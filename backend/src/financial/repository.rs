// src/financial/repository.rs
//
// Sprint P2.5 — Financeiro.
// P2.6.2a — tenant_id em 100% das queries (defense in depth).
//
// Acesso a dados:
//   - Configuração
//   - Plano de contas
//   - Centro de custo
//   - Contas a receber/pagar
//   - Lançamentos (append-only)
//   - Alertas

use super::models::{
    AlertaFinanceiro, AlertaFinanceiroOcorrencia, CentroCusto, ConfiguracaoFinanceira, ContaPagar,
    ContaReceber, Lancamento, OrigemPagar, OrigemReceber, PlanoConta, Severidade, StatusConta,
    TipoEntidadeAlerta, TipoLancamento, TipoPlanoConta,
};
use crate::banco_de_dados::obter_conexao;
use crate::servicos::ErroAplicacao;
use chrono::{DateTime, NaiveDate, Utc};
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

fn get_dt(r: &Row, i: usize) -> Result<DateTime<Utc>, ErroAplicacao> {
    use mysql::Value;
    match r.get::<Value, _>(i) {
        Some(Value::Date(y, m, d, h, mi, s, _us)) => {
            use chrono::NaiveDate;
            let nd = NaiveDate::from_ymd_opt(y as i32, m as u32, d as u32).unwrap();
            let nt = nd.and_hms_opt(h as u32, mi as u32, s as u32).unwrap();
            Ok(DateTime::<Utc>::from_naive_utc_and_offset(nt, Utc))
        }
        Some(Value::NULL) | None => Ok(Utc::now()),
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

fn get_date(r: &Row, i: usize) -> Result<NaiveDate, ErroAplicacao> {
    use mysql::Value;
    match r.get::<Value, _>(i) {
        Some(Value::Date(y, m, d, _, _, _, _)) => {
            Ok(NaiveDate::from_ymd_opt(y as i32, m as u32, d as u32)
                .ok_or_else(|| err("data inválida"))?)
        }
        Some(Value::NULL) | None => Ok(Utc::now().date_naive()),
        _ => Err(err("tipo errado")),
    }
}

fn get_opt_date(r: &Row, i: usize) -> Result<Option<NaiveDate>, ErroAplicacao> {
    use mysql::Value;
    match r.get::<Value, _>(i) {
        Some(Value::Date(y, m, d, _, _, _, _)) => Ok(Some(
            NaiveDate::from_ymd_opt(y as i32, m as u32, d as u32)
                .ok_or_else(|| err("data inválida"))?,
        )),
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

fn naivedate_to_str(d: NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

// =============================================================================
// Configuração
// =============================================================================

pub fn obter_configuracao(tenant_id: i32) -> Result<ConfiguracaoFinanceira, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<Row> = conn
        .exec_first(
            r#"SELECT id, dias_vencimento_padrao, centro_custo_padrao_receber_id,
                      plano_conta_padrao_receber_id, centro_custo_padrao_pagar_id,
                      plano_conta_padrao_pagar_id, moeda_padrao, data_atualizacao
               FROM configuracao_financeira WHERE tenant_id = :tenant_id"#,
            params! { "tenant_id" => tenant_id },
        )
        .map_err(err)?;
    match row {
        Some(r) => Ok(ConfiguracaoFinanceira {
            id: get_i64(&r, 0)? as u32,
            dias_vencimento_padrao: get_i64(&r, 1)? as u32,
            centro_custo_padrao_receber_id: get_opt_i64(&r, 2)?.map(|n| n as u32),
            plano_conta_padrao_receber_id: get_opt_i64(&r, 3)?.map(|n| n as u32),
            centro_custo_padrao_pagar_id: get_opt_i64(&r, 4)?.map(|n| n as u32),
            plano_conta_padrao_pagar_id: get_opt_i64(&r, 5)?.map(|n| n as u32),
            moeda_padrao: get_str(&r, 6)?,
            data_atualizacao: get_dt(&r, 7)?,
        }),
        None => Err(err(
            "configuração financeira não inicializada para este tenant",
        )),
    }
}

pub fn atualizar_configuracao(
    tenant_id: i32,
    dias: u32,
    cc_rec: Option<u32>,
    pc_rec: Option<u32>,
    cc_pag: Option<u32>,
    pc_pag: Option<u32>,
    moeda: &str,
) -> Result<(), ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let affected = conn
        .exec_iter(
            r#"UPDATE configuracao_financeira
           SET dias_vencimento_padrao = ?,
               centro_custo_padrao_receber_id = ?,
               plano_conta_padrao_receber_id = ?,
               centro_custo_padrao_pagar_id = ?,
               plano_conta_padrao_pagar_id = ?,
               moeda_padrao = ?
           WHERE tenant_id = ?"#,
            (dias, cc_rec, pc_rec, cc_pag, pc_pag, moeda, tenant_id),
        )
        .map_err(err)?
        .affected_rows();
    if affected == 0 {
        return Err(err("configuração não encontrada para este tenant"));
    }
    Ok(())
}

// =============================================================================
// Plano de Contas
// =============================================================================

pub fn listar_plano_contas(
    tenant_id: i32,
    ativo_apenas: bool,
) -> Result<Vec<PlanoConta>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let sql = if ativo_apenas {
        "SELECT id, codigo, nome, tipo, pai_id, ativo, descricao, data_criacao FROM plano_contas WHERE tenant_id = :tenant_id AND ativo = TRUE ORDER BY codigo"
    } else {
        "SELECT id, codigo, nome, tipo, pai_id, ativo, descricao, data_criacao FROM plano_contas WHERE tenant_id = :tenant_id ORDER BY codigo"
    };
    let rows: Vec<Row> = conn
        .exec(sql, params! { "tenant_id" => tenant_id })
        .map_err(err)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(PlanoConta {
            id: get_i64(&r, 0)? as u32,
            codigo: get_str(&r, 1)?,
            nome: get_str(&r, 2)?,
            tipo: TipoPlanoConta::from_db_str(&get_str(&r, 3)?),
            pai_id: get_opt_i64(&r, 4)?.map(|n| n as u32),
            ativo: get_bool(&r, 5)?,
            descricao: get_opt_str(&r, 6)?,
            data_criacao: get_dt(&r, 7)?,
        });
    }
    Ok(out)
}

pub fn obter_plano_conta(tenant_id: i32, id: u32) -> Result<Option<PlanoConta>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<Row> = conn
        .exec_first(
            "SELECT id, codigo, nome, tipo, pai_id, ativo, descricao, data_criacao FROM plano_contas WHERE id = ? AND tenant_id = ?",
            (id, tenant_id),
        )
        .map_err(err)?;
    match row {
        Some(r) => Ok(Some(PlanoConta {
            id: get_i64(&r, 0)? as u32,
            codigo: get_str(&r, 1)?,
            nome: get_str(&r, 2)?,
            tipo: TipoPlanoConta::from_db_str(&get_str(&r, 3)?),
            pai_id: get_opt_i64(&r, 4)?.map(|n| n as u32),
            ativo: get_bool(&r, 5)?,
            descricao: get_opt_str(&r, 6)?,
            data_criacao: get_dt(&r, 7)?,
        })),
        None => Ok(None),
    }
}

