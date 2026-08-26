// src/crm/repository.rs
//
// Camada de acesso ao DB do CRM. Apenas queries, sem regra de negócio.
// Toda mutação aqui retorna `Result<_, ErroAplicacao>`. A camada de
// service é que decide se registra audit, timeline, etc.
//
// P2.6.2a — tenant_id obrigatório em toda query (defense in depth).

use mysql::params;
use mysql::prelude::Queryable;

use super::models::{Contato, Equipamento, Observacao, Tag, TimelineEvento};
use crate::banco_de_dados::conexao::obter_conexao;
use crate::servicos::ErroAplicacao;

// =============================================================================
// Timeline
// =============================================================================

/// Insere um evento na timeline. Não dispara triggers (responsabilidade
/// do service).
pub fn inserir_evento_timeline(
    tenant_id: i32,
    cliente_id: u32,
    tipo_db: &str,
    descricao: &str,
    payload: Option<&serde_json::Value>,
    usuario_id: Option<u32>,
    username: Option<&str>,
) -> Result<u64, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        r#"INSERT INTO cliente_timeline_eventos
           (tenant_id, cliente_id, tipo, descricao, payload, usuario_id, username)
           VALUES (:tid, :cid, :tipo, :desc, :payload, :uid, :user)"#,
        params! {
            "tid" => tenant_id,
            "cid" => cliente_id,
            "tipo" => tipo_db,
            "desc" => descricao,
            "payload" => payload.map(|v| v.to_string()),
            "uid" => usuario_id,
            "user" => username,
        },
    )
    .map_err(ErroAplicacao::from)?;
    Ok(conn.last_insert_id())
}

/// Lista eventos da timeline de um cliente (mais recentes primeiro).
pub fn listar_timeline(
    tenant_id: i32,
    cliente_id: u32,
    limite: u32,
) -> Result<Vec<TimelineEvento>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let rows: Vec<TimelineEvento> = conn
        .exec(
            r#"SELECT id, cliente_id, tipo, descricao, payload, usuario_id, username,
                  DATE_FORMAT(data_hora, '%Y-%m-%dT%H:%i:%s.%fZ') AS data_hora
           FROM cliente_timeline_eventos
           WHERE tenant_id = :tid AND cliente_id = :cid
           ORDER BY data_hora DESC
           LIMIT :lim"#,
            params! { "tid" => tenant_id, "cid" => cliente_id, "lim" => limite },
        )
        .map_err(ErroAplicacao::from)?;
    Ok(rows)
}

// =============================================================================
// Observações
// =============================================================================

pub fn criar_observacao(
    tenant_id: i32,
    cliente_id: u32,
    conteudo: &str,
    autor_id: Option<u32>,
    autor_username: Option<&str>,
) -> Result<Observacao, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        r#"INSERT INTO cliente_observacoes (tenant_id, cliente_id, conteudo, autor_id, autor_username)
           VALUES (:tid, :cid, :cont, :aid, :user)"#,
        params! {
            "tid" => tenant_id,
            "cid" => cliente_id,
            "cont" => conteudo,
            "aid" => autor_id,
            "user" => autor_username,
        },
    )
    .map_err(ErroAplicacao::from)?;
    let id = conn.last_insert_id() as u32;
    obter_observacao_por_id(tenant_id, id)?.ok_or(ErroAplicacao::Desconhecido(
        "observacao criada mas nao encontrada".into(),
    ))
}

pub fn obter_observacao_por_id(
    tenant_id: i32,
    id: u32,
) -> Result<Option<Observacao>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<Observacao> = conn
        .exec_first(
            r#"SELECT id, cliente_id, conteudo, autor_id, autor_username, editada,
                      vezes_editada,
                      DATE_FORMAT(created_at, '%Y-%m-%dT%H:%i:%s.%fZ') AS created_at,
                      DATE_FORMAT(updated_at, '%Y-%m-%dT%H:%i:%s.%fZ') AS updated_at
               FROM cliente_observacoes WHERE id = :id AND tenant_id = :tid"#,
            params! { "id" => id, "tid" => tenant_id },
        )
        .map_err(ErroAplicacao::from)?;
    Ok(row)
}

