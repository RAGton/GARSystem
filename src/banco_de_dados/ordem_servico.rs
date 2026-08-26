// src/banco_de_dados/ordem_servico.rs
// P2.6.2a — tenant_id obrigatório em toda query.

use super::conexao::obter_conexao;
use crate::servicos::{ErroAplicacao, HistoricoEdicao, OrdemServico, PecaOS, SituacaoOS, StatusOS};
use mysql::{
    params,
    prelude::{FromRow, Queryable},
    FromRowError, Row, Transaction, Value,
};
use std::collections::HashMap;

// ============================================================================
// Constantes e tipos
// ============================================================================

/// Erro lançado quando a edição da OS exigiria estoque negativo.
#[derive(Debug, thiserror::Error)]
pub enum ErroEstoque {
    #[error("Estoque insuficiente para a peça {peca_id}: disponível {disponivel}, necessário {necessario}")]
    Insuficiente {
        peca_id: u32,
        disponivel: i32,
        necessario: i32,
    },
}

// ============================================================================
// Helpers internos
// ============================================================================

/// Aplica um delta de estoque a uma peça, dentro de uma transação.
/// Usa `SELECT ... FOR UPDATE` para serializar concorrência na peça.
/// Emite registro em `movimentos_estoque` para auditoria.
///
/// Retorna erro se o estoque resultante ficaria negativo.
#[allow(clippy::too_many_arguments)]
fn ajustar_estoque(
    tx: &mut Transaction,
    tenant_id: i32,
    peca_id: u32,
    delta: i32,
    tipo_movimento: &str,
    referencia_id: u32,
    motivo: &str,
    usuario: &str,
) -> Result<(), ErroAplicacao> {
    // Lock pessimista na linha da peça. Bloqueia outros ajustes concorrentes.
    // (P2.6.2a) defense in depth: WHERE tenant_id filtra o lock por tenant.
    let row: Option<(i32,)> = tx
        .exec_first(
            "SELECT estoque_atual FROM pecas WHERE id = ? AND tenant_id = ? FOR UPDATE",
            (peca_id, tenant_id),
        )
        .map_err(ErroAplicacao::from)?;
    let row = row.ok_or_else(|| {
        ErroAplicacao::BancoDeDadosQuery(format!("peça {} não encontrada neste tenant", peca_id))
    })?;
    let estoque_anterior = row.0;
    let estoque_novo = estoque_anterior + delta;
    if estoque_novo < 0 {
        return Err(ErroAplicacao::Desconhecido(format!(
            "Estoque insuficiente para peça {}: disponível={} delta={} novo={}",
            peca_id, estoque_anterior, delta, estoque_novo
        )));
    }
    tx.exec_drop(
        "UPDATE pecas SET estoque_atual = ? WHERE id = ? AND tenant_id = ?",
        (estoque_novo, peca_id, tenant_id),
    )
    .map_err(ErroAplicacao::from)?;
    tx.exec_drop(
        r#"INSERT INTO movimentos_estoque
             (tenant_id, peca_id, tipo_movimento, referencia_id, quantidade_movimentada,
              estoque_anterior, estoque_novo, usuario, motivo)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        (
            tenant_id,
            peca_id,
            tipo_movimento,
            referencia_id,
            delta,
            estoque_anterior,
            estoque_novo,
            usuario,
            motivo,
        ),
    )
    .map_err(ErroAplicacao::from)?;
    Ok(())
}

fn registrar_historico(
    tx: &mut Transaction,
    tenant_id: i32,
    os_id: u32,
    usuario: &str,
    campo: &str,
    antigo: &str,
    novo: &str,
) -> Result<(), mysql::Error> {
    tx.exec_drop(
        r"INSERT INTO historico_edicoes
            (tenant_id, ordem_servico_id, usuario, campo_alterado, valor_antigo, valor_novo)
          VALUES (:tenant_id, :os_id, :usuario, :campo, :antigo, :novo)",
        params! { "tenant_id" => tenant_id, "os_id" => os_id, "usuario" => usuario, "campo" => campo, "antigo" => antigo, "novo" => novo },
    )?;
    Ok(())
}