// =============================================================================
// Centro de Custo
// =============================================================================

pub fn listar_centros_custo(
    tenant_id: i32,
    ativo_apenas: bool,
) -> Result<Vec<CentroCusto>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let sql = if ativo_apenas {
        "SELECT id, codigo, nome, descricao, ativo, data_criacao FROM centros_custo WHERE tenant_id = ? AND ativo = TRUE ORDER BY nome"
    } else {
        "SELECT id, codigo, nome, descricao, ativo, data_criacao FROM centros_custo WHERE tenant_id = ? ORDER BY nome"
    };
    let rows: Vec<Row> = conn.exec(sql, (tenant_id,)).map_err(err)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(CentroCusto {
            id: get_i64(&r, 0)? as u32,
            codigo: get_str(&r, 1)?,
            nome: get_str(&r, 2)?,
            descricao: get_opt_str(&r, 3)?,
            ativo: get_bool(&r, 4)?,
            data_criacao: get_dt(&r, 5)?,
        });
    }
    Ok(out)
}

pub fn obter_centro_custo(tenant_id: i32, id: u32) -> Result<Option<CentroCusto>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<Row> = conn
        .exec_first(
            "SELECT id, codigo, nome, descricao, ativo, data_criacao FROM centros_custo WHERE id = ? AND tenant_id = ?",
            (id, tenant_id),
        )
        .map_err(err)?;
    match row {
        Some(r) => Ok(Some(CentroCusto {
            id: get_i64(&r, 0)? as u32,
            codigo: get_str(&r, 1)?,
            nome: get_str(&r, 2)?,
            descricao: get_opt_str(&r, 3)?,
            ativo: get_bool(&r, 4)?,
            data_criacao: get_dt(&r, 5)?,
        })),
        None => Ok(None),
    }
}

