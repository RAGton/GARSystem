// src/os_mobile/repository.rs
//
// Camada de acesso a dados — Sprint P2.3.
// Operações CRUD para checklist, evoluções, assinaturas, auditoria offline.

use super::models::{
    Assinatura, AuditoriaCampo, Checklist, ChecklistCompleto, ChecklistItem, ChecklistTemplate,
    ChecklistTemplateCompleto, ChecklistTemplateItem, Evolucao, OsResumo, TipoAcaoAuditoria,
    TipoEvolucao,
};
use crate::banco_de_dados::obter_conexao;
use crate::servicos::ErroAplicacao;
use chrono::{DateTime, Utc};
use mysql::prelude::Queryable;
use mysql::Row;

fn err<E: std::fmt::Display>(e: E) -> ErroAplicacao {
    ErroAplicacao::Desconhecido(e.to_string())
}

// =============================================================================
// Helpers
// =============================================================================

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

fn get_f64(r: &Row, i: usize) -> Result<f64, ErroAplicacao> {
    use mysql::Value;
    match r.get::<Value, _>(i) {
        Some(Value::Float(f)) => Ok(f as f64),
        Some(Value::Double(f)) => Ok(f),
        Some(Value::NULL) => Ok(0.0),
        _ => Err(err("tipo errado")),
    }
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

fn get_bool(r: &Row, i: usize) -> Result<bool, ErroAplicacao> {
    use mysql::Value;
    match r.get::<Value, _>(i) {
        Some(Value::Int(n)) => Ok(n != 0),
        Some(Value::UInt(n)) => Ok(n != 0),
        Some(Value::Bytes(b)) => {
            // MySQL TINYINT(1) pode vir como '0'/'1' ou 0/1
            if b.is_empty() {
                Ok(false)
            } else {
                Ok(b[0] != b'0')
            }
        }
        Some(Value::NULL) | None => Ok(false),
        _ => Err(err("tipo errado")),
    }
}

fn get_dt(r: &Row, i: usize) -> Result<DateTime<Utc>, ErroAplicacao> {
    use mysql::Value;
    match r.get::<Value, _>(i) {
        Some(Value::Date(y, m, d, h, mi, s, _us)) => {
            use chrono::NaiveDate;
            let nd = NaiveDate::from_ymd_opt(y as i32, m as u32, d as u32).unwrap();
            let nt = nd.and_hms_opt(h as u32, mi as u32, s as u32).unwrap();
            Ok(DateTime::<Utc>::from_naive_utc_and_offset(nt, Utc))
        }
        Some(Value::NULL) | None => Ok(chrono::Utc::now()),
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

fn get_opt_json(r: &Row, i: usize) -> Result<Option<serde_json::Value>, ErroAplicacao> {
    use mysql::Value;
    match r.get::<Value, _>(i) {
        Some(Value::Bytes(b)) => {
            let s = String::from_utf8_lossy(&b);
            Ok(serde_json::from_str(&s).ok())
        }
        Some(Value::NULL) | None => Ok(None),
        _ => Ok(None),
    }
}

// =============================================================================
// Templates
// =============================================================================

pub fn criar_template(
    tenant_id: i32,
    nome: &str,
    descricao: Option<&str>,
    usuario_criacao_id: u32,
) -> Result<u32, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        "INSERT INTO os_checklist_templates (nome, descricao, usuario_criacao_id, tenant_id) VALUES (?, ?, ?, ?)",
        (nome, descricao, usuario_criacao_id, tenant_id),
    )
    .map_err(err)?;
    Ok(conn.last_insert_id() as u32)
}

pub fn adicionar_item_template(
    tenant_id: i32,
    template_id: u32,
    ordem: u32,
    texto: &str,
    obrigatorio: bool,
) -> Result<u32, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        "INSERT INTO os_checklist_template_itens (template_id, ordem, texto, obrigatorio, tenant_id) VALUES (?, ?, ?, ?, ?)",
        (template_id, ordem, texto, obrigatorio, tenant_id),
    )
    .map_err(err)?;
    Ok(conn.last_insert_id() as u32)
}