/// Agrupa peças por id, somando quantidades. Útil para normalizar
/// a entrada antes do diff.
fn agregar_pecas(pecas: &[PecaOS]) -> HashMap<u32, u32> {
    let mut acc: HashMap<u32, u32> = HashMap::new();
    for p in pecas {
        *acc.entry(p.id_peca).or_insert(0) += p.quantidade;
    }
    acc
}

// ============================================================================
// Listagem / busca
// ============================================================================

pub fn listar_ordens_servico(tenant_id: i32) -> Result<Vec<OrdemServico>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let ordens = conn.exec_map(
        r#"
        SELECT
            os.id, c.nome, e.descricao, os.defeito_relatado, os.status, os.parecer_tecnico,
            os.situacao, e.numero_serie, os.observacoes, os.tecnico_responsavel,
            os.atendente, c.telefone, DATE_FORMAT(os.data_chegada, '%d/%m/%Y %H:%i'), DATE_FORMAT(os.prazo_entrega, '%d/%m/%Y')
        FROM ordens_servico os
        LEFT JOIN clientes c ON os.cliente_id = c.id
        JOIN equipamentos e ON os.equipamento_id = e.id
        WHERE os.tenant_id = :tenant_id
        ORDER BY os.id DESC
        LIMIT 100
        "#,
        params! { "tenant_id" => tenant_id },
        OrdemServico::from_row,
    )?;
    Ok(ordens)
}

pub fn buscar_os_por_id(tenant_id: i32, id: u32) -> Result<OrdemServico, ErroAplicacao> {
    let mut conn = obter_conexao()?;

    let row = conn.exec_first(
        r#"SELECT os.id, c.nome, e.descricao, os.defeito_relatado, os.status, os.parecer_tecnico,
            os.situacao, e.numero_serie, os.observacoes, os.tecnico_responsavel, os.atendente,
            c.telefone, DATE_FORMAT(os.data_chegada, '%d/%m/%Y %H:%i'), DATE_FORMAT(os.prazo_entrega, '%d/%m/%Y')
           FROM ordens_servico os LEFT JOIN clientes c ON os.cliente_id = c.id
           JOIN equipamentos e ON os.equipamento_id = e.id WHERE os.id = :id AND os.tenant_id = :tenant_id"#,
        params! { "id" => id, "tenant_id" => tenant_id },
    )?.ok_or(ErroAplicacao::OsNaoEncontrada)?;

    let mut os = OrdemServico::from_row_opt(row)?;

    os.historico_edicoes = conn.exec_map(
        "SELECT usuario, DATE_FORMAT(data_hora, '%d/%m/%Y %H:%i'), campo_alterado, valor_antigo, valor_novo FROM historico_edicoes WHERE ordem_servico_id = :id AND tenant_id = :tenant_id ORDER BY data_hora DESC",
        params! { "id" => id, "tenant_id" => tenant_id },
        |(usuario, data_hora, campo_alterado, valor_antigo, valor_novo)| HistoricoEdicao { usuario, data_hora, campo_alterado, valor_antigo, valor_novo },
    )?;

    os.pecas = conn.exec_map(
        "SELECT p.id, p.codigo_interno, p.descricao, osp.quantidade, osp.preco_venda_unitario FROM ordem_servico_pecas osp JOIN pecas p ON osp.peca_id = p.id WHERE osp.ordem_servico_id = :id AND osp.tenant_id = :tenant_id",
        params! { "id" => id, "tenant_id" => tenant_id },
        |(id_peca, codigo_interno, descricao, quantidade, preco_venda_unitario): (u32, String, String, u32, f64)| PecaOS {
            id_peca, codigo_interno, descricao, quantidade, preco_venda_unitario,
            preco_total: preco_venda_unitario * quantidade as f64,
        },
    )?;

    match super::servico::listar_servicos_da_os(tenant_id, id) {
        Ok(list) => os.servicos = list,
        Err(e) => {
            eprintln!("Falha ao carregar serviços da OS {}: {}", id, e);
            os.servicos = Vec::new();
        }
    }
    os.total_servicos = os.servicos.iter().map(|s| s.preco_total).sum();

    os.total_pecas = os.pecas.iter().map(|p| p.preco_total).sum();
    Ok(os)
}

