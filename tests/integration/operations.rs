// tests/integration/operations.rs
//
// Testes de integração — Sprint P2.4.

#[path = "common.rs"]
mod common;

use gar_system::operations::{self as ops, TipoEvento};

fn ctx_padrao() -> ops::Contexto {
    ops::Contexto {
        usuario_id: 1,
        username: Some("teste".into()),
        ip: Some("127.0.0.1".into()),
        user_agent: Some("integration".into()),
        request_id: Some("test".into()),
    }
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn workflow_default_existe() {
    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    let wf = ops::obter_workflow_padrao().expect("workflow");
    assert!(wf.ativo);
    assert_eq!(wf.entidade_tipo, "OS");

    let estados = ops::listar_estados().expect("estados");
    assert!(estados.len() >= 10);
    let inicial = estados.iter().find(|e| e.eh_inicial).expect("inicial");
    assert_eq!(inicial.slug, "RECEBIDO");

    let transicoes = ops::listar_transicoes().expect("transicoes");
    assert!(transicoes.len() >= 10);
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn mover_estado_valido_registra_movimentacao() {
    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    use mysql::prelude::Queryable;
    let mut conn = pool.get_conn().expect("conn");
    conn.exec_drop("INSERT INTO clientes (nome) VALUES ('C')", ())
        .expect("cli");
    let cid = conn.last_insert_id() as u32;
    conn.exec_drop(
        "INSERT INTO ordens_servico (cliente_id, status, descricao) VALUES (?, 'ABERTA', 'OS')",
        (cid,),
    )
    .expect("os");
    let os_id = conn.last_insert_id() as u32;

    // Recebido → Diagnóstico
    let mov = ops::mover_estado(os_id, "DIAGNOSTICO", None, &ctx_padrao()).expect("mover");
    assert_eq!(mov.entidade_id, os_id);

    // Status atualizado
    let status = ops::obter_status_workflow(os_id).expect("status").unwrap();
    assert_eq!(status.estado_atual.slug, "DIAGNOSTICO");
    assert_eq!(status.historico.len(), 1);
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn mover_estado_invalido_retorna_erro() {
    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    use mysql::prelude::Queryable;
    let mut conn = pool.get_conn().expect("conn");
    conn.exec_drop("INSERT INTO clientes (nome) VALUES ('C')", ())
        .expect("cli");
    let cid = conn.last_insert_id() as u32;
    conn.exec_drop(
        "INSERT INTO ordens_servico (cliente_id, status, descricao) VALUES (?, 'ABERTA', 'OS')",
        (cid,),
    )
    .expect("os");
    let os_id = conn.last_insert_id() as u32;

    // Recebido → FINALIZADO não é uma transição válida
    let r = ops::mover_estado(os_id, "FINALIZADO", None, &ctx_padrao());
    assert!(matches!(
        r,
        Err(ops::ErroOperacao::TransicaoInvalida(_, _))
    ));
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn kanban_organiza_por_estado() {
    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    use mysql::prelude::Queryable;
    let mut conn = pool.get_conn().expect("conn");
    conn.exec_drop("INSERT INTO clientes (nome) VALUES ('C1'), ('C2')", ())
        .expect("cli");
    let cid1 = conn.last_insert_id() as u32 - 1;
    let cid2 = conn.last_insert_id() as u32;
    conn.exec_drop(
        "INSERT INTO ordens_servico (cliente_id, status, descricao) VALUES (?, 'ABERTA', 'A')",
        (cid1,),
    )
    .expect("os1");
    conn.exec_drop(
        "INSERT INTO ordens_servico (cliente_id, status, descricao) VALUES (?, 'EM_DIAGNOSTICO', 'B')",
        (cid2,),
    )
    .expect("os2");

    let cols = ops::kanban().expect("kanban");
    let recebido = cols.iter().find(|c| c.estado.slug == "RECEBIDO").expect("RECEBIDO");
    let diagnostico = cols.iter().find(|c| c.estado.slug == "DIAGNOSTICO").expect("DIAG");
    assert_eq!(recebido.cards.len(), 1);
    assert_eq!(diagnostico.cards.len(), 1);
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn criar_evento_agenda_sem_conflito() {
    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    let inicio = chrono::Utc::now() + chrono::Duration::days(1);
    let fim = inicio + chrono::Duration::hours(2);
    let id = ops::criar_evento_agenda(
        1, None, None, "Visita X", Some("Atendimento"), TipoEvento::Visita,
        inicio, fim, None, None, None, &ctx_padrao(), false,
    )
    .expect("criar");
    assert!(id > 0);

    // Segundo evento conflitante deve falhar
    let r = ops::criar_evento_agenda(
        1, None, None, "Outro", None, TipoEvento::Visita,
        inicio + chrono::Duration::minutes(30),
        inicio + chrono::Duration::hours(3),
        None, None, None, &ctx_padrao(), false,
    );
    assert!(matches!(r, Err(ops::ErroOperacao::ConflitoAgenda)));
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn calcular_sla_detecta_violacao() {
    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    use mysql::prelude::Queryable;
    let mut conn = pool.get_conn().expect("conn");
    conn.exec_drop("INSERT INTO clientes (nome) VALUES ('C')", ())
        .expect("cli");
    let cid = conn.last_insert_id() as u32;
    conn.exec_drop(
        "INSERT INTO ordens_servico (cliente_id, status, descricao) VALUES (?, 'ABERTA', 'OS')",
        (cid,),
    )
    .expect("os");
    let os_id = conn.last_insert_id() as u32;

    // Inserir eventos SLA com intervalo grande (10h de diagnóstico > 1h do SLA default)
    let t0 = chrono::Utc::now() - chrono::Duration::hours(10);
    gar_system::operations::repository::registrar_evento_sla(
        os_id, gar_system::operations::TipoSlaEvento::Recebido, t0, Some(1), None,
    ).expect("rec");
    gar_system::operations::repository::registrar_evento_sla(
        os_id, gar_system::operations::TipoSlaEvento::DiagnosticoInicio, t0, Some(1), None,
    ).expect("diag_ini");
    gar_system::operations::repository::registrar_evento_sla(
        os_id,
        gar_system::operations::TipoSlaEvento::DiagnosticoFim,
        t0 + chrono::Duration::hours(10),
        Some(1),
        None,
    )
    .expect("diag_fim");

    let calc = ops::calcular_sla_os(os_id).expect("calcular");
    assert!(calc.sla_violado);
    assert!(calc.motivo_violacao.as_deref().unwrap_or("").contains("diagnóstico"));
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn verificar_alertas_cria_ocorrencia_para_os_parada() {
    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    use mysql::prelude::Queryable;
    let mut conn = pool.get_conn().expect("conn");
    conn.exec_drop("INSERT INTO clientes (nome) VALUES ('C')", ())
        .expect("cli");
    let cid = conn.last_insert_id() as u32;
    conn.exec_drop(
        "INSERT INTO ordens_servico (cliente_id, status, descricao) VALUES (?, 'ABERTA', 'OS')",
        (cid,),
    )
    .expect("os");
    let os_id = conn.last_insert_id() as u32;

    // Backdate a última movimentação para 5 dias atrás
    let t = chrono::Utc::now() - chrono::Duration::days(5);
    conn.exec_drop(
        "UPDATE ordens_servico SET ultima_movimentacao_data = ? WHERE id = ?",
        (t.format("%Y-%m-%d %H:%M:%S").to_string(), os_id),
    )
    .expect("update");
    // Forçar uma movimentação para a OS
    let _ = ops::mover_estado(os_id, "DIAGNOSTICO", None, &ctx_padrao());
    conn.exec_drop(
        "UPDATE workflow_movimentacoes SET data_movimentacao = ? WHERE entidade_id = ?",
        (t.format("%Y-%m-%d %H:%M:%S").to_string(), os_id),
    )
    .expect("update2");
    // Status de volta para ABERTA (para ser detectado como "parado")
    conn.exec_drop(
        "UPDATE ordens_servico SET status = 'ABERTA' WHERE id = ?",
        (os_id,),
    )
    .expect("update3");

    let criadas = ops::verificar_alertas(&ctx_padrao()).expect("verificar");
    assert!(criadas >= 1, "deveria ter criado pelo menos 1 alerta");

    let pendentes = ops::listar_alertas_pendentes(50).expect("listar");
    assert!(!pendentes.is_empty());
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn dashboard_executivo_retorna_metricas_basicas() {
    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    use mysql::prelude::Queryable;
    let mut conn = pool.get_conn().expect("conn");
    conn.exec_drop("INSERT INTO clientes (nome) VALUES ('C')", ())
        .expect("cli");
    let cid = conn.last_insert_id() as u32;
    for s in &["ABERTA", "EM_ANDAMENTO", "FINALIZADA", "ENTREGUE"] {
        conn.exec_drop(
            "INSERT INTO ordens_servico (cliente_id, status, descricao) VALUES (?, ?, 'OS')",
            (cid, *s),
        )
        .expect("os");
    }

    let dash = ops::dashboard_executivo().expect("dash");
    assert!(dash.os_abertas >= 1);
    assert!(dash.os_concluidas >= 1);
    assert!(dash.os_total >= 4);
}
