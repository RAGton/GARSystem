// src/operations/repository.rs
//
// Camada de acesso a dados — Sprint P2.4.
// Workflow, Kanban, Agenda, SLA, Alertas.

use super::models::{
    Agenda, Alerta, AlertaOcorrencia, EventoAgenda, KanbanCard, KanbanColuna, Severidade,
    SlaCalculo, SlaConfig, StatusEvento, TipoEvento, TipoSlaEvento, WorkflowDefinicao,
    WorkflowEstado, WorkflowMovimentacao, WorkflowTransicao,
};
use crate::banco_de_dados::obter_conexao;
use crate::servicos::ErroAplicacao;
use chrono::{DateTime, Utc};
use mysql::prelude::Queryable;
use mysql::{params, Row};

fn err<E: std::fmt::Display>(e: E) -> ErroAplicacao {
    ErroAplicacao::Desconhecido(e.to_string())
}

fn get_str(r: &Row, i: usize) -> Result<String, ErroAplicacao> {
    use mysql::Value;
    match r.get::<Value, _>(i) {
        Some(Value::Bytes(b)) => Ok(String::from_utf8_lossy(&b).into_owned()),
        Some(Value::NULL) | None => Ok(String::new()),
        _ => Err(err("tipo errado")),
    }
}

fn get_opt_str(r: &Row, i: usize) -> Result<Option<String>, ErroAplicacao> {
    use mysql::Value;
    match r.get::<Value, _>(i) {
        Some(Value::Bytes(b)) => Ok(Some(String::from_utf8_lossy(&b).into_owned())),
        Some(Value::NULL) | None => Ok(None),
        _ => Err(err("tipo errado")),
    }
}

fn get_i64(r: &Row, i: usize) -> Result<i64, ErroAplicacao> {
    use mysql::Value;
    match r.get::<Value, _>(i) {
        Some(Value::Int(n)) => Ok(n),
        Some(Value::UInt(n)) => Ok(n as i64),
        Some(Value::NULL) => Ok(0),
        _ => Err(err("tipo errado")),
    }
}

fn get_opt_i64(r: &Row, i: usize) -> Result<Option<i64>, ErroAplicacao> {
    use mysql::Value;
    match r.get::<Value, _>(i) {
        Some(Value::Int(n)) => Ok(Some(n)),
        Some(Value::UInt(n)) => Ok(Some(n as i64)),
        Some(Value::NULL) | None => Ok(None),
        _ => Err(err("tipo errado")),
    }
}

fn get_bool(r: &Row, i: usize) -> Result<bool, ErroAplicacao> {
    use mysql::Value;
    match r.get::<Value, _>(i) {
        Some(Value::Int(n)) => Ok(n != 0),
        Some(Value::UInt(n)) => Ok(n != 0),
        Some(Value::Bytes(b)) => Ok(!b.is_empty() && b[0] != b'0'),
        Some(Value::NULL) | None => Ok(false),
        _ => Err(err("tipo errado")),
    }
}

fn get_opt_dt(r: &Row, i: usize) -> Result<Option<DateTime<Utc>>, ErroAplicacao> {
    use mysql::Value;
    match r.get::<Value, _>(i) {
        Some(Value::Date(y, m, d, h, mi, s, _us)) => {
            use chrono::NaiveDate;
            let nd = NaiveDate::from_ymd_opt(y as i32, m as u32, d as u32).unwrap();
            let nt = nd.and_hms_opt(h as u32, mi as u32, s as u32).unwrap();
            Ok(Some(DateTime::<Utc>::from_naive_utc_and_offset(nt, Utc)))
        }
        Some(Value::NULL) | None => Ok(None),
        _ => Err(err("tipo errado")),
    }
}

fn get_dt(r: &Row, i: usize) -> Result<DateTime<Utc>, ErroAplicacao> {
    Ok(get_opt_dt(r, i)?.unwrap_or_else(Utc::now))
}

fn get_opt_f64(r: &Row, i: usize) -> Result<Option<f64>, ErroAplicacao> {
    use mysql::Value;
    match r.get::<Value, _>(i) {
        Some(Value::Float(f)) => Ok(Some(f as f64)),
        Some(Value::Double(f)) => Ok(Some(f)),
        Some(Value::NULL) | None => Ok(None),
        _ => Err(err("tipo errado")),
    }
}

// =============================================================================
// Workflow Engine
// =============================================================================

