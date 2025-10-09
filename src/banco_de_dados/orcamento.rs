use super::conexao::obter_conexao;
use crate::servicos::{ErroAplicacao, Orcamento as ServicoOrcamento, OrcamentoItem as ServicoItem};
use mysql::params;
use mysql::prelude::Queryable;

pub fn criar_orcamento(o: &ServicoOrcamento) -> Result<u32, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    // Inserir orcamento
    conn.exec_drop(
        "INSERT INTO orcamentos (cliente_id, total) VALUES (:cliente_id, :total)",
        params! { "cliente_id" => o.cliente_id, "total" => o.total },
    )?;
    let id = conn.last_insert_id() as u32;
    for item in &o.items {
        conn.exec_drop(
            "INSERT INTO orcamento_items (orcamento_id, descricao, quantidade, preco_unitario, preco_total) VALUES (:orc_id, :descricao, :qtd, :pu, :pt)",
            params! { "orc_id" => id, "descricao" => &item.descricao, "qtd" => item.quantidade, "pu" => item.preco_unitario, "pt" => item.preco_total },
        )?;
    }
    Ok(id)
}

pub fn obter_orcamento(id: u32) -> Result<ServicoOrcamento, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<(u32, u32, f64)> = conn.exec_first(
        "SELECT id, cliente_id, total FROM orcamentos WHERE id = :id",
        params! { "id" => id },
    )?;
    if let Some((id, cliente_id, total)) = row {
        let items = conn.exec_map(
            "SELECT descricao, quantidade, preco_unitario, preco_total FROM orcamento_items WHERE orcamento_id = :id",
            params! { "id" => id },
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
            "Orçamento não encontrado".into(),
        ))
    }
}
