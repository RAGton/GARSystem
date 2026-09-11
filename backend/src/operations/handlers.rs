// src/operations/handlers.rs
//! Handlers HTTP do módulo Operacional (workflow, kanban, SLA, agenda,
//! alertas e dashboard executivo).
//!
//! Compilado apenas pelo binário `senior-system-server` (declarado em
//! `server.rs` via `#[path]`).
//!
//! ## Contrato comum
//!
//! - `Claims` obrigatório + `check_perm` explícito por rota.
//! - `tenant_id` sempre de `auth::tenant_do_usuario(&claims)`.
//! - **Toda** transição de estado de OS passa por
//!   `operations::service::mover_estado` (state machine central). Nenhum
//!   handler escreve `ordens_servico.status` direto.

use axum::{
    extract::{Json, Path, Query},
    http::StatusCode,
    response::Json as AxJson,
};
use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
use serde::Deserialize;

use crate::auth::{self, Claims};
use crate::operations::models::{
    Alerta, DashboardExecutivo, EventoAgenda, KanbanColuna, SlaCalculo, StatusEvento, TipoEvento,
    WorkflowEstado, WorkflowMovimentacao,
};
use crate::operations::service::{self, Contexto, ErroOperacao};
use crate::{erro_padrao, ErroApi};
use crate::check_perm;

type Resposta<T> = Result<AxJson<T>, (StatusCode, AxJson<ErroApi>)>;
type RespostaStatus = Result<StatusCode, (StatusCode, AxJson<ErroApi>)>;

fn contexto(claims: &Claims) -> Contexto {
    Contexto {
        usuario_id: if claims.uid > 0 { claims.uid as u32 } else { 0 },
        username: Some(claims.sub.clone()),
        ip: None,
        user_agent: None,
        request_id: None,
    }
}

/// Traduz `ErroOperacao` para a resposta HTTP padronizada.
///
/// Erros de regra de negócio (transição inválida, motivo obrigatório,
/// conflito de agenda) são **4xx**, não 500 — o cliente pode corrigir.
fn map_erro_operacao(e: ErroOperacao) -> (StatusCode, AxJson<ErroApi>) {
    let (code, msg): (&'static str, String) = match &e {
        ErroOperacao::WorkflowNaoEncontrado => (
            "NOT_FOUND",
            "Nenhum workflow ativo configurado para este tenant".to_string(),
        ),
        ErroOperacao::OsNaoEncontrada => {
            ("NOT_FOUND", "Ordem de serviço não encontrada".to_string())
        }
        ErroOperacao::EstadoInvalido(s) => ("VALIDATION", format!("Estado inválido: {}", s)),
        ErroOperacao::TransicaoInvalida(de, para) => (
            "CONFLICT",
            format!("Transição inválida de '{}' para '{}'", de, para),
        ),
        ErroOperacao::MotivoObrigatorio => (
            "VALIDATION",
            "Esta transição exige o campo 'motivo'".to_string(),
        ),
        ErroOperacao::ArquivoObrigatorio => (
            "VALIDATION",
            "Esta transição exige anexo de arquivo".to_string(),
        ),
        ErroOperacao::ConflitoAgenda => (
            "CONFLICT",
            "Técnico já tem evento neste intervalo".to_string(),
        ),
        ErroOperacao::Desconhecido(m) => {
            tracing::error!(target: "api", "ErroOperacao::Desconhecido: {}", m);
            ("INTERNAL", "Erro interno no módulo operacional".to_string())
        }
    };
    tracing::warn!(target: "api", code = code, "operations: {:?}", e);
    erro_padrao(code, msg, None)
}

async fn executar<T, F>(f: F, contexto_erro: &'static str) -> Resposta<T>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, ErroOperacao> + Send + 'static,
{
    match tokio::task::spawn_blocking(f).await {
        Ok(Ok(v)) => Ok(AxJson(v)),
        Ok(Err(e)) => Err(map_erro_operacao(e)),
        Err(_) => Err(erro_padrao("INTERNAL", contexto_erro, None)),
    }
}

fn limite_sanitizado(bruto: Option<u32>, default: u32) -> u32 {
    match bruto {
        Some(0) | None => default,
        Some(n) if n > 200 => 200,
        Some(n) => n,
    }
}

/// `YYYY-MM-DD` (dia inteiro) ou RFC3339. `fim_do_dia` empurra a data pura
/// para 23:59:59 — assim `?de=2026-01-01&ate=2026-01-01` cobre o dia todo.
fn parse_data(bruto: &str, fim_do_dia: bool) -> Option<DateTime<Utc>> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(bruto) {
        return Some(dt.with_timezone(&Utc));
    }
    let d = NaiveDate::parse_from_str(bruto.trim(), "%Y-%m-%d").ok()?;
    let hora = if fim_do_dia {
        d.and_hms_opt(23, 59, 59)?
    } else {
        d.and_hms_opt(0, 0, 0)?
    };
    Some(Utc.from_utc_datetime(&hora))
}