pub fn obter_workflow_default(tenant_id: i32) -> Result<Option<WorkflowDefinicao>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<Row> = conn
        .exec_first(
            "SELECT id, nome, descricao, entidade_tipo, ativo, data_criacao, usuario_criacao_id
             FROM workflow_definicoes WHERE tenant_id = ? AND ativo = TRUE ORDER BY id LIMIT 1",
            (tenant_id,),
        )
        .map_err(err)?;
    match row {
        Some(r) => Ok(Some(WorkflowDefinicao {
            id: get_i64(&r, 0)? as u32,
            nome: get_str(&r, 1)?,
            descricao: get_opt_str(&r, 2)?,
            entidade_tipo: get_str(&r, 3)?,
            ativo: get_bool(&r, 4)?,
            data_criacao: get_dt(&r, 5)?,
            usuario_criacao_id: get_i64(&r, 6)? as u32,
        })),
        None => Ok(None),
    }
}

pub fn listar_estados(
    tenant_id: i32,
    workflow_id: u32,
) -> Result<Vec<WorkflowEstado>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let rows: Vec<Row> = conn
        .exec(
            r#"SELECT id, workflow_id, nome, slug, descricao, cor, ordem,
                      eh_inicial, eh_final, requer_responsavel, bloqueia_sla
               FROM workflow_estados WHERE tenant_id = ? AND workflow_id = ? ORDER BY ordem"#,
            (tenant_id, workflow_id),
        )
        .map_err(err)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(WorkflowEstado {
            id: get_i64(&r, 0)? as u32,
            workflow_id: get_i64(&r, 1)? as u32,
            nome: get_str(&r, 2)?,
            slug: get_str(&r, 3)?,
            descricao: get_opt_str(&r, 4)?,
            cor: get_opt_str(&r, 5)?,
            ordem: get_i64(&r, 6)? as u32,
            eh_inicial: get_bool(&r, 7)?,
            eh_final: get_bool(&r, 8)?,
            requer_responsavel: get_bool(&r, 9)?,
            bloqueia_sla: get_bool(&r, 10)?,
        });
    }
    Ok(out)
}

pub fn listar_transicoes(
    tenant_id: i32,
    workflow_id: u32,
) -> Result<Vec<WorkflowTransicao>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let rows: Vec<Row> = conn
        .exec(
            r#"SELECT id, workflow_id, estado_origem_id, estado_destino_id,
                      requer_papel, exige_motivo, exige_arquivo
               FROM workflow_transicoes WHERE tenant_id = ? AND workflow_id = ?"#,
            (tenant_id, workflow_id),
        )
        .map_err(err)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(WorkflowTransicao {
            id: get_i64(&r, 0)? as u32,
            workflow_id: get_i64(&r, 1)? as u32,
            estado_origem_id: get_i64(&r, 2)? as u32,
            estado_destino_id: get_i64(&r, 3)? as u32,
            requer_papel: get_opt_str(&r, 4)?,
            exige_motivo: get_bool(&r, 5)?,
            exige_arquivo: get_bool(&r, 6)?,
        });
    }
    Ok(out)
}

pub fn obter_estado_por_slug(
    tenant_id: i32,
    workflow_id: u32,
    slug: &str,
) -> Result<Option<WorkflowEstado>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<Row> = conn
        .exec_first(
            r#"SELECT id, workflow_id, nome, slug, descricao, cor, ordem,
                      eh_inicial, eh_final, requer_responsavel, bloqueia_sla
               FROM workflow_estados WHERE tenant_id = ? AND workflow_id = ? AND slug = ?"#,
            (tenant_id, workflow_id, slug),
        )
        .map_err(err)?;
    match row {
        Some(r) => Ok(Some(WorkflowEstado {
            id: get_i64(&r, 0)? as u32,
            workflow_id: get_i64(&r, 1)? as u32,
            nome: get_str(&r, 2)?,
            slug: get_str(&r, 3)?,
            descricao: get_opt_str(&r, 4)?,
            cor: get_opt_str(&r, 5)?,
            ordem: get_i64(&r, 6)? as u32,
            eh_inicial: get_bool(&r, 7)?,
            eh_final: get_bool(&r, 8)?,
            requer_responsavel: get_bool(&r, 9)?,
            bloqueia_sla: get_bool(&r, 10)?,
        })),
        None => Ok(None),
    }
}

