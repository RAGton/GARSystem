use super::conexao::obter_conexao;
use crate::servicos::{Cliente, ErroAplicacao};
use mysql::params;
use mysql::prelude::Queryable;

/// Lista clientes de UM tenant (P2.6.2a).
///
/// **Mudança**: agora filtra por `tenant_id`. Caller é responsável por
/// passar o tenant correto (extraído de Claims). Em SUPER_ADMIN, passa
/// o tenant específico (não bypass para listagem).
pub fn listar_clientes(tenant_id: i32) -> Result<Vec<Cliente>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let rows = conn.exec_map(
        "SELECT id, nome, email, telefone FROM clientes WHERE tenant_id = :tenant_id",
        params! { "tenant_id" => tenant_id },
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

/// Soma de gastos de um cliente dentro do tenant (P2.6.2a).
pub fn obter_gastos_por_cliente(tenant_id: i32, id: u32) -> Result<f64, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let total: Option<f64> = conn.exec_first(
        "SELECT SUM(valor) FROM movimentacoes WHERE tenant_id = :tenant_id AND cliente_id = :id",
        params! { "tenant_id" => tenant_id, "id" => id },
    )?;
    Ok(total.unwrap_or(0.0))
}

/// Crédito de um cliente dentro do tenant (P2.6.2a).
pub fn obter_credito_cliente(tenant_id: i32, id: u32) -> Result<f64, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let credito: Option<f64> = conn.exec_first(
        "SELECT credito FROM clientes WHERE tenant_id = :tenant_id AND id = :id",
        params! { "tenant_id" => tenant_id, "id" => id },
    )?;
    Ok(credito.unwrap_or(0.0))
}

/// Cria ou atualiza cliente dentro de UM tenant (P2.6.2a).
///
/// Defense in depth: WHERE tenant_id filtra updates por tenant.
pub fn criar_ou_atualizar_cliente(tenant_id: i32, c: &Cliente) -> Result<u32, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    if c.id == 0 {
        conn.exec_drop(
            "INSERT INTO clientes (tenant_id, nome, email, telefone, cpf_cnpj, endereco, inscricao_estadual, credito) VALUES (:tenant_id, :nome, :email, :telefone, :cpf_cnpj, :endereco, :ie, :credito)",
            params! {
                "tenant_id" => tenant_id,
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
        let affected = conn.exec_iter(
            "UPDATE clientes SET nome=:nome, email=:email, telefone=:telefone, cpf_cnpj=:cpf_cnpj, endereco=:endereco, inscricao_estadual=:ie, credito=:credito WHERE id=:id AND tenant_id=:tenant_id",
            params! {
                "id" => c.id,
                "tenant_id" => tenant_id,
                "nome" => &c.nome,
                "email" => &c.email,
                "telefone" => &c.telefone,
                "cpf_cnpj" => &c.cpf_cnpj,
                "endereco" => &c.endereco,
                "ie" => &c.inscricao_estadual,
                "credito" => c.credito_disponivel,
            },
        )?.affected_rows();
        if affected == 0 {
            return Err(ErroAplicacao::BancoDeDadosQuery(
                "cliente não encontrado neste tenant".into(),
            ));
        }
        Ok(c.id)
    }
}

/// Obtém um cliente por ID dentro do tenant (P2.6.2a).
pub fn obter_cliente_por_id(tenant_id: i32, id: u32) -> Result<Cliente, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    type ClienteRow = (
        u32,
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        f64,
    );
    let row: Option<ClienteRow> = conn.exec_first(
        "SELECT id, nome, email, telefone, endereco, inscricao_estadual, cpf_cnpj, credito FROM clientes WHERE id = :id AND tenant_id = :tenant_id",
        params! { "id" => id, "tenant_id" => tenant_id },
    )?;
    let (id, nome, email, telefone, endereco, inscricao_estadual, cpf_cnpj, credito) =
        row.ok_or_else(|| ErroAplicacao::BancoDeDadosQuery("cliente não encontrado".into()))?;
    Ok(Cliente {
        id,
        nome,
        email,
        telefone,
        endereco,
        inscricao_estadual,
        cpf_cnpj,
        credito_disponivel: credito,
    })
}