// =============================================================================
// Contas a Receber
// =============================================================================

pub fn inserir_conta_receber(
    tenant_id: i32,
    cliente_id: u32,
    origem_tipo: OrigemReceber,
    origem_id: Option<u32>,
    descricao: &str,
    valor: i64,
    vencimento: NaiveDate,
    data_competencia: NaiveDate,
    centro_custo_id: u32,
    plano_conta_id: u32,
    observacao: Option<&str>,
    usuario_criacao_id: u32,
) -> Result<u32, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        r#"INSERT INTO contas_receber
            (tenant_id, cliente_id, origem_tipo, origem_id, descricao, valor, valor_pago,
             vencimento, data_competencia, status, centro_custo_id, plano_conta_id,
             observacao, usuario_criacao_id)
           VALUES (?, ?, ?, ?, ?, ?, 0, ?, ?, 'PENDENTE', ?, ?, ?, ?)"#,
        (
            tenant_id,
            cliente_id,
            origem_tipo.as_db_str(),
            origem_id,
            descricao,
            valor,
            naivedate_to_str(vencimento),
            naivedate_to_str(data_competencia),
            centro_custo_id,
            plano_conta_id,
            observacao,
            usuario_criacao_id,
        ),
    )
    .map_err(err)?;
    Ok(conn.last_insert_id() as u32)
}

pub fn obter_conta_receber(tenant_id: i32, id: u32) -> Result<Option<ContaReceber>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<Row> = conn
        .exec_first(
            r#"SELECT id, cliente_id, origem_tipo, origem_id, descricao, valor, valor_pago,
                      vencimento, data_pagamento, data_competencia, status, centro_custo_id,
                      plano_conta_id, observacao, usuario_criacao_id, usuario_pagamento_id,
                      data_criacao, data_atualizacao
               FROM contas_receber WHERE id = ? AND tenant_id = ?"#,
            (id, tenant_id),
        )
        .map_err(err)?;
    match row {
        Some(r) => Ok(Some(ContaReceber {
            id: get_i64(&r, 0)? as u32,
            cliente_id: get_i64(&r, 1)? as u32,
            origem_tipo: OrigemReceber::from_db_str(&get_str(&r, 2)?),
            origem_id: get_opt_i64(&r, 3)?.map(|n| n as u32),
            descricao: get_str(&r, 4)?,
            valor: get_i64(&r, 5)?,
            valor_pago: get_i64(&r, 6)?,
            vencimento: get_date(&r, 7)?,
            data_pagamento: get_opt_date(&r, 8)?,
            data_competencia: get_date(&r, 9)?,
            status: StatusConta::from_db_str(&get_str(&r, 10)?),
            centro_custo_id: get_i64(&r, 11)? as u32,
            plano_conta_id: get_i64(&r, 12)? as u32,
            observacao: get_opt_str(&r, 13)?,
            usuario_criacao_id: get_i64(&r, 14)? as u32,
            usuario_pagamento_id: get_opt_i64(&r, 15)?.map(|n| n as u32),
            data_criacao: get_dt(&r, 16)?,
            data_atualizacao: get_dt(&r, 17)?,
        })),
        None => Ok(None),
    }
}