pub fn obter_estado(tenant_id: i32, id: u32) -> Result<Option<WorkflowEstado>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<Row> = conn
        .exec_first(
            r#"SELECT id, workflow_id, nome, slug, descricao, cor, ordem,
                      eh_inicial, eh_final, requer_responsavel, bloqueia_sla
               FROM workflow_estados WHERE tenant_id = ? AND id = ?"#,
            (tenant_id, id),
        )
        .map_err(err)?;
    match row {
        Some(r) => Ok(Some(WorkflowEstado {
            id: get_i64(&r, 0)? as u32,
            workflow_id: get_i64(&r, 1)? as u32,
            nome: get_str(&r, 2)?,
            slug: get_str(&r, 3)?,
            descricao: get_opt_str(&r, 4)?,
            cor: get_opt_str(&r, 5)?,
            ordem: get_i64(&r, 6)? as u32,
            eh_inicial: get_bool(&r, 7)?,
            eh_final: get_bool(&r, 8)?,
            requer_responsavel: get_bool(&r, 9)?,
            bloqueia_sla: get_bool(&r, 10)?,
        })),
        None => Ok(None),
    }
}

pub fn registrar_movimentacao(
    tenant_id: i32,
    workflow_id: u32,
    entidade_id: u32,
    estado_origem_id: Option<u32>,
    estado_destino_id: u32,
    usuario_id: Option<u32>,
    username: Option<&str>,
    motivo: Option<&str>,
    duracao_anterior_seg: Option<u32>,
    ip: Option<&str>,
) -> Result<u32, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        r#"INSERT INTO workflow_movimentacoes
            (workflow_id, entidade_id, estado_origem_id, estado_destino_id,
             usuario_id, username, motivo, duracao_no_estado_anterior_segundos, ip_origem, tenant_id)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        (
            workflow_id,
            entidade_id,
            estado_origem_id,
            estado_destino_id,
            usuario_id,
            username,
            motivo,
            duracao_anterior_seg,
            ip,
            tenant_id,
        ),
    )
    .map_err(err)?;
    Ok(conn.last_insert_id() as u32)
}

pub fn listar_movimentacoes(
    tenant_id: i32,
    workflow_id: u32,
    entidade_id: u32,
    limite: u32,
) -> Result<Vec<WorkflowMovimentacao>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let rows: Vec<Row> = conn
        .exec(
            r#"SELECT id, workflow_id, entidade_id, estado_origem_id, estado_destino_id,
                      usuario_id, username, motivo, data_movimentacao,
                      duracao_no_estado_anterior_segundos, ip_origem
               FROM workflow_movimentacoes
               WHERE tenant_id = ? AND workflow_id = ? AND entidade_id = ?
               ORDER BY data_movimentacao DESC LIMIT ?"#,
            (tenant_id, workflow_id, entidade_id, limite),
        )
        .map_err(err)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(WorkflowMovimentacao {
            id: get_i64(&r, 0)? as u32,
            workflow_id: get_i64(&r, 1)? as u32,
            entidade_id: get_i64(&r, 2)? as u32,
            estado_origem_id: get_opt_i64(&r, 3)?.map(|n| n as u32),
            estado_destino_id: get_i64(&r, 4)? as u32,
            usuario_id: get_opt_i64(&r, 5)?.map(|n| n as u32),
            username: get_opt_str(&r, 6)?,
            motivo: get_opt_str(&r, 7)?,
            data_movimentacao: get_dt(&r, 8)?,
            duracao_no_estado_anterior_segundos: get_opt_i64(&r, 9)?.map(|n| n as u32),
            ip_origem: get_opt_str(&r, 10)?,
        });
    }
    Ok(out)
}

pub fn ultima_movimentacao(
    tenant_id: i32,
    workflow_id: u32,
    entidade_id: u32,
) -> Result<Option<WorkflowMovimentacao>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<Row> = conn
        .exec_first(
            r#"SELECT id, workflow_id, entidade_id, estado_origem_id, estado_destino_id,
                      usuario_id, username, motivo, data_movimentacao,
                      duracao_no_estado_anterior_segundos, ip_origem
               FROM workflow_movimentacoes
               WHERE tenant_id = ? AND workflow_id = ? AND entidade_id = ?
               ORDER BY data_movimentacao DESC LIMIT 1"#,
            (tenant_id, workflow_id, entidade_id),
        )
        .map_err(err)?;
    match row {
        Some(r) => Ok(Some(WorkflowMovimentacao {
            id: get_i64(&r, 0)? as u32,
            workflow_id: get_i64(&r, 1)? as u32,
            entidade_id: get_i64(&r, 2)? as u32,
            estado_origem_id: get_opt_i64(&r, 3)?.map(|n| n as u32),
            estado_destino_id: get_i64(&r, 4)? as u32,
            usuario_id: get_opt_i64(&r, 5)?.map(|n| n as u32),
            username: get_opt_str(&r, 6)?,
            motivo: get_opt_str(&r, 7)?,
            data_movimentacao: get_dt(&r, 8)?,
            duracao_no_estado_anterior_segundos: get_opt_i64(&r, 9)?.map(|n| n as u32),
            ip_origem: get_opt_str(&r, 10)?,
        })),
        None => Ok(None),
    }
}

