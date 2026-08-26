// src/cotacao_orcamento/repository.rs
//
// Acesso ao DB do módulo Cotação/Orçamento.

use mysql::params;
use mysql::prelude::Queryable;

use super::models::{
    Aprovacao, CategoriaItem, Cotacao, CotacaoAnexo, CotacaoCompleta, CotacaoItem, Decisao,
    HistoricoEntrada, Orcamento, OrcamentoCompleto, OrcamentoItem, StatusCotacao, StatusOrcamento,
    TipoAnexo,
};
use crate::banco_de_dados::conexao::obter_conexao;
use crate::servicos::ErroAplicacao;

// =============================================================================
// Cotações
// =============================================================================

pub fn criar_cotacao(
    tenant_id: i32,
    os_id: Option<u32>,
    cliente_id: u32,
    descricao: &str,
    observacoes: Option<&str>,
    criado_por: Option<u32>,
    criado_por_username: Option<&str>,
) -> Result<Cotacao, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        r#"INSERT INTO cotacoes
           (os_id, cliente_id, descricao, observacoes, status,
            criado_por, criado_por_username, tenant_id)
           VALUES (:os, :cid, :desc, :obs, 'RASCUNHO', :cp, :cpu, :tid)"#,
        params! {
            "os" => os_id,
            "cid" => cliente_id,
            "desc" => descricao,
            "obs" => observacoes,
            "cp" => criado_por,
            "cpu" => criado_por_username,
            "tid" => tenant_id,
        },
    )
    .map_err(ErroAplicacao::from)?;
    let id = conn.last_insert_id() as u32;
    obter_cotacao(tenant_id, id)?
        .ok_or(ErroAplicacao::Desconhecido("cotação não encontrada".into()))
}

pub fn obter_cotacao(tenant_id: i32, id: u32) -> Result<Option<Cotacao>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<Cotacao> = conn
        .exec_first(
            r#"SELECT id, os_id, cliente_id, descricao, observacoes, status,
                      criado_por, criado_por_username,
                      decidido_por, decidido_por_username,
                      DATE_FORMAT(decidido_em, '%Y-%m-%dT%H:%i:%s.%fZ') AS decidido_em,
                      decisao_observacao, orcamento_id,
                      DATE_FORMAT(created_at, '%Y-%m-%dT%H:%i:%s.%fZ') AS created_at,
                      DATE_FORMAT(updated_at, '%Y-%m-%dT%H:%i:%s.%fZ') AS updated_at
               FROM cotacoes WHERE tenant_id = :tid AND id = :id"#,
            params! { "id" => id, "tid" => tenant_id },
        )
        .map_err(ErroAplicacao::from)?;
    Ok(row)
}

pub fn listar_cotacoes(
    tenant_id: i32,
    status: Option<&str>,
    cliente_id: Option<u32>,
    limite: u32,
) -> Result<Vec<Cotacao>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let mut sql = String::from(
        r#"SELECT id, os_id, cliente_id, descricao, observacoes, status,
                  criado_por, criado_por_username,
                  decidido_por, decidido_por_username,
                  DATE_FORMAT(decidido_em, '%Y-%m-%dT%H:%i:%s.%fZ') AS decidido_em,
                  decisao_observacao, orcamento_id,
                  DATE_FORMAT(created_at, '%Y-%m-%dT%H:%i:%s.%fZ') AS created_at,
                  DATE_FORMAT(updated_at, '%Y-%m-%dT%H:%i:%s.%fZ') AS updated_at
           FROM cotacoes WHERE tenant_id = :tid"#,
    );
    let mut conds: Vec<String> = Vec::new();
    if status.is_some() {
        conds.push("status = :status".into());
    }
    if cliente_id.is_some() {
        conds.push("cliente_id = :cid".into());
    }
    if !conds.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conds.join(" AND "));
    }
    sql.push_str(" ORDER BY created_at DESC LIMIT :lim");

    let mut p = std::collections::HashMap::<&str, mysql::Value>::new();
    if let Some(s) = status {
        p.insert("status", s.into());
    }
    if let Some(c) = cliente_id {
        p.insert("cid", c.into());
    }
    p.insert("lim", limite.into());

    // Use exec_map with params! macro
    let rows: Vec<Cotacao> = if let Some(s) = status {
        if let Some(c) = cliente_id {
            conn.exec(
                &sql,
                params! { "status" => s, "cid" => c, "lim" => limite,
                    "tid" => tenant_id,
                },
            )
            .map_err(ErroAplicacao::from)?
        } else {
            conn.exec(
                &sql,
                params! { "status" => s, "lim" => limite,
                    "tid" => tenant_id,
                },
            )
            .map_err(ErroAplicacao::from)?
        }
    } else if let Some(c) = cliente_id {
        conn.exec(
            &sql,
            params! { "cid" => c, "lim" => limite,
                "tid" => tenant_id,
            },
        )
        .map_err(ErroAplicacao::from)?
    } else {
        conn.exec(
            &sql,
            params! { "lim" => limite,
                "tid" => tenant_id,
            },
        )
        .map_err(ErroAplicacao::from)?
    };
    Ok(rows)
}