pub fn listar_templates(
    tenant_id: i32,
    ativo_apenas: bool,
) -> Result<Vec<ChecklistTemplate>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let sql = if ativo_apenas {
        "SELECT id, nome, descricao, ativo, data_criacao, usuario_criacao_id FROM os_checklist_templates WHERE tenant_id = ? AND ativo = TRUE ORDER BY nome"
    } else {
        "SELECT id, nome, descricao, ativo, data_criacao, usuario_criacao_id FROM os_checklist_templates WHERE tenant_id = ? ORDER BY nome"
    };
    let rows: Vec<Row> = conn.exec(sql, (tenant_id,)).map_err(err)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(ChecklistTemplate {
            id: get_i64(&r, 0)? as u32,
            nome: get_str(&r, 1)?,
            descricao: get_opt_str(&r, 2)?,
            ativo: get_bool(&r, 3)?,
            data_criacao: get_dt(&r, 4)?,
            usuario_criacao_id: get_i64(&r, 5)? as u32,
        });
    }
    Ok(out)
}

pub fn obter_template(
    tenant_id: i32,
    id: u32,
) -> Result<Option<ChecklistTemplateCompleto>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<Row> = conn
        .exec_first(
            "SELECT id, nome, descricao, ativo, data_criacao, usuario_criacao_id FROM os_checklist_templates WHERE tenant_id = ? AND id = ?",
            (id, tenant_id),
        )
        .map_err(err)?;
    let Some(r) = row else { return Ok(None) };
    let template = ChecklistTemplate {
        id: get_i64(&r, 0)? as u32,
        nome: get_str(&r, 1)?,
        descricao: get_opt_str(&r, 2)?,
        ativo: get_bool(&r, 3)?,
        data_criacao: get_dt(&r, 4)?,
        usuario_criacao_id: get_i64(&r, 5)? as u32,
    };
    let itens_rows: Vec<Row> = conn
        .exec(
            "SELECT id, template_id, ordem, texto, obrigatorio FROM os_checklist_template_itens WHERE tenant_id = ? AND template_id = ? ORDER BY ordem",
            (id,),
        )
        .map_err(err)?;
    let mut itens = Vec::new();
    for ir in itens_rows {
        itens.push(ChecklistTemplateItem {
            id: get_i64(&ir, 0)? as u32,
            template_id: get_i64(&ir, 1)? as u32,
            ordem: get_i64(&ir, 2)? as u32,
            texto: get_str(&ir, 3)?,
            obrigatorio: get_bool(&ir, 4)?,
        });
    }
    Ok(Some(ChecklistTemplateCompleto { template, itens }))
}

// =============================================================================
// Checklists aplicados
// =============================================================================

pub fn criar_checklist(
    tenant_id: i32,
    os_id: u32,
    template_id: Option<u32>,
    titulo: &str,
) -> Result<u32, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        "INSERT INTO os_checklists (os_id, template_id, titulo, tenant_id) VALUES (?, ?, ?, ?)",
        (os_id, template_id, titulo),
    )
    .map_err(err)?;
    let checklist_id = conn.last_insert_id() as u32;

    // Popula com base no template (se houver)
    if let Some(tid) = template_id {
        conn.exec_drop(
            r#"INSERT INTO os_checklist_itens
                (checklist_id, template_item_id, ordem, texto, obrigatorio, tenant_id)
               SELECT ?, id, ordem, texto, obrigatorio, ?
               FROM os_checklist_template_itens
               WHERE tenant_id = ? AND template_id = ?
               ORDER BY ordem"#,
            (checklist_id, tid, tenant_id, tid),
        )
        .map_err(err)?;
    }
    // Atualiza contador
    atualizar_contadores_checklist(tenant_id, checklist_id)?;
    Ok(checklist_id)
}

pub fn adicionar_item_checklist(
    tenant_id: i32,
    checklist_id: u32,
    texto: &str,
    obrigatorio: bool,
) -> Result<u32, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let max_ordem: Option<i64> = conn
        .exec_first(
            "SELECT COALESCE(MAX(ordem), 0) + 1 FROM os_checklist_itens WHERE tenant_id = ? AND checklist_id = ?",
            (checklist_id,),
        )
        .map_err(err)?
        .flatten();
    conn.exec_drop(
        "INSERT INTO os_checklist_itens (checklist_id, ordem, texto, obrigatorio, tenant_id) VALUES (?, ?, ?, ?, ?)",
        (checklist_id, max_ordem.unwrap_or(1), texto, obrigatorio),
    )
    .map_err(err)?;
    let id = conn.last_insert_id() as u32;
    atualizar_contadores_checklist(tenant_id, checklist_id)?;
    Ok(id)
}