impl FromRow for OrdemServico {
    fn from_row_opt(mut row: Row) -> Result<Self, FromRowError> {
        let data_chegada: Option<String> = row
            .take("DATE_FORMAT(os.data_chegada, '%d/%m/%Y %H:%i')")
            .ok_or_else(|| FromRowError(row.clone()))?;

        Ok(OrdemServico {
            id: row.take("id").ok_or_else(|| FromRowError(row.clone()))?,
            cliente: row
                .take("nome")
                .map(|v: Value| {
                    if v == Value::NULL {
                        "Cliente não encontrado".to_string()
                    } else {
                        mysql::from_value(v)
                    }
                })
                .unwrap_or_default(),
            equipamento: row
                .take("descricao")
                .ok_or_else(|| FromRowError(row.clone()))?,
            defeito_relatado: row
                .take("defeito_relatado")
                .ok_or_else(|| FromRowError(row.clone()))?,
            status: row
                .take("status")
                .map(|s: String| {
                    serde_json::from_str(&format!("\"{}\"", s)).unwrap_or(StatusOS::Aberta)
                })
                .ok_or_else(|| FromRowError(row.clone()))?,
            parecer_tecnico: row.take("parecer_tecnico").unwrap_or_default(),
            situacao: row
                .take("situacao")
                .map(|s: String| {
                    serde_json::from_str(&format!("\"{}\"", s)).unwrap_or(SituacaoOS::Orcamento)
                })
                .ok_or_else(|| FromRowError(row.clone()))?,
            numero_serie_equipamento: row.take("numero_serie").unwrap_or_default(),
            observacoes: row.take("observacoes").unwrap_or_default(),
            nome_tecnico_responsavel: row.take("tecnico_responsavel").unwrap_or_default(),
            atendente: row.take("atendente").unwrap_or_default(),
            telefone_cliente: row.take("telefone").unwrap_or_default(),
            prazo_entrega: row
                .take("DATE_FORMAT(os.prazo_entrega, '%d/%m/%Y')")
                .unwrap_or_default(),
            horario_abertura: data_chegada.clone().unwrap_or_default(),
            data_chegada: data_chegada.unwrap_or_default(),
            historico_edicoes: Vec::new(),
            pecas: Vec::new(),
            total_pecas: 0.0,
            servicos: Vec::new(),
            total_servicos: 0.0,
        })
    }
}

// ============================================================================
// Atualização (com diff de estoque correto e lock pessimista)
// ============================================================================