// =============================================================================
// DTOs
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct AlertasQuery {
    pub severidade: Option<String>,
    pub limite: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct MoverPayload {
    /// ID da `WorkflowTransicao` a executar (resolve o estado destino).
    pub transicao_id: Option<u32>,
    /// Alternativa: slug do estado destino (`RECEBIDO`, `EM_EXECUCAO`, ...).
    pub novo_estado: Option<String>,
    pub motivo: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AgendaQuery {
    pub de: Option<String>,
    pub ate: Option<String>,
    pub tecnico_id: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct EventoAgendaPayload {
    pub tecnico_id: u32,
    pub os_id: Option<u32>,
    pub cliente_id: Option<u32>,
    pub titulo: String,
    pub descricao: Option<String>,
    pub tipo: Option<String>,
    /// RFC3339 ou `YYYY-MM-DD`.
    pub inicio: String,
    pub fim: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub endereco: Option<String>,
    #[serde(default)]
    pub permitir_conflito: bool,
}

#[derive(Debug, Deserialize)]
pub struct AtualizarEventoPayload {
    /// `AGENDADO | CONFIRMADO | EM_ANDAMENTO | CONCLUIDO | CANCELADO | FALTOU`
    pub status: String,
}

// =============================================================================
// Dashboard executivo
// =============================================================================

/// `GET /dashboard/executivo` — perm `os.view`. Fonte do dashboard do front.
pub async fn dashboard_executivo(claims: Claims) -> Resposta<DashboardExecutivo> {
    check_perm(&claims, "os.view")?;
    let tenant = auth::tenant_do_usuario(&claims);
    executar(
        move || service::dashboard_executivo(tenant),
        "Erro ao montar dashboard executivo",
    )
    .await
}

// =============================================================================
// Alertas
// =============================================================================

/// `GET /alertas?severidade=critical|warning|info&limite=50` — perm `os.view`.
pub async fn listar_alertas(claims: Claims, Query(q): Query<AlertasQuery>) -> Resposta<Vec<Alerta>> {
    check_perm(&claims, "os.view")?;
    let tenant = auth::tenant_do_usuario(&claims);
    let limite = limite_sanitizado(q.limite, 50) as usize;
    let filtro = q.severidade.map(|s| s.trim().to_uppercase());
    executar(
        move || {
            let mut alertas = service::listar_alertas_ativos(tenant)?;
            if let Some(sev) = &filtro {
                alertas.retain(|a| a.severidade.as_db_str() == sev.as_str());
            }
            alertas.truncate(limite);
            Ok(alertas)
        },
        "Erro ao listar alertas",
    )
    .await
}

/// `POST /alertas/{id}/resolver` — perm `os.edit`.
pub async fn resolver_alerta(claims: Claims, Path(id): Path<u32>) -> RespostaStatus {
    check_perm(&claims, "os.edit")?;
    let tenant = auth::tenant_do_usuario(&claims);
    match tokio::task::spawn_blocking(move || service::marcar_alerta_resolvido(tenant, id)).await {
        Ok(Ok(true)) => Ok(StatusCode::NO_CONTENT),
        Ok(Ok(false)) => Err(erro_padrao("NOT_FOUND", "Alerta não encontrado", None)),
        Ok(Err(e)) => Err(map_erro_operacao(e)),
        Err(_) => Err(erro_padrao("INTERNAL", "Erro ao resolver alerta", None)),
    }
}

// =============================================================================
// Workflow / Kanban
// =============================================================================

/// `GET /os/kanban` — perm `os.view`.
pub async fn kanban(claims: Claims) -> Resposta<Vec<KanbanColuna>> {
    check_perm(&claims, "os.view")?;
    let tenant = auth::tenant_do_usuario(&claims);
    executar(move || service::kanban(tenant), "Erro ao montar kanban").await
}

/// `GET /workflow/definicoes` — perm `os.view`. Lista os estados do workflow.
pub async fn listar_estados(claims: Claims) -> Resposta<Vec<WorkflowEstado>> {
    check_perm(&claims, "os.view")?;
    let tenant = auth::tenant_do_usuario(&claims);
    executar(
        move || service::listar_estados(tenant),
        "Erro ao listar estados do workflow",
    )
    .await
}

/// `POST /os/{id}/mover` — perm `os.edit`.
///
/// Aceita `transicao_id` (preferido pelo front, vem do kanban) ou
/// `novo_estado` (slug). O `transicao_id` é resolvido para o estado destino
/// **no servidor** — o cliente não escolhe o destino arbitrariamente.
pub async fn mover_estado(
    claims: Claims,
    Path(id): Path<u32>,
    Json(p): Json<MoverPayload>,
) -> Resposta<WorkflowMovimentacao> {
    check_perm(&claims, "os.edit")?;
    if p.transicao_id.is_none() && p.novo_estado.is_none() {
        return Err(erro_padrao(
            "VALIDATION",
            "informe 'transicao_id' ou 'novo_estado'",
            None,
        ));
    }
    let tenant = auth::tenant_do_usuario(&claims);
    let ctx = contexto(&claims);
    executar(
        move || {
            // Resolve o slug destino a partir da transição, quando aplicável.
            let slug = match (p.transicao_id, p.novo_estado.clone()) {
                (Some(tid), _) => {
                    let transicoes = service::listar_transicoes(tenant)?;
                    let t = transicoes
                        .iter()
                        .find(|t| t.id == tid)
                        .ok_or_else(|| ErroOperacao::EstadoInvalido(format!("transicao {}", tid)))?;
                    let estados = service::listar_estados(tenant)?;
                    estados
                        .iter()
                        .find(|e| e.id == t.estado_destino_id)
                        .map(|e| e.slug.clone())
                        .ok_or_else(|| {
                            ErroOperacao::EstadoInvalido(format!(
                                "estado destino {}",
                                t.estado_destino_id
                            ))
                        })?
                }
                (None, Some(slug)) => slug,
                (None, None) => unreachable!("validado acima"),
            };
            service::mover_estado(tenant, id, &slug, p.motivo.as_deref(), &ctx)
        },
        "Erro ao mover estado da OS",
    )
    .await
}

// =============================================================================
// SLA
// =============================================================================

/// `GET /sla/os/{id}` — perm `os.view`.
pub async fn sla_os(claims: Claims, Path(id): Path<u32>) -> Resposta<SlaCalculo> {
    check_perm(&claims, "os.view")?;
    let tenant = auth::tenant_do_usuario(&claims);
    executar(
        move || service::calcular_sla_os(tenant, id),
        "Erro ao calcular SLA",
    )
    .await
}

// =============================================================================
// Agenda
// =============================================================================

/// `GET /agenda?de=YYYY-MM-DD&ate=YYYY-MM-DD&tecnico_id=N` — perm `os.view`.
///
/// Sem `de`/`ate`, devolve a janela dos próximos 30 dias.
pub async fn listar_agenda(
    claims: Claims,
    Query(q): Query<AgendaQuery>,
) -> Resposta<Vec<EventoAgenda>> {
    check_perm(&claims, "os.view")?;
    let tenant = auth::tenant_do_usuario(&claims);
    let agora = Utc::now();
    let inicio = match q.de.as_deref() {
        Some(s) => parse_data(s, false)
            .ok_or_else(|| erro_padrao("VALIDATION", "parâmetro 'de' inválido", None))?,
        None => agora - Duration::days(1),
    };
    let fim = match q.ate.as_deref() {
        Some(s) => parse_data(s, true)
            .ok_or_else(|| erro_padrao("VALIDATION", "parâmetro 'ate' inválido", None))?,
        None => agora + Duration::days(30),
    };
    if fim < inicio {
        return Err(erro_padrao("VALIDATION", "'ate' deve ser >= 'de'", None));
    }
    let tecnico = q.tecnico_id;
    executar(
        move || service::listar_eventos_agenda(tenant, tecnico, inicio, fim),
        "Erro ao listar agenda",
    )
    .await
}

/// `POST /agenda` — perm `os.edit`.
pub async fn criar_evento_agenda(
    claims: Claims,
    Json(p): Json<EventoAgendaPayload>,
) -> Result<(StatusCode, AxJson<serde_json::Value>), (StatusCode, AxJson<ErroApi>)> {
    check_perm(&claims, "os.edit")?;
    let tenant = auth::tenant_do_usuario(&claims);
    let ctx = contexto(&claims);
    let inicio = parse_data(&p.inicio, false)
        .ok_or_else(|| erro_padrao("VALIDATION", "campo 'inicio' inválido", None))?;
    let fim = parse_data(&p.fim, false)
        .ok_or_else(|| erro_padrao("VALIDATION", "campo 'fim' inválido", None))?;
    let tipo = TipoEvento::from_db_str(
        &p.tipo
            .clone()
            .unwrap_or_else(|| "OUTRO".to_string())
            .to_uppercase(),
    );
    let resultado = tokio::task::spawn_blocking(move || {
        service::criar_evento_agenda(
            tenant,
            p.tecnico_id,
            p.os_id,
            p.cliente_id,
            &p.titulo,
            p.descricao.as_deref(),
            tipo,
            inicio,
            fim,
            p.latitude,
            p.longitude,
            p.endereco.as_deref(),
            &ctx,
            p.permitir_conflito,
        )
    })
    .await;
    match resultado {
        Ok(Ok(id)) => Ok((
            StatusCode::CREATED,
            AxJson(serde_json::json!({ "id": id })),
        )),
        Ok(Err(e)) => Err(map_erro_operacao(e)),
        Err(_) => Err(erro_padrao("INTERNAL", "Erro ao criar evento", None)),
    }
}

/// `GET /agenda/{id}` — perm `os.view`.
pub async fn obter_evento_agenda(claims: Claims, Path(id): Path<u32>) -> Resposta<EventoAgenda> {
    check_perm(&claims, "os.view")?;
    let tenant = auth::tenant_do_usuario(&claims);
    match tokio::task::spawn_blocking(move || service::obter_evento_agenda(tenant, id)).await {
        Ok(Ok(Some(ev))) => Ok(AxJson(ev)),
        Ok(Ok(None)) => Err(erro_padrao("NOT_FOUND", "Evento não encontrado", None)),
        Ok(Err(e)) => Err(map_erro_operacao(e)),
        Err(_) => Err(erro_padrao("INTERNAL", "Erro ao obter evento", None)),
    }
}

/// `PUT /agenda/{id}` — perm `os.edit`. Atualiza o status do evento.
pub async fn atualizar_evento_agenda(
    claims: Claims,
    Path(id): Path<u32>,
    Json(p): Json<AtualizarEventoPayload>,
) -> RespostaStatus {
    check_perm(&claims, "os.edit")?;
    let tenant = auth::tenant_do_usuario(&claims);
    let status = StatusEvento::from_db_str(&p.status.trim().to_uppercase());
    match tokio::task::spawn_blocking(move || service::atualizar_status_evento(tenant, id, status))
        .await
    {
        Ok(Ok(true)) => Ok(StatusCode::NO_CONTENT),
        Ok(Ok(false)) => Err(erro_padrao("NOT_FOUND", "Evento não encontrado", None)),
        Ok(Err(e)) => Err(map_erro_operacao(e)),
        Err(_) => Err(erro_padrao("INTERNAL", "Erro ao atualizar evento", None)),
    }
}

/// `DELETE /agenda/{id}` — perm `os.edit`.
pub async fn remover_evento_agenda(claims: Claims, Path(id): Path<u32>) -> RespostaStatus {
    check_perm(&claims, "os.edit")?;
    let tenant = auth::tenant_do_usuario(&claims);
    match tokio::task::spawn_blocking(move || service::remover_evento(tenant, id)).await {
        Ok(Ok(true)) => Ok(StatusCode::NO_CONTENT),
        Ok(Ok(false)) => Err(erro_padrao("NOT_FOUND", "Evento não encontrado", None)),
        Ok(Err(e)) => Err(map_erro_operacao(e)),
        Err(_) => Err(erro_padrao("INTERNAL", "Erro ao remover evento", None)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_data_aceita_iso_simples() {
        let d = parse_data("2026-01-15", false).expect("data válida");
        assert_eq!(d.format("%Y-%m-%d %H:%M:%S").to_string(), "2026-01-15 00:00:00");
    }

    #[test]
    fn parse_data_fim_do_dia() {
        let d = parse_data("2026-01-15", true).expect("data válida");
        assert_eq!(d.format("%H:%M:%S").to_string(), "23:59:59");
    }

    #[test]
    fn parse_data_aceita_rfc3339() {
        assert!(parse_data("2026-01-15T10:30:00Z", false).is_some());
    }

    #[test]
    fn parse_data_rejeita_lixo() {
        assert!(parse_data("ontem", false).is_none());
    }

    #[test]
    fn erro_transicao_invalida_e_409() {
        let (status, _) = map_erro_operacao(ErroOperacao::TransicaoInvalida(
            "RECEBIDO".into(),
            "ENTREGUE".into(),
        ));
        assert_eq!(status, StatusCode::CONFLICT);
    }

    #[test]
    fn erro_motivo_obrigatorio_e_400() {
        let (status, _) = map_erro_operacao(ErroOperacao::MotivoObrigatorio);
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn erro_os_nao_encontrada_e_404() {
        let (status, _) = map_erro_operacao(ErroOperacao::OsNaoEncontrada);
        assert_eq!(status, StatusCode::NOT_FOUND);
    }
}