pub fn marcar_item(
    tenant_id: i32,
    item_id: u32,
    concluido: bool,
    observacao: Option<&str>,
) -> Result<bool, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let now_expr = if concluido { "NOW()" } else { "NULL" };
    let sql = format!(
        "UPDATE os_checklist_itens SET concluido = ?, data_conclusao = {}, observacao = COALESCE(?, observacao) WHERE tenant_id = ? AND id = ?",
        now_expr
    );
    conn.exec_drop(&sql, (concluido, observacao, item_id))
        .map_err(err)?;
    let affected = conn.affected_rows();
    // Atualiza contadores do pai
    let checklist_id: Option<u32> = conn
        .exec_first(
            "SELECT checklist_id FROM os_checklist_itens WHERE tenant_id = ? AND id = ?",
            (item_id,),
        )
        .map_err(err)?
        .flatten();
    if let Some(cid) = checklist_id {
        atualizar_contadores_checklist(tenant_id, cid)?;
    }
    Ok(affected > 0)
}

pub fn atualizar_contadores_checklist(
    tenant_id: i32,
    checklist_id: u32,
) -> Result<(), ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        r#"UPDATE os_checklists c
           SET total_itens = (SELECT COUNT(*) FROM os_checklist_itens WHERE tenant_id = c.tenant_id AND checklist_id = c.id),
               concluidos = (SELECT COUNT(*) FROM os_checklist_itens WHERE checklist_id = c.id AND concluido = TRUE),
               data_conclusao = CASE
                   WHEN (SELECT COUNT(*) FROM os_checklist_itens WHERE checklist_id = c.id) > 0
                    AND (SELECT COUNT(*) FROM os_checklist_itens WHERE checklist_id = c.id AND concluido = TRUE)
                       = (SELECT COUNT(*) FROM os_checklist_itens WHERE checklist_id = c.id)
                   THEN COALESCE(c.data_conclusao, NOW())
                   WHEN (SELECT COUNT(*) FROM os_checklist_itens WHERE checklist_id = c.id AND concluido = TRUE) = 0
                   THEN NULL
                   ELSE c.data_conclusao
               END
           WHERE c.id = ?"#,
        (checklist_id, tenant_id),
    )
    .map_err(err)?;
    Ok(())
}

pub fn obter_checklist(
    tenant_id: i32,
    id: u32,
) -> Result<Option<ChecklistCompleto>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<Row> = conn
        .exec_first(
            r#"SELECT id, os_id, template_id, titulo, data_inicio, data_conclusao, total_itens, concluidos
               FROM os_checklists WHERE tenant_id = ? AND id = ?"#,
            (id, tenant_id),
        )
        .map_err(err)?;
    let Some(r) = row else { return Ok(None) };
    let checklist = Checklist {
        id: get_i64(&r, 0)? as u32,
        os_id: get_i64(&r, 1)? as u32,
        template_id: get_opt_i64(&r, 2)?.map(|n| n as u32),
        titulo: get_str(&r, 3)?,
        data_inicio: get_opt_dt(&r, 4)?,
        data_conclusao: get_opt_dt(&r, 5)?,
        total_itens: get_i64(&r, 6)? as u32,
        concluidos: get_i64(&r, 7)? as u32,
    };
    let itens_rows: Vec<Row> = conn
        .exec(
            r#"SELECT id, checklist_id, template_item_id, ordem, texto, obrigatorio, concluido, data_conclusao, observacao
               FROM os_checklist_itens WHERE tenant_id = ? AND checklist_id = ? ORDER BY ordem"#,
            (id,),
        )
        .map_err(err)?;
    let mut itens = Vec::new();
    for ir in itens_rows {
        itens.push(ChecklistItem {
            id: get_i64(&ir, 0)? as u32,
            checklist_id: get_i64(&ir, 1)? as u32,
            template_item_id: get_opt_i64(&ir, 2)?.map(|n| n as u32),
            ordem: get_i64(&ir, 3)? as u32,
            texto: get_str(&ir, 4)?,
            obrigatorio: get_bool(&ir, 5)?,
            concluido: get_bool(&ir, 6)?,
            data_conclusao: get_opt_dt(&ir, 7)?,
            observacao: get_opt_str(&ir, 8)?,
        });
    }
    Ok(Some(ChecklistCompleto { checklist, itens }))
}