pub fn atualizar_status_cotacao(
    tenant_id: i32,
    id: u32,
    novo_status: StatusCotacao,
) -> Result<(), ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        "UPDATE cotacoes SET status = :s WHERE tenant_id = :tid AND id = :id",
        params! { "s" => novo_status.as_db_str(), "id" => id, "tid" => tenant_id },
    )
    .map_err(ErroAplicacao::from)?;
    Ok(())
}

pub fn definir_orcamento_da_cotacao(
    tenant_id: i32,
    cotacao_id: u32,
    orcamento_id: u32,
) -> Result<(), ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        "UPDATE cotacoes SET orcamento_id = :oid WHERE tenant_id = :tid AND id = :cid",
        params! { "oid" => orcamento_id, "tid" => tenant_id, "cid" => cotacao_id },
    )
    .map_err(ErroAplicacao::from)?;
    Ok(())
}

pub fn remover_cotacao(tenant_id: i32, id: u32) -> Result<bool, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let affected = conn
        .exec_iter(
            "DELETE FROM cotacoes WHERE tenant_id = :tid AND id = :id",
            params! { "id" => id,
                "tid" => tenant_id,
            },
        )
        .map_err(ErroAplicacao::from)?
        .affected_rows();
    Ok(affected > 0)
}

// =============================================================================
// Itens de cotação
// =============================================================================

pub fn adicionar_item_cotacao(
    tenant_id: i32,
    cotacao_id: u32,
    nome: &str,
    quantidade: f64,
    valor_estimado: f64,
    observacao: Option<&str>,
) -> Result<CotacaoItem, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        r#"INSERT INTO cotacao_itens
           (cotacao_id, nome, quantidade, valor_estimado, observacao, ordem, tenant_id)
           SELECT :cid, :nome, :qtd, :valor, :obs,
                  COALESCE(MAX(ordem), -1) + 1, :tid
           FROM cotacao_itens WHERE tenant_id = :tid AND cotacao_id = :cid"#,
        params! {
            "cid" => cotacao_id,
            "nome" => nome,
            "qtd" => quantidade,
            "valor" => valor_estimado,
            "obs" => observacao,
            "tid" => tenant_id,
        },
    )
    .map_err(ErroAplicacao::from)?;
    let id = conn.last_insert_id() as u32;
    obter_item_cotacao(tenant_id, id)?
        .ok_or(ErroAplicacao::Desconhecido("item não encontrado".into()))
}

pub fn obter_item_cotacao(tenant_id: i32, id: u32) -> Result<Option<CotacaoItem>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<CotacaoItem> = conn
        .exec_first(
            "SELECT id, cotacao_id, nome, quantidade, valor_estimado, observacao, ordem FROM cotacao_itens WHERE tenant_id = :tid AND id = :id",
            params! { "id" => id,
            "tid" => tenant_id,
        },
        )
        .map_err(ErroAplicacao::from)?;
    Ok(row)
}

