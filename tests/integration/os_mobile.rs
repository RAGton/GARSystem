// tests/integration/os_mobile.rs
//
// Testes de integração — Sprint P2.3.

#[path = "common.rs"]
mod common;

use gar_system::os_mobile::{self as osm, TipoEvolucao};

fn ctx_padrao() -> osm::Contexto {
    osm::Contexto {
        usuario_id: 1,
        username: Some("teste".into()),
        ip: Some("127.0.0.1".into()),
        user_agent: Some("integration".into()),
        request_id: Some("test".into()),
        cliente_device_id: Some("device-uuid-1".into()),
    }
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn criar_template_listar_e_obter() {
    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    let id = osm::criar_template("Template X", Some("desc"), &ctx_padrao()).expect("criar");
    let _ = osm::adicionar_item_template(id, "Item 1", true).expect("item1");
    let _ = osm::adicionar_item_template(id, "Item 2", false).expect("item2");

    let lista = osm::listar_templates(false).expect("listar");
    assert!(lista.iter().any(|t| t.id == id));

    let completo = osm::obter_template(id).expect("obter").unwrap();
    assert_eq!(completo.itens.len(), 2);
    assert_eq!(completo.template.nome, "Template X");
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn criar_checklist_a_partir_de_template_popula_itens() {
    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    // Usa o template seed "Manutenção Padrão" (id=1)
    let template = osm::obter_template(1).expect("obter template 1");
    if template.is_none() {
        // Cria manualmente se seed não rodou
        let _ = osm::criar_template("Manutenção Padrão", Some("seed"), &ctx_padrao());
    }
    let t_id = template.as_ref().map(|t| t.template.id).unwrap_or(1);
    if template.as_ref().map(|t| t.itens.is_empty()).unwrap_or(true) {
        osm::adicionar_item_template(t_id, "item a", true).expect("ia");
        osm::adicionar_item_template(t_id, "item b", false).expect("ib");
    }

    // Cria OS
    use mysql::prelude::Queryable;
    let mut conn = pool.get_conn().expect("conn");
    conn.exec_drop("INSERT INTO clientes (nome) VALUES ('C')", ()).expect("cli");
    let cid = conn.last_insert_id() as u32;
    conn.exec_drop(
        "INSERT INTO ordens_servico (cliente_id, status, descricao) VALUES (?, 'ABERTA', 'OS teste')",
        (cid,),
    )
    .expect("os");
    let os_id = conn.last_insert_id() as u32;

    let checklist_id = osm::criar_checklist(os_id, Some(t_id), "Checklist OS", &ctx_padrao())
        .expect("criar checklist");
    let c = osm::obter_checklist(checklist_id).expect("obter").unwrap();
    assert!(c.itens.len() >= 2);
    assert_eq!(c.checklist.os_id, os_id);
    assert_eq!(c.checklist.total_itens, c.itens.len() as u32);
    assert_eq!(c.checklist.concluidos, 0);
    assert!(!c.checklist.is_completo());
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn marcar_item_completa_checklist() {
    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    use mysql::prelude::Queryable;
    let mut conn = pool.get_conn().expect("conn");
    conn.exec_drop("INSERT INTO clientes (nome) VALUES ('C')", ()).expect("cli");
    let cid = conn.last_insert_id() as u32;
    conn.exec_drop(
        "INSERT INTO ordens_servico (cliente_id, status, descricao) VALUES (?, 'ABERTA', 'OS teste')",
        (cid,),
    )
    .expect("os");
    let os_id = conn.last_insert_id() as u32;

    let cid_chk = osm::criar_checklist(os_id, None, "Ad-hoc", &ctx_padrao()).expect("criar");
    let i1 = osm::adicionar_item_checklist(cid_chk, "Tarefa A", true).expect("a");
    let i2 = osm::adicionar_item_checklist(cid_chk, "Tarefa B", false).expect("b");

    osm::marcar_item(i1, true, None, &ctx_padrao(), None).expect("marcar a");
    let c = osm::obter_checklist(cid_chk).expect("obter").unwrap();
    assert_eq!(c.checklist.concluidos, 1);
    assert!(!c.checklist.is_completo());

    osm::marcar_item(i2, true, Some("ok"), &ctx_padrao(), None).expect("marcar b");
    let c = osm::obter_checklist(cid_chk).expect("obter").unwrap();
    assert_eq!(c.checklist.concluidos, 2);
    assert!(c.checklist.is_completo());
    assert!(c.checklist.data_conclusao.is_some());
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn registrar_evolucao_persiste_e_lista() {
    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    use mysql::prelude::Queryable;
    let mut conn = pool.get_conn().expect("conn");
    conn.exec_drop("INSERT INTO clientes (nome) VALUES ('C')", ()).expect("cli");
    let cid = conn.last_insert_id() as u32;
    conn.exec_drop(
        "INSERT INTO ordens_servico (cliente_id, status, descricao) VALUES (?, 'ABERTA', 'OS teste')",
        (cid,),
    )
    .expect("os");
    let os_id = conn.last_insert_id() as u32;

    let id = osm::registrar_evolucao(
        os_id,
        "Encontrei capacitor estourado",
        TipoEvolucao::Problema,
        Some(-23.5505),
        Some(-46.6333),
        &ctx_padrao(),
        None,
    )
    .expect("registrar");
    assert!(id > 0);

    let lista = osm::listar_evolucoes(os_id).expect("listar");
    assert_eq!(lista.len(), 1);
    assert_eq!(lista[0].tipo, TipoEvolucao::Problema);
    assert!(lista[0].latitude.is_some());
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn auditoria_offline_dedupe_por_device_evento() {
    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    use mysql::prelude::Queryable;
    let mut conn = pool.get_conn().expect("conn");
    conn.exec_drop("INSERT INTO clientes (nome) VALUES ('C')", ()).expect("cli");
    let cid = conn.last_insert_id() as u32;
    conn.exec_drop(
        "INSERT INTO ordens_servico (cliente_id, status, descricao) VALUES (?, 'ABERTA', 'OS teste')",
        (cid,),
    )
    .expect("os");
    let os_id = conn.last_insert_id() as u32;

    let data = chrono::Utc::now();
    // Primeira inserção
    let id1 = gar_system::os_mobile::repository::registrar_auditoria_campo(
        Some("device-123"),
        1,
        gar_system::os_mobile::TipoAcaoAuditoria::StatusAlterado,
        Some(os_id),
        None, None, None, None, None,
        data,
        Some(&serde_json::json!({"novo_status":"EM_ANDAMENTO"})),
    )
    .expect("insert 1");
    // Segunda inserção com mesma chave = deve ser ignorada (INSERT IGNORE)
    let id2 = gar_system::os_mobile::repository::registrar_auditoria_campo(
        Some("device-123"),
        1,
        gar_system::os_mobile::TipoAcaoAuditoria::StatusAlterado,
        Some(os_id),
        None, None, None, None, None,
        data,
        Some(&serde_json::json!({"novo_status":"EM_ANDAMENTO"})),
    )
    .expect("insert 2");

    // Deve haver 1 registro (id1 e id2 retornam o mesmo id se dedup funcionou,
    // ou id2 = 0 se for IGNORE)
    let lista = osm::listar_auditoria_os(os_id).expect("listar");
    assert_eq!(lista.len(), 1, "auditoria deve deduplicar");
    let _ = (id1, id2);
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn dashboard_tecnico_retorna_contadores() {
    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    use mysql::prelude::Queryable;
    let mut conn = pool.get_conn().expect("conn");
    conn.exec_drop("INSERT INTO clientes (nome) VALUES ('C')", ()).expect("cli");
    let cid = conn.last_insert_id() as u32;
    // 3 OS em estados diferentes
    for s in &["ABERTA", "EM_ANDAMENTO", "CONCLUIDA"] {
        conn.exec_drop(
            "INSERT INTO ordens_servico (cliente_id, status, descricao) VALUES (?, ?, 'OS')",
            (cid, *s),
        )
        .expect("os");
    }

    let dash = osm::dashboard_tecnico(None).expect("dashboard");
    assert!(dash.os_abertas >= 1);
    assert!(dash.os_em_andamento >= 1);
    assert!(dash.os_concluidas >= 1);
    // OS sem checklist
    assert!(dash.os_sem_checklist >= 3);
}
