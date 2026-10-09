// tests/integration/common.rs
//
// Helpers compartilhados pelos testes de integração com MySQL real.
//
// ATENÇÃO: Estes testes requerem MySQL rodando. Por padrão estão
// desabilitados (via `#[ignore]`). Para executar:
//
//   # Localmente:
//   docker run -d --name mysql-test -p 3306:3306 \
//     -e MYSQL_ROOT_PASSWORD=test -e MYSQL_DATABASE=gar_system_test \
//     mysql:8.0
//   cargo test --test integration -- --include-ignored
//
//   # Via script:
//   ./scripts/run-integration-tests.sh
//
// Os testes NÃO foram executados no sandbox (sem MySQL/Docker). Devem
// ser executados em ambiente de dev ou CI antes de deploy em prod.

#![allow(dead_code)]

use mysql::prelude::Queryable;
use mysql::Pool;
use std::env;
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;

/// URL padrão de conexão ao MySQL de teste.
pub fn test_mysql_url() -> String {
    env::var("TEST_MYSQL_URL")
        .unwrap_or_else(|_| "mysql://root:test@127.0.0.1:3306/gar_system_test".to_string())
}

/// Pool de conexões para o MySQL de teste.
pub async fn setup_pool() -> Result<Pool, mysql::Error> {
    let url = test_mysql_url();
    Pool::new(url.as_str())
}

/// Aplica as migrations usando o binário do servidor.
/// Equivalente a rodar `cargo run --bin gar-system-server` uma vez.
pub fn aplicar_migrations() -> Result<(), String> {
    let status = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "gar-system-server",
            "--",
            "--migrate-only",
        ])
        .env("DATABASE_URL", test_mysql_url())
        .env("JWT_SECRET", "test-secret-for-integration-tests")
        .env("APP_ENV", "test")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| format!("Falha ao rodar migration: {}", e))?;
    if !status.success() {
        return Err(format!("Migration falhou com status: {}", status));
    }
    Ok(())
}

/// Limpa todas as tabelas (ordem importa por FK).
/// Usar entre testes para isolamento.
pub fn truncate_all(pool: &Pool) -> Result<(), mysql::Error> {
    let mut conn = pool.get_conn()?;
    // Ordem: dependentes primeiro
    conn.query_drop("SET FOREIGN_KEY_CHECKS = 0")?;
    let tabelas = [
        "audit_log",
        "ordem_servico_pecas",
        "ordens_servico",
        "movimentacoes",
        "pecas",
        "fornecedores",
        "clientes",
        "usuarios",
        "orcamento_itens",
        "orcamentos",
        "schema_migrations",
    ];
    for t in tabelas {
        let _ = conn.query_drop(format!("TRUNCATE TABLE {}", t));
    }
    conn.query_drop("SET FOREIGN_KEY_CHECKS = 1")?;
    Ok(())
}

/// Cria um usuário admin para testes.
pub fn criar_admin(pool: &Pool, username: &str, senha: &str) -> Result<u32, mysql::Error> {
    use mysql::params;
    let mut conn = pool.get_conn()?;
    let hash = bcrypt::hash(senha, 4).unwrap(); // cost baixo para velocidade
    conn.exec_drop(
        "INSERT INTO usuarios (nome_usuario, senha_hash, papel, ativo) VALUES (:u, :h, 'Administrador', 1)",
        params! { "u" => username, "h" => hash },
    )?;
    Ok(conn.last_insert_id() as u32)
}

/// Spawna o servidor numa porta aleatória e retorna o Child + URL.
pub fn spawn_server(port: u16) -> Result<(Child, String), String> {
    let child = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "gar-system-server",
            "--",
            "--port",
            &port.to_string(),
        ])
        .env("DATABASE_URL", test_mysql_url())
        .env("JWT_SECRET", "test-secret-for-integration-tests")
        .env("APP_ENV", "test")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Falha ao iniciar servidor: {}", e))?;
    Ok((child, format!("http://127.0.0.1:{}", port)))
}

/// Aguarda o servidor responder em /livez.
pub async fn esperar_servidor_pronto(url: &str) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap();
    for _ in 0..30 {
        let url = url.to_string();
        let client = client.clone();
        let res = client.get(format!("{}/livez", url)).send().await;
        if let Ok(r) = res {
            if r.status().is_success() {
                return Ok(());
            }
        }
        sleep(Duration::from_millis(500)).await;
    }
    Err("Servidor não respondeu em /livez em 15s".to_string())
}

/// Faz login e retorna o token JWT.
pub fn login(url: &str, usuario: &str, senha: &str) -> Result<String, String> {
    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(format!("{}/login", url))
        .json(&serde_json::json!({"usuario": usuario, "senha": senha}))
        .send()
        .map_err(|e| format!("Login falhou: {}", e))?;
    if !resp.status().is_success() {
        return Err(format!("Login status: {}", resp.status()));
    }
    let body: serde_json::Value = resp.json().map_err(|e| e.to_string())?;
    body.get("token")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Resposta sem token".to_string())
}
