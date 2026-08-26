use super::conexao::obter_conexao;
use crate::servicos::{ErroAplicacao, Orcamento as ServicoOrcamento, OrcamentoItem as ServicoItem};
use mysql::params;
use mysql::prelude::Queryable;

pub fn criar_orcamento(tenant_id: i32, o: &ServicoOrcamento) -> Result<u32, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        "INSERT INTO orcamentos (tenant_id, cliente_id, total) VALUES (:tenant_id, :cliente_id, :total)",
        params! { "tenant_id" => tenant_id, "cliente_id" => o.cliente_id, "total" => o.total },
    )?;
    let id = conn.last_insert_id() as u32;
    for item in &o.items {
        conn.exec_drop(
            "INSERT INTO orcamento_items (tenant_id, orcamento_id, descricao, quantidade, preco_unitario, preco_total) VALUES (:tenant_id, :orc_id, :descricao, :qtd, :pu, :pt)",
            params! { "tenant_id" => tenant_id, "orc_id" => id, "descricao" => &item.descricao, "qtd" => item.quantidade, "pu" => item.preco_unitario, "pt" => item.preco_total },
        )?;
    }
    Ok(id)
}

pub fn obter_orcamento(tenant_id: i32, id: u32) -> Result<ServicoOrcamento, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<(u32, u32, f64)> = conn.exec_first(
        "SELECT id, cliente_id, total FROM orcamentos WHERE id = :id AND tenant_id = :tenant_id",
        params! { "id" => id, "tenant_id" => tenant_id },
    )?;
    if let Some((id, cliente_id, total)) = row {
        let items = conn.exec_map(
            "SELECT descricao, quantidade, preco_unitario, preco_total FROM orcamento_items WHERE orcamento_id = :id AND tenant_id = :tenant_id",
            params! { "id" => id, "tenant_id" => tenant_id },
            |(descricao, quantidade, preco_unitario, preco_total)| ServicoItem {
                descricao,
                quantidade,
                preco_unitario,
                preco_total,
            },
        )?;
        Ok(ServicoOrcamento {
            id,
            cliente_id,
            items,
            total,
        })
    } else {
        Err(ErroAplicacao::Desconhecido(
            "Orçamento não encontrado neste tenant".into(),
        ))
    }
}
