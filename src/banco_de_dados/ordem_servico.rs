// src/banco_de_dados/ordem_servico.rs

use super::conexao::obter_conexao;
use crate::servicos::{ErroAplicacao, HistoricoEdicao, OrdemServico, PecaOS, SituacaoOS, StatusOS};
use mysql::{
    params,
    prelude::{FromRow, Queryable},
    FromRowError, Row, Transaction, Value,
};

// --- [CORREÇÃO APLICADA AQUI] ---
// A assinatura completa da função foi restaurada.
fn registrar_historico(
    tx: &mut Transaction,
    os_id: u32,
    usuario: &str,
    campo: &str,
    antigo: &str,
    novo: &str,
) -> Result<(), mysql::Error> {
    tx.exec_drop(
        r"INSERT INTO historico_edicoes
            (ordem_servico_id, usuario, campo_alterado, valor_antigo, valor_novo)
          VALUES (:os_id, :usuario, :campo, :antigo, :novo)",
        params! { os_id, usuario, campo, antigo, novo },
    )?;
    Ok(())
}

pub fn listar_ordens_servico() -> Result<Vec<OrdemServico>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    // --- [CORREÇÃO APLICADA AQUI] ---
    // Usamos query_map, mas passamos a referência à nossa implementação manual `OrdemServico::from_row`.
    // Isso evita o erro da tupla grande.
    let ordens = conn.query_map(
        r#"
        SELECT
            os.id, c.nome, e.descricao, os.defeito_relatado, os.status, os.parecer_tecnico,
            os.situacao, e.numero_serie, os.observacoes, os.tecnico_responsavel,
            os.atendente, c.telefone, DATE_FORMAT(os.data_chegada, '%d/%m/%Y %H:%i'), DATE_FORMAT(os.prazo_entrega, '%d/%m/%Y')
        FROM ordens_servico os
        LEFT JOIN clientes c ON os.cliente_id = c.id
        JOIN equipamentos e ON os.equipamento_id = e.id
        ORDER BY os.id DESC
        LIMIT 100
        "#,
        OrdemServico::from_row,
    )?;
    Ok(ordens)
}

pub fn buscar_os_por_id(id: u32) -> Result<OrdemServico, ErroAplicacao> {
    let mut conn = obter_conexao()?;

    let row = conn.exec_first(
        r#"SELECT os.id, c.nome, e.descricao, os.defeito_relatado, os.status, os.parecer_tecnico,
            os.situacao, e.numero_serie, os.observacoes, os.tecnico_responsavel, os.atendente,
            c.telefone, DATE_FORMAT(os.data_chegada, '%d/%m/%Y %H:%i'), DATE_FORMAT(os.prazo_entrega, '%d/%m/%Y')
           FROM ordens_servico os LEFT JOIN clientes c ON os.cliente_id = c.id
           JOIN equipamentos e ON os.equipamento_id = e.id WHERE os.id = :id"#,
        params! { "id" => id },
    )?.ok_or(ErroAplicacao::OsNaoEncontrada)?;

    // --- [CORREÇÃO APLICADA AQUI] ---
    // O '?' agora funciona corretamente porque implementamos `From<FromRowError>` no servicos.rs
    let mut os = OrdemServico::from_row_opt(row)?;

    os.historico_edicoes = conn.exec_map(
        "SELECT usuario, DATE_FORMAT(data_hora, '%d/%m/%Y %H:%i'), campo_alterado, valor_antigo, valor_novo FROM historico_edicoes WHERE ordem_servico_id = :id ORDER BY data_hora DESC",
        params! { "id" => id },
        |(usuario, data_hora, campo_alterado, valor_antigo, valor_novo)| HistoricoEdicao { usuario, data_hora, campo_alterado, valor_antigo, valor_novo },
    )?;

    os.pecas = conn.exec_map(
        "SELECT p.id, p.codigo_interno, p.descricao, osp.quantidade, osp.preco_venda_unitario FROM ordem_servico_pecas osp JOIN pecas p ON osp.peca_id = p.id WHERE osp.ordem_servico_id = :id",
        params! { "id" => id },
        |(id_peca, codigo_interno, descricao, quantidade, preco_venda_unitario): (u32, String, String, u32, f64)| PecaOS {
            id_peca, codigo_interno, descricao, quantidade, preco_venda_unitario,
            preco_total: preco_venda_unitario * quantidade as f64,
        },
    )?;

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
        })
    }
}

