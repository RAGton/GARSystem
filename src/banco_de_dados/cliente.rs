use super::conexao::obter_conexao;
use crate::servicos::{Cliente, ErroAplicacao};
use mysql::params;
use mysql::prelude::Queryable;

pub fn listar_clientes() -> Result<Vec<Cliente>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    // Query simplificada: traz id, nome, email, telefone
    let rows = conn.query_map(
        "SELECT id, nome, email, telefone FROM clientes",
        |(id, nome, email, telefone)| Cliente {
            id,
            nome,
            email,
            telefone,
            endereco: None,
            inscricao_estadual: None,
            cpf_cnpj: None,
            credito_disponivel: 0.0,
        },
    )?;
    Ok(rows)
}

pub fn obter_gastos_por_cliente(id: u32) -> Result<f64, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let total: Option<f64> = conn.exec_first(
        "SELECT SUM(valor) FROM movimentacoes WHERE cliente_id = :id",
        params! {"id" => id},
    )?;
    Ok(total.unwrap_or(0.0))
}

pub fn obter_credito_cliente(id: u32) -> Result<f64, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let credito: Option<f64> = conn.exec_first(
        "SELECT credito FROM clientes WHERE id = :id",
        params! {"id" => id},
    )?;
    Ok(credito.unwrap_or(0.0))
}

pub fn criar_ou_atualizar_cliente(c: &Cliente) -> Result<u32, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    if c.id == 0 {
        conn.exec_drop(
            "INSERT INTO clientes (nome, email, telefone, cpf_cnpj, endereco, inscricao_estadual, credito) VALUES (:nome, :email, :telefone, :cpf_cnpj, :endereco, :ie, :credito)",
            params! {
                "nome" => &c.nome,
                "email" => &c.email,
                "telefone" => &c.telefone,
                "cpf_cnpj" => &c.cpf_cnpj,
                "endereco" => &c.endereco,
                "ie" => &c.inscricao_estadual,
                "credito" => c.credito_disponivel,
            },
        )?;
        let id = conn.last_insert_id() as u32;
        Ok(id)
    } else {
        conn.exec_drop(
            "UPDATE clientes SET nome=:nome, email=:email, telefone=:telefone, cpf_cnpj=:cpf_cnpj, endereco=:endereco, inscricao_estadual=:ie, credito=:credito WHERE id=:id",
            params! {
                "id" => c.id,
                "nome" => &c.nome,
                "email" => &c.email,
                "telefone" => &c.telefone,
                "cpf_cnpj" => &c.cpf_cnpj,
                "endereco" => &c.endereco,
                "ie" => &c.inscricao_estadual,
                "credito" => c.credito_disponivel,
            },
        )?;
        Ok(c.id)
    }
}