pub fn atualizar_os(
    tenant_id: i32,
    os_novo: &OrdemServico,
    usuario_logado: &str,
) -> Result<(), ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let mut tx = conn.start_transaction(mysql::TxOpts::default())?;

    // Lock pessimista da OS para serializar edições concorrentes na mesma OS.
    // (P2.6.2a) defense in depth: WHERE tenant_id.
    let os_lock: Option<Row> = tx
        .exec_first(
            "SELECT id FROM ordens_servico WHERE id = ? AND tenant_id = ? FOR UPDATE",
            (os_novo.id, tenant_id),
        )
        .map_err(ErroAplicacao::from)?;
    if os_lock.is_none() {
        return Err(ErroAplicacao::OsNaoEncontrada);
    }

    // Re-busca o estado atual DENTRO da transação (após o lock) para
    // garantir leitura consistente.
    let os_antigo = buscar_os_por_id_no_tx(&mut tx, tenant_id, os_novo.id)?;

    // --- 1. Histórico de edições de campos escalares ---
    if os_antigo.parecer_tecnico != os_novo.parecer_tecnico {
        registrar_historico(
            &mut tx,
            tenant_id,
            os_novo.id,
            usuario_logado,
            "Parecer Técnico",
            &os_antigo.parecer_tecnico,
            &os_novo.parecer_tecnico,
        )?;
    }
    if os_antigo.status != os_novo.status {
        registrar_historico(
            &mut tx,
            tenant_id,
            os_novo.id,
            usuario_logado,
            "Status",
            &format!("{:?}", os_antigo.status),
            &format!("{:?}", os_novo.status),
        )?;
    }
    if os_antigo.situacao != os_novo.situacao {
        registrar_historico(
            &mut tx,
            tenant_id,
            os_novo.id,
            usuario_logado,
            "Situação",
            &format!("{:?}", os_antigo.situacao),
            &format!("{:?}", os_novo.situacao),
        )?;
    }
    if os_antigo.observacoes != os_novo.observacoes {
        registrar_historico(
            &mut tx,
            tenant_id,
            os_novo.id,
            usuario_logado,
            "Observações",
            &os_antigo.observacoes,
            &os_novo.observacoes,
        )?;
    }
    if os_antigo.nome_tecnico_responsavel != os_novo.nome_tecnico_responsavel {
        registrar_historico(
            &mut tx,
            tenant_id,
            os_novo.id,
            usuario_logado,
            "Técnico Responsável",
            &os_antigo.nome_tecnico_responsavel,
            &os_novo.nome_tecnico_responsavel,
        )?;
    }

    // --- 2. Update dos campos escalares ---
    let status_str = serde_json::to_string(&os_novo.status)?.replace('"', "");
    let situacao_str = serde_json::to_string(&os_novo.situacao)?.replace('"', "");
    let affected = tx
        .exec_iter(
            r#"UPDATE ordens_servico
              SET parecer_tecnico     = :parecer,
                  status              = :status,
                  situacao            = :situacao,
                  observacoes         = :obs,
                  tecnico_responsavel = :tecnico
            WHERE id = :id AND tenant_id = :tenant_id"#,
            params! {
                "parecer" => &os_novo.parecer_tecnico,
                "status"  => &status_str,
                "situacao"=> &situacao_str,
                "obs"     => &os_novo.observacoes,
                "tecnico" => &os_novo.nome_tecnico_responsavel,
                "id"      => os_novo.id,
                "tenant_id" => tenant_id,
            },
        )?
        .affected_rows();
    if affected == 0 {
        return Err(ErroAplicacao::OsNaoEncontrada);
    }

    // --- 3. Diff de peças (estoque) ---
    let pecas_antigas_db = tx
        .exec_map(
            "SELECT peca_id, quantidade FROM ordem_servico_pecas WHERE ordem_servico_id = ? AND tenant_id = ?",
            (os_novo.id, tenant_id),
            |(peca_id, qtd): (u32, u32)| (peca_id, qtd),
        )
        .map_err(ErroAplicacao::from)?;
    let mut pecas_antigas: HashMap<u32, i32> = pecas_antigas_db
        .into_iter()
        .map(|(id, q)| (id, q as i32))
        .collect();
    let pecas_novas = agregar_pecas(&os_novo.pecas);

    // União das chaves para iterar todas as peças envolvidas.
    let mut todas: std::collections::HashSet<u32> = pecas_antigas.keys().copied().collect();
    todas.extend(pecas_novas.keys().copied());

    // Bloqueia todas as peças envolvidas em uma ordem determinística
    // (por id) para evitar deadlocks.
    let mut ids_ordenados: Vec<u32> = todas.into_iter().collect();
    ids_ordenados.sort();
    for peca_id in &ids_ordenados {
        tx.exec_drop(
            "SELECT id FROM pecas WHERE id = ? AND tenant_id = ? FOR UPDATE",
            (peca_id, tenant_id),
        )
        .map_err(ErroAplicacao::from)?;
    }

    // Calcula e aplica o delta.
    for peca_id in ids_ordenados {
        let qtd_antiga = pecas_antigas.remove(&peca_id).unwrap_or(0);
        let qtd_nova = *pecas_novas.get(&peca_id).unwrap_or(&0) as i32;
        let delta = qtd_antiga - qtd_nova;
        if delta == 0 {
            continue;
        }
        let tipo = "Ajuste por Edição";
        let motivo = if delta > 0 {
            format!("OS {} editada: devolução de {} un.", os_novo.id, delta)
        } else {
            format!("OS {} editada: baixa de {} un.", os_novo.id, -delta)
        };
        ajustar_estoque(
            &mut tx,
            tenant_id,
            peca_id,
            delta,
            tipo,
            os_novo.id,
            &motivo,
            usuario_logado,
        )?;
    }

    // --- 4. Persistir peças e serviços da nova versão ---
    tx.exec_drop(
        "DELETE FROM ordem_servico_pecas WHERE ordem_servico_id = :id AND tenant_id = :tenant_id",
        params! { "id" => os_novo.id, "tenant_id" => tenant_id },
    )?;
    for peca in &os_novo.pecas {
        tx.exec_drop(
            "INSERT INTO ordem_servico_pecas (tenant_id, ordem_servico_id, peca_id, quantidade, preco_venda_unitario)
             VALUES (?, ?, ?, ?, ?)",
            (tenant_id, os_novo.id, peca.id_peca, peca.quantidade, peca.preco_venda_unitario),
        )?;
    }

    tx.exec_drop(
        "DELETE FROM ordem_servico_servicos WHERE ordem_servico_id = :id AND tenant_id = :tenant_id",
        params! { "id" => os_novo.id, "tenant_id" => tenant_id },
    )?;
    for s in &os_novo.servicos {
        tx.exec_drop(
            "INSERT INTO ordem_servico_servicos (tenant_id, ordem_servico_id, servico_id, quantidade, preco_unitario)
             VALUES (?, ?, ?, ?, ?)",
            (tenant_id, os_novo.id, s.id_servico, s.quantidade, s.preco_unitario),
        )?;
    }

    tx.commit()?;
    Ok(())
}

