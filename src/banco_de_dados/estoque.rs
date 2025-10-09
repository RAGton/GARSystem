// src/banco_de_dados/estoque.rs

// ...existing code...
use super::conexao::obter_conexao;
use crate::servicos::{ErroAplicacao, Fornecedor, Peca};
use mysql::prelude::*;

pub fn listar_pecas() -> Result<Vec<Peca>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let pecas = conn.query_map(
        "SELECT id, codigo_interno, part_number, descricao, fabricante, localizacao, estoque_atual, estoque_minimo, preco_custo, preco_venda FROM pecas ORDER BY descricao",
        |(id, codigo_interno, part_number, descricao, fabricante, localizacao, estoque_atual, estoque_minimo, preco_custo, preco_venda): (u32, String, String, String, String, String, i32, i32, f64, f64)| {
            // por compatibilidade, usamos 'descricao' também como 'nome' curto
            Peca { id, nome: descricao.clone(), codigo_interno, part_number, descricao, fabricante, localizacao, estoque_atual, estoque_minimo, preco_custo, preco_venda }
        }
    )?;
    Ok(pecas)
}

pub fn criar_peca(peca: &Peca) -> Result<(), ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        "INSERT INTO pecas (codigo_interno, part_number, descricao, fabricante, localizacao, estoque_atual, estoque_minimo, preco_custo, preco_venda) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        (&peca.codigo_interno, &peca.part_number, &peca.descricao, &peca.fabricante, &peca.localizacao, peca.estoque_atual, peca.estoque_minimo, peca.preco_custo, peca.preco_venda)
    )?;
    Ok(())
}

pub fn atualizar_peca(peca: &Peca) -> Result<(), ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        "UPDATE pecas SET codigo_interno = ?, part_number = ?, descricao = ?, fabricante = ?, localizacao = ?, estoque_minimo = ?, preco_custo = ?, preco_venda = ? WHERE id = ?",
        (&peca.codigo_interno, &peca.part_number, &peca.descricao, &peca.fabricante, &peca.localizacao, peca.estoque_minimo, peca.preco_custo, peca.preco_venda, peca.id)
    )?;
    Ok(())
}

pub fn listar_fornecedores() -> Result<Vec<Fornecedor>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let fornecedores = conn.query_map(
        "SELECT id, nome, cnpj, contato, telefone, email FROM fornecedores ORDER BY nome",
        |(id, nome, cnpj, contato, telefone, email)| Fornecedor {
            id,
            nome,
            cnpj,
            contato,
            telefone,
            email,
        },
    )?;
    Ok(fornecedores)
}

pub fn criar_fornecedor(fornecedor: &Fornecedor) -> Result<(), ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        "INSERT INTO fornecedores (nome, cnpj, contato, telefone, email) VALUES (?, ?, ?, ?, ?)",
        (
            &fornecedor.nome,
            &fornecedor.cnpj,
            &fornecedor.contato,
            &fornecedor.telefone,
            &fornecedor.email,
        ),
    )?;
    Ok(())
}