// Kanban: agrupa OS por estado (lê direto de ordens_servico)
pub fn kanban_por_estado(
    tenant_id: i32,
    workflow_id: u32,
) -> Result<Vec<KanbanColuna>, ErroAplicacao> {
    use std::collections::HashMap;
    let estados = listar_estados(tenant_id, workflow_id)?;
    let transicoes = listar_transicoes(tenant_id, workflow_id)?;
    let mut colunas: HashMap<u32, KanbanColuna> = HashMap::new();
    for e in &estados {
        let transicoes_validas: Vec<u32> = transicoes
            .iter()
            .filter(|t| t.estado_origem_id == e.id)
            .map(|t| t.id)
            .collect();
        colunas.insert(
            e.id,
            KanbanColuna {
                estado: e.clone(),
                transicoes_validas,
                cards: Vec::new(),
            },
        );
    }

    let mut conn = obter_conexao()?;
    let rows: Vec<Row> = conn
        .exec(
            r#"SELECT o.id, COALESCE(c.nome, ''), o.descricao, o.tecnico_id, u.username,
                      o.ultima_movimentacao_data
               FROM ordens_servico o
               LEFT JOIN clientes c ON c.id = o.cliente_id AND c.tenant_id = o.tenant_id
               LEFT JOIN usuarios u ON u.id = o.tecnico_id
               WHERE o.tenant_id = ? AND o.status NOT IN ('FINALIZADA','CANCELADA','ENTREGUE')
                 AND o.workflow_estado_id IS NOT NULL"#,
            (tenant_id,),
        )
        .map_err(err)?;

    for r in rows {
        let card = KanbanCard {
            os_id: get_i64(&r, 0)? as u32,
            cliente_nome: get_str(&r, 1)?,
            descricao: get_opt_str(&r, 2)?,
            responsavel_id: get_opt_i64(&r, 3)?.map(|n| n as u32),
            responsavel_nome: get_opt_str(&r, 4)?,
            data_entrada: get_opt_dt(&r, 5)?.unwrap_or_else(Utc::now),
            prioridade: None,
            bloqueia_sla: false,
        };
        // Mapeia por nome do status (compat)
        // O Kanban usa o workflow_estado_id quando setado;
        // caso contrário, mapeia por nome do status antigo.
        let status_nome = get_str(&r, 1)?; // placeholder
        let _ = status_nome;
        // Tenta mapear pelo status atual da OS
        let os_status: String = conn
            .exec_first::<String, _, _>(
                "SELECT status FROM ordens_servico WHERE tenant_id = ? AND id = ?",
                (tenant_id, get_i64(&r, 0)? as u32),
            )
            .ok()
            .flatten()
            .unwrap_or_default();
        if let Some(estado) = estados
            .iter()
            .find(|e| slug_matches_legacy(&e.slug, &os_status))
        {
            if let Some(c) = colunas.get_mut(&estado.id) {
                c.cards.push(card);
            }
        }
    }
    Ok(colunas.into_values().collect())
}

fn slug_matches_legacy(slug: &str, os_status: &str) -> bool {
    let s = os_status.to_uppercase().replace('-', "_");
    if s == slug {
        return true;
    }
    // Mapeamentos aproximados
    match slug {
        "RECEBIDO" => s == "ABERTA" || s == "RECEBIDO",
        "DIAGNOSTICO" => s == "EM_DIAGNOSTICO" || s == "DIAGNOSTICO",
        "AGUARDANDO_PECA" => s.contains("AGUARDANDO_PEC"),
        "ORCAMENTO" => s == "ORCAMENTO" || s == "EM_ORCAMENTO",
        "AGUARDANDO_APROVACAO" => s.contains("AGUARDANDO_APROV"),
        "APROVADO" => s == "APROVADO",
        "EXECUCAO" => s == "EM_ANDAMENTO" || s == "EM_EXECUCAO" || s == "EXECUCAO",
        "TESTE" => s == "EM_TESTE" || s == "TESTE",
        "FINALIZADO" => s == "FINALIZADA" || s == "FINALIZADO",
        "ENTREGUE" => s == "ENTREGUE",
        _ => false,
    }
}

// =============================================================================
// Agenda
// =============================================================================