pub fn listar_itens_cotacao(
    tenant_id: i32,
    cotacao_id: u32,
) -> Result<Vec<CotacaoItem>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let rows: Vec<CotacaoItem> = conn.exec(
        "SELECT id, cotacao_id, nome, quantidade, valor_estimado, observacao, ordem FROM cotacao_itens WHERE tenant_id = :tid AND cotacao_id = :cid ORDER BY ordem, id",
        params! { "cid" => cotacao_id,
            "tid" => tenant_id,
        },
    )
    .map_err(ErroAplicacao::from)?;
    Ok(rows)
}

pub fn remover_item_cotacao(tenant_id: i32, id: u32) -> Result<bool, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let affected = conn
        .exec_iter(
            "DELETE FROM cotacao_itens WHERE tenant_id = :tid AND id = :id",
            params! { "id" => id,
                "tid" => tenant_id,
            },
        )
        .map_err(ErroAplicacao::from)?
        .affected_rows();
    Ok(affected > 0)
}

// =============================================================================
// Anexos de cotação
// =============================================================================

pub fn adicionar_anexo(
    tenant_id: i32,
    cotacao_id: u32,
    tipo: TipoAnexo,
    nome: &str,
    arquivo_path: &str,
    tamanho_bytes: Option<i64>,
    mime_type: Option<&str>,
    hash_sha256: Option<&str>,
    usuario_id: Option<u32>,
    username: Option<&str>,
) -> Result<CotacaoAnexo, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        r#"INSERT INTO cotacao_anexos
           (cotacao_id, tipo, nome, arquivo_path, tamanho_bytes, mime_type,
            hash_sha256, usuario_id, username, tenant_id)
           VALUES (:cid, :tipo, :nome, :path, :tam, :mime, :hash, :uid, :user, :tid)"#,
        params! {
            "cid" => cotacao_id,
            "tipo" => tipo.as_db_str(),
            "nome" => nome,
            "path" => arquivo_path,
            "tam" => tamanho_bytes,
            "mime" => mime_type,
            "hash" => hash_sha256,
            "uid" => usuario_id,
            "user" => username,
            "tid" => tenant_id,
        },
    )
    .map_err(ErroAplicacao::from)?;
    let id = conn.last_insert_id() as u32;
    obter_anexo(tenant_id, id)?.ok_or(ErroAplicacao::Desconhecido("anexo não encontrado".into()))
}

pub fn obter_anexo(tenant_id: i32, id: u32) -> Result<Option<CotacaoAnexo>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<CotacaoAnexo> = conn
        .exec_first(
            r#"SELECT id, cotacao_id, tipo, nome, arquivo_path, tamanho_bytes, mime_type,
                      hash_sha256, usuario_id, username,
                      DATE_FORMAT(data_upload, '%Y-%m-%dT%H:%i:%s.%fZ') AS data_upload
               FROM cotacao_anexos WHERE tenant_id = :tid AND id = :id"#,
            params! { "id" => id,
                "tid" => tenant_id,
            },
        )
        .map_err(ErroAplicacao::from)?;
    Ok(row)
}

pub fn listar_anexos(tenant_id: i32, cotacao_id: u32) -> Result<Vec<CotacaoAnexo>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let rows: Vec<CotacaoAnexo> = conn.exec(
        r#"SELECT id, cotacao_id, tipo, nome, arquivo_path, tamanho_bytes, mime_type,
                  hash_sha256, usuario_id, username,
                  DATE_FORMAT(data_upload, '%Y-%m-%dT%H:%i:%s.%fZ') AS data_upload
           FROM cotacao_anexos WHERE tenant_id = :tid AND cotacao_id = :cid ORDER BY data_upload DESC"#,
        params! { "cid" => cotacao_id,
            "tid" => tenant_id,
        },
    )
    .map_err(ErroAplicacao::from)?;
    Ok(rows)
}

pub fn remover_anexo(tenant_id: i32, id: u32) -> Result<bool, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let affected = conn
        .exec_iter(
            "DELETE FROM cotacao_anexos WHERE tenant_id = :tid AND id = :id",
            params! { "id" => id,
                "tid" => tenant_id,
            },
        )
        .map_err(ErroAplicacao::from)?
        .affected_rows();
    Ok(affected > 0)
}