// ============================================================================
// Criação
// ============================================================================

pub fn criar_os(tenant_id: i32, os: &mut OrdemServico) -> Result<u32, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let mut tx = conn.start_transaction(mysql::TxOpts::default())?;

    // Resolve / cria cliente (no mesmo tenant).
    let cliente_id: i32 = match tx
        .exec_first(
            "SELECT id FROM clientes WHERE nome = ? AND tenant_id = ?",
            (os.cliente.clone(), tenant_id),
        )
        .map_err(ErroAplicacao::from)?
    {
        Some((id,)) => id,
        None => {
            tx.exec_drop(
                "INSERT INTO clientes (tenant_id, nome, telefone) VALUES (?, ?, ?)",
                (tenant_id, os.cliente.clone(), os.telefone_cliente.clone()),
            )
            .map_err(ErroAplicacao::from)?;
            tx.last_insert_id().unwrap_or(0) as i32
        }
    };

    // Resolve / cria equipamento.
    let equipamento_id: i32 = match tx
        .exec_first(
            "SELECT id FROM equipamentos WHERE numero_serie = ? AND tenant_id = ?",
            (os.numero_serie_equipamento.clone(), tenant_id),
        )
        .map_err(ErroAplicacao::from)?
    {
        Some((id,)) => id,
        None => {
            tx.exec_drop(
                "INSERT INTO equipamentos (tenant_id, cliente_id, descricao, numero_serie) VALUES (?, ?, ?, ?)",
                (
                    tenant_id,
                    cliente_id,
                    os.equipamento.clone(),
                    os.numero_serie_equipamento.clone(),
                ),
            )
            .map_err(ErroAplicacao::from)?;
            tx.last_insert_id().unwrap_or(0) as i32
        }
    };

    tx.exec_drop(
        "INSERT INTO ordens_servico
           (tenant_id, cliente_id, equipamento_id, defeito_relatado, observacoes, parecer_tecnico,
            status, situacao, atendente, tecnico_responsavel, prazo_entrega)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        (
            tenant_id,
            cliente_id,
            equipamento_id,
            os.defeito_relatado.clone(),
            os.observacoes.clone(),
            os.parecer_tecnico.clone(),
            serde_json::to_string(&os.status)?.replace('"', ""),
            serde_json::to_string(&os.situacao)?.replace('"', ""),
            os.atendente.clone(),
            os.nome_tecnico_responsavel.clone(),
            os.prazo_entrega.clone(),
        ),
    )
    .map_err(ErroAplicacao::from)?;
    let novo_id = tx.last_insert_id().unwrap_or(0) as u32;

    // Inserir peças e dar baixa no estoque (com lock pessimista).
    let pecas_agregadas = agregar_pecas(&os.pecas);
    let mut ids: Vec<u32> = pecas_agregadas.keys().copied().collect();
    ids.sort();
    for peca_id in &ids {
        tx.exec_drop(
            "SELECT id FROM pecas WHERE id = ? AND tenant_id = ? FOR UPDATE",
            (peca_id, tenant_id),
        )
        .map_err(ErroAplicacao::from)?;
    }
    for peca_id in ids {
        let qtd = *pecas_agregadas.get(&peca_id).unwrap_or(&0) as i32;
        if qtd == 0 {
            continue;
        }
        let pv = os
            .pecas
            .iter()
            .find(|p| p.id_peca == peca_id)
            .map(|p| p.preco_venda_unitario)
            .unwrap_or(0.0);
        tx.exec_drop(
            "INSERT INTO ordem_servico_pecas
               (tenant_id, ordem_servico_id, peca_id, quantidade, preco_venda_unitario)
             VALUES (?, ?, ?, ?, ?)",
            (tenant_id, novo_id, peca_id, qtd, pv),
        )
        .map_err(ErroAplicacao::from)?;
        ajustar_estoque(
            &mut tx,
            tenant_id,
            peca_id,
            -qtd,
            "Saída OS",
            novo_id,
            &format!("OS {} criada", novo_id),
            "system",
        )?;
    }

    // Gravar serviços.
    tx.exec_drop(
        "DELETE FROM ordem_servico_servicos WHERE ordem_servico_id = :id AND tenant_id = :tenant_id",
        params! { "id" => novo_id, "tenant_id" => tenant_id },
    )?;
    for s in &os.servicos {
        tx.exec_drop(
            "INSERT INTO ordem_servico_servicos
               (tenant_id, ordem_servico_id, servico_id, quantidade, preco_unitario)
             VALUES (?, ?, ?, ?, ?)",
            (
                tenant_id,
                novo_id,
                s.id_servico,
                s.quantidade,
                s.preco_unitario,
            ),
        )?;
    }

    tx.commit()?;
    os.id = novo_id;
    Ok(novo_id)
}

