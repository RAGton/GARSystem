// tests/integration/financial.rs
//
// Testes de integração — Sprint P2.5.

#[path = "common.rs"]
mod common;

use senior_system::financial::{self as fin, OrigemPagar, OrigemReceber, StatusConta};

fn ctx_padrao() -> fin::Contexto {
    fin::Contexto {
        usuario_id: 1,
        username: Some("teste".into()),
        ip: Some("127.0.0.1".into()),
        user_agent: Some("integration".into()),
        request_id: Some("test".into()),
    }
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn configuracao_inicial_existe() {
    let _pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    let cfg = fin::obter_configuracao().expect("config");
    assert_eq!(cfg.dias_vencimento_padrao, 7);
    assert_eq!(cfg.moeda_padrao, "BRL");
    assert!(cfg.plano_conta_padrao_receber_id.is_some());
    assert!(cfg.centro_custo_padrao_receber_id.is_some());
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn criar_e_pagar_conta_receber() {
    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    use mysql::prelude::Queryable;
    let mut conn = pool.get_conn().expect("conn");
    conn.exec_drop("INSERT INTO clientes (nome) VALUES ('C')", ()).expect("cli");
    let cid = conn.last_insert_id() as u32;

    let venc = chrono::Utc::now().date_naive() + chrono::Duration::days(7);
    let id = fin::criar_conta_receber(
        cid,
        OrigemReceber::Manual,
        None,
        "Manutenção preventiva",
        100_00,  // R$ 100,00
        venc,
        1,  // centro_custo LAB
        1,  // plano_conta Manutenção
        None,
        &ctx_padrao(),
    )
    .expect("criar");
    assert!(id > 0);

    // Pagar parcial
    let c1 = fin::pagar_conta_receber(
        id, 30_00, chrono::Utc::now().date_naive(), Some("PIX"), &ctx_padrao(),
    ).expect("pagar parcial");
    assert_eq!(c1.status, StatusConta::Parcial);
    assert_eq!(c1.valor_pago, 30_00);

    // Pagar restante
    let c2 = fin::pagar_conta_receber(
        id, 70_00, chrono::Utc::now().date_naive(), Some("PIX"), &ctx_padrao(),
    ).expect("pagar restante");
    assert_eq!(c2.status, StatusConta::Pago);
    assert_eq!(c2.valor_pago, 100_00);

    // Tentar pagar de novo (já pago)
    let r = fin::pagar_conta_receber(
        id, 10_00, chrono::Utc::now().date_naive(), None, &ctx_padrao(),
    );
    assert!(matches!(r, Err(fin::ErroFinanceiro::ContaJaFinalizada)));
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn criar_e_pagar_conta_pagar() {
    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    let venc = chrono::Utc::now().date_naive() + chrono::Duration::days(15);
    let id = fin::criar_conta_pagar(
        "Fornecedor X",
        Some("12.345.678/0001-00"),
        "Compra de peças",
        50_000_00,  // R$ 50.000
        venc,
        4,  // ADM
        11,  // Contabilidade
        OrigemPagar::Manual,
        None,
        None,
        &ctx_padrao(),
    )
    .expect("criar");
    assert!(id > 0);

    let c = fin::pagar_conta_pagar(
        id, 50_000_00, chrono::Utc::now().date_naive(), Some("TRANSFERENCIA"), &ctx_padrao(),
    ).expect("pagar");
    assert_eq!(c.status, StatusConta::Pago);
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn valor_pago_excede_saldo() {
    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    use mysql::prelude::Queryable;
    let mut conn = pool.get_conn().expect("conn");
    conn.exec_drop("INSERT INTO clientes (nome) VALUES ('C')", ()).expect("cli");
    let cid = conn.last_insert_id() as u32;
    let id = fin::criar_conta_receber(
        cid, OrigemReceber::Manual, None, "x", 100_00,
        chrono::Utc::now().date_naive() + chrono::Duration::days(7),
        1, 1, None, &ctx_padrao(),
    ).expect("criar");
    let r = fin::pagar_conta_receber(
        id, 200_00, chrono::Utc::now().date_naive(), None, &ctx_padrao(),
    );
    assert!(matches!(r, Err(fin::ErroFinanceiro::ValorPagoExcede)));
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn status_efetivo_atrasado() {
    use chrono::Datelike;
    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    use mysql::prelude::Queryable;
    let mut conn = pool.get_conn().expect("conn");
    conn.exec_drop("INSERT INTO clientes (nome) VALUES ('C')", ()).expect("cli");
    let cid = conn.last_insert_id() as u32;
    let venc = chrono::Utc::now().date_naive() - chrono::Duration::days(10);
    let id = fin::criar_conta_receber(
        cid, OrigemReceber::Manual, None, "x", 100_00,
        venc, 1, 1, None, &ctx_padrao(),
    ).expect("criar");
    let c = fin::obter_conta_receber(id).expect("obter").unwrap();
    assert_eq!(c.calcular_status_efetivo(chrono::Utc::now().date_naive()), StatusConta::Atrasado);
    // Garantir que hoje.year() existe (sanity)
    let _ = chrono::Utc::now().year();
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn fluxo_caixa_calcula_saldo() {
    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    use mysql::prelude::Queryable;
    let mut conn = pool.get_conn().expect("conn");
    conn.exec_drop("INSERT INTO clientes (nome) VALUES ('C')", ()).expect("cli");
    let cid = conn.last_insert_id() as u32;
    // 1 conta a receber de R$ 1000
    let id = fin::criar_conta_receber(
        cid, OrigemReceber::Manual, None, "x", 100_000_00,
        chrono::Utc::now().date_naive() + chrono::Duration::days(7),
        1, 1, None, &ctx_padrao(),
    ).expect("criar");
    // Paga R$ 600
    let _ = fin::pagar_conta_receber(
        id, 60_000_00, chrono::Utc::now().date_naive(), Some("PIX"), &ctx_padrao(),
    ).expect("pagar");

    let hoje = chrono::Utc::now().date_naive();
    let fluxo = fin::calcular_fluxo(hoje, hoje + chrono::Duration::days(30)).expect("fluxo");
    // Saldo realizado = +R$ 600 (60000_00 centavos)
    assert_eq!(fluxo.saldo_realizado, 60_000_00);
    // Saldo projetado = realizado + pendente a receber (40_000_00) - pendente a pagar (0)
    assert_eq!(fluxo.saldo_projetado, 100_000_00);
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn gerar_conta_receber_de_orcamento_aprovado() {
    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    use mysql::prelude::Queryable;
    let mut conn = pool.get_conn().expect("conn");
    conn.exec_drop("INSERT INTO clientes (nome) VALUES ('C')", ()).expect("cli");
    let cid = conn.last_insert_id() as u32;

    // Insere orçamento direto no banco
    conn.exec_drop(
        "INSERT INTO orcamentos (cliente_id, status, total) VALUES (?, 'AGUARDANDO_APROVACAO', 950.0)",
        (cid,),
    ).expect("orc");
    let oid = conn.last_insert_id() as u32;

    // Aprova via hook do cotacao_orcamento
    use senior_system::cotacao_orcamento::{models::Decisao, service as co_svc};
    let ctx = senior_system::cotacao_orcamento::Contexto {
        usuario_id: Some(1),
        username: Some("teste".into()),
        ip: None,
        user_agent: None,
        request_id: None,
    };
    co_svc::decidir_orcamento(oid, Decisao::Aprovado, Some("ok"), &ctx).expect("decidir");

    // Verifica que ContaReceber foi gerada
    let lista = fin::listar_contas_receber(Some(cid), Some(StatusConta::Pendente), 10).expect("listar");
    assert_eq!(lista.len(), 1);
    assert_eq!(lista[0].origem_tipo, OrigemReceber::Orcamento);
    assert_eq!(lista[0].origem_id, Some(oid));
    assert_eq!(lista[0].valor, 95_000);  // 950.00 * 100
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn dashboard_financeiro_retorna_metricas() {
    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    use mysql::prelude::Queryable;
    let mut conn = pool.get_conn().expect("conn");
    conn.exec_drop("INSERT INTO clientes (nome) VALUES ('C')", ()).expect("cli");
    let cid = conn.last_insert_id() as u32;
    let _ = fin::criar_conta_receber(
        cid, OrigemReceber::Manual, None, "x", 50_000_00,
        chrono::Utc::now().date_naive() + chrono::Duration::days(7),
        1, 1, None, &ctx_padrao(),
    ).expect("criar");
    let _ = fin::criar_conta_pagar(
        "Forn", None, "x", 20_000_00,
        chrono::Utc::now().date_naive() + chrono::Duration::days(15),
        4, 11, OrigemPagar::Manual, None, None, &ctx_padrao(),
    ).expect("criar");

    let dash = fin::dashboard().expect("dashboard");
    assert!(dash.contas_receber_pendentes >= 1);
    assert!(dash.contas_pagar_pendentes >= 1);
    assert!(dash.receita_prevista >= 50_000_00);
    assert!(dash.despesa_prevista >= 20_000_00);
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn verificar_alertas_cria_ocorrencia_atrasada() {
    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    use mysql::prelude::Queryable;
    let mut conn = pool.get_conn().expect("conn");
    conn.exec_drop("INSERT INTO clientes (nome) VALUES ('C')", ()).expect("cli");
    let cid = conn.last_insert_id() as u32;
    let _ = fin::criar_conta_receber(
        cid, OrigemReceber::Manual, None, "x", 100_00,
        chrono::Utc::now().date_naive() - chrono::Duration::days(5),  // atrasada
        1, 1, None, &ctx_padrao(),
    ).expect("criar");

    let criadas = fin::verificar_alertas().expect("verificar");
    assert!(criadas >= 1);

    let pendentes = fin::listar_alertas_pendentes(50).expect("listar");
    assert!(!pendentes.is_empty());
    assert!(pendentes.iter().any(|a| a.severidade == fin::Severidade::Critical));
}