pub fn listar_contas_receber(
    tenant_id: i32,
    cliente_id: Option<u32>,
    status: Option<StatusConta>,
    limite: u32,
) -> Result<Vec<ContaReceber>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let mut sql = String::from(
        r#"SELECT id, cliente_id, origem_tipo, origem_id, descricao, valor, valor_pago,
                  vencimento, data_pagamento, data_competencia, status, centro_custo_id,
                  plano_conta_id, observacao, usuario_criacao_id, usuario_pagamento_id,
                  data_criacao, data_atualizacao
           FROM contas_receber WHERE tenant_id = ?"#,
    );
    if cliente_id.is_some() {
        sql.push_str(" AND cliente_id = ?");
    }
    if status.is_some() {
        sql.push_str(" AND status = ?");
    }
    sql.push_str(" ORDER BY vencimento ASC LIMIT ?");

    let lim = limite as i64;
    let rows: Vec<Row> = if let (Some(c), Some(s)) = (cliente_id, status) {
        conn.exec(&sql, (tenant_id, c, s.as_db_str(), lim))
            .map_err(err)?
    } else if let Some(c) = cliente_id {
        conn.exec(&sql, (tenant_id, c, lim)).map_err(err)?
    } else if let Some(s) = status {
        conn.exec(&sql, (tenant_id, s.as_db_str(), lim))
            .map_err(err)?
    } else {
        conn.exec(&sql, (tenant_id, lim)).map_err(err)?
    };
    let mut out = Vec::new();
    for r in rows {
        out.push(row_para_conta_receber(r)?);
    }
    Ok(out)
}

fn row_para_conta_receber(r: Row) -> Result<ContaReceber, ErroAplicacao> {
    Ok(ContaReceber {
        id: get_i64(&r, 0)? as u32,
        cliente_id: get_i64(&r, 1)? as u32,
        origem_tipo: OrigemReceber::from_db_str(&get_str(&r, 2)?),
        origem_id: get_opt_i64(&r, 3)?.map(|n| n as u32),
        descricao: get_str(&r, 4)?,
        valor: get_i64(&r, 5)?,
        valor_pago: get_i64(&r, 6)?,
        vencimento: get_date(&r, 7)?,
        data_pagamento: get_opt_date(&r, 8)?,
        data_competencia: get_date(&r, 9)?,
        status: StatusConta::from_db_str(&get_str(&r, 10)?),
        centro_custo_id: get_i64(&r, 11)? as u32,
        plano_conta_id: get_i64(&r, 12)? as u32,
        observacao: get_opt_str(&r, 13)?,
        usuario_criacao_id: get_i64(&r, 14)? as u32,
        usuario_pagamento_id: get_opt_i64(&r, 15)?.map(|n| n as u32),
        data_criacao: get_dt(&r, 16)?,
        data_atualizacao: get_dt(&r, 17)?,
    })
}

pub fn atualizar_pagamento_conta_receber(
    tenant_id: i32,
    id: u32,
    valor_pago: i64,
    data_pagamento: NaiveDate,
    novo_status: StatusConta,
    usuario_id: u32,
) -> Result<bool, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let affected = conn
        .exec_iter(
            r#"UPDATE contas_receber
           SET valor_pago = ?, data_pagamento = ?, status = ?, usuario_pagamento_id = ?
           WHERE id = ? AND tenant_id = ?"#,
            (
                valor_pago,
                naivedate_to_str(data_pagamento),
                novo_status.as_db_str(),
                usuario_id,
                id,
                tenant_id,
            ),
        )
        .map_err(err)?
        .affected_rows();
    Ok(affected > 0)
}

pub fn cancelar_conta_receber(tenant_id: i32, id: u32) -> Result<bool, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let affected = conn
        .exec_iter(
            "UPDATE contas_receber SET status = 'CANCELADO' WHERE id = ? AND tenant_id = ?",
            (id, tenant_id),
        )
        .map_err(err)?
        .affected_rows();
    Ok(affected > 0)
}

// =============================================================================
// Contas a Pagar
// =============================================================================