pub fn listar_checklists_os(
    tenant_id: i32,
    os_id: u32,
) -> Result<Vec<ChecklistCompleto>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let rows: Vec<Row> = conn
        .exec(
            "SELECT id FROM os_checklists WHERE tenant_id = ? AND os_id = ? ORDER BY id",
            (tenant_id, os_id),
        )
        .map_err(err)?;
    let mut out = Vec::new();
    for r in rows {
        let id = get_i64(&r, 0)? as u32;
        if let Some(c) = obter_checklist(tenant_id, id)? {
            out.push(c);
        }
    }
    Ok(out)
}

pub fn remover_checklist(tenant_id: i32, id: u32) -> Result<bool, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        "DELETE FROM os_checklist_itens WHERE tenant_id = ? AND checklist_id = ?",
        (id, tenant_id),
    )
    .map_err(err)?;
    conn.exec_drop(
        "DELETE FROM os_checklists WHERE tenant_id = ? AND id = ?",
        (id,),
    )
    .map_err(err)?;
    Ok(conn.affected_rows() > 0)
}

// =============================================================================
// Evoluções
// =============================================================================

pub fn criar_evolucao(
    tenant_id: i32,
    os_id: u32,
    usuario_id: u32,
    texto: &str,
    tipo: TipoEvolucao,
    latitude: Option<f64>,
    longitude: Option<f64>,
    data_evento_device: Option<DateTime<Utc>>,
) -> Result<u32, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let data_sql = match data_evento_device {
        Some(dt) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
        None => chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };
    conn.exec_drop(
        r#"INSERT INTO os_evolucoes (os_id, usuario_id, texto, tipo, latitude, longitude, data_evolucao, tenant_id)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
        (os_id, usuario_id, texto, tipo.as_db_str(), latitude, longitude, data_sql, tenant_id),
    )
    .map_err(err)?;
    Ok(conn.last_insert_id() as u32)
}

pub fn listar_evolucoes_os(tenant_id: i32, os_id: u32) -> Result<Vec<Evolucao>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let rows: Vec<Row> = conn
        .exec(
            r#"SELECT id, os_id, usuario_id, texto, tipo, data_evolucao, latitude, longitude
               FROM os_evolucoes WHERE tenant_id = ? AND os_id = ? ORDER BY data_evolucao DESC"#,
            (os_id, tenant_id),
        )
        .map_err(err)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(Evolucao {
            id: get_i64(&r, 0)? as u32,
            os_id: get_i64(&r, 1)? as u32,
            usuario_id: get_i64(&r, 2)? as u32,
            texto: get_str(&r, 3)?,
            tipo: TipoEvolucao::from_db_str(&get_str(&r, 4)?),
            data_evolucao: get_dt(&r, 5)?,
            latitude: get_opt_f64(&r, 6)?,
            longitude: get_opt_f64(&r, 7)?,
        });
    }
    Ok(out)
}

// =============================================================================
// Assinaturas
// =============================================================================

pub fn criar_assinatura(
    tenant_id: i32,
    os_id: u32,
    arquivo_id: u32,
    nome_assinante: &str,
    documento_assinante: Option<&str>,
    ip_assinatura: Option<&str>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    observacao: Option<&str>,
) -> Result<u32, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        r#"INSERT INTO os_assinaturas
            (os_id, arquivo_id, nome_assinante, documento_assinante, ip_assinatura, latitude, longitude, observacao, tenant_id)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        (
            os_id,
            arquivo_id,
            nome_assinante,
            documento_assinante,
            ip_assinatura,
            latitude,
            longitude,
            observacao,
            tenant_id,
        ),
    )
    .map_err(err)?;
    Ok(conn.last_insert_id() as u32)
}

