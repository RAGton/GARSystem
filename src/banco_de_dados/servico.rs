// src/banco_de_dados/servico.rs

use super::conexao::obter_conexao;
use crate::servicos::ErroAplicacao;
use crate::servicos::{Servico, ServicoOS};
use mysql::{params, prelude::Queryable};

pub fn listar_servicos() -> Result<Vec<Servico>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let rows = conn.query_map(
        "SELECT id, nome, descricao, preco FROM servicos ORDER BY id",
        |(id, nome, descricao, preco)| Servico {
            id,
            nome,
            descricao,
            preco,
        },
    )?;
    Ok(rows)
}

pub fn criar_servico(s: &Servico) -> Result<u32, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        "INSERT INTO servicos (nome, descricao, preco) VALUES (:nome, :desc, :preco)",
        params! { "nome" => &s.nome, "desc" => &s.descricao, "preco" => s.preco },
    )?;
    Ok(conn.last_insert_id() as u32)
}

// Funções para associar serviços a uma OS
pub fn listar_servicos_da_os(os_id: u32) -> Result<Vec<ServicoOS>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let items = conn.exec_map(
        "SELECT s.id, s.nome, s.descricao, osi.quantidade, osi.preco_unitario FROM ordem_servico_servicos osi JOIN servicos s ON osi.servico_id = s.id WHERE osi.ordem_servico_id = :id",
        params! { "id" => os_id },
        |(id_servico, nome, descricao, quantidade, preco_unitario): (u32, String, String, u32, f64)| ServicoOS {
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

pub fn gravar_servicos_na_os(os_id: u32, servicos: &Vec<ServicoOS>) -> Result<(), ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let mut tx = conn.start_transaction(mysql::TxOpts::default())?;
    tx.exec_drop(
        "DELETE FROM ordem_servico_servicos WHERE ordem_servico_id = :id",
        params! { "id" => os_id },
    )?;
    for s in servicos {
        tx.exec_drop(
            "INSERT INTO ordem_servico_servicos (ordem_servico_id, servico_id, quantidade, preco_unitario) VALUES (?, ?, ?, ?)",
            (os_id, s.id_servico, s.quantidade, s.preco_unitario),
        )?;
    }
    tx.commit()?;
    Ok(())
}