// =============================================================================
// Cotação completa (helper)
// =============================================================================

pub fn obter_cotacao_completa(
    tenant_id: i32,
    id: u32,
) -> Result<Option<CotacaoCompleta>, ErroAplicacao> {
    let cotacao = match obter_cotacao(tenant_id, id)? {
        Some(c) => c,
        None => return Ok(None),
    };
    let itens = listar_itens_cotacao(tenant_id, id)?;
    let anexos = listar_anexos(tenant_id, id)?;
    Ok(Some(CotacaoCompleta {
        cotacao,
        itens,
        anexos,
    }))
}

// =============================================================================
// Orçamentos
// =============================================================================

pub fn criar_orcamento(
    tenant_id: i32,
    cliente_id: u32,
    cotacao_origem_id: Option<u32>,
    observacoes: Option<&str>,
    criado_por: Option<u32>,
    criado_por_username: Option<&str>,
) -> Result<Orcamento, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        r#"INSERT INTO orcamentos
           (cliente_id, total, subtotal, status, cotacao_origem_id, observacoes,
            criado_por, criado_por_username, tenant_id)
           VALUES (:cid, 0, 0, 'RASCUNHO', :cot, :obs, :cp, :cpu, :tid)"#,
        params! {
            "cid" => cliente_id,
            "cot" => cotacao_origem_id,
            "obs" => observacoes,
            "cp" => criado_por,
            "cpu" => criado_por_username,
            "tid" => tenant_id,
        },
    )
    .map_err(ErroAplicacao::from)?;
    let id = conn.last_insert_id() as u32;
    obter_orcamento(tenant_id, id)?.ok_or(ErroAplicacao::Desconhecido(
        "orçamento não encontrado".into(),
    ))
}

pub fn obter_orcamento(tenant_id: i32, id: u32) -> Result<Option<Orcamento>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<Orcamento> = conn
        .exec_first(
            r#"SELECT id, cliente_id, total, status, desconto, subtotal, impostos_estimado,
                      observacoes, cotacao_origem_id, criado_por, criado_por_username,
                      decidido_por, decidido_por_username,
                      DATE_FORMAT(decidido_em, '%Y-%m-%dT%H:%i:%s.%fZ') AS decidido_em,
                      decisao_observacao,
                      DATE_FORMAT(created_at, '%Y-%m-%dT%H:%i:%s.%fZ') AS created_at,
                      DATE_FORMAT(updated_at, '%Y-%m-%dT%H:%i:%s.%fZ') AS updated_at
               FROM orcamentos WHERE tenant_id = :tid AND id = :id"#,
            params! { "id" => id,
                "tid" => tenant_id,
            },
        )
        .map_err(ErroAplicacao::from)?;
    Ok(row)
}