pub fn criar_evento_agenda(
    tenant_id: i32,
    agenda_id: u32,
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
    usuario_criacao_id: u32,
) -> Result<u32, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let ini = inicio.format("%Y-%m-%d %H:%M:%S").to_string();
    let fim_s = fim.format("%Y-%m-%d %H:%M:%S").to_string();
    let tipo_str = tipo.as_db_str();
    let sql = r#"INSERT INTO eventos_agenda
        (agenda_id, tecnico_id, os_id, cliente_id, titulo, descricao, tipo,
         status, inicio, fim, latitude, longitude, endereco, usuario_criacao_id, tenant_id)
       VALUES (:a, :b, :c, :d, :e, :f, :g, 'AGENDADO', :h, :i, :j, :k, :l, :m, :tid)"#;
    conn.exec_drop(
        sql,
        params! {
            "a" => agenda_id,
            "b" => tecnico_id,
            "c" => os_id,
            "d" => cliente_id,
            "e" => titulo,
            "f" => descricao,
            "g" => tipo_str,
            "h" => ini,
            "i" => fim_s,
            "j" => latitude,
            "k" => longitude,
            "l" => endereco,
            "m" => usuario_criacao_id,
            "tid" => tenant_id,
        },
    )
    .map_err(err)?;
    Ok(conn.last_insert_id() as u32)
}

pub fn obter_evento_agenda(tenant_id: i32, id: u32) -> Result<Option<EventoAgenda>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<Row> = conn
        .exec_first(
            r#"SELECT id, agenda_id, tecnico_id, os_id, cliente_id, titulo, descricao,
                      tipo, status, inicio, fim, latitude, longitude, endereco,
                      usuario_criacao_id, data_criacao, data_atualizacao
               FROM eventos_agenda WHERE tenant_id = ? AND id = ?"#,
            (tenant_id, id),
        )
        .map_err(err)?;
    match row {
        Some(r) => Ok(Some(row_para_evento(r)?)),
        None => Ok(None),
    }
}

pub fn listar_eventos_periodo(
    tenant_id: i32,
    tecnico_id: Option<u32>,
    inicio: DateTime<Utc>,
    fim: DateTime<Utc>,
) -> Result<Vec<EventoAgenda>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let ini = inicio.format("%Y-%m-%d %H:%M:%S").to_string();
    let fim_s = fim.format("%Y-%m-%d %H:%M:%S").to_string();
    let rows: Vec<Row> = if let Some(t) = tecnico_id {
        conn.exec(
            r#"SELECT id, agenda_id, tecnico_id, os_id, cliente_id, titulo, descricao,
                      tipo, status, inicio, fim, latitude, longitude, endereco,
                      usuario_criacao_id, data_criacao, data_atualizacao
               FROM eventos_agenda
               WHERE tenant_id = ? AND tecnico_id = ? AND inicio < ? AND fim > ?
               ORDER BY inicio"#,
            (tenant_id, t, fim_s, ini),
        )
        .map_err(err)?
    } else {
        conn.exec(
            r#"SELECT id, agenda_id, tecnico_id, os_id, cliente_id, titulo, descricao,
                      tipo, status, inicio, fim, latitude, longitude, endereco,
                      usuario_criacao_id, data_criacao, data_atualizacao
               FROM eventos_agenda
               WHERE tenant_id = ? AND inicio < ? AND fim > ?
               ORDER BY inicio"#,
            (tenant_id, fim_s, ini),
        )
        .map_err(err)?
    };
    let mut out = Vec::new();
    for r in rows {
        out.push(row_para_evento(r)?);
    }
    Ok(out)
}

pub fn atualizar_status_evento(
    tenant_id: i32,
    id: u32,
    status: StatusEvento,
) -> Result<bool, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        "UPDATE eventos_agenda SET status = ? WHERE tenant_id = ? AND id = ?",
        (status.as_db_str(), tenant_id, id),
    )
    .map_err(err)?;
    Ok(conn.affected_rows() > 0)
}

pub fn remover_evento(tenant_id: i32, id: u32) -> Result<bool, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        "DELETE FROM eventos_agenda WHERE tenant_id = ? AND id = ?",
        (tenant_id, id),
    )
    .map_err(err)?;
    Ok(conn.affected_rows() > 0)
}

pub fn garantir_agenda_tecnico(tenant_id: i32, tecnico_id: u32) -> Result<u32, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let existente: Option<u32> = conn
        .exec_first(
            "SELECT id FROM agendas WHERE tenant_id = ? AND tecnico_id = ?",
            (tenant_id, tecnico_id),
        )
        .map_err(err)?
        .flatten();
    if let Some(id) = existente {
        return Ok(id);
    }
    conn.exec_drop(
        "INSERT INTO agendas (nome, tecnico_id, tenant_id) VALUES (?, ?, ?)",
        (format!("Agenda #{}", tecnico_id), tecnico_id, tenant_id),
    )
    .map_err(err)?;
    Ok(conn.last_insert_id() as u32)
}

