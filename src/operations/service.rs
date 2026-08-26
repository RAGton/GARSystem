// src/operations/service.rs
//
// Camada de regras — Sprint P2.4.
//
// Orquestra:
//   1. Workflow engine (state machine centralizada)
//   2. Kanban
//   3. Agenda técnica
//   4. Cálculo de SLA
//   5. Alertas operacionais
//   6. Dashboard executivo

use super::models::{
    Agenda, Alerta, AlertaOcorrencia, DashboardExecutivo, EventoAgenda, KanbanCard, KanbanColuna,
    Severidade, SlaCalculo, SlaConfig, StatusEvento, TipoEvento, TipoSlaEvento, WorkflowDefinicao,
    WorkflowEstado, WorkflowMovimentacao, WorkflowStatus, WorkflowTransicao,
};
use super::repository;
use crate::crm::repository as crm_repo;
use crate::servicos::ErroAplicacao;
use chrono::{DateTime, Utc};
use mysql::params;
use mysql::prelude::Queryable;
use mysql::Row;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Contexto {
    pub usuario_id: u32,
    pub username: Option<String>,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ErroOperacao {
    #[error("workflow não encontrado")]
    WorkflowNaoEncontrado,
    #[error("estado inválido: {0}")]
    EstadoInvalido(String),
    #[error("transição inválida de '{0}' para '{1}'")]
    TransicaoInvalida(String, String),
    #[error("motivo obrigatório para esta transição")]
    MotivoObrigatorio,
    #[error("arquivo obrigatório para esta transição")]
    ArquivoObrigatorio,
    #[error("conflito de agenda")]
    ConflitoAgenda,
    #[error("OS não encontrada")]
    OsNaoEncontrada,
    #[error("desconhecido: {0}")]
    Desconhecido(String),
}

impl From<ErroOperacao> for ErroAplicacao {
    fn from(e: ErroOperacao) -> Self {
        ErroAplicacao::Desconhecido(format!("{:?}", e))
    }
}

impl From<ErroAplicacao> for ErroOperacao {
    fn from(e: ErroAplicacao) -> Self {
        ErroOperacao::Desconhecido(format!("{:?}", e))
    }
}

// =============================================================================
// Workflow Engine
// =============================================================================

/// Retorna o workflow padrão (ativo, entidade_tipo='OS').
pub fn obter_workflow_padrao(tenant_id: i32) -> Result<WorkflowDefinicao, ErroOperacao> {
    repository::obter_workflow_default(tenant_id)?.ok_or(ErroOperacao::WorkflowNaoEncontrado)
}

/// Move uma OS para um novo estado, registrando movimentação e timeline CRM.
pub fn mover_estado(
    tenant_id: i32,
    os_id: u32,
    novo_estado_slug: &str,
    motivo: Option<&str>,
    ctx: &Contexto,
) -> Result<WorkflowMovimentacao, ErroOperacao> {
    if os_id == 0 {
        return Err(ErroOperacao::OsNaoEncontrada);
    }
    let workflow = obter_workflow_padrao(tenant_id)?;
    let novo_estado = repository::obter_estado_por_slug(tenant_id, workflow.id, novo_estado_slug)?
        .ok_or_else(|| ErroOperacao::EstadoInvalido(novo_estado_slug.to_string()))?;
    let estados = repository::listar_estados(tenant_id, workflow.id)?;
    let transicoes = repository::listar_transicoes(tenant_id, workflow.id)?;

    // Estado atual
    let atual = repository::ultima_movimentacao(tenant_id, workflow.id, os_id)?;
    let estado_atual_id = atual.as_ref().map(|m| m.estado_destino_id);
    let estado_atual_slug = estado_atual_id
        .and_then(|id| estados.iter().find(|e| e.id == id).map(|e| e.slug.clone()))
        .unwrap_or_else(|| "RECEBIDO".to_string());

    if let Some(atual_id) = estado_atual_id {
        if atual_id == novo_estado.id {
            // No-op: já está nesse estado. Retornar a última movimentação.
            return Ok(atual.unwrap());
        }
        // Verifica transição
        let trans = transicoes
            .iter()
            .find(|t| t.estado_origem_id == atual_id && t.estado_destino_id == novo_estado.id);
        let trans = match trans {
            Some(t) => t,
            None => {
                return Err(ErroOperacao::TransicaoInvalida(
                    estado_atual_slug,
                    novo_estado_slug.to_string(),
                ))
            }
        };
        if trans.exige_motivo && motivo.map(|s| s.trim().is_empty()).unwrap_or(true) {
            return Err(ErroOperacao::MotivoObrigatorio);
        }
    }

    // Duração no estado anterior
    let duracao_anterior_seg = atual.as_ref().map(|a| {
        let delta = Utc::now().signed_duration_since(a.data_movimentacao);
        delta.num_seconds().max(0) as u32
    });

    // Registra
    let mov_id = repository::registrar_movimentacao(
        tenant_id,
        workflow.id,
        os_id,
        estado_atual_id,
        novo_estado.id,
        Some(ctx.usuario_id),
        ctx.username.as_deref(),
        motivo,
        duracao_anterior_seg,
        ctx.ip.as_deref(),
    )?;
    let _ = mov_id;

    // Atualiza `ultima_movimentacao_data` na OS (best-effort, schema é legado)
    let _ = atualizar_ultima_movimentacao_os(tenant_id, os_id);

    // Timeline CRM
    let payload = serde_json::json!({
        "workflow_id": workflow.id,
        "estado_origem": estado_atual_slug,
        "estado_destino": novo_estado.slug,
        "motivo": motivo,
    });
    let _ = inserir_evento_timeline_para_os(
        tenant_id,
        os_id,
        "OS_WORKFLOW_MOVIDA",
        &format!("OS movida: {} → {}", estado_atual_slug, novo_estado.slug),
        Some(&payload),
        Some(ctx.usuario_id),
        ctx.username.as_deref(),
    );

    // SLA events
    registrar_evento_sla_por_transicao(
        tenant_id,
        os_id,
        &estado_atual_slug,
        &novo_estado.slug,
        ctx,
    );

    // Retornar a movimentação criada
    let mov = repository::ultima_movimentacao(tenant_id, workflow.id, os_id)?
        .ok_or_else(|| ErroOperacao::Desconhecido("movimentação sumiu".into()))?;
    Ok(mov)
}

fn registrar_evento_sla_por_transicao(
    tenant_id: i32,
    os_id: u32,
    estado_origem: &str,
    estado_destino: &str,
    ctx: &Contexto,
) {
    let evento = match (estado_origem, estado_destino) {
        ("RECEBIDO", "DIAGNOSTICO") => Some(TipoSlaEvento::DiagnosticoInicio),
        ("DIAGNOSTICO", "AGUARDANDO_PECA") => Some(TipoSlaEvento::PecaSolicitada),
        ("AGUARDANDO_PECA", "DIAGNOSTICO") => Some(TipoSlaEvento::PecaRecebida),
        (_, "AGUARDANDO_APROVACAO") => Some(TipoSlaEvento::AprovacaoSolicitada),
        ("AGUARDANDO_APROVACAO", "APROVADO") => Some(TipoSlaEvento::AprovacaoRecebida),
        ("AGUARDANDO_APROVACAO", "EXECUCAO") => Some(TipoSlaEvento::AprovacaoRecebida),
        ("APROVADO", "EXECUCAO") => Some(TipoSlaEvento::ExecucaoInicio),
        ("EXECUCAO", "TESTE") => Some(TipoSlaEvento::TesteInicio),
        ("TESTE", "FINALIZADO") => Some(TipoSlaEvento::TesteFim),
        (_, "FINALIZADO") => Some(TipoSlaEvento::Finalizado),
        (_, "ENTREGUE") => Some(TipoSlaEvento::Entregue),
        (_, "CANCELADO") => Some(TipoSlaEvento::Cancelado),
        _ => None,
    };
    if let Some(t) = evento {
        let _ = repository::registrar_evento_sla(
            tenant_id,
            os_id,
            t,
            Utc::now(),
            Some(ctx.usuario_id),
            None,
        );
    }
}

fn atualizar_ultima_movimentacao_os(tenant_id: i32, os_id: u32) -> Result<(), ErroAplicacao> {
    use mysql::prelude::Queryable;
    let mut conn = crate::banco_de_dados::obter_conexao()?;
    // Idempotente: cria coluna se não existir (não temos DDL aqui, então só tenta)
    let _ = conn.exec_drop(
        "UPDATE ordens_servico SET ultima_movimentacao_data = NOW() WHERE tenant_id = ? AND id = ?",
        (tenant_id, os_id),
    );
    Ok(())
}

fn inserir_evento_timeline_para_os(
    tenant_id: i32,
    os_id: u32,
    tipo: &str,
    descricao: &str,
    payload: Option<&serde_json::Value>,
    usuario_id: Option<u32>,
    username: Option<&str>,
) -> Result<(), ErroAplicacao> {
    use mysql::prelude::Queryable;
    let mut conn = crate::banco_de_dados::obter_conexao()?;
    let cid: Option<u32> = conn
        .exec_first(
            "SELECT cliente_id FROM ordens_servico WHERE id = ? AND tenant_id = ?",
            (os_id, tenant_id),
        )
        .ok()
        .flatten();
    if let Some(cliente_id) = cid {
        let _ = crm_repo::inserir_evento_timeline(
            tenant_id, cliente_id, tipo, descricao, payload, usuario_id, username,
        );
    }
    Ok(())
}

/// Retorna o status completo do workflow de uma OS.
pub fn obter_status_workflow(
    tenant_id: i32,
    os_id: u32,
) -> Result<Option<WorkflowStatus>, ErroOperacao> {
    let workflow = obter_workflow_padrao(tenant_id)?;
    let estados = repository::listar_estados(tenant_id, workflow.id)?;
    let transicoes = repository::listar_transicoes(tenant_id, workflow.id)?;
    let historico = repository::listar_movimentacoes(tenant_id, workflow.id, os_id, 100)?;

    let ultima = repository::ultima_movimentacao(tenant_id, workflow.id, os_id)?;
    let estado_atual_id = ultima.as_ref().map(|m| m.estado_destino_id);
    let estado_atual = match estado_atual_id {
        Some(id) => estados.iter().find(|e| e.id == id).cloned(),
        None => estados.iter().find(|e| e.eh_inicial).cloned(),
    }
    .ok_or_else(|| ErroOperacao::EstadoInvalido("estado inicial".into()))?;

    let transicoes_validas: Vec<u32> = transicoes
        .iter()
        .filter(|t| t.estado_origem_id == estado_atual.id)
        .map(|t| t.id)
        .collect();

    Ok(Some(WorkflowStatus {
        workflow,
        estado_atual,
        estados,
        transicoes,
        transicoes_validas,
        historico,
    }))
}

pub fn listar_estados(tenant_id: i32) -> Result<Vec<WorkflowEstado>, ErroOperacao> {
    let workflow = obter_workflow_padrao(tenant_id)?;
    Ok(repository::listar_estados(tenant_id, workflow.id)?)
}

pub fn listar_transicoes(tenant_id: i32) -> Result<Vec<WorkflowTransicao>, ErroOperacao> {
    let workflow = obter_workflow_padrao(tenant_id)?;
    Ok(repository::listar_transicoes(tenant_id, workflow.id)?)
}

pub fn kanban(tenant_id: i32) -> Result<Vec<KanbanColuna>, ErroOperacao> {
    let workflow = obter_workflow_padrao(tenant_id)?;
    Ok(repository::kanban_por_estado(tenant_id, workflow.id)?)
}

pub fn kanban_para_tecnico(
    tenant_id: i32,
    tecnico_id: u32,
) -> Result<Vec<KanbanColuna>, ErroOperacao> {
    let mut cols = kanban(tenant_id)?;
    for c in &mut cols {
        c.cards
            .retain(|card| card.responsavel_id == Some(tecnico_id));
    }
    Ok(cols)
}

// =============================================================================
// Agenda
// =============================================================================

pub fn criar_evento_agenda(
    tenant_id: i32,
    tecnico_id: u32,
    os_id: Option<u32>,
    cliente_id: Option<u32>,
    titulo: &str,
    descricao: Option<&str>,
    tipo: TipoEvento,
    inicio: DateTime<Utc>,
    fim: DateTime<Utc>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    endereco: Option<&str>,
    ctx: &Contexto,
    permitir_conflito: bool,
) -> Result<u32, ErroOperacao> {
    if titulo.trim().is_empty() {
        return Err(ErroOperacao::Desconhecido("título obrigatório".into()));
    }
    if fim <= inicio {
        return Err(ErroOperacao::Desconhecido(
            "fim deve ser depois de início".into(),
        ));
    }
    // Verifica conflito
    if !permitir_conflito {
        let eventos = repository::listar_eventos_periodo(tenant_id, Some(tecnico_id), inicio, fim)?;
        for e in eventos {
            if e.conflita_com_com(&e, inicio, fim) {
                return Err(ErroOperacao::ConflitoAgenda);
            }
        }
    }
    let agenda_id = repository::garantir_agenda_tecnico(tenant_id, tecnico_id)?;
    let id = repository::criar_evento_agenda(
        tenant_id,
        agenda_id,
        tecnico_id,
        os_id,
        cliente_id,
        titulo,
        descricao,
        tipo,
        inicio,
        fim,
        latitude,
        longitude,
        endereco,
        ctx.usuario_id,
    )?;
    Ok(id)
}

// Trait de conveniência
trait Conflita {
    fn conflita_com_com(&self, _outro: &Self, _inicio: DateTime<Utc>, _fim: DateTime<Utc>) -> bool;
}
impl Conflita for EventoAgenda {
    fn conflita_com_com(&self, _outro: &Self, inicio: DateTime<Utc>, fim: DateTime<Utc>) -> bool {
        self.inicio < fim && inicio < self.fim && !matches!(self.status, StatusEvento::Cancelado)
    }
}

pub fn listar_eventos_agenda(
    tenant_id: i32,
    tecnico_id: Option<u32>,
    inicio: DateTime<Utc>,
    fim: DateTime<Utc>,
) -> Result<Vec<EventoAgenda>, ErroOperacao> {
    Ok(repository::listar_eventos_periodo(
        tenant_id, tecnico_id, inicio, fim,
    )?)
}

pub fn obter_evento_agenda(tenant_id: i32, id: u32) -> Result<Option<EventoAgenda>, ErroOperacao> {
    Ok(repository::obter_evento_agenda(tenant_id, id)?)
}

pub fn atualizar_status_evento(
    tenant_id: i32,
    id: u32,
    status: StatusEvento,
) -> Result<bool, ErroOperacao> {
    Ok(repository::atualizar_status_evento(tenant_id, id, status)?)
}

pub fn remover_evento(tenant_id: i32, id: u32) -> Result<bool, ErroOperacao> {
    Ok(repository::remover_evento(tenant_id, id)?)
}

pub fn detectar_conflitos(
    tenant_id: i32,
    tecnico_id: u32,
    inicio: DateTime<Utc>,
    fim: DateTime<Utc>,
) -> Result<Vec<EventoAgenda>, ErroOperacao> {
    let eventos = repository::listar_eventos_periodo(tenant_id, Some(tecnico_id), inicio, fim)?;
    Ok(eventos
        .into_iter()
        .filter(|e| {
            e.inicio < fim && inicio < e.fim && !matches!(e.status, StatusEvento::Cancelado)
        })
        .collect())
}

// =============================================================================
// SLA
// =============================================================================

pub fn obter_sla_default(tenant_id: i32) -> Result<Option<SlaConfig>, ErroOperacao> {
    Ok(repository::obter_sla_default(tenant_id)?)
}

pub fn calcular_sla_os(tenant_id: i32, os_id: u32) -> Result<SlaCalculo, ErroOperacao> {
    let config = repository::obter_sla_default(tenant_id)?
        .ok_or_else(|| ErroOperacao::Desconhecido("sem SLA config".into()))?;
    let eventos = repository::listar_eventos_sla(tenant_id, os_id)?;
    let calc = calcular_tempos(&eventos, &config);
    let motivo = if calc.sla_violado {
        Some(gerar_motivo_violacao(&calc, &config))
    } else {
        None
    };
    repository::upsert_sla_calculo(
        tenant_id,
        os_id,
        Some(config.id),
        calc.tempo_diagnostico_segundos,
        calc.tempo_espera_peca_segundos,
        calc.tempo_espera_aprovacao_segundos,
        calc.tempo_execucao_segundos,
        calc.tempo_total_segundos,
        calc.sla_violado,
        motivo.as_deref(),
    )?;
    repository::obter_sla_calculo(tenant_id, os_id)?
        .ok_or_else(|| ErroOperacao::Desconhecido("cálculo sumiu".into()))
}

struct CalcTemp {
    tempo_diagnostico_segundos: Option<u32>,
    tempo_espera_peca_segundos: Option<u32>,
    tempo_espera_aprovacao_segundos: Option<u32>,
    tempo_execucao_segundos: Option<u32>,
    tempo_total_segundos: Option<u32>,
    sla_violado: bool,
}

fn calcular_tempos(eventos: &[(TipoSlaEvento, DateTime<Utc>)], config: &SlaConfig) -> CalcTemp {
    let lookup = |t: TipoSlaEvento| -> Option<DateTime<Utc>> {
        eventos.iter().find(|(et, _)| *et == t).map(|(_, d)| *d)
    };
    let recebido = lookup(TipoSlaEvento::Recebido);
    let diag_ini = lookup(TipoSlaEvento::DiagnosticoInicio);
    let diag_fim = lookup(TipoSlaEvento::DiagnosticoFim);
    let peca_sol = lookup(TipoSlaEvento::PecaSolicitada);
    let peca_rec = lookup(TipoSlaEvento::PecaRecebida);
    let aprov_sol = lookup(TipoSlaEvento::AprovacaoSolicitada);
    let aprov_rec = lookup(TipoSlaEvento::AprovacaoRecebida);
    let exec_ini = lookup(TipoSlaEvento::ExecucaoInicio);
    let exec_fim = lookup(TipoSlaEvento::ExecucaoFim);
    let finalizado = lookup(TipoSlaEvento::Finalizado);
    let entregue = lookup(TipoSlaEvento::Entregue);

    let dur = |ini: DateTime<Utc>, fim: DateTime<Utc>| -> u32 {
        let s = fim.signed_duration_since(ini).num_seconds().max(0);
        s as u32
    };
    let horas = |seg: u32| seg / 3600;

    let tempo_diagnostico = match (diag_ini, diag_fim) {
        (Some(a), Some(b)) => Some(dur(a, b)),
        _ => None,
    };
    let tempo_espera_peca = match (peca_sol, peca_rec) {
        (Some(a), Some(b)) => Some(dur(a, b)),
        _ => None,
    };
    let tempo_espera_aprovacao = match (aprov_sol, aprov_rec) {
        (Some(a), Some(b)) => Some(dur(a, b)),
        _ => None,
    };
    let tempo_execucao = match (exec_ini, exec_fim) {
        (Some(a), Some(b)) => Some(dur(a, b)),
        _ => None,
    };
    let tempo_total = match (recebido, entregue) {
        (Some(a), Some(b)) => Some(dur(a, b)),
        _ => match (recebido, finalizado) {
            (Some(a), Some(b)) => Some(dur(a, b)),
            _ => None,
        },
    };

    let mut violado = false;
    if let (Some(seg), Some(max_h)) = (tempo_diagnostico, config.max_horas_diagnostico) {
        if horas(seg) > max_h {
            violado = true;
        }
    }
    if let (Some(seg), Some(max_h)) = (tempo_execucao, config.max_horas_execucao) {
        if horas(seg) > max_h {
            violado = true;
        }
    }
    if let (Some(seg), Some(max_h)) = (tempo_total, config.max_horas_total) {
        if horas(seg) > max_h {
            violado = true;
        }
    }
    if let (Some(seg), Some(max_h)) = (tempo_espera_peca, config.max_horas_espera_peca) {
        if horas(seg) > max_h {
            violado = true;
        }
    }
    if let (Some(seg), Some(max_h)) = (tempo_espera_aprovacao, config.max_horas_espera_aprovacao) {
        if horas(seg) > max_h {
            violado = true;
        }
    }

    CalcTemp {
        tempo_diagnostico_segundos: tempo_diagnostico,
        tempo_espera_peca_segundos: tempo_espera_peca,
        tempo_espera_aprovacao_segundos: tempo_espera_aprovacao,
        tempo_execucao_segundos: tempo_execucao,
        tempo_total_segundos: tempo_total,
        sla_violado: violado,
    }
}

fn gerar_motivo_violacao(calc: &CalcTemp, config: &SlaConfig) -> String {
    let horas = |s: u32| s / 3600;
    let mut motivos = Vec::new();
    if let (Some(s), Some(m)) = (
        calc.tempo_diagnostico_segundos,
        config.max_horas_diagnostico,
    ) {
        if horas(s) > m {
            motivos.push(format!("diagnóstico {}h > {}h", horas(s), m));
        }
    }
    if let (Some(s), Some(m)) = (calc.tempo_execucao_segundos, config.max_horas_execucao) {
        if horas(s) > m {
            motivos.push(format!("execução {}h > {}h", horas(s), m));
        }
    }
    if let (Some(s), Some(m)) = (calc.tempo_total_segundos, config.max_horas_total) {
        if horas(s) > m {
            motivos.push(format!("total {}h > {}h", horas(s), m));
        }
    }
    motivos.join("; ")
}

// =============================================================================
// Alertas
// =============================================================================

pub fn listar_alertas_ativos(tenant_id: i32) -> Result<Vec<Alerta>, ErroOperacao> {
    Ok(repository::listar_alertas_ativos(tenant_id)?)
}

/// Verifica alertas ativos e cria ocorrências. Idempotente por dia.
pub fn verificar_alertas(tenant_id: i32, _ctx: &Contexto) -> Result<u32, ErroOperacao> {
    let alertas = repository::listar_alertas_ativos(tenant_id)?;
    let workflow = obter_workflow_padrao(tenant_id)?;
    let mut criadas = 0u32;
    for alerta in alertas {
        match alerta.condicao_tipo.as_str() {
            "TEMPO_NO_ESTADO" => {
                criadas += verificar_tempo_no_estado(tenant_id, &alerta, &workflow)?;
            }
            "CHECKLIST_PENDENTE" => {
                // Stub: implementação virá em P2.4.x
            }
            _ => {}
        }
    }
    Ok(criadas)
}

fn verificar_tempo_no_estado(
    tenant_id: i32,
    alerta: &Alerta,
    workflow: &WorkflowDefinicao,
) -> Result<u32, ErroOperacao> {
    let threshold = match alerta.condicao_threshold_horas {
        Some(t) => t as i64,
        None => return Ok(0),
    };
    let estado_slug_filtro = alerta.condicao_estado_slug.clone();
    let mut conn = crate::banco_de_dados::obter_conexao()?;
    use mysql::prelude::Queryable;
    let mut sql = String::from(
        r#"SELECT os.id, MAX(m.data_movimentacao) as ultima
           FROM ordens_servico os
           INNER JOIN workflow_movimentacoes m ON m.workflow_id = ? AND m.entidade_id = os.id AND m.tenant_id = os.tenant_id
           WHERE os.tenant_id = ? AND os.status NOT IN ('ENTREGUE','CANCELADA','FINALIZADA')"#,
    );
    if let Some(_slug) = &estado_slug_filtro {
        sql.push_str(" AND os.status = ?");
    }
    sql.push_str(" GROUP BY os.id HAVING TIMESTAMPDIFF(HOUR, ultima, NOW()) >= ?");
    let rows: Vec<Row> = if let Some(slug) = &estado_slug_filtro {
        conn.exec(&sql, (workflow.id, tenant_id, slug.as_str(), threshold))
            .map_err(|e| ErroOperacao::Desconhecido(e.to_string()))?
    } else {
        conn.exec(&sql, (workflow.id, tenant_id, threshold))
            .map_err(|e| ErroOperacao::Desconhecido(e.to_string()))?
    };
    let mut count = 0u32;
    for r in rows {
        let os_id: u32 = match r.get::<mysql::Value, _>(0) {
            Some(mysql::Value::Int(n)) => n as u32,
            _ => continue,
        };
        let contexto = serde_json::json!({ "os_id": os_id });
        let _ = repository::inserir_ocorrencia(
            tenant_id,
            alerta.id,
            os_id,
            alerta.severidade,
            &format!("{} (OS #{})", alerta.titulo, os_id),
            Some(&contexto),
        );
        count += 1;
    }
    Ok(count)
}

pub fn listar_alertas_pendentes(
    tenant_id: i32,
    limite: u32,
) -> Result<Vec<AlertaOcorrencia>, ErroOperacao> {
    Ok(repository::listar_ocorrencias_pendentes(tenant_id, limite)?)
}

pub fn listar_alertas_por_entidade(
    tenant_id: i32,
    entidade_id: u32,
) -> Result<Vec<AlertaOcorrencia>, ErroOperacao> {
    Ok(repository::listar_ocorrencias_por_entidade(
        tenant_id,
        entidade_id,
    )?)
}

pub fn marcar_alerta_visualizado(
    tenant_id: i32,
    id: u32,
    ctx: &Contexto,
) -> Result<bool, ErroOperacao> {
    Ok(repository::marcar_visualizado(
        tenant_id,
        id,
        ctx.usuario_id,
    )?)
}

pub fn marcar_alerta_resolvido(tenant_id: i32, id: u32) -> Result<bool, ErroOperacao> {
    Ok(repository::marcar_resolvido(tenant_id, id)?)
}

// =============================================================================
// Dashboard Executivo
// =============================================================================

pub fn dashboard_executivo(tenant_id: i32) -> Result<DashboardExecutivo, ErroOperacao> {
    let mut conn = crate::banco_de_dados::obter_conexao()?;
    use mysql::prelude::Queryable;
    let dash = DashboardExecutivo::default();

    // P2.6.2a: dashboard consolidado em 5 queries (CASE WHEN + GROUP BY)
    // Reduz 13+2N queries para 5 (sem N+1)
    let stats: Option<(i64, i64, i64, i64, i64, i64, i64, Option<f64>, i64, i64)> = conn
        .exec_first(
            r#"
            SELECT
                SUM(CASE WHEN os.status NOT IN ('ENTREGUE','CANCELADA','FINALIZADA') THEN 1 ELSE 0 END) as abertas,
                SUM(CASE WHEN os.status IN ('FINALIZADA','ENTREGUE') THEN 1 ELSE 0 END) as concluidas,
                COUNT(*) as total,
                SUM(CASE WHEN os.status LIKE 'AGUARDANDO_PEC%' THEN 1 ELSE 0 END) as aguardando_peca,
                SUM(CASE WHEN os.status LIKE 'AGUARDANDO_APR%' THEN 1 ELSE 0 END) as aguardando_aprov,
                SUM(CASE WHEN os.status IN ('EM_ANDAMENTO','EM_EXECUCAO','EXECUCAO','TESTE') THEN 1 ELSE 0 END) as em_execucao,
                SUM(CASE WHEN os.status = 'ENTREGUE' THEN 1 ELSE 0 END) as entregues,
                (SELECT AVG(tempo_total_segundos)/3600.0 FROM sla_calculos WHERE tempo_total_segundos IS NOT NULL) as tempo_medio_h,
                (SELECT COUNT(*) FROM alertas_ocorrencias WHERE resolvido = FALSE) as alertas_pendentes,
                (SELECT COUNT(*) FROM alertas_ocorrencias WHERE resolvido = FALSE AND severidade = 'CRITICAL') as alertas_criticos
            FROM ordens_servico os
            WHERE os.tenant_id = :tenant_id
            "#,
            params! { "tenant_id" => tenant_id },
        )
        .map_err(|e| ErroOperacao::Desconhecido(e.to_string()))?;
    let (
        abertas,
        concluidas,
        total,
        aguardando_peca,
        aguardando_aprov,
        em_execucao,
        entregues,
        tempo_medio_h_opt,
        alertas_pendentes,
        alertas_criticos,
    ) = stats.unwrap_or((0, 0, 0, 0, 0, 0, 0, None, 0, 0));
    let tempo_medio_h = tempo_medio_h_opt.map(|f| f as f32);

    let atrasadas: i64 = conn
        .exec_first(
            "SELECT COUNT(*) FROM sla_calculos WHERE tenant_id = :tenant_id AND sla_violado = TRUE",
            params! { "tenant_id" => tenant_id },
        )
        .ok()
        .flatten()
        .unwrap_or(0);

    // Técnicos — 1 query (CASE WHEN inclui taxa de aprovação + tempo médio)
    let tecnicos_rows: Vec<Row> = conn
        .exec(
            r#"SELECT u.id, u.username,
                      COUNT(o.id) as total,
                      SUM(CASE WHEN o.status IN ('FINALIZADA','ENTREGUE') THEN 1 ELSE 0 END) as concluidas,
                      AVG(c.tempo_total_segundos)/3600.0 as tempo_medio_h
               FROM usuarios u
               LEFT JOIN ordens_servico o ON o.tecnico_id = u.id AND o.tenant_id = :tenant_id
               LEFT JOIN sla_calculos c ON c.os_id = o.id
               WHERE u.tenant_id = :tenant_id
               GROUP BY u.id, u.username
               HAVING total > 0
               ORDER BY total DESC
               LIMIT 20"#,
            params! { "tenant_id" => tenant_id },
        )
        .map_err(|e| ErroOperacao::Desconhecido(e.to_string()))?;
    let mut tecnicos = Vec::new();
    for r in tecnicos_rows {
        let tid: u32 = match r.get::<mysql::Value, _>(0) {
            Some(mysql::Value::UInt(n)) => n as u32,
            Some(mysql::Value::Int(n)) => n as u32,
            _ => continue,
        };
        let nome: String = match r.get::<mysql::Value, _>(1) {
            Some(mysql::Value::Bytes(b)) => String::from_utf8_lossy(&b).into_owned(),
            _ => continue,
        };
        let total: u32 = match r.get::<mysql::Value, _>(2) {
            Some(mysql::Value::Int(n)) => n as u32,
            Some(mysql::Value::UInt(n)) => n as u32,
            _ => 0,
        };
        let concluidas_t: u32 = match r.get::<mysql::Value, _>(3) {
            Some(mysql::Value::Int(n)) => n as u32,
            Some(mysql::Value::UInt(n)) => n as u32,
            _ => 0,
        };
        let tempo_medio: Option<f32> = r.get::<Option<f64>, _>(4).flatten().map(|f| f as f32);
        let taxa_aprovacao = if total > 0 {
            Some((concluidas_t as f32 / total as f32) * 100.0)
        } else {
            None
        };
        tecnicos.push(super::models::TecnicoStats {
            tecnico_id: tid,
            tecnico_nome: nome,
            os_total: total,
            os_concluidas: concluidas_t,
            tempo_medio_horas: tempo_medio,
            taxa_aprovacao,
        });
    }

    let _ = dash;
    let tempo_medio_por_tecnico = if !tecnicos.is_empty() {
        let vals: Vec<f32> = tecnicos
            .iter()
            .filter_map(|t| t.tempo_medio_horas)
            .collect();
        if vals.is_empty() {
            None
        } else {
            Some(vals.iter().sum::<f32>() / vals.len() as f32)
        }
    } else {
        None
    };

    // Top clientes — 1 query (sem N+1)
    let cli_rows: Vec<Row> = conn
        .exec(
            r#"SELECT c.id, c.nome, COUNT(o.id) as total,
                      SUM(CASE WHEN o.status IN ('FINALIZADA','ENTREGUE') THEN 1 ELSE 0 END) as concluidas,
                      AVG(s.tempo_total_segundos)/3600.0 as tempo_medio_h
               FROM clientes c
               INNER JOIN ordens_servico o ON o.cliente_id = c.id AND o.tenant_id = :tenant_id
               LEFT JOIN sla_calculos s ON s.os_id = o.id
               WHERE c.tenant_id = :tenant_id
               GROUP BY c.id, c.nome
               ORDER BY total DESC
               LIMIT 10"#,
            params! { "tenant_id" => tenant_id },
        )
        .map_err(|e| ErroOperacao::Desconhecido(e.to_string()))?;
    let mut top_clientes = Vec::new();
    for r in cli_rows {
        let cid: u32 = match r.get::<mysql::Value, _>(0) {
            Some(mysql::Value::Int(n)) => n as u32,
            Some(mysql::Value::UInt(n)) => n as u32,
            _ => continue,
        };
        let nome: String = match r.get::<mysql::Value, _>(1) {
            Some(mysql::Value::Bytes(b)) => String::from_utf8_lossy(&b).into_owned(),
            _ => continue,
        };
        let total: u32 = match r.get::<mysql::Value, _>(2) {
            Some(mysql::Value::Int(n)) => n as u32,
            Some(mysql::Value::UInt(n)) => n as u32,
            _ => 0,
        };
        let concluidas_c: u32 = match r.get::<mysql::Value, _>(3) {
            Some(mysql::Value::Int(n)) => n as u32,
            Some(mysql::Value::UInt(n)) => n as u32,
            _ => 0,
        };
        let tempo_medio: Option<f32> = r.get::<Option<f64>, _>(4).flatten().map(|f| f as f32);
        top_clientes.push(super::models::ClienteStats {
            cliente_id: cid,
            cliente_nome: nome,
            os_total: total,
            os_concluidas: concluidas_c,
            tempo_medio_horas: tempo_medio,
        });
    }

    Ok(DashboardExecutivo {
        os_abertas: abertas as u32,
        os_atrasadas: atrasadas as u32,
        os_aguardando_peca: aguardando_peca as u32,
        os_aguardando_aprovacao: aguardando_aprov as u32,
        os_em_execucao: em_execucao as u32,
        os_concluidas: concluidas as u32,
        os_entregues: entregues as u32,
        os_total: total as u32,
        alertas_pendentes: alertas_pendentes as u32,
        alertas_criticos: alertas_criticos as u32,
        tempo_medio_resolucao_horas: tempo_medio_h,
        tempo_medio_por_tecnico_horas: tempo_medio_por_tecnico,
        tecnicos,
        top_clientes,
    })
}

// =============================================================================
// Testes
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contexto_default() {
        let c = Contexto::default();
        assert_eq!(c.usuario_id, 0);
    }

    #[test]
    fn calcular_tempos_sem_eventos() {
        let config = SlaConfig {
            id: 1,
            nome: "x".into(),
            descricao: None,
            ativo: true,
            max_horas_diagnostico: Some(24),
            max_horas_execucao: Some(72),
            max_horas_total: Some(240),
            max_horas_espera_peca: Some(168),
            max_horas_espera_aprovacao: Some(72),
            data_criacao: Utc::now(),
        };
        let calc = calcular_tempos(&[], &config);
        assert!(!calc.sla_violado);
        assert!(calc.tempo_diagnostico_segundos.is_none());
    }

    #[test]
    fn calcular_tempos_dentro_do_sla() {
        use chrono::Duration;
        let config = SlaConfig {
            id: 1,
            nome: "x".into(),
            descricao: None,
            ativo: true,
            max_horas_diagnostico: Some(24),
            max_horas_execucao: Some(72),
            max_horas_total: Some(240),
            max_horas_espera_peca: Some(168),
            max_horas_espera_aprovacao: Some(72),
            data_criacao: Utc::now(),
        };
        let inicio = Utc::now();
        let eventos = vec![
            (TipoSlaEvento::Recebido, inicio),
            (
                TipoSlaEvento::DiagnosticoInicio,
                inicio + Duration::hours(1),
            ),
            (TipoSlaEvento::DiagnosticoFim, inicio + Duration::hours(2)),
        ];
        let calc = calcular_tempos(&eventos, &config);
        assert!(!calc.sla_violado);
        assert_eq!(calc.tempo_diagnostico_segundos, Some(3600));
    }

    #[test]
    fn calcular_tempos_violado() {
        use chrono::Duration;
        let config = SlaConfig {
            id: 1,
            nome: "x".into(),
            descricao: None,
            ativo: true,
            max_horas_diagnostico: Some(1), // muito apertado
            max_horas_execucao: Some(72),
            max_horas_total: Some(240),
            max_horas_espera_peca: Some(168),
            max_horas_espera_aprovacao: Some(72),
            data_criacao: Utc::now(),
        };
        let inicio = Utc::now();
        let eventos = vec![
            (TipoSlaEvento::Recebido, inicio),
            (TipoSlaEvento::DiagnosticoInicio, inicio),
            (TipoSlaEvento::DiagnosticoFim, inicio + Duration::hours(5)),
        ];
        let calc = calcular_tempos(&eventos, &config);
        assert!(calc.sla_violado);
        assert!(gerar_motivo_violacao(&calc, &config).contains("diagnóstico"));
    }
}
