// src/banco_de_dados/estoque.rs
// P2.6.2a — tenant_id obrigatório em toda query.

use super::conexao::obter_conexao;
use crate::servicos::{ErroAplicacao, Fornecedor, Peca};
use mysql::params;
use mysql::prelude::*;

pub fn listar_pecas(tenant_id: i32) -> Result<Vec<Peca>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let pecas = conn.exec_map(
        "SELECT id, codigo_interno, part_number, descricao, fabricante, localizacao, estoque_atual, estoque_minimo, preco_custo, preco_venda FROM pecas WHERE tenant_id = :tenant_id ORDER BY descricao",
        params! { "tenant_id" => tenant_id },
        |(id, codigo_interno, part_number, descricao, fabricante, localizacao, estoque_atual, estoque_minimo, preco_custo, preco_venda): (u32, String, String, String, String, String, i32, i32, f64, f64)| {
            Peca { id, nome: descricao.clone(), codigo_interno, part_number, descricao, fabricante, localizacao, estoque_atual, estoque_minimo, preco_custo, preco_venda }
        }
    )?;
    Ok(pecas)
}

pub fn criar_peca(tenant_id: i32, peca: &Peca) -> Result<(), ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        "INSERT INTO pecas (tenant_id, codigo_interno, part_number, descricao, fabricante, localizacao, estoque_atual, estoque_minimo, preco_custo, preco_venda) VALUES (:tenant_id, :codigo_interno, :part_number, :descricao, :fabricante, :localizacao, :estoque_atual, :estoque_minimo, :preco_custo, :preco_venda)",
        params! {
            "tenant_id" => tenant_id,
            "codigo_interno" => &peca.codigo_interno,
            "part_number" => &peca.part_number,
            "descricao" => &peca.descricao,
            "fabricante" => &peca.fabricante,
            "localizacao" => &peca.localizacao,
            "estoque_atual" => peca.estoque_atual,
            "estoque_minimo" => peca.estoque_minimo,
            "preco_custo" => peca.preco_custo,
            "preco_venda" => peca.preco_venda,
        }
    )?;
    Ok(())
}

pub fn atualizar_peca(tenant_id: i32, peca: &Peca) -> Result<(), ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let affected = conn.exec_iter(
        "UPDATE pecas SET codigo_interno = :codigo_interno, part_number = :part_number, descricao = :descricao, fabricante = :fabricante, localizacao = :localizacao, estoque_minimo = :estoque_minimo, preco_custo = :preco_custo, preco_venda = :preco_venda WHERE id = :id AND tenant_id = :tenant_id",
        params! {
            "tenant_id" => tenant_id,
            "id" => peca.id,
            "codigo_interno" => &peca.codigo_interno,
            "part_number" => &peca.part_number,
            "descricao" => &peca.descricao,
            "fabricante" => &peca.fabricante,
            "localizacao" => &peca.localizacao,
            "estoque_minimo" => peca.estoque_minimo,
            "preco_custo" => peca.preco_custo,
            "preco_venda" => peca.preco_venda,
        }
    )?.affected_rows();
    if affected == 0 {
        return Err(ErroAplicacao::BancoDeDadosQuery(
            "peça não encontrada neste tenant".into(),
        ));
    }
    Ok(())
}

pub fn listar_fornecedores(tenant_id: i32) -> Result<Vec<Fornecedor>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let fornecedores = conn.exec_map(
        "SELECT id, nome, cnpj, contato, telefone, email FROM fornecedores WHERE tenant_id = :tenant_id ORDER BY nome",
        params! { "tenant_id" => tenant_id },
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

pub fn criar_fornecedor(tenant_id: i32, fornecedor: &Fornecedor) -> Result<(), ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        "INSERT INTO fornecedores (tenant_id, nome, cnpj, contato, telefone, email) VALUES (:tenant_id, :nome, :cnpj, :contato, :telefone, :email)",
        params! {
            "tenant_id" => tenant_id,
            "nome" => &fornecedor.nome,
            "cnpj" => &fornecedor.cnpj,
            "contato" => &fornecedor.contato,
            "telefone" => &fornecedor.telefone,
            "email" => &fornecedor.email,
        },
    )?;
    Ok(())
}