pub fn atualizar_os(os_novo: &OrdemServico, usuario_logado: &str) -> Result<(), ErroAplicacao> {
    let os_antigo = buscar_os_por_id(os_novo.id)?;
    let mut conn = obter_conexao()?;
    let mut tx = conn.start_transaction(mysql::TxOpts::default())?;

    if os_antigo.parecer_tecnico != os_novo.parecer_tecnico {
        registrar_historico(
            &mut tx,
            os_novo.id,
            usuario_logado,
            "Parecer Técnico",
            &os_antigo.parecer_tecnico,
            &os_novo.parecer_tecnico,
        )?;
    }
    // ... (restante das chamadas para registrar_historico)
    if os_antigo.status != os_novo.status {
        registrar_historico(
            &mut tx,
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
            os_novo.id,
            usuario_logado,
            "Técnico Responsável",
            &os_antigo.nome_tecnico_responsavel,
            &os_novo.nome_tecnico_responsavel,
        )?;
    }

    let status_str = serde_json::to_string(&os_novo.status)?.replace('"', "");
    let situacao_str = serde_json::to_string(&os_novo.situacao)?.replace('"', "");
    tx.exec_drop(
        r#"UPDATE ordens_servico SET parecer_tecnico = :parecer, status = :status, situacao = :situacao, observacoes = :obs, tecnico_responsavel = :tecnico WHERE id = :id"#,
        params! { "parecer" => &os_novo.parecer_tecnico, "status" => &status_str, "situacao" => &situacao_str, "obs" => &os_novo.observacoes, "tecnico" => &os_novo.nome_tecnico_responsavel, "id" => os_novo.id, },
    )?;

    tx.exec_drop(
        "DELETE FROM ordem_servico_pecas WHERE ordem_servico_id = :id",
        params! { "id" => os_novo.id },
    )?;

    for peca in &os_novo.pecas {
        tx.exec_drop(
            "INSERT INTO ordem_servico_pecas (ordem_servico_id, peca_id, quantidade, preco_venda_unitario) VALUES (?, ?, ?, ?)",
            (os_novo.id, peca.id_peca, peca.quantidade, peca.preco_venda_unitario)
        )?;

        let (estoque_anterior,): (i32,) = tx
            .exec_first(
                "SELECT estoque_atual FROM pecas WHERE id = ?",
                (peca.id_peca,),
            )?
            .unwrap_or((0,));
        let estoque_novo = estoque_anterior - peca.quantidade as i32;

        tx.exec_drop(
            "UPDATE pecas SET estoque_atual = ? WHERE id = ?",
            (estoque_novo, peca.id_peca),
        )?;

        tx.exec_drop(
            r#"INSERT INTO movimentos_estoque (peca_id, tipo_movimento, referencia_id, quantidade_movimentada, estoque_anterior, estoque_novo, usuario) VALUES (?, 'Saída OS', ?, ?, ?, ?, ?)"#,
            (peca.id_peca, os_novo.id, -(peca.quantidade as i32), estoque_anterior, estoque_novo, usuario_logado)
        )?;
    }

    tx.commit()?;
    Ok(())
}

/// Cria uma nova ordem de serviço e retorna o id criado
pub fn criar_os(os: &mut OrdemServico) -> Result<u32, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let mut tx = conn.start_transaction(mysql::TxOpts::default())?;

    // Primeiro, tentamos resolver/insrir cliente e equipamento minimalmente.
    // Aqui assumimos que `clientes` e `equipamentos` já existem ou são criados externamente.
    // Para simplicidade, vamos inserir um cliente se não existir (nome fornecido).
    let cliente_id: i32 = tx.exec_first(
        "SELECT id FROM clientes WHERE nome = ?",
        (os.cliente.clone(),),
    )?.map(|(id,)| id).unwrap_or_else(|| {
        let res = tx.exec_drop(
            "INSERT INTO clientes (nome, telefone) VALUES (?, ?)",
            (os.cliente.clone(), os.telefone_cliente.clone()),
        );
        // pega o último id
        let id = tx.last_insert_id().unwrap_or(0) as i32;
        id
    });

    // Para equipamento, faremos uma inserção simples se não houver número de série.
    let equipamento_id: i32 = tx.exec_first(
        "SELECT id FROM equipamentos WHERE numero_serie = ?",
        (os.numero_serie_equipamento.clone(),),
    )?.map(|(id,)| id).unwrap_or_else(|| {
        let res = tx.exec_drop(
            "INSERT INTO equipamentos (cliente_id, descricao, numero_serie) VALUES (?, ?, ?)",
            (cliente_id, os.equipamento.clone(), os.numero_serie_equipamento.clone()),
        );
        tx.last_insert_id().unwrap_or(0) as i32
    });

    tx.exec_drop(
        "INSERT INTO ordens_servico (cliente_id, equipamento_id, defeito_relatado, observacoes, parecer_tecnico, status, situacao, atendente, tecnico_responsavel, prazo_entrega) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        (
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
    )?;

    let novo_id = tx.last_insert_id().unwrap_or(0) as u32;

    // Inserir peças e movimentações
    for p in &os.pecas {
        tx.exec_drop(
            "INSERT INTO ordem_servico_pecas (ordem_servico_id, peca_id, quantidade, preco_venda_unitario) VALUES (?, ?, ?, ?)",
            (novo_id, p.id_peca, p.quantidade, p.preco_venda_unitario),
        )?;

        // Atualiza estoque
        let estoque_anterior: i32 = tx.exec_first("SELECT estoque_atual FROM pecas WHERE id = ?", (p.id_peca,))?.map(|(v,)| v).unwrap_or(0);
        let estoque_novo = estoque_anterior - p.quantidade as i32;
        tx.exec_drop("UPDATE pecas SET estoque_atual = ? WHERE id = ?", (estoque_novo, p.id_peca))?;
        tx.exec_drop(r#"INSERT INTO movimentos_estoque (peca_id, tipo_movimento, referencia_id, quantidade_movimentada, estoque_anterior, estoque_novo, usuario) VALUES (?, 'Saída OS', ?, ?, ?, ?, ?)"#, (p.id_peca, novo_id, -(p.quantidade as i32), estoque_anterior, estoque_novo, "system"))?;
    }

    tx.commit()?;

    os.id = novo_id;
    Ok(novo_id)
}