pub fn inserir_conta_pagar(
    tenant_id: i32,
    fornecedor: &str,
    fornecedor_doc: Option<&str>,
    descricao: &str,
    valor: i64,
    vencimento: NaiveDate,
    data_competencia: NaiveDate,
    centro_custo_id: u32,
    plano_conta_id: u32,
    origem_tipo: OrigemPagar,
    origem_id: Option<u32>,
    observacao: Option<&str>,
    usuario_criacao_id: u32,
) -> Result<u32, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        r#"INSERT INTO contas_pagar
            (tenant_id, fornecedor, fornecedor_doc, descricao, valor, valor_pago,
             vencimento, data_competencia, status, centro_custo_id, plano_conta_id,
             origem_tipo, origem_id, observacao, usuario_criacao_id)
           VALUES (:tid, :fornecedor, :doc, :desc, :valor, 0, :venc, :comp, 'PENDENTE', :cc, :pc, :origem_tipo, :origem_id, :obs, :user)"#,
        params! {
            "tid" => tenant_id,
            "fornecedor" => fornecedor,
            "doc" => fornecedor_doc,
            "desc" => descricao,
            "valor" => valor,
            "venc" => naivedate_to_str(vencimento),
            "comp" => naivedate_to_str(data_competencia),
            "cc" => centro_custo_id,
            "pc" => plano_conta_id,
            "origem_tipo" => origem_tipo.as_db_str(),
            "origem_id" => origem_id,
            "obs" => observacao,
            "user" => usuario_criacao_id,
        },
    )
    .map_err(err)?;
    Ok(conn.last_insert_id() as u32)
}

pub fn obter_conta_pagar(tenant_id: i32, id: u32) -> Result<Option<ContaPagar>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<Row> = conn
        .exec_first(
            r#"SELECT id, fornecedor, fornecedor_doc, descricao, valor, valor_pago,
                      vencimento, data_pagamento, data_competencia, status, centro_custo_id,
                      plano_conta_id, origem_tipo, origem_id, observacao, usuario_criacao_id,
                      usuario_pagamento_id, data_criacao, data_atualizacao
               FROM contas_pagar WHERE id = ? AND tenant_id = ?"#,
            (id, tenant_id),
        )
        .map_err(err)?;
    match row {
        Some(r) => Ok(Some(row_para_conta_pagar(r)?)),
        None => Ok(None),
    }
}

pub fn listar_contas_pagar(
    tenant_id: i32,
    status: Option<StatusConta>,
    limite: u32,
) -> Result<Vec<ContaPagar>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let mut sql = String::from(
        r#"SELECT id, fornecedor, fornecedor_doc, descricao, valor, valor_pago,
                  vencimento, data_pagamento, data_competencia, status, centro_custo_id,
                  plano_conta_id, origem_tipo, origem_id, observacao, usuario_criacao_id,
                  usuario_pagamento_id, data_criacao, data_atualizacao
           FROM contas_pagar WHERE tenant_id = ?"#,
    );
    if status.is_some() {
        sql.push_str(" AND status = ?");
    }
    sql.push_str(" ORDER BY vencimento ASC LIMIT ?");
    let lim = limite as i64;
    let rows: Vec<Row> = if let Some(s) = status {
        conn.exec(&sql, (tenant_id, s.as_db_str(), lim))
            .map_err(err)?
    } else {
        conn.exec(&sql, (tenant_id, lim)).map_err(err)?
    };
    let mut out = Vec::new();
    for r in rows {
        out.push(row_para_conta_pagar(r)?);
    }
    Ok(out)
}

fn row_para_conta_pagar(r: Row) -> Result<ContaPagar, ErroAplicacao> {
    Ok(ContaPagar {
        id: get_i64(&r, 0)? as u32,
        fornecedor: get_str(&r, 1)?,
        fornecedor_doc: get_opt_str(&r, 2)?,
        descricao: get_str(&r, 3)?,
        valor: get_i64(&r, 4)?,
        valor_pago: get_i64(&r, 5)?,
        vencimento: get_date(&r, 6)?,
        data_pagamento: get_opt_date(&r, 7)?,
        data_competencia: get_date(&r, 8)?,
        status: StatusConta::from_db_str(&get_str(&r, 9)?),
        centro_custo_id: get_i64(&r, 10)? as u32,
        plano_conta_id: get_i64(&r, 11)? as u32,
        origem_tipo: OrigemPagar::from_db_str(&get_str(&r, 12)?),
        origem_id: get_opt_i64(&r, 13)?.map(|n| n as u32),
        observacao: get_opt_str(&r, 14)?,
        usuario_criacao_id: get_i64(&r, 15)? as u32,
        usuario_pagamento_id: get_opt_i64(&r, 16)?.map(|n| n as u32),
        data_criacao: get_dt(&r, 17)?,
        data_atualizacao: get_dt(&r, 18)?,
    })
}