pub fn listar_observacoes(
    tenant_id: i32,
    cliente_id: u32,
    limite: u32,
) -> Result<Vec<Observacao>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let rows: Vec<Observacao> = conn
        .exec(
            r#"SELECT id, cliente_id, conteudo, autor_id, autor_username, editada,
                  vezes_editada,
                  DATE_FORMAT(created_at, '%Y-%m-%dT%H:%i:%s.%fZ') AS created_at,
                  DATE_FORMAT(updated_at, '%Y-%m-%dT%H:%i:%s.%fZ') AS updated_at
           FROM cliente_observacoes
           WHERE tenant_id = :tid AND cliente_id = :cid
           ORDER BY created_at DESC
           LIMIT :lim"#,
            params! { "tid" => tenant_id, "cid" => cliente_id, "lim" => limite },
        )
        .map_err(ErroAplicacao::from)?;
    Ok(rows)
}

/// Edita uma observação. Salva o histórico em `cliente_observacao_edicoes`
/// e incrementa `vezes_editada` + marca `editada=true`.
pub fn editar_observacao(
    tenant_id: i32,
    id: u32,
    conteudo_novo: &str,
    editor_id: Option<u32>,
    editor_username: Option<&str>,
) -> Result<Observacao, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    // 1. Buscar conteúdo atual (defense in depth: WHERE tenant_id)
    let atual: Option<(String,)> = conn
        .exec_first(
            "SELECT conteudo FROM cliente_observacoes WHERE id = :id AND tenant_id = :tid",
            params! { "id" => id, "tid" => tenant_id },
        )
        .map_err(ErroAplicacao::from)?;
    let conteudo_anterior = match atual {
        Some((s,)) => s,
        None => {
            return Err(ErroAplicacao::Desconhecido(
                "observacao nao encontrada neste tenant".into(),
            ))
        }
    };
    // 2. Salvar edição no audit
    conn.exec_drop(
        r#"INSERT INTO cliente_observacao_edicoes
           (tenant_id, observacao_id, conteudo_anterior, conteudo_novo, editor_id, editor_username)
           VALUES (:tid, :oid, :ant, :nov, :eid, :user)"#,
        params! {
            "tid" => tenant_id,
            "oid" => id,
            "ant" => &conteudo_anterior,
            "nov" => conteudo_novo,
            "eid" => editor_id,
            "user" => editor_username,
        },
    )
    .map_err(ErroAplicacao::from)?;
    // 3. Atualizar a observação (defense in depth)
    let affected = conn
        .exec_iter(
            r#"UPDATE cliente_observacoes
           SET conteudo = :cont, editada = TRUE, vezes_editada = vezes_editada + 1
           WHERE id = :id AND tenant_id = :tid"#,
            params! { "cont" => conteudo_novo, "id" => id, "tid" => tenant_id },
        )
        .map_err(ErroAplicacao::from)?
        .affected_rows();
    if affected == 0 {
        return Err(ErroAplicacao::Desconhecido(
            "observacao nao encontrada neste tenant".into(),
        ));
    }
    // 4. Retornar
    obter_observacao_por_id(tenant_id, id)?.ok_or(ErroAplicacao::Desconhecido(
        "observacao editada mas nao encontrada".into(),
    ))
}

// =============================================================================
// Tags
// =============================================================================

pub fn listar_tags(tenant_id: i32) -> Result<Vec<Tag>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let rows: Vec<Tag> = conn
        .exec(
            r#"SELECT id, nome, cor, descricao, publica,
                  DATE_FORMAT(created_at, '%Y-%m-%dT%H:%i:%s.%fZ') AS created_at
           FROM cliente_tags
           WHERE tenant_id = :tid
           ORDER BY nome"#,
            params! { "tid" => tenant_id },
        )
        .map_err(ErroAplicacao::from)?;
    Ok(rows)
}