pub fn obter_assinatura(tenant_id: i32, os_id: u32) -> Result<Option<Assinatura>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<Row> = conn
        .exec_first(
            r#"SELECT id, os_id, arquivo_id, nome_assinante, documento_assinante, data_assinatura,
                      ip_assinatura, latitude, longitude, observacao
               FROM os_assinaturas WHERE tenant_id = ? AND os_id = ? ORDER BY data_assinatura DESC LIMIT 1"#,
            (os_id, tenant_id),
        )
        .map_err(err)?;
    match row {
        Some(r) => Ok(Some(Assinatura {
            id: get_i64(&r, 0)? as u32,
            os_id: get_i64(&r, 1)? as u32,
            arquivo_id: get_i64(&r, 2)? as u32,
            nome_assinante: get_str(&r, 3)?,
            documento_assinante: get_opt_str(&r, 4)?,
            data_assinatura: get_dt(&r, 5)?,
            ip_assinatura: get_opt_str(&r, 6)?,
            latitude: get_opt_f64(&r, 7)?,
            longitude: get_opt_f64(&r, 8)?,
            observacao: get_opt_str(&r, 9)?,
        })),
        None => Ok(None),
    }
}

// =============================================================================
// Auditoria offline
// =============================================================================

pub fn registrar_auditoria_campo(
    tenant_id: i32,
    cliente_device_id: Option<&str>,
    usuario_id: u32,
    tipo_acao: TipoAcaoAuditoria,
    os_id: Option<u32>,
    checklist_id: Option<u32>,
    checklist_item_id: Option<u32>,
    evolucao_id: Option<u32>,
    arquivo_id: Option<u32>,
    assinatura_id: Option<u32>,
    data_evento_device: DateTime<Utc>,
    payload: Option<&serde_json::Value>,
) -> Result<u32, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let data_dev = data_evento_device.format("%Y-%m-%d %H:%M:%S").to_string();
    let payload_str = payload.map(|p| p.to_string());
    // INSERT IGNORE — dedupe se mesmo device+evento já chegou
    conn.exec_drop(
        r#"INSERT IGNORE INTO os_auditoria_campo
            (cliente_device_id, usuario_id, tipo_acao, os_id, checklist_id, checklist_item_id,
             evolucao_id, arquivo_id, assinatura_id, data_evento_device, payload_json, tenant_id)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        (
            cliente_device_id,
            usuario_id,
            tipo_acao.as_db_str(),
            os_id,
            checklist_id,
            checklist_item_id,
            evolucao_id,
            arquivo_id,
            assinatura_id,
            data_dev,
            payload_str.as_deref(),
            tenant_id,
        ),
    )
    .map_err(err)?;
    Ok(conn.last_insert_id() as u32)
}

pub fn listar_auditoria_os(
    tenant_id: i32,
    os_id: u32,
) -> Result<Vec<AuditoriaCampo>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let rows: Vec<Row> = conn
        .exec(
            r#"SELECT id, cliente_device_id, usuario_id, tipo_acao, os_id, checklist_id,
                      checklist_item_id, evolucao_id, arquivo_id, assinatura_id,
                      data_evento_device, data_sincronizacao, processado, payload_json
               FROM os_auditoria_campo WHERE tenant_id = ? AND os_id = ? ORDER BY data_evento_device DESC"#,
            (tenant_id, os_id),
        )
        .map_err(err)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(AuditoriaCampo {
            id: get_i64(&r, 0)? as u32,
            cliente_device_id: get_opt_str(&r, 1)?,
            usuario_id: get_i64(&r, 2)? as u32,
            tipo_acao: parse_tipo_acao(&get_str(&r, 3)?),
            os_id: get_opt_i64(&r, 4)?.map(|n| n as u32),
            checklist_id: get_opt_i64(&r, 5)?.map(|n| n as u32),
            checklist_item_id: get_opt_i64(&r, 6)?.map(|n| n as u32),
            evolucao_id: get_opt_i64(&r, 7)?.map(|n| n as u32),
            arquivo_id: get_opt_i64(&r, 8)?.map(|n| n as u32),
            assinatura_id: get_opt_i64(&r, 9)?.map(|n| n as u32),
            data_evento_device: get_dt(&r, 10)?,
            data_sincronizacao: get_dt(&r, 11)?,
            processado: get_bool(&r, 12)?,
            payload_json: get_opt_json(&r, 13)?,
        });
    }
    Ok(out)
}

