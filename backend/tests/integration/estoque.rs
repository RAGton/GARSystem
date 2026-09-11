// tests/integration/estoque.rs
//
// Testes de integração do módulo de estoque.
// Regressão crítica P0-5: atualizar OS não pode duplicar estoque.

#[path = "common.rs"]
mod common;

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn criar_peca_persiste_no_banco() {
    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    let mut conn = pool.get_conn().expect("conn");
    use mysql::params;
    conn.exec_drop(
        "INSERT INTO pecas (codigo_interno, part_number, descricao, fabricante, localizacao, estoque_atual, estoque_minimo, preco_custo, preco_venda) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        ("P001", "PN001", "Parafuso M5", "Genérico", "A1", 100, 10, 0.50, 1.00),
    ).expect("insert");

    let count: u64 = conn.query_first("SELECT COUNT(*) FROM pecas").unwrap().unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn atualizar_peca_altera_quantidade() {
    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    let mut conn = pool.get_conn().expect("conn");
    use mysql::params;
    conn.exec_drop(
        "INSERT INTO pecas (codigo_interno, part_number, descricao, fabricante, localizacao, estoque_atual, estoque_minimo, preco_custo, preco_venda) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        ("P001", "PN001", "Parafuso", "Gen", "A1", 100, 10, 0.50, 1.00),
    ).expect("insert");
    let id = conn.last_insert_id();

    conn.exec_drop(
        "UPDATE pecas SET estoque_atual = 50 WHERE id = :id",
        params! { "id" => id },
    ).expect("update");

    let novo: i32 = conn.exec_first(
        "SELECT estoque_atual FROM pecas WHERE id = :id",
        params! { "id" => id },
    ).unwrap().unwrap();
    assert_eq!(novo, 50);
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn regressao_atualizar_os_nao_duplica_estoque() {
    // P0-5: este é o teste de regressão crítico.
    // Antes do P0, atualizar uma OS 5x com a mesma peça decrementava
    // estoque 5x. Agora deve decrementar apenas o delta.
    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    let mut conn = pool.get_conn().expect("conn");
    use mysql::params;
    // Setup: peça com 100 em estoque
    conn.exec_drop(
        "INSERT INTO pecas (codigo_interno, part_number, descricao, fabricante, localizacao, estoque_atual, estoque_minimo, preco_custo, preco_venda) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        ("P001", "PN001", "P", "F", "L", 100, 0, 1.0, 2.0),
    ).expect("insert peca");
    let peca_id = conn.last_insert_id();

    // Setup: cliente
    conn.exec_drop(
        "INSERT INTO clientes (nome, email, telefone) VALUES (?, ?, ?)",
        ("Cliente", "c@x.com", "123"),
    ).expect("insert cliente");
    let cliente_id = conn.last_insert_id();

    // Setup: equipamento
    conn.exec_drop(
        "INSERT INTO equipamentos (cliente_id, descricao, numero_serie) VALUES (?, ?, ?)",
        (cliente_id, "Notebook", "SN123"),
    ).expect("insert eq");
    let eq_id = conn.last_insert_id();

    // Cria OS
    let os = crate::servicos::OrdemServico {
        id: 0,
        cliente_nome: "Cliente".into(),
        equipamento_descricao: "Notebook".into(),
        defeito_relatado: "Tela quebrada".into(),
        status: crate::servicos::StatusOS::Aberta,
        parecer_tecnico: None,
        situacao: crate::servicos::SituacaoOS::EmAndamento,
        numero_serie: "SN123".into(),
        observacoes: None,
        tecnico_responsavel: None,
        atendente: None,
        telefone: "123".into(),
        data_chegada: "01/01/2026 10:00".into(),
        prazo_entrega: "15/01/2026".into(),
    };
    let os_id = crate::banco_de_dados::ordem_servico::criar_ordem_servico(
        cliente_id, eq_id, &os, "admin"
    ).expect("criar OS");

    // Adiciona 1 peça à OS
    crate::banco_de_dados::ordem_servico::adicionar_pecas_os(
        os_id, &[(peca_id, 1)], "admin"
    ).expect("add peca");

    // Verifica: estoque foi decrementado de 100 para 99
    let apos_add: i32 = conn.exec_first(
        "SELECT estoque_atual FROM pecas WHERE id = :id",
        params! { "id" => peca_id },
    ).unwrap().unwrap();
    assert_eq!(apos_add, 99, "estoque deveria ser 99 após adicionar 1");

    // Atualiza OS com a MESMA peça/quantidade 5x (simulando N edições)
    for _ in 0..5 {
        crate::banco_de_dados::ordem_servico::atualizar_pecas_os(
            os_id, &[(peca_id, 1)], "admin"
        ).expect("atualizar peca");
    }

    // Estoque DEVE continuar em 99 (delta zero a cada update)
    let apos_updates: i32 = conn.exec_first(
        "SELECT estoque_atual FROM pecas WHERE id = :id",
        params! { "id" => peca_id },
    ).unwrap().unwrap();
    assert_eq!(apos_updates, 99, "estoque NÃO deve mudar em updates com mesma peça/quantidade");
}