pub fn obter_agenda(tenant_id: i32, id: u32) -> Result<Option<Agenda>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<Row> = conn
        .exec_first(
            "SELECT id, nome, tecnico_id, cor, ativo, data_criacao FROM agendas WHERE tenant_id = ? AND id = ?",
            (tenant_id, id),
        )
        .map_err(err)?;
    match row {
        Some(r) => Ok(Some(Agenda {
            id: get_i64(&r, 0)? as u32,
            nome: get_str(&r, 1)?,
            tecnico_id: get_opt_i64(&r, 2)?.map(|n| n as u32),
            cor: get_opt_str(&r, 3)?,
            ativo: get_bool(&r, 4)?,
            data_criacao: get_dt(&r, 5)?,
        })),
        None => Ok(None),
    }
}

fn row_para_evento(r: Row) -> Result<EventoAgenda, ErroAplicacao> {
    Ok(EventoAgenda {
        id: get_i64(&r, 0)? as u32,
        agenda_id: get_i64(&r, 1)? as u32,
        tecnico_id: get_i64(&r, 2)? as u32,
        os_id: get_opt_i64(&r, 3)?.map(|n| n as u32),
        cliente_id: get_opt_i64(&r, 4)?.map(|n| n as u32),
        titulo: get_str(&r, 5)?,
        descricao: get_opt_str(&r, 6)?,
        tipo: TipoEvento::from_db_str(&get_str(&r, 7)?),
        status: StatusEvento::from_db_str(&get_str(&r, 8)?),
        inicio: get_dt(&r, 9)?,
        fim: get_dt(&r, 10)?,
        latitude: get_opt_f64(&r, 11)?,
        longitude: get_opt_f64(&r, 12)?,
        endereco: get_opt_str(&r, 13)?,
        usuario_criacao_id: get_i64(&r, 14)? as u32,
        data_criacao: get_dt(&r, 15)?,
        data_atualizacao: get_dt(&r, 16)?,
    })
}

// =============================================================================
// SLA
// =============================================================================

pub fn obter_sla_default(tenant_id: i32) -> Result<Option<SlaConfig>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<Row> = conn
        .exec_first(
            r#"SELECT id, nome, descricao, ativo,
                      max_horas_diagnostico, max_horas_execucao, max_horas_total,
                      max_horas_espera_peca, max_horas_espera_aprovacao, data_criacao
               FROM sla_config WHERE tenant_id = ? AND ativo = TRUE ORDER BY id LIMIT 1"#,
            (tenant_id,),
        )
        .map_err(err)?;
    match row {
        Some(r) => Ok(Some(SlaConfig {
            id: get_i64(&r, 0)? as u32,
            nome: get_str(&r, 1)?,
            descricao: get_opt_str(&r, 2)?,
            ativo: get_bool(&r, 3)?,
            max_horas_diagnostico: get_opt_i64(&r, 4)?.map(|n| n as u32),
            max_horas_execucao: get_opt_i64(&r, 5)?.map(|n| n as u32),
            max_horas_total: get_opt_i64(&r, 6)?.map(|n| n as u32),
            max_horas_espera_peca: get_opt_i64(&r, 7)?.map(|n| n as u32),
            max_horas_espera_aprovacao: get_opt_i64(&r, 8)?.map(|n| n as u32),
            data_criacao: get_dt(&r, 9)?,
        })),
        None => Ok(None),
    }
}

pub fn registrar_evento_sla(
    tenant_id: i32,
    os_id: u32,
    tipo: TipoSlaEvento,
    data_evento: DateTime<Utc>,
    usuario_id: Option<u32>,
    observacao: Option<&str>,
) -> Result<u32, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let de = data_evento.format("%Y-%m-%d %H:%M:%S").to_string();
    conn.exec_drop(
        r#"INSERT INTO sla_eventos (os_id, tipo, data_evento, usuario_id, observacao, tenant_id)
           VALUES (?, ?, ?, ?, ?, ?)"#,
        (
            os_id,
            tipo.as_db_str(),
            de,
            usuario_id,
            observacao,
            tenant_id,
        ),
    )
    .map_err(err)?;
    Ok(conn.last_insert_id() as u32)
}