/// Versão leve: retorna só o id do checklist pai + os_id (sem carregar itens).
/// Usado em `service::marcar_item` para descobrir a OS.
pub fn obter_checklist_direto(
    tenant_id: i32,
    item_id: u32,
) -> Result<Option<ChecklistDireto>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<Row> = conn
        .exec_first(
            r#"SELECT c.id, c.os_id
               FROM os_checklists c
               INNER JOIN os_checklist_itens i ON i.checklist_id = c.id AND i.tenant_id = c.tenant_id
               WHERE i.tenant_id = ? AND i.id = ?"#,
            (tenant_id, item_id),
        )
        .map_err(err)?;
    match row {
        Some(r) => Ok(Some(ChecklistDireto {
            id: get_i64(&r, 0)? as u32,
            os_id: get_i64(&r, 1)? as u32,
        })),
        None => Ok(None),
    }
}

#[derive(Debug, Clone)]
pub struct ChecklistDireto {
    pub id: u32,
    pub os_id: u32,
}

/// Lista os checklists mais recentes que ainda têm itens pendentes.
pub fn listar_checklists_pendentes_recentes(
    tenant_id: i32,
    limite: u32,
) -> Result<Vec<ChecklistCompleto>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let rows: Vec<Row> = conn
        .exec(
            r#"SELECT DISTINCT c.id
               FROM os_checklists c
               INNER JOIN os_checklist_itens i ON i.checklist_id = c.id AND i.tenant_id = c.tenant_id
               WHERE i.tenant_id = ? AND i.concluido = FALSE
               ORDER BY c.id DESC LIMIT ?"#,
            (tenant_id, limite),
        )
        .map_err(err)?;
    let mut out = Vec::new();
    for r in rows {
        let id = get_i64(&r, 0)? as u32;
        if let Some(c) = obter_checklist(tenant_id, id)? {
            out.push(c);
        }
    }
    Ok(out)
}

fn parse_tipo_acao(s: &str) -> TipoAcaoAuditoria {
    match s {
        "OS_CRIADA" => TipoAcaoAuditoria::OsCriada,
        "OS_ATUALIZADA" => TipoAcaoAuditoria::OsAtualizada,
        "STATUS_ALTERADO" => TipoAcaoAuditoria::StatusAlterado,
        "CHECKLIST_ITEM_CONCLUIDO" => TipoAcaoAuditoria::ChecklistItemConcluido,
        "CHECKLIST_CONCLUIDO" => TipoAcaoAuditoria::ChecklistConcluido,
        "ANEXO_ENVIADO" => TipoAcaoAuditoria::AnexoEnviado,
        "EVOLUCAO_REGISTRADA" => TipoAcaoAuditoria::EvolucaoRegistrada,
        "ASSINATURA_REGISTRADA" => TipoAcaoAuditoria::AssinaturaRegistrada,
        _ => TipoAcaoAuditoria::OsAtualizada,
    }
}

// =============================================================================
// Dashboard do técnico
// =============================================================================

pub fn dashboard_tecnico(
    tenant_id: i32,
    usuario_id: Option<u32>,
) -> Result<super::models::DashboardTecnico, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let user_param: Option<u32> = usuario_id;

    // QUERY 1: Contadores de OS (CASE WHEN + COUNT condicional — 1 query, antes eram 4)
    let counters: mysql::Row = conn.exec_first(
        r#"SELECT
              SUM(CASE WHEN status NOT IN ('CONCLUIDA','CANCELADA','FINALIZADA','AGUARDANDO_PECAS_ABERTA') THEN 1 ELSE 0 END) as abertas,
              SUM(CASE WHEN status IN ('EM_ANDAMENTO','EM_EXECUCAO') THEN 1 ELSE 0 END) as em_andamento,
              SUM(CASE WHEN status IN ('AGUARDANDO_PECA','AGUARDANDO_PECAS','AGUARDANDO_PECAS_ABERTA') THEN 1 ELSE 0 END) as aguardando_peca,
              SUM(CASE WHEN status IN ('CONCLUIDA','FINALIZADA') THEN 1 ELSE 0 END) as concluidas
           FROM ordens_servico
           WHERE tenant_id = ? AND (? IS NULL OR tecnico_id = ?)"#,
        (tenant_id, user_param, user_param),
    ).map_err(err)?.ok_or_else(|| ErroAplicacao::Desconhecido("counters vazio".into()))?;

    // QUERY 2: Checklists pendentes (1 query — antes 1)
    let chk: mysql::Row = conn.exec_first(
        r#"SELECT
              (SELECT COUNT(DISTINCT c.id) FROM os_checklists c
                 INNER JOIN os_checklist_itens i ON i.checklist_id = c.id
                 WHERE c.tenant_id = ? AND i.concluido = FALSE) as checklists_pendentes,
              (SELECT COUNT(*) FROM ordens_servico o
                 WHERE o.tenant_id = ? AND o.status NOT IN ('CONCLUIDA','CANCELADA','FINALIZADA')
                   AND NOT EXISTS (SELECT 1 FROM os_checklists c WHERE c.os_id = o.id AND c.tenant_id = o.tenant_id)) as os_sem_checklist"#,
        (tenant_id, tenant_id),
    ).map_err(err)?.ok_or_else(|| ErroAplicacao::Desconhecido("chk vazio".into()))?;

    Ok(super::models::DashboardTecnico {
        os_abertas: get_i64_from_row(&counters, 0) as u32,
        os_em_andamento: get_i64_from_row(&counters, 1) as u32,
        os_aguardando_peca: get_i64_from_row(&counters, 2) as u32,
        os_concluidas: get_i64_from_row(&counters, 3) as u32,
        checklists_pendentes: get_i64_from_row(&chk, 0) as u32,
        os_sem_checklist: get_i64_from_row(&chk, 1) as u32,
        proximas_os: Vec::new(),
        checklists_recentes: Vec::new(),
    })
}