pub fn atualizar_pagamento_conta_pagar(
    tenant_id: i32,
    id: u32,
    valor_pago: i64,
    data_pagamento: NaiveDate,
    novo_status: StatusConta,
    usuario_id: u32,
) -> Result<bool, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let affected = conn
        .exec_iter(
            r#"UPDATE contas_pagar
           SET valor_pago = ?, data_pagamento = ?, status = ?, usuario_pagamento_id = ?
           WHERE id = ? AND tenant_id = ?"#,
            (
                valor_pago,
                naivedate_to_str(data_pagamento),
                novo_status.as_db_str(),
                usuario_id,
                id,
                tenant_id,
            ),
        )
        .map_err(err)?
        .affected_rows();
    Ok(affected > 0)
}

// =============================================================================
// Lançamentos
// =============================================================================

pub fn inserir_lancamento(
    tenant_id: i32,
    tipo: TipoLancamento,
    conta_receber_id: Option<u32>,
    conta_pagar_id: Option<u32>,
    valor: i64,
    data_lancamento: NaiveDate,
    data_competencia: NaiveDate,
    descricao: &str,
    plano_conta_id: u32,
    centro_custo_id: u32,
    forma_pagamento: Option<&str>,
    lancamento_estornado_id: Option<u32>,
    lancamento_origem_id: Option<u32>,
    usuario_id: u32,
) -> Result<u32, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let tipo_str = tipo.as_db_str();
    let dl = naivedate_to_str(data_lancamento);
    let dc = naivedate_to_str(data_competencia);
    let sql = r#"INSERT INTO lancamentos
        (tenant_id, tipo, conta_receber_id, conta_pagar_id, valor, data_lancamento,
         data_competencia, descricao, plano_conta_id, centro_custo_id,
         forma_pagamento, lancamento_estornado_id, lancamento_origem_id, usuario_id)
       VALUES (:tid, :a, :b, :c, :d, :e, :f, :g, :h, :i, :j, :k, :l, :m)"#;
    conn.exec_drop(
        sql,
        params! {
            "tid" => tenant_id,
            "a" => tipo_str,
            "b" => conta_receber_id,
            "c" => conta_pagar_id,
            "d" => valor,
            "e" => dl,
            "f" => dc,
            "g" => descricao,
            "h" => plano_conta_id,
            "i" => centro_custo_id,
            "j" => forma_pagamento,
            "k" => lancamento_estornado_id,
            "l" => lancamento_origem_id,
            "m" => usuario_id,
        },
    )
    .map_err(err)?;
    Ok(conn.last_insert_id() as u32)
}

pub fn obter_lancamento(tenant_id: i32, id: u32) -> Result<Option<Lancamento>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<Row> = conn
        .exec_first(
            r#"SELECT id, tipo, conta_receber_id, conta_pagar_id, valor,
                      data_lancamento, data_competencia, descricao, plano_conta_id,
                      centro_custo_id, forma_pagamento, lancamento_estornado_id,
                      lancamento_origem_id, usuario_id, data_criacao
               FROM lancamentos WHERE id = ? AND tenant_id = ?"#,
            (id, tenant_id),
        )
        .map_err(err)?;
    match row {
        Some(r) => Ok(Some(Lancamento {
            id: get_i64(&r, 0)? as u32,
            tipo: TipoLancamento::from_db_str(&get_str(&r, 1)?),
            conta_receber_id: get_opt_i64(&r, 2)?.map(|n| n as u32),
            conta_pagar_id: get_opt_i64(&r, 3)?.map(|n| n as u32),
            valor: get_i64(&r, 4)?,
            data_lancamento: get_date(&r, 5)?,
            data_competencia: get_date(&r, 6)?,
            descricao: get_str(&r, 7)?,
            plano_conta_id: get_i64(&r, 8)? as u32,
            centro_custo_id: get_i64(&r, 9)? as u32,
            forma_pagamento: get_opt_str(&r, 10)?,
            lancamento_estornado_id: get_opt_i64(&r, 11)?.map(|n| n as u32),
            lancamento_origem_id: get_opt_i64(&r, 12)?.map(|n| n as u32),
            usuario_id: get_i64(&r, 13)? as u32,
            data_criacao: get_dt(&r, 14)?,
        })),
        None => Ok(None),
    }
}