// ============================================================================
// Helpers privados
// ============================================================================

/// Versão interna de `buscar_os_por_id` que recebe uma transação já aberta.
fn buscar_os_por_id_no_tx(
    tx: &mut Transaction,
    tenant_id: i32,
    id: u32,
) -> Result<OrdemServico, ErroAplicacao> {
    let row: Option<Row> = tx
        .exec_first(
            r#"SELECT os.id, c.nome, e.descricao, os.defeito_relatado, os.status, os.parecer_tecnico,
                      os.situacao, e.numero_serie, os.observacoes, os.tecnico_responsavel, os.atendente,
                      c.telefone, DATE_FORMAT(os.data_chegada, '%d/%m/%Y %H:%i'),
                      DATE_FORMAT(os.prazo_entrega, '%d/%m/%Y')
                 FROM ordens_servico os
                 LEFT JOIN clientes c ON os.cliente_id = c.id
                 JOIN equipamentos e ON os.equipamento_id = e.id
                WHERE os.id = :id AND os.tenant_id = :tenant_id"#,
            params! { "id" => id, "tenant_id" => tenant_id },
        )?;
    let row = row.ok_or(ErroAplicacao::OsNaoEncontrada)?;
    let mut os = OrdemServico::from_row_opt(row)?;
    os.pecas = tx.exec_map(
        "SELECT p.id, p.codigo_interno, p.descricao, osp.quantidade, osp.preco_venda_unitario
               FROM ordem_servico_pecas osp
               JOIN pecas p ON osp.peca_id = p.id
              WHERE osp.ordem_servico_id = :id AND osp.tenant_id = :tenant_id",
        params! { "id" => id, "tenant_id" => tenant_id },
        |(id_peca, codigo_interno, descricao, quantidade, preco_venda_unitario): (
            u32,
            String,
            String,
            u32,
            f64,
        )| PecaOS {
            id_peca,
            codigo_interno,
            descricao,
            quantidade,
            preco_venda_unitario,
            preco_total: preco_venda_unitario * quantidade as f64,
        },
    )?;
    os.total_pecas = os.pecas.iter().map(|p| p.preco_total).sum();
    Ok(os)
}