fn get_i64_from_row(row: &mysql::Row, idx: usize) -> i64 {
    match row.get::<mysql::Value, _>(idx) {
        Some(mysql::Value::Int(n)) => n,
        Some(mysql::Value::UInt(n)) => n as i64,
        _ => 0,
    }
}

pub fn listar_os_resumo(
    tenant_id: i32,
    usuario_id: Option<u32>,
    limite: u32,
) -> Result<Vec<OsResumo>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let mut sql = String::from(
        r#"SELECT o.id, o.cliente_id, COALESCE(c.nome, ''), o.equipamento_id, o.descricao,
                  o.status, o.descricao, o.data_criacao,
                  (SELECT COUNT(*) FROM arquivo_vinculos v
                    INNER JOIN arquivos a ON a.id = v.arquivo_id AND a.tenant_id = v.tenant_id
                    WHERE v.tenant_id = ? AND v.entidade_tipo = 'OS' AND v.entidade_id = o.id AND a.removido_em IS NULL),
                  (SELECT COUNT(*) FROM os_checklists WHERE tenant_id = ? AND os_id = o.id),
                  (SELECT COUNT(*) FROM os_evolucoes WHERE tenant_id = ? AND os_id = o.id)
           FROM ordens_servico o
           LEFT JOIN clientes c ON c.id = o.cliente_id AND c.tenant_id = o.tenant_id
           WHERE o.tenant_id = ? AND o.status NOT IN ('CONCLUIDA','CANCELADA','FINALIZADA')"#,
    );
    if usuario_id.is_some() {
        sql.push_str(" AND o.tecnico_id = ?");
    }
    sql.push_str(" ORDER BY o.data_criacao DESC LIMIT ?");

    let rows: Vec<Row> = if let Some(uid) = usuario_id {
        conn.exec(
            &sql,
            (tenant_id, tenant_id, tenant_id, tenant_id, uid, limite),
        )
        .map_err(err)?
    } else {
        conn.exec(&sql, (tenant_id, tenant_id, tenant_id, tenant_id, limite))
            .map_err(err)?
    };

    let mut out = Vec::new();
    for r in rows {
        out.push(OsResumo {
            id: get_i64(&r, 0)? as u32,
            cliente_id: get_i64(&r, 1)? as u32,
            cliente_nome: get_str(&r, 2)?,
            equipamento_id: get_opt_i64(&r, 3)?.map(|n| n as u32),
            equipamento_descricao: get_opt_str(&r, 4)?,
            status: get_str(&r, 5)?,
            descricao: get_opt_str(&r, 6)?,
            data_criacao: get_dt(&r, 7)?,
            data_inicio: None,
            data_conclusao: None,
            total_anexos: get_i64(&r, 8)? as u32,
            total_checklists: get_i64(&r, 9)? as u32,
            total_evolucoes: get_i64(&r, 10)? as u32,
        });
    }
    Ok(out)
}
