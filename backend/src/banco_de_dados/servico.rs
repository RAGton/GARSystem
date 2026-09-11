// src/banco_de_dados/servico.rs
// P2.6.2a — tenant_id obrigatório.

use super::conexao::obter_conexao;
use crate::servicos::ErroAplicacao;
use crate::servicos::{Servico, ServicoOS};
use mysql::{params, prelude::Queryable};

pub fn listar_servicos(tenant_id: i32) -> Result<Vec<Servico>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let rows = conn.exec_map(
        "SELECT id, nome, descricao, preco FROM servicos WHERE tenant_id = :tenant_id ORDER BY id",
        params! { "tenant_id" => tenant_id },
        |(id, nome, descricao, preco)| Servico {
            id,
            nome,
            descricao,
            preco,
        },
    )?;
    Ok(rows)
}

pub fn criar_servico(tenant_id: i32, s: &Servico) -> Result<u32, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        "INSERT INTO servicos (tenant_id, nome, descricao, preco) VALUES (:tenant_id, :nome, :desc, :preco)",
        params! { "tenant_id" => tenant_id, "nome" => &s.nome, "desc" => &s.descricao, "preco" => s.preco },
    )?;
    Ok(conn.last_insert_id() as u32)
}

/// Lista serviços de uma OS dentro do tenant.
pub fn listar_servicos_da_os(tenant_id: i32, os_id: u32) -> Result<Vec<ServicoOS>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let items = conn.exec_map(
        "SELECT s.id, s.nome, s.descricao, osi.quantidade, osi.preco_unitario
           FROM ordem_servico_servicos osi
           JOIN servicos s ON osi.servico_id = s.id
          WHERE osi.ordem_servico_id = :id AND osi.tenant_id = :tenant_id",
        params! { "id" => os_id, "tenant_id" => tenant_id },
        |(id_servico, nome, descricao, quantidade, preco_unitario): (
            u32,
            String,
            String,
            u32,
            f64,
        )| ServicoOS {
            id_servico,
            nome,
            descricao,
            quantidade,
            preco_unitario,
            preco_total: preco_unitario * quantidade as f64,
        },
    )?;
    Ok(items)
}

pub fn gravar_servicos_na_os(
    tenant_id: i32,
    os_id: u32,
    servicos: &Vec<ServicoOS>,
) -> Result<(), ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let mut tx = conn.start_transaction(mysql::TxOpts::default())?;
    tx.exec_drop(
        "DELETE FROM ordem_servico_servicos WHERE ordem_servico_id = :id AND tenant_id = :tenant_id",
        params! { "id" => os_id, "tenant_id" => tenant_id },
    )?;
    for s in servicos {
        tx.exec_drop(
            "INSERT INTO ordem_servico_servicos (tenant_id, ordem_servico_id, servico_id, quantidade, preco_unitario) VALUES (?, ?, ?, ?, ?)",
            (tenant_id, os_id, s.id_servico, s.quantidade, s.preco_unitario),
        )?;
    }
    tx.commit()?;
    Ok(())
}