pub fn listar_eventos_sla(
    tenant_id: i32,
    os_id: u32,
) -> Result<Vec<(TipoSlaEvento, DateTime<Utc>)>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let rows: Vec<Row> = conn
        .exec(
            "SELECT tipo, data_evento FROM sla_eventos WHERE tenant_id = ? AND os_id = ? ORDER BY data_evento",
            (tenant_id, os_id),
        )
        .map_err(err)?;
    let mut out = Vec::new();
    for r in rows {
        out.push((TipoSlaEvento::from_db_str(&get_str(&r, 0)?), get_dt(&r, 1)?));
    }
    Ok(out)
}

pub fn upsert_sla_calculo(
    tenant_id: i32,
    os_id: u32,
    sla_config_id: Option<u32>,
    diag_seg: Option<u32>,
    peca_seg: Option<u32>,
    aprov_seg: Option<u32>,
    exec_seg: Option<u32>,
    total_seg: Option<u32>,
    violado: bool,
    motivo: Option<&str>,
) -> Result<(), ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        r#"INSERT INTO sla_calculos
            (os_id, sla_config_id, tempo_diagnostico_segundos, tempo_espera_peca_segundos,
             tempo_espera_aprovacao_segundos, tempo_execucao_segundos, tempo_total_segundos,
             sla_violado, motivo_violacao, tenant_id)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
           ON DUPLICATE KEY UPDATE
             sla_config_id = VALUES(sla_config_id),
             tempo_diagnostico_segundos = VALUES(tempo_diagnostico_segundos),
             tempo_espera_peca_segundos = VALUES(tempo_espera_peca_segundos),
             tempo_espera_aprovacao_segundos = VALUES(tempo_espera_aprovacao_segundos),
             tempo_execucao_segundos = VALUES(tempo_execucao_segundos),
             tempo_total_segundos = VALUES(tempo_total_segundos),
             sla_violado = VALUES(sla_violado),
             motivo_violacao = VALUES(motivo_violacao),
             data_calculo = NOW(),
             tenant_id = VALUES(tenant_id)"#,
        (
            os_id,
            sla_config_id,
            diag_seg,
            peca_seg,
            aprov_seg,
            exec_seg,
            total_seg,
            violado,
            motivo,
            tenant_id,
        ),
    )
    .map_err(err)?;
    Ok(())
}

pub fn obter_sla_calculo(tenant_id: i32, os_id: u32) -> Result<Option<SlaCalculo>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<Row> = conn
        .exec_first(
            r#"SELECT id, os_id, sla_config_id, tempo_diagnostico_segundos, tempo_espera_peca_segundos,
                      tempo_espera_aprovacao_segundos, tempo_execucao_segundos, tempo_total_segundos,
                      sla_violado, motivo_violacao, data_calculo
               FROM sla_calculos WHERE tenant_id = ? AND os_id = ?"#,
            (tenant_id, os_id),
        )
        .map_err(err)?;
    match row {
        Some(r) => Ok(Some(SlaCalculo {
            id: get_i64(&r, 0)? as u32,
            os_id: get_i64(&r, 1)? as u32,
            sla_config_id: get_opt_i64(&r, 2)?.map(|n| n as u32),
            tempo_diagnostico_segundos: get_opt_i64(&r, 3)?.map(|n| n as u32),
            tempo_espera_peca_segundos: get_opt_i64(&r, 4)?.map(|n| n as u32),
            tempo_espera_aprovacao_segundos: get_opt_i64(&r, 5)?.map(|n| n as u32),
            tempo_execucao_segundos: get_opt_i64(&r, 6)?.map(|n| n as u32),
            tempo_total_segundos: get_opt_i64(&r, 7)?.map(|n| n as u32),
            sla_violado: get_bool(&r, 8)?,
            motivo_violacao: get_opt_str(&r, 9)?,
            data_calculo: get_dt(&r, 10)?,
        })),
        None => Ok(None),
    }
}

// =============================================================================
// Alertas
// =============================================================================

pub fn listar_alertas_ativos(tenant_id: i32) -> Result<Vec<Alerta>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let rows: Vec<Row> = conn
        .exec(
            r#"SELECT id, codigo, titulo, descricao, severidade, ativo, condicao_tipo,
                      condicao_threshold_horas, condicao_estado_slug, entidade_tipo, data_criacao
               FROM alertas WHERE tenant_id = ? AND ativo = TRUE ORDER BY id"#,
            (tenant_id,),
        )
        .map_err(err)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(Alerta {
            id: get_i64(&r, 0)? as u32,
            codigo: get_str(&r, 1)?,
            titulo: get_str(&r, 2)?,
            descricao: get_opt_str(&r, 3)?,
            severidade: Severidade::from_db_str(&get_str(&r, 4)?),
            ativo: get_bool(&r, 5)?,
            condicao_tipo: get_str(&r, 6)?,
            condicao_threshold_horas: get_opt_i64(&r, 7)?.map(|n| n as u32),
            condicao_estado_slug: get_opt_str(&r, 8)?,
            entidade_tipo: get_str(&r, 9)?,
            data_criacao: get_dt(&r, 10)?,
        });
    }
    Ok(out)
}

