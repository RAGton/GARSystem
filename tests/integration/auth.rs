// tests/integration/auth.rs
//
// Testes de integração de autenticação.
//
// Requer MySQL real (ver `tests/integration/common.rs`).
//
// #[ignore] por padrão — execute com:
//   cargo test --test integration -- --include-ignored

#[path = "common.rs"]
mod common;

#[tokio::test]
#[ignore = "requer MySQL real; ver tests/integration/common.rs"]
async fn login_sucesso_retorna_token() {
    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");
    common::criar_admin(&pool, "admin", "senha12345").expect("criar admin");

    let (mut server, url) = common::spawn_server(33001).expect("spawn");
    common::esperar_servidor_pronto(&url).await.expect("pronto");

    let token = common::login(&url, "admin", "senha12345").expect("login");
    assert!(!token.is_empty(), "token não pode ser vazio");

    server.kill().expect("kill");
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn login_senha_errada_401() {
    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");
    common::criar_admin(&pool, "admin", "senha12345").expect("criar admin");

    let (mut server, url) = common::spawn_server(33002).expect("spawn");
    common::esperar_servidor_pronto(&url).await.expect("pronto");

    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(format!("{}/login", url))
        .json(&serde_json::json!({"usuario": "admin", "senha": "errada"}))
        .send()
        .expect("send");
    assert_eq!(resp.status().as_u16(), 401, "deveria ser 401");

    server.kill().expect("kill");
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn login_usuario_inexistente_401() {
    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    let (mut server, url) = common::spawn_server(33003).expect("spawn");
    common::esperar_servidor_pronto(&url).await.expect("pronto");

    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(format!("{}/login", url))
        .json(&serde_json::json!({"usuario": "fantasma", "senha": "qualquer"}))
        .send()
        .expect("send");
    assert_eq!(resp.status().as_u16(), 401);

    // Anti-enumeração: o tempo de resposta deve ser similar ao caso
    // "senha errada". Esse teste valida só o status code — para timing,
    // seria necessário medir e comparar, fora do escopo aqui.

    server.kill().expect("kill");
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn acesso_sem_token_401() {
    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");

    let (mut server, url) = common::spawn_server(33004).expect("spawn");
    common::esperar_servidor_pronto(&url).await.expect("pronto");

    let client = reqwest::blocking::Client::new();
    let resp = client
        .get(format!("{}/usuarios", url))
        .send()
        .expect("send");
    assert_eq!(resp.status().as_u16(), 401, "sem token = 401");

    server.kill().expect("kill");
}