pub fn listar_orcamentos(
    tenant_id: i32,
    status: Option<&str>,
    cliente_id: Option<u32>,
    limite: u32,
) -> Result<Vec<Orcamento>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let base_sql = r#"SELECT id, cliente_id, total, status, desconto, subtotal, impostos_estimado,
                  observacoes, cotacao_origem_id, criado_por, criado_por_username,
                  decidido_por, decidido_por_username,
                  DATE_FORMAT(decidido_em, '%Y-%m-%dT%H:%i:%s.%fZ') AS decidido_em,
                  decisao_observacao,
                  DATE_FORMAT(created_at, '%Y-%m-%dT%H:%i:%s.%fZ') AS created_at,
                  DATE_FORMAT(updated_at, '%Y-%m-%dT%H:%i:%s.%fZ') AS updated_at
           FROM orcamentos WHERE tenant_id = :tid"#;
    let rows: Vec<Orcamento> = match (status, cliente_id) {
        (Some(s), Some(c)) => conn
            .exec(
                format!(
                    "{} WHERE status = :s AND cliente_id = :c ORDER BY created_at DESC LIMIT :lim",
                    base_sql
                ),
                params! { "s" => s, "c" => c, "lim" => limite,
                    "tid" => tenant_id,
                },
            )
            .map_err(ErroAplicacao::from)?,
        (Some(s), None) => conn
            .exec(
                format!(
                    "{} WHERE status = :s ORDER BY created_at DESC LIMIT :lim",
                    base_sql
                ),
                params! { "s" => s, "lim" => limite,
                    "tid" => tenant_id,
                },
            )
            .map_err(ErroAplicacao::from)?,
        (None, Some(c)) => conn
            .exec(
                format!(
                    "{} WHERE cliente_id = :c ORDER BY created_at DESC LIMIT :lim",
                    base_sql
                ),
                params! { "c" => c, "lim" => limite,
                    "tid" => tenant_id,
                },
            )
            .map_err(ErroAplicacao::from)?,
        (None, None) => conn
            .exec(
                format!("{} ORDER BY created_at DESC LIMIT :lim", base_sql),
                params! { "lim" => limite,
                    "tid" => tenant_id,
                },
            )
            .map_err(ErroAplicacao::from)?,
    };
    Ok(rows)
}

pub fn atualizar_status_orcamento(
    tenant_id: i32,
    id: u32,
    novo_status: StatusOrcamento,
) -> Result<(), ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        "UPDATE orcamentos SET status = :s WHERE tenant_id = :tid AND id = :id",
        params! { "s" => novo_status.as_db_str(), "id" => id,
            "tid" => tenant_id,
        },
    )
    .map_err(ErroAplicacao::from)?;
    Ok(())
}

pub fn atualizar_totais_orcamento(
    tenant_id: i32,
    id: u32,
    subtotal: f64,
    desconto: f64,
    impostos_estimado: f64,
    total: f64,
) -> Result<(), ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        r#"UPDATE orcamentos
           SET subtotal = :sub, desconto = :desc, impostos_estimado = :imp, total = :tot
           WHERE tenant_id = :tid AND id = :id"#,
        params! {
            "sub" => subtotal,
            "desc" => desconto,
            "imp" => impostos_estimado,
            "tot" => total,
            "id" => id,
            "tid" => tenant_id,
        },
    )
    .map_err(ErroAplicacao::from)?;
    Ok(())
}

pub fn registrar_decisao_orcamento(
    tenant_id: i32,
    id: u32,
    decisao: Decisao,
    observacao: Option<&str>,
    decidido_por: Option<u32>,
    decidido_por_username: Option<&str>,
) -> Result<(), ErroAplicacao> {
    let mut conn = obter_conexao()?;
    // 1. Atualiza a decisão no orçamento
    conn.exec_drop(
        r#"UPDATE orcamentos
           SET decidido_por = :uid, decidido_por_username = :user,
               decidido_em = NOW(), decisao_observacao = :obs,
               status = :status
           WHERE tenant_id = :tid AND id = :id"#,
        params! {
            "uid" => decidido_por,
            "user" => decidido_por_username,
            "obs" => observacao,
            "status" => decisao.as_db_str(),
            "id" => id,
            "tid" => tenant_id,
        },
    )
    .map_err(ErroAplicacao::from)?;
    // 2. Registra na tabela de aprovações (audit)
    conn.exec_drop(
        r#"INSERT INTO orcamento_aprovacoes
           (orcamento_id, decisao, observacao, decidido_por, decidido_por_username, tenant_id)
           VALUES (:oid, :dec, :obs, :uid, :user, :tid)"#,
        params! {
            "oid" => id,
            "dec" => decisao.as_db_str(),
            "obs" => observacao,
            "uid" => decidido_por,
            "user" => decidido_por_username,
            "tid" => tenant_id,
        },
    )
    .map_err(ErroAplicacao::from)?;
    Ok(())
}