pub fn inserir_ocorrencia(
    tenant_id: i32,
    alerta_id: u32,
    entidade_id: u32,
    severidade: Severidade,
    mensagem: &str,
    contexto: Option<&serde_json::Value>,
) -> Result<u32, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let ctx_str = contexto.map(|c| c.to_string());
    conn.exec_drop(
        r#"INSERT INTO alertas_ocorrencias
            (alerta_id, entidade_id, severidade, mensagem, contexto_json, tenant_id)
           VALUES (?, ?, ?, ?, ?, ?)"#,
        (
            alerta_id,
            entidade_id,
            severidade.as_db_str(),
            mensagem,
            ctx_str.as_deref(),
            tenant_id,
        ),
    )
    .map_err(err)?;
    Ok(conn.last_insert_id() as u32)
}

pub fn listar_ocorrencias_pendentes(
    tenant_id: i32,
    limite: u32,
) -> Result<Vec<AlertaOcorrencia>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let rows: Vec<Row> = conn
        .exec(
            r#"SELECT id, alerta_id, entidade_id, data_ocorrencia, severidade, mensagem,
                      contexto_json, visualizado, data_visualizacao, usuario_visualizacao_id,
                      resolvido, data_resolucao
               FROM alertas_ocorrencias
               WHERE tenant_id = ? AND resolvido = FALSE
               ORDER BY FIELD(severidade,'CRITICAL','WARNING','INFO'), data_ocorrencia DESC
               LIMIT ?"#,
            (tenant_id, limite),
        )
        .map_err(err)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(row_para_ocorrencia(r)?);
    }
    Ok(out)
}

pub fn listar_ocorrencias_por_entidade(
    tenant_id: i32,
    entidade_id: u32,
) -> Result<Vec<AlertaOcorrencia>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let rows: Vec<Row> = conn
        .exec(
            r#"SELECT id, alerta_id, entidade_id, data_ocorrencia, severidade, mensagem,
                      contexto_json, visualizado, data_visualizacao, usuario_visualizacao_id,
                      resolvido, data_resolucao
               FROM alertas_ocorrencias
               WHERE tenant_id = ? AND entidade_id = ?
               ORDER BY data_ocorrencia DESC"#,
            (tenant_id, entidade_id),
        )
        .map_err(err)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(row_para_ocorrencia(r)?);
    }
    Ok(out)
}

pub fn marcar_visualizado(tenant_id: i32, id: u32, usuario_id: u32) -> Result<bool, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        "UPDATE alertas_ocorrencias SET visualizado = TRUE, data_visualizacao = NOW(), usuario_visualizacao_id = ? WHERE tenant_id = ? AND id = ?",
        (usuario_id, id, tenant_id),
    )
    .map_err(err)?;
    Ok(conn.affected_rows() > 0)
}

pub fn marcar_resolvido(tenant_id: i32, id: u32) -> Result<bool, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        "UPDATE alertas_ocorrencias SET resolvido = TRUE, data_resolucao = NOW() WHERE tenant_id = ? AND id = ?",
        (tenant_id, id),
    )
    .map_err(err)?;
    Ok(conn.affected_rows() > 0)
}

fn row_para_ocorrencia(r: Row) -> Result<AlertaOcorrencia, ErroAplicacao> {
    use mysql::Value;
    let ctx: Option<serde_json::Value> = match r.get::<Value, _>(6) {
        Some(Value::Bytes(b)) => {
            let s = String::from_utf8_lossy(&b);
            serde_json::from_str(&s).ok()
        }
        _ => None,
    };
    Ok(AlertaOcorrencia {
        id: get_i64(&r, 0)? as u32,
        alerta_id: get_i64(&r, 1)? as u32,
        entidade_id: get_i64(&r, 2)? as u32,
        data_ocorrencia: get_dt(&r, 3)?,
        severidade: Severidade::from_db_str(&get_str(&r, 4)?),
        mensagem: get_str(&r, 5)?,
        contexto_json: ctx,
        visualizado: get_bool(&r, 7)?,
        data_visualizacao: get_opt_dt(&r, 8)?,
        usuario_visualizacao_id: get_opt_i64(&r, 9)?.map(|n| n as u32),
        resolvido: get_bool(&r, 10)?,
        data_resolucao: get_opt_dt(&r, 11)?,
    })
}
