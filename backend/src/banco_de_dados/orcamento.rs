use super::conexao::obter_conexao;
use super::pagination::{Paginacao, Pagina};
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

/// Lista orçamentos do tenant com paginação. Ordenação padrão:
/// `data_criacao DESC` (mais recentes primeiro).
pub fn listar_orcamentos_paginado(
    tenant_id: i32,
    pag: Paginacao,
) -> Result<Pagina<ServicoOrcamento>, ErroAplicacao> {
    let pag = pag.sanitizar();
    let mut conn = obter_conexao()?;

    let total: u64 = conn
        .exec_first(
            "SELECT COUNT(*) FROM orcamentos WHERE tenant_id = :tenant_id",
            params! { "tenant_id" => tenant_id },
        )?
        .unwrap_or(0);

    let rows: Vec<(u32, u32, f64)> = conn.exec(
        "SELECT id, cliente_id, total FROM orcamentos
         WHERE tenant_id = :tenant_id
         ORDER BY data_criacao DESC, id DESC
         LIMIT :lim OFFSET :off",
        params! {
            "tenant_id" => tenant_id,
            "lim" => pag.limit,
            "off" => pag.offset(),
        },
    )?;

    let items: Vec<ServicoOrcamento> = rows
        .into_iter()
        .map(|(id, cliente_id, total)| {
            // Busca itens de cada orçamento. Para listas pequenas é OK;
            // quando passar de 100k, refatorar para query única com JOIN.
            let items = conn
                .exec_map(
                    "SELECT descricao, quantidade, preco_unitario, preco_total
                     FROM orcamento_items
                     WHERE orcamento_id = :id AND tenant_id = :tenant_id
                     ORDER BY id",
                    params! { "id" => id, "tenant_id" => tenant_id },
                    |(descricao, quantidade, preco_unitario, preco_total)| ServicoItem {
                        descricao,
                        quantidade,
                        preco_unitario,
                        preco_total,
                    },
                )?;
            Ok::<_, ErroAplicacao>(ServicoOrcamento {
                id,
                cliente_id,
                items,
                total,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Pagina::from_items(
        items,
        pag.page,
        pag.limit,
        total,
    ))
}