// =============================================================================
// Alertas
// =============================================================================

pub fn listar_alertas_financeiros_ativos(
    tenant_id: i32,
) -> Result<Vec<AlertaFinanceiro>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let rows: Vec<Row> = conn
        .exec(
            "SELECT id, codigo, titulo, descricao, severidade, condicao_tipo,
                    condicao_threshold_dias, condicao_threshold_valor, ativo, data_criacao
             FROM alertas_financeiros WHERE tenant_id = ? AND ativo = TRUE ORDER BY id",
            (tenant_id,),
        )
        .map_err(err)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(AlertaFinanceiro {
            id: get_i64(&r, 0)? as u32,
            codigo: get_str(&r, 1)?,
            titulo: get_str(&r, 2)?,
            descricao: get_opt_str(&r, 3)?,
            severidade: Severidade::from_db_str(&get_str(&r, 4)?),
            condicao_tipo: get_str(&r, 5)?,
            condicao_threshold_dias: get_opt_i64(&r, 6)?.map(|n| n as u32),
            condicao_threshold_valor: get_opt_i64(&r, 7)?,
            ativo: get_bool(&r, 8)?,
            data_criacao: get_dt(&r, 9)?,
        });
    }
    Ok(out)
}

pub fn inserir_ocorrencia_financeira(
    tenant_id: i32,
    alerta_id: u32,
    entidade_tipo: TipoEntidadeAlerta,
    entidade_id: u32,
    severidade: Severidade,
    mensagem: &str,
    contexto: Option<&serde_json::Value>,
) -> Result<u32, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let ctx = contexto.map(|c| c.to_string());
    conn.exec_drop(
        r#"INSERT INTO alertas_financeiros_ocorrencias
            (tenant_id, alerta_id, entidade_tipo, entidade_id, severidade, mensagem, contexto_json)
           VALUES (?, ?, ?, ?, ?, ?, ?)"#,
        (
            tenant_id,
            alerta_id,
            entidade_tipo.as_db_str(),
            entidade_id,
            severidade.as_db_str(),
            mensagem,
            ctx.as_deref(),
        ),
    )
    .map_err(err)?;
    Ok(conn.last_insert_id() as u32)
}

pub fn listar_ocorrencias_financeiras_pendentes(
    tenant_id: i32,
    limite: u32,
) -> Result<Vec<AlertaFinanceiroOcorrencia>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let rows: Vec<Row> = conn
        .exec(
            r#"SELECT id, alerta_id, entidade_tipo, entidade_id, data_ocorrencia,
                      severidade, mensagem, contexto_json, visualizado, data_visualizacao,
                      resolvido, data_resolucao
               FROM alertas_financeiros_ocorrencias
               WHERE tenant_id = ? AND resolvido = FALSE
               ORDER BY FIELD(severidade,'CRITICAL','WARNING','INFO'), data_ocorrencia DESC
               LIMIT ?"#,
            (tenant_id, limite),
        )
        .map_err(err)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(AlertaFinanceiroOcorrencia {
            id: get_i64(&r, 0)? as u32,
            alerta_id: get_i64(&r, 1)? as u32,
            entidade_tipo: TipoEntidadeAlerta::from_db_str(&get_str(&r, 2)?),
            entidade_id: get_i64(&r, 3)? as u32,
            data_ocorrencia: get_dt(&r, 4)?,
            severidade: Severidade::from_db_str(&get_str(&r, 5)?),
            mensagem: get_str(&r, 6)?,
            contexto_json: get_opt_json(&r, 7)?,
            visualizado: get_bool(&r, 8)?,
            data_visualizacao: get_opt_dt(&r, 9)?,
            resolvido: get_bool(&r, 10)?,
            data_resolucao: get_opt_dt(&r, 11)?,
        });
    }
    Ok(out)
}

pub fn marcar_ocorrencia_financeira_resolvida(
    tenant_id: i32,
    id: u32,
) -> Result<bool, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let affected = conn.exec_iter(
        "UPDATE alertas_financeiros_ocorrencias SET resolvido = TRUE, data_resolucao = NOW() WHERE id = ? AND tenant_id = ?",
        (id, tenant_id),
    ).map_err(err)?.affected_rows();
    Ok(affected > 0)
}