pub fn registrar_historico(
    tenant_id: i32,
    orcamento_id: u32,
    status_anterior: Option<StatusOrcamento>,
    status_novo: StatusOrcamento,
    observacao: Option<&str>,
    usuario_id: Option<u32>,
    username: Option<&str>,
) -> Result<(), ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        r#"INSERT INTO orcamento_historico
           (orcamento_id, status_anterior, status_novo, observacao, usuario_id, username, tenant_id)
           VALUES (:oid, :ant, :nov, :obs, :uid, :user, :tid)"#,
        params! {
            "oid" => orcamento_id,
            "ant" => status_anterior.map(|s| s.as_db_str()),
            "nov" => status_novo.as_db_str(),
            "obs" => observacao,
            "uid" => usuario_id,
            "user" => username,
            "tid" => tenant_id,
        },
    )
    .map_err(ErroAplicacao::from)?;
    Ok(())
}

pub fn listar_aprovacoes(
    tenant_id: i32,
    orcamento_id: u32,
) -> Result<Vec<Aprovacao>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let rows: Vec<Aprovacao> = conn
        .exec(
            r#"SELECT id, orcamento_id, decisao, observacao, decidido_por, decidido_por_username,
                  DATE_FORMAT(data_hora, '%Y-%m-%dT%H:%i:%s.%fZ') AS data_hora
           FROM orcamento_aprovacoes
           WHERE tenant_id = :tid AND orcamento_id = :oid
           ORDER BY data_hora DESC"#,
            params! { "tid" => tenant_id, "oid" => orcamento_id },
        )
        .map_err(ErroAplicacao::from)?;
    Ok(rows)
}

pub fn listar_historico(
    tenant_id: i32,
    orcamento_id: u32,
) -> Result<Vec<HistoricoEntrada>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let rows: Vec<HistoricoEntrada> = conn
        .exec(
            r#"SELECT id, orcamento_id, status_anterior, status_novo, observacao,
                  usuario_id, username,
                  DATE_FORMAT(data_hora, '%Y-%m-%dT%H:%i:%s.%fZ') AS data_hora
           FROM orcamento_historico
           WHERE tenant_id = :tid AND orcamento_id = :oid
           ORDER BY data_hora DESC"#,
            params! { "tid" => tenant_id, "oid" => orcamento_id },
        )
        .map_err(ErroAplicacao::from)?;
    Ok(rows)
}

pub fn remover_orcamento(tenant_id: i32, id: u32) -> Result<bool, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let affected = conn
        .exec_iter(
            "DELETE FROM orcamentos WHERE tenant_id = :tid AND id = :id",
            params! { "id" => id,
                "tid" => tenant_id,
            },
        )
        .map_err(ErroAplicacao::from)?
        .affected_rows();
    Ok(affected > 0)
}

// =============================================================================
// Itens de orçamento
// =============================================================================

pub fn adicionar_item_orcamento(
    tenant_id: i32,
    orcamento_id: u32,
    descricao: &str,
    quantidade: u32,
    preco_unitario: f64,
    categoria: CategoriaItem,
    peca_id: Option<u32>,
    servico_id: Option<u32>,
    observacao: Option<&str>,
) -> Result<OrcamentoItem, ErroAplicacao> {
    let preco_total = quantidade as f64 * preco_unitario;
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        r#"INSERT INTO orcamento_items
           (orcamento_id, descricao, quantidade, preco_unitario, preco_total, categoria,
            peca_id, servico_id, observacao, tenant_id)
           VALUES (:oid, :desc, :qtd, :pu, :pt, :cat, :pid, :sid, :obs, :tid)"#,
        params! {
            "oid" => orcamento_id,
            "desc" => descricao,
            "qtd" => quantidade,
            "pu" => preco_unitario,
            "pt" => preco_total,
            "cat" => categoria.as_db_str(),
            "pid" => peca_id,
            "sid" => servico_id,
            "obs" => observacao,
            "tid" => tenant_id,
        },
    )
    .map_err(ErroAplicacao::from)?;
    let id = conn.last_insert_id() as u32;
    obter_item_orcamento(tenant_id, id)?
        .ok_or(ErroAplicacao::Desconhecido("item não encontrado".into()))
}