pub fn criar_tag(
    tenant_id: i32,
    nome: &str,
    cor: &str,
    descricao: Option<&str>,
) -> Result<Tag, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        "INSERT INTO cliente_tags (tenant_id, nome, cor, descricao) VALUES (:tid, :n, :c, :d)",
        params! { "tid" => tenant_id, "n" => nome, "c" => cor, "d" => descricao },
    )
    .map_err(ErroAplicacao::from)?;
    let id = conn.last_insert_id() as u32;
    let row: Option<Tag> = conn
        .exec_first(
            r#"SELECT id, nome, cor, descricao, publica,
                      DATE_FORMAT(created_at, '%Y-%m-%dT%H:%i:%s.%fZ') AS created_at
               FROM cliente_tags WHERE id = :id AND tenant_id = :tid"#,
            params! { "id" => id, "tid" => tenant_id },
        )
        .map_err(ErroAplicacao::from)?;
    row.ok_or(ErroAplicacao::Desconhecido(
        "tag criada mas nao encontrada".into(),
    ))
}

pub fn listar_tags_do_cliente(tenant_id: i32, cliente_id: u32) -> Result<Vec<Tag>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let rows: Vec<Tag> = conn
        .exec(
            r#"SELECT t.id, t.nome, t.cor, t.descricao, t.publica,
                  DATE_FORMAT(t.created_at, '%Y-%m-%dT%H:%i:%s.%fZ') AS created_at
           FROM cliente_tags t
           JOIN cliente_tag_atribuicoes a ON a.tag_id = t.id
           WHERE a.tenant_id = :tid AND a.cliente_id = :cid
           ORDER BY t.nome"#,
            params! { "tid" => tenant_id, "cid" => cliente_id },
        )
        .map_err(ErroAplicacao::from)?;
    Ok(rows)
}

pub fn atribuir_tag(
    tenant_id: i32,
    cliente_id: u32,
    tag_id: u32,
    atribuido_por: Option<u32>,
    atribuido_por_username: Option<&str>,
) -> Result<(), ErroAplicacao> {
    let mut conn = obter_conexao()?;
    // INSERT IGNORE para idempotência
    conn.exec_drop(
        r#"INSERT IGNORE INTO cliente_tag_atribuicoes
           (tenant_id, cliente_id, tag_id, atribuido_por, atribuido_por_username)
           VALUES (:tid, :cid, :tid_tag, :uid, :user)"#,
        params! {
            "tid" => tenant_id,
            "cid" => cliente_id,
            "tid_tag" => tag_id,
            "uid" => atribuido_por,
            "user" => atribuido_por_username,
        },
    )
    .map_err(ErroAplicacao::from)?;
    Ok(())
}

pub fn remover_tag(tenant_id: i32, cliente_id: u32, tag_id: u32) -> Result<bool, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let affected = conn
        .exec_iter(
            "DELETE FROM cliente_tag_atribuicoes WHERE tenant_id = :tid AND cliente_id = :cid AND tag_id = :tid_tag",
            params! { "tid" => tenant_id, "cid" => cliente_id, "tid_tag" => tag_id },
        )
        .map_err(ErroAplicacao::from)?
        .affected_rows();
    Ok(affected > 0)
}

// =============================================================================
// Contatos
// =============================================================================

