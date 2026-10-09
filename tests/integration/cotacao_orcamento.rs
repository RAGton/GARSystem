// tests/integration/cotacao_orcamento.rs
//
// Testes de integração do módulo Cotação/Orçamento.
// Requer MySQL real. Ver `tests/integration/common.rs`.
//
// #[ignore] por padrão.

#[path = "common.rs"]
mod common;

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn cotacao_criar_listar_obter() {
    use gar_system::cotacao_orcamento::service as co_service;

    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    use mysql::prelude::Queryable;
    let mut conn = pool.get_conn().expect("conn");
    conn.exec_drop("INSERT INTO clientes (nome) VALUES (?)", ("Cliente Q",))
        .expect("insert cliente");
    let cid = conn.last_insert_id() as u32;

    let ctx = co_service::Contexto::default();
    let cot = co_service::criar_cotacao(
        None, cid, "Notebook não liga", None, &ctx,
    ).expect("criar");
    assert_eq!(cot.cliente_id, cid);
    assert_eq!(cot.status, "RASCUNHO");

    // Listar
    let lista = co_service::listar_cotacoes(Some("RASCUNHO"), Some(cid), 10).expect("listar");
    assert_eq!(lista.len(), 1);
    assert_eq!(lista[0].id, cot.id);

    // Obter completa
    let completa = co_service::obter_cotacao_completa(cot.id).expect("obter");
    assert!(completa.is_some());
    assert_eq!(completa.unwrap().itens.len(), 0);
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn cotacao_workflow_completo() {
    use gar_system::cotacao_orcamento::{
        models::StatusCotacao,
        service as co_service,
    };

    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    use mysql::prelude::Queryable;
    let mut conn = pool.get_conn().expect("conn");
    conn.exec_drop("INSERT INTO clientes (nome) VALUES (?)", ("Cliente WF",))
        .expect("insert cliente");
    let cid = conn.last_insert_id() as u32;

    let ctx = co_service::Contexto::default();
    // 1. Criar cotação
    let cot = co_service::criar_cotacao(None, cid, "Teste", None, &ctx).expect("criar");
    assert_eq!(cot.status, "RASCUNHO");

    // 2. RASCUNHO → AGUARDANDO_COTACAO
    let cot = co_service::transicionar_status_cotacao(
        cot.id, StatusCotacao::AguardandoCotacao, &ctx,
    ).expect("trans1");
    assert_eq!(cot.status, "AGUARDANDO_COTACAO");

    // 3. Adicionar item
    co_service::adicionar_item_cotacao(
        cot.id, "Memória 8GB", 2.0, 150.0, None, &ctx,
    ).expect("add item");

    // 4. AGUARDANDO_COTACAO → COTADO
    let cot = co_service::transicionar_status_cotacao(
        cot.id, StatusCotacao::Cotado, &ctx,
    ).expect("trans2");
    assert_eq!(cot.status, "COTADO");

    // 5. COTADO → AGUARDANDO_APROVACAO
    let cot = co_service::transicionar_status_cotacao(
        cot.id, StatusCotacao::AguardandoAprovacao, &ctx,
    ).expect("trans3");
    assert_eq!(cot.status, "AGUARDANDO_APROVACAO");

    // 6. Transição inválida (não pode ir direto para FINALIZADO)
    let r = co_service::transicionar_status_cotacao(
        cot.id, StatusCotacao::Finalizado, &ctx,
    );
    assert!(r.is_err());
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn anexos_persistem_e_listam() {
    use gar_system::cotacao_orcamento::{
        models::TipoAnexo,
        service as co_service,
    };

    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    use mysql::prelude::Queryable;
    let mut conn = pool.get_conn().expect("conn");
    conn.exec_drop("INSERT INTO clientes (nome) VALUES (?)", ("C",))
        .expect("insert");
    let cid = conn.last_insert_id() as u32;
    let ctx = co_service::Contexto::default();
    let cot = co_service::criar_cotacao(None, cid, "Teste", None, &ctx).expect("criar");

    let _a1 = co_service::adicionar_anexo(
        cot.id, TipoAnexo::Imagem, "foto.jpg", "/tmp/foto.jpg",
        Some(1024), Some("image/jpeg"), Some("abc123"),
        &ctx,
    ).expect("anexo 1");
    let _a2 = co_service::adicionar_anexo(
        cot.id, TipoAnexo::Pdf, "orc.pdf", "/tmp/orc.pdf",
        Some(2048), Some("application/pdf"), None,
        &ctx,
    ).expect("anexo 2");

    let anexos = co_service::listar_anexos(cot.id).expect("listar");
    assert_eq!(anexos.len(), 2);
    assert!(anexos.iter().any(|a| a.tipo == "IMAGEM"));
    assert!(anexos.iter().any(|a| a.tipo == "PDF"));
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn cotacao_para_orcamento_copia_itens_e_calcula_total() {
    use gar_system::cotacao_orcamento::{
        models::StatusCotacao,
        service as co_service,
    };

    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    use mysql::prelude::Queryable;
    let mut conn = pool.get_conn().expect("conn");
    conn.exec_drop("INSERT INTO clientes (nome) VALUES (?)", ("C",))
        .expect("insert");
    let cid = conn.last_insert_id() as u32;
    let ctx = co_service::Contexto::default();

    // Cria cotação e adiciona itens
    let cot = co_service::criar_cotacao(None, cid, "Notebook", None, &ctx).expect("criar");
    co_service::transicionar_status_cotacao(cot.id, StatusCotacao::AguardandoCotacao, &ctx).expect("t1");
    co_service::adicionar_item_cotacao(cot.id, "Tela", 1.0, 800.0, None, &ctx).expect("i1");
    co_service::adicionar_item_cotacao(cot.id, "Bateria", 1.0, 200.0, None, &ctx).expect("i2");
    co_service::transicionar_status_cotacao(cot.id, StatusCotacao::Cotado, &ctx).expect("t2");

    // Converte em orçamento
    let orc = co_service::converter_cotacao_para_orcamento(
        cot.id, 50.0, 0.0, &ctx,
    ).expect("converter");

    // Total esperado: (800 + 200) - 50 + 0 = 950
    assert_eq!(orc.subtotal, 1000.0);
    assert_eq!(orc.desconto, 50.0);
    assert_eq!(orc.total, 950.0);
    assert_eq!(orc.cotacao_origem_id, Some(cot.id));
    assert_eq!(orc.status, "AGUARDANDO_APROVACAO");
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn orcamento_decisao_aprova_rejeita() {
    use gar_system::cotacao_orcamento::{
        models::{Decisao, StatusOrcamento},
        service as co_service,
    };

    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    use mysql::prelude::Queryable;
    let mut conn = pool.get_conn().expect("conn");
    conn.exec_drop("INSERT INTO clientes (nome) VALUES (?)", ("C",))
        .expect("insert");
    let cid = conn.last_insert_id() as u32;
    let ctx = co_service::Contexto::default();
    let orc = co_service::criar_orcamento_vazio(cid, None, None, &ctx).expect("criar");

    // Aprovar direto não deve funcionar (precisa estar AGUARDANDO_APROVACAO)
    let r = co_service::decidir_orcamento(orc.id, Decisao::Aprovado, None, &ctx);
    assert!(r.is_err());

    // Forçar status para AGUARDANDO_APROVACAO via SQL (já que não há endpoint pra isso ainda)
    conn.exec_drop(
        "UPDATE orcamentos SET status = 'AGUARDANDO_APROVACAO' WHERE id = :id",
        mysql::params! { "id" => orc.id },
    ).expect("update");

    // Agora aprova
    let orc2 = co_service::decidir_orcamento(orc.id, Decisao::Aprovado, Some("ok"), &ctx).expect("aprovar");
    assert_eq!(orc2.status, "APROVADO");

    // Histórico deve ter 2 entradas (RASCUNHO→AGUARDANDO_APROVACAO, AGUARDANDO_APROVACAO→APROVADO)
    let hist = co_service::listar_historico(orc.id).expect("hist");
    assert_eq!(hist.len(), 2);

    // Aprovações deve ter 1
    let aprovs = co_service::listar_aprovacoes(orc.id).expect("aprovs");
    assert_eq!(aprovs.len(), 1);
    assert_eq!(aprovs[0].decisao, "APROVADO");

    // Finaliza
    let orc3 = co_service::finalizar_orcamento(orc.id, &ctx).expect("finalizar");
    assert_eq!(orc3.status, "FINALIZADO");
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn desconto_recalcula_total() {
    use gar_system::cotacao_orcamento::service as co_service;

    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    use mysql::prelude::Queryable;
    let mut conn = pool.get_conn().expect("conn");
    conn.exec_drop("INSERT INTO clientes (nome) VALUES (?)", ("C",))
        .expect("insert");
    let cid = conn.last_insert_id() as u32;
    let ctx = co_service::Contexto::default();
    let orc = co_service::criar_orcamento_vazio(cid, None, None, &ctx).expect("criar");
    co_service::adicionar_item_orcamento(
        orc.id, "Serviço", 1, 500.0,
        gar_system::cotacao_orcamento::models::CategoriaItem::Servico,
        None, None, None, &ctx,
    ).expect("item");

    // Sem desconto
    let o1 = co_service::obter_orcamento(orc.id).expect("obter").unwrap();
    assert_eq!(o1.subtotal, 500.0);
    assert_eq!(o1.total, 500.0);

    // Aplica desconto
    let o2 = co_service::aplicar_desconto(orc.id, 100.0, &ctx).expect("desconto");
    assert_eq!(o2.desconto, 100.0);
    assert_eq!(o2.total, 400.0);
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn dashboard_conta_por_status() {
    use gar_system::cotacao_orcamento::{
        models::StatusCotacao,
        service as co_service,
    };

    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    use mysql::prelude::Queryable;
    let mut conn = pool.get_conn().expect("conn");
    conn.exec_drop("INSERT INTO clientes (nome) VALUES (?)", ("C",))
        .expect("insert");
    let cid = conn.last_insert_id() as u32;
    let ctx = co_service::Contexto::default();

    // Cria 3 cotações: 1 rascunho, 1 cotado, 1 aprovado
    let c1 = co_service::criar_cotacao(None, cid, "A", None, &ctx).expect("c1");
    let c2 = co_service::criar_cotacao(None, cid, "B", None, &ctx).expect("c2");
    let c3 = co_service::criar_cotacao(None, cid, "C", None, &ctx).expect("c3");
    co_service::transicionar_status_cotacao(c2.id, StatusCotacao::AguardandoCotacao, &ctx).expect("t1");
    co_service::transicionar_status_cotacao(c2.id, StatusCotacao::Cotado, &ctx).expect("t2");
    co_service::transicionar_status_cotacao(c3.id, StatusCotacao::AguardandoCotacao, &ctx).expect("t3");
    co_service::transicionar_status_cotacao(c3.id, StatusCotacao::Cotado, &ctx).expect("t4");
    co_service::transicionar_status_cotacao(c3.id, StatusCotacao::AguardandoAprovacao, &ctx).expect("t5");
    co_service::transicionar_status_cotacao(c3.id, StatusCotacao::Aprovado, &ctx).expect("t6");

    let dash = co_service::dashboard_cotacoes();
    // 1 em RASCUNHO (c1) + 1 COTADO (c2) = 2 abertas
    assert_eq!(dash.cotacoes_abertas, 2);
    // 1 AGUARDANDO_APROVACAO? Não, c3 foi para APROVADO direto
    assert_eq!(dash.aguardando_aprovacao, 0);
    assert_eq!(dash.aprovadas, 1);
}