pub fn obter_item_orcamento(
    tenant_id: i32,
    id: u32,
) -> Result<Option<OrcamentoItem>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<OrcamentoItem> = conn
        .exec_first(
            r#"SELECT id, orcamento_id, descricao, quantidade, preco_unitario, preco_total,
                      categoria, peca_id, servico_id, observacao
               FROM orcamento_items WHERE tenant_id = :tid AND id = :id"#,
            params! { "id" => id,
                "tid" => tenant_id,
            },
        )
        .map_err(ErroAplicacao::from)?;
    Ok(row)
}

pub fn listar_itens_orcamento(
    tenant_id: i32,
    orcamento_id: u32,
) -> Result<Vec<OrcamentoItem>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let rows: Vec<OrcamentoItem> = conn
        .exec(
            r#"SELECT id, orcamento_id, descricao, quantidade, preco_unitario, preco_total,
                  categoria, peca_id, servico_id, observacao
           FROM orcamento_items WHERE tenant_id = :tid AND orcamento_id = :oid ORDER BY id"#,
            params! { "oid" => orcamento_id,
                "tid" => tenant_id,
            },
        )
        .map_err(ErroAplicacao::from)?;
    Ok(rows)
}

pub fn remover_item_orcamento(tenant_id: i32, id: u32) -> Result<bool, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let affected = conn
        .exec_iter(
            "DELETE FROM orcamento_items WHERE tenant_id = :tid AND id = :id",
            params! { "id" => id,
                "tid" => tenant_id,
            },
        )
        .map_err(ErroAplicacao::from)?
        .affected_rows();
    Ok(affected > 0)
}

pub fn obter_orcamento_completo(
    tenant_id: i32,
    id: u32,
) -> Result<Option<OrcamentoCompleto>, ErroAplicacao> {
    let orcamento = match obter_orcamento(tenant_id, id)? {
        Some(o) => o,
        None => return Ok(None),
    };
    let itens = listar_itens_orcamento(tenant_id, id)?;
    Ok(Some(OrcamentoCompleto { orcamento, itens }))
}

// =============================================================================
// Dashboard
// =============================================================================

pub fn contar_cotacoes_por_status(
    tenant_id: i32,
) -> Result<(u64, u64, u64, u64, u64), ErroAplicacao> {
    let mut conn = obter_conexao()?;
    // (abertas, aguardando_aprovacao, aprovadas, rejeitadas, finalizadas)
    // "abertas" = RASCUNHO + AGUARDANDO_COTACAO + COTADO
    let abertas: u64 = conn
        .exec_first(
            "SELECT COUNT(*) FROM cotacoes WHERE tenant_id = ? AND status IN ('RASCUNHO','AGUARDANDO_COTACAO','COTADO')",
            (tenant_id,),
        )
        .map_err(ErroAplicacao::from)?
        .unwrap_or(0u64);
    let aguardando: u64 = conn
        .exec_first(
            "SELECT COUNT(*) FROM cotacoes WHERE tenant_id = ? AND status = 'AGUARDANDO_APROVACAO'",
            (tenant_id,),
        )
        .map_err(ErroAplicacao::from)?
        .unwrap_or(0u64);
    let aprovadas: u64 = conn
        .exec_first(
            "SELECT COUNT(*) FROM cotacoes WHERE tenant_id = ? AND status = 'APROVADO'",
            (tenant_id,),
        )
        .map_err(ErroAplicacao::from)?
        .unwrap_or(0u64);
    let rejeitadas: u64 = conn
        .exec_first(
            "SELECT COUNT(*) FROM cotacoes WHERE tenant_id = ? AND status = 'REJEITADO'",
            (tenant_id,),
        )
        .map_err(ErroAplicacao::from)?
        .unwrap_or(0u64);
    let finalizadas: u64 = conn
        .exec_first(
            "SELECT COUNT(*) FROM cotacoes WHERE tenant_id = ? AND status = 'FINALIZADO'",
            (tenant_id,),
        )
        .map_err(ErroAplicacao::from)?
        .unwrap_or(0u64);
    Ok((abertas, aguardando, aprovadas, rejeitadas, finalizadas))
}