pub fn criar_contato(
    tenant_id: i32,
    cliente_id: u32,
    tipo: &str,
    valor: &str,
    rotulo: Option<&str>,
    principal: bool,
    observacao: Option<&str>,
) -> Result<Contato, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    // Se for principal, desmarca outros principais
    if principal {
        conn.exec_drop(
            "UPDATE cliente_contatos SET principal = FALSE WHERE tenant_id = :tid AND cliente_id = :cid",
            params! { "tid" => tenant_id, "cid" => cliente_id },
        )
        .map_err(ErroAplicacao::from)?;
    }
    conn.exec_drop(
        r#"INSERT INTO cliente_contatos
           (tenant_id, cliente_id, tipo, valor, rotulo, principal, observacao)
           VALUES (:tid, :cid, :tipo, :valor, :rot, :princ, :obs)"#,
        params! {
            "tid" => tenant_id,
            "cid" => cliente_id,
            "tipo" => tipo,
            "valor" => valor,
            "rot" => rotulo,
            "princ" => principal,
            "obs" => observacao,
        },
    )
    .map_err(ErroAplicacao::from)?;
    let id = conn.last_insert_id() as u32;
    obter_contato_por_id(tenant_id, id)?.ok_or(ErroAplicacao::Desconhecido(
        "contato criado mas nao encontrado".into(),
    ))
}

pub fn obter_contato_por_id(tenant_id: i32, id: u32) -> Result<Option<Contato>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<Contato> = conn
        .exec_first(
            r#"SELECT id, cliente_id, tipo, valor, rotulo, principal, observacao,
                      DATE_FORMAT(created_at, '%Y-%m-%dT%H:%i:%s.%fZ') AS created_at,
                      DATE_FORMAT(updated_at, '%Y-%m-%dT%H:%i:%s.%fZ') AS updated_at
               FROM cliente_contatos WHERE id = :id AND tenant_id = :tid"#,
            params! { "id" => id, "tid" => tenant_id },
        )
        .map_err(ErroAplicacao::from)?;
    Ok(row)
}

pub fn listar_contatos(tenant_id: i32, cliente_id: u32) -> Result<Vec<Contato>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let rows: Vec<Contato> = conn
        .exec(
            r#"SELECT id, cliente_id, tipo, valor, rotulo, principal, observacao,
                  DATE_FORMAT(created_at, '%Y-%m-%dT%H:%i:%s.%fZ') AS created_at,
                  DATE_FORMAT(updated_at, '%Y-%m-%dT%H:%i:%s.%fZ') AS updated_at
           FROM cliente_contatos
           WHERE tenant_id = :tid AND cliente_id = :cid
           ORDER BY principal DESC, tipo, valor"#,
            params! { "tid" => tenant_id, "cid" => cliente_id },
        )
        .map_err(ErroAplicacao::from)?;
    Ok(rows)
}

pub fn remover_contato(tenant_id: i32, id: u32) -> Result<bool, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let affected = conn
        .exec_iter(
            "DELETE FROM cliente_contatos WHERE id = :id AND tenant_id = :tid",
            params! { "id" => id, "tid" => tenant_id },
        )
        .map_err(ErroAplicacao::from)?
        .affected_rows();
    Ok(affected > 0)
}

// =============================================================================
// Equipamentos
// =============================================================================

pub fn criar_equipamento(
    tenant_id: i32,
    cliente_id: u32,
    descricao: &str,
    marca: Option<&str>,
    modelo: Option<&str>,
    numero_serie: Option<&str>,
    patrimonio: Option<&str>,
    observacao: Option<&str>,
    tipo: &str,
) -> Result<Equipamento, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        r#"INSERT INTO equipamentos
           (tenant_id, cliente_id, descricao, marca, modelo, numero_serie, patrimonio, observacao, tipo)
           VALUES (:tid, :cid, :desc, :marca, :modelo, :sn, :pat, :obs, :tipo)"#,
        params! {
            "tid" => tenant_id,
            "cid" => cliente_id,
            "desc" => descricao,
            "marca" => marca,
            "modelo" => modelo,
            "sn" => numero_serie,
            "pat" => patrimonio,
            "obs" => observacao,
            "tipo" => tipo,
        },
    )
    .map_err(ErroAplicacao::from)?;
    let id = conn.last_insert_id() as u32;
    obter_equipamento_por_id(tenant_id, id)?.ok_or(ErroAplicacao::Desconhecido(
        "equipamento criado mas nao encontrado".into(),
    ))
}

