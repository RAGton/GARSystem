// tests/integration/crm.rs
//
// Testes de integração do CRM.
//
// Requer MySQL real. Ver `tests/integration/common.rs`.
//
// #[ignore] por padrão — execute com:
//   cargo test --test integration -- --include-ignored

#[path = "common.rs"]
mod common;

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn observacao_criar_persiste_e_gera_timeline() {
    use gar_system::crm::service as crm_service;
    use gar_system::crm::models::TipoEventoTimeline;

    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    // 1. Cria cliente diretamente no DB
    use mysql::prelude::Queryable;
    let mut conn = pool.get_conn().expect("conn");
    conn.exec_drop(
        "INSERT INTO clientes (nome, email, telefone) VALUES (?, ?, ?)",
        ("Cliente Teste", "c@x.com", "1111-1111"),
    )
    .expect("insert cliente");
    let cliente_id = conn.last_insert_id() as u32;

    // 2. Cria observação
    let ctx = crm_service::Contexto {
        usuario_id: Some(1),
        username: Some("admin".to_string()),
        ip: None,
        user_agent: None,
        request_id: Some("rid-1".to_string()),
    };
    let obs = crm_service::adicionar_observacao(
        cliente_id,
        "Cliente VIP, prefere atendimento pela manhã",
        &ctx,
    )
    .expect("add obs");
    assert!(!obs.conteudo.is_empty());
    assert_eq!(obs.vezes_editada, 0);
    assert!(!obs.editada);

    // 3. Verifica que timeline tem o evento
    let timeline = crm_service::listar_timeline(cliente_id, 10).expect("timeline");
    assert_eq!(timeline.len(), 1);
    assert_eq!(timeline[0].tipo, TipoEventoTimeline::ObservacaoAdicionada.as_db_str());
    assert!(timeline[0].descricao.contains("admin"));
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn observacao_editar_incrementa_vezes_editada() {
    use gar_system::crm::service as crm_service;

    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    use mysql::prelude::Queryable;
    let mut conn = pool.get_conn().expect("conn");
    conn.exec_drop(
        "INSERT INTO clientes (nome) VALUES (?)",
        ("Cliente Edit",),
    )
    .expect("insert cliente");
    let cid = conn.last_insert_id() as u32;

    let ctx = crm_service::Contexto {
        usuario_id: Some(1),
        username: Some("admin".to_string()),
        ip: None,
        user_agent: None,
        request_id: Some("rid-2".to_string()),
    };
    let obs = crm_service::adicionar_observacao(cid, "v1", &ctx).expect("add");
    assert_eq!(obs.vezes_editada, 0);

    let obs2 = crm_service::editar_observacao(obs.id, "v2", &ctx).expect("edit");
    assert!(obs2.editada);
    assert_eq!(obs2.vezes_editada, 1);
    assert_eq!(obs2.conteudo, "v2");

    let obs3 = crm_service::editar_observacao(obs.id, "v3", &ctx).expect("edit2");
    assert_eq!(obs3.vezes_editada, 2);
    assert_eq!(obs3.conteudo, "v3");
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn tag_atribuir_e_remover() {
    use gar_system::crm::service as crm_service;

    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    use mysql::prelude::Queryable;
    let mut conn = pool.get_conn().expect("conn");
    conn.exec_drop(
        "INSERT INTO clientes (nome) VALUES (?)",
        ("Cliente Tag",),
    )
    .expect("insert cliente");
    let cid = conn.last_insert_id() as u32;

    // Tag "VIP" foi pré-populada na seed da migration 0004
    let tags = crm_service::listar_tags().expect("tags");
    let vip = tags.iter().find(|t| t.nome == "VIP").expect("VIP existe");

    let ctx = crm_service::Contexto::default();
    crm_service::atribuir_tag(cid, vip.id, &ctx).expect("atribuir");
    let tags_cliente = crm_service::listar_tags_do_cliente(cid).expect("tags cliente");
    assert_eq!(tags_cliente.len(), 1);
    assert_eq!(tags_cliente[0].nome, "VIP");

    let removed = crm_service::remover_tag(cid, vip.id, &ctx).expect("remover");
    assert!(removed);
    let tags_cliente2 = crm_service::listar_tags_do_cliente(cid).expect("tags cliente 2");
    assert!(tags_cliente2.is_empty());
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn contato_adicionar_unico_principal() {
    use gar_system::crm::service as crm_service;

    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    use mysql::prelude::Queryable;
    let mut conn = pool.get_conn().expect("conn");
    conn.exec_drop(
        "INSERT INTO clientes (nome) VALUES (?)",
        ("Cliente Contato",),
    )
    .expect("insert cliente");
    let cid = conn.last_insert_id() as u32;

    let ctx = crm_service::Contexto::default();
    let _c1 = crm_service::adicionar_contato(
        cid, "TELEFONE", "1111-1111", None, true, None, &ctx,
    ).expect("c1");
    // Tenta adicionar outro telefone como principal — o primeiro deve virar não-principal
    let _c2 = crm_service::adicionar_contato(
        cid, "WHATSAPP", "9999-9999", None, true, None, &ctx,
    ).expect("c2");

    let contatos = crm_service::listar_contatos(cid).expect("contatos");
    assert_eq!(contatos.len(), 2);
    // Apenas o segundo (mais recente) deve estar como principal
    let principais: Vec<_> = contatos.iter().filter(|c| c.principal).collect();
    assert_eq!(principais.len(), 1);
    assert_eq!(principais[0].valor, "9999-9999");
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn busca_global_encontra_por_nome_cpf_telefone() {
    use gar_system::crm::service as crm_service;

    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    use mysql::prelude::Queryable;
    let mut conn = pool.get_conn().expect("conn");
    conn.exec_drop(
        "INSERT INTO clientes (nome, cpf_cnpj, telefone) VALUES (?, ?, ?)",
        ("João Silva", "123.456.789-00", "(11) 98765-4321"),
    )
    .expect("insert 1");
    conn.exec_drop(
        "INSERT INTO clientes (nome, cpf_cnpj, telefone) VALUES (?, ?, ?)",
        ("Maria Souza", "987.654.321-00", "(21) 91234-5678"),
    )
    .expect("insert 2");

    // Busca por nome
    let r1 = crm_service::buscar_clientes("João", 20).expect("busca nome");
    assert_eq!(r1.len(), 1);
    assert_eq!(r1[0]["nome"], "João Silva");

    // Busca por CPF
    let r2 = crm_service::buscar_clientes("987.654", 20).expect("busca cpf");
    assert_eq!(r2.len(), 1);
    assert_eq!(r2[0]["nome"], "Maria Souza");

    // Busca por telefone
    let r3 = crm_service::buscar_clientes("98765", 20).expect("busca tel");
    assert_eq!(r3.len(), 1);
    assert_eq!(r3[0]["nome"], "João Silva");

    // Busca vazia
    let r4 = crm_service::buscar_clientes("xyzzy", 20).expect("busca vazia");
    assert_eq!(r4.len(), 0);
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn equipamento_adicionar_com_tipo() {
    use gar_system::crm::service as crm_service;

    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    use mysql::prelude::Queryable;
    let mut conn = pool.get_conn().expect("conn");
    conn.exec_drop(
        "INSERT INTO clientes (nome) VALUES (?)",
        ("Cliente Eq",),
    )
    .expect("insert cliente");
    let cid = conn.last_insert_id() as u32;

    let ctx = crm_service::Contexto::default();
    let eq = crm_service::adicionar_equipamento(
        cid,
        "Notebook Dell",
        Some("Dell"),
        Some("Inspiron 15"),
        Some("SN-12345"),
        Some("PAT-0001"),
        Some("sem carregador"),
        "NOTEBOOK",
        &ctx,
    )
    .expect("add eq");
    assert_eq!(eq.tipo, "NOTEBOOK");
    assert_eq!(eq.marca, Some("Dell".to_string()));
    assert_eq!(eq.patrimonio, Some("PAT-0001".to_string()));

    let eqs = crm_service::listar_equipamentos(cid).expect("listar");
    assert_eq!(eqs.len(), 1);
    assert_eq!(eqs[0].id, eq.id);
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn dashboard_agrega_tudo() {
    use gar_system::crm::service as crm_service;

    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    use mysql::prelude::Queryable;
    let mut conn = pool.get_conn().expect("conn");
    conn.exec_drop(
        "INSERT INTO clientes (nome, email) VALUES (?, ?)",
        ("Cliente Dash", "d@x.com"),
    )
    .expect("insert cliente");
    let cid = conn.last_insert_id() as u32;

    let ctx = crm_service::Contexto::default();
    crm_service::adicionar_observacao(cid, "obs 1", &ctx).expect("obs");
    crm_service::adicionar_contato(cid, "EMAIL", "d@x.com", None, true, None, &ctx).expect("contato");
    crm_service::adicionar_equipamento(cid, "PC", None, None, None, None, None, "DESKTOP", &ctx).expect("eq");

    let dash = crm_service::dashboard_cliente(cid).expect("dashboard");
    assert!(dash.cliente["nome"].as_str().unwrap().contains("Cliente Dash"));
    assert_eq!(dash.observacoes_recentes.len(), 1);
    assert_eq!(dash.contatos.len(), 1);
    assert_eq!(dash.equipamentos.len(), 1);
    assert!(!dash.timeline.is_empty());
}