pub fn obter_equipamento_por_id(
    tenant_id: i32,
    id: u32,
) -> Result<Option<Equipamento>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<Equipamento> = conn
        .exec_first(
            r#"SELECT id, cliente_id, descricao, marca, modelo, numero_serie,
                      patrimonio, observacao, tipo,
                      DATE_FORMAT(created_at, '%Y-%m-%dT%H:%i:%s.%fZ') AS created_at,
                      DATE_FORMAT(updated_at, '%Y-%m-%dT%H:%i:%s.%fZ') AS updated_at
               FROM equipamentos WHERE id = :id AND tenant_id = :tid"#,
            params! { "id" => id, "tid" => tenant_id },
        )
        .map_err(ErroAplicacao::from)?;
    Ok(row)
}

pub fn listar_equipamentos(
    tenant_id: i32,
    cliente_id: u32,
) -> Result<Vec<Equipamento>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let rows: Vec<Equipamento> = conn
        .exec(
            r#"SELECT id, cliente_id, descricao, marca, modelo, numero_serie,
                  patrimonio, observacao, tipo,
                  DATE_FORMAT(created_at, '%Y-%m-%dT%H:%i:%s.%fZ') AS created_at,
                  DATE_FORMAT(updated_at, '%Y-%m-%dT%H:%i:%s.%fZ') AS updated_at
           FROM equipamentos
           WHERE tenant_id = :tid AND cliente_id = :cid
           ORDER BY descricao"#,
            params! { "tid" => tenant_id, "cid" => cliente_id },
        )
        .map_err(ErroAplicacao::from)?;
    Ok(rows)
}

pub fn remover_equipamento(tenant_id: i32, id: u32) -> Result<bool, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let affected = conn
        .exec_iter(
            "DELETE FROM equipamentos WHERE id = :id AND tenant_id = :tid",
            params! { "id" => id, "tid" => tenant_id },
        )
        .map_err(ErroAplicacao::from)?
        .affected_rows();
    Ok(affected > 0)
}

// =============================================================================
// Busca global
// =============================================================================

/// Busca global: nome, cpf/cnpj, telefone, email, equipamento, serial.
/// Retorna clientes que batem. Defense in depth: filtra por tenant.
pub fn buscar_clientes(
    tenant_id: i32,
    query: &str,
    limite: u32,
) -> Result<Vec<serde_json::Value>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let like = format!("%{}%", query);
    let rows: Vec<mysql::Row> = conn
        .exec(
            r#"
            SELECT DISTINCT c.id, c.nome, c.email, c.telefone, c.cpf_cnpj
            FROM clientes c
            LEFT JOIN cliente_contatos ct ON ct.cliente_id = c.id AND ct.tenant_id = c.tenant_id
            LEFT JOIN equipamentos e ON e.cliente_id = c.id AND e.tenant_id = c.tenant_id
            WHERE c.tenant_id = :tid
               AND (c.nome LIKE :q
                 OR c.cpf_cnpj LIKE :q
                 OR c.telefone LIKE :q
                 OR c.email LIKE :q
                 OR ct.valor LIKE :q
                 OR e.descricao LIKE :q
                 OR e.marca LIKE :q
                 OR e.modelo LIKE :q
                 OR e.numero_serie LIKE :q
                 OR e.patrimonio LIKE :q)
            ORDER BY c.nome
            LIMIT :lim
            "#,
            params! { "tid" => tenant_id, "q" => &like, "lim" => limite },
        )
        .map_err(ErroAplicacao::from)?;
    let result: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|row| {
            let (id, nome, email, telefone, cpf_cnpj): (
                u32,
                String,
                Option<String>,
                Option<String>,
                Option<String>,
            ) = mysql::prelude::FromRow::from_row(row);
            serde_json::json!({
                "id": id,
                "nome": nome,
                "email": email,
                "telefone": telefone,
                "cpf_cnpj": cpf_cnpj,
            })
        })
        .collect();
    Ok(result)
}
