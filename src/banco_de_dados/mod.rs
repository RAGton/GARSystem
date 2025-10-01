// src/banco_de_dados/mod.rs

use crate::servicos::{ErroAplicacao, InfoUsuario, PapelUsuario};
use mysql::{params, prelude::Queryable, OptsBuilder, Pool, PooledConn};
use once_cell::sync::Lazy;
use std::sync::Mutex;

#[derive(Debug)]
struct Usuario {
    id: i32,
    nome_usuario: String,
    hash_senha: String,
    papel: String,
}

static POOL_DB: Lazy<Mutex<Pool>> = Lazy::new(|| {
    let url_db = "mysql://rocha:200519@localhost:3306/senior_system";
    let opts = OptsBuilder::from_opts(mysql::Opts::from_url(url_db).expect("URL do DB inválida"));
    let pool = Pool::new(opts).expect("Não foi possível criar o pool de conexões.");
    Mutex::new(pool)
});

fn obter_conexao() -> Result<PooledConn, ErroAplicacao> {
    POOL_DB
        .lock()
        .unwrap()
        .get_conn()
        .map_err(|_| ErroAplicacao::BancoDeDadosConexao)
}

pub fn inicializar() {
    let mut conn = obter_conexao().expect("Falha ao obter conexão para inicializar o DB");
    conn.query_drop(r"CREATE TABLE IF NOT EXISTS users (id INT AUTO_INCREMENT PRIMARY KEY, username VARCHAR(255) NOT NULL UNIQUE, password_hash VARCHAR(255) NOT NULL, role ENUM('ADM', 'Tecnico', 'Financeiro', 'Vendedor', 'Gerente', 'Atendente') NOT NULL DEFAULT 'Vendedor')").expect("Falha ao criar/verificar tabela 'users'");
    let user_exists: Option<String> = conn
        .query_first("SELECT role FROM users WHERE username = 'admin'")
        .unwrap_or(None);
    if user_exists.is_none() {
        let senha = "1234";
        let hash_senha = bcrypt::hash(senha, bcrypt::DEFAULT_COST).expect("Falha ao hashear senha");
        conn.exec_drop("INSERT INTO users (username, password_hash, role) VALUES (:username, :password_hash, :role)", params! { "username" => "admin", "password_hash" => hash_senha, "role" => "ADM" }).expect("Falha ao inserir admin");
    } else if user_exists != Some("ADM".to_string()) {
        conn.exec_drop("UPDATE users SET role = 'ADM' WHERE username = 'admin'", ())
            .expect("Falha ao atualizar papel do admin");
    }
}

fn encontrar_usuario_por_nome(conn: &mut PooledConn, nome_usuario: &str) -> Option<Usuario> {
    conn.exec_first(
        "SELECT id, username, password_hash, role FROM users WHERE username = :username",
        params! { "username" => nome_usuario },
    )
    .ok()
    .flatten()
    .map(|(id, nome_usuario, hash_senha, papel)| Usuario {
        id,
        nome_usuario,
        hash_senha,
        papel,
    })
}

pub fn verificar_senha_e_obter_papel(
    nome_usuario: &str,
    senha: &str,
) -> Result<PapelUsuario, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    match encontrar_usuario_por_nome(&mut conn, nome_usuario) {
        Some(usuario) => {
            if bcrypt::verify(senha, &usuario.hash_senha).unwrap_or(false) {
                match usuario.papel.as_str() {
                    "ADM" => Ok(PapelUsuario::ADM),
                    "Tecnico" => Ok(PapelUsuario::Tecnico),
                    "Financeiro" => Ok(PapelUsuario::Financeiro),
                    "Vendedor" => Ok(PapelUsuario::Vendedor),
                    "Gerente" => Ok(PapelUsuario::Gerente),
                    "Atendente" => Ok(PapelUsuario::Atendente),
                    _ => Err(ErroAplicacao::Desconhecido(
                        "Papel de usuário inválido no DB.".to_string(),
                    )),
                }
            } else {
                Err(ErroAplicacao::SenhaInvalida)
            }
        }
        None => Err(ErroAplicacao::UsuarioNaoEncontrado),
    }
}

pub fn criar_usuario(
    nome_usuario: &str,
    senha: &str,
    papel: PapelUsuario,
) -> Result<(), ErroAplicacao> {
    let mut conn = obter_conexao()?;
    if encontrar_usuario_por_nome(&mut conn, nome_usuario).is_some() {
        return Err(ErroAplicacao::UsuarioJaExiste);
    }
    let hash_senha = bcrypt::hash(senha, bcrypt::DEFAULT_COST)
        .map_err(|e| ErroAplicacao::Desconhecido(format!("Falha no hash: {}", e)))?;
    let papel_str = format!("{:?}", papel);
    conn.exec_drop("INSERT INTO users (username, password_hash, role) VALUES (:username, :password_hash, :role)", params! { "username" => nome_usuario, "password_hash" => hash_senha, "role" => papel_str }).map_err(|e| ErroAplicacao::BancoDeDadosQuery(e.to_string()))?;
    Ok(())
}

pub fn listar_todos_usuarios() -> Vec<InfoUsuario> {
    if let Ok(mut conn) = obter_conexao() {
        return conn
            .query_map(
                "SELECT id, username FROM users ORDER BY username ASC",
                |(id, nome_usuario)| InfoUsuario { id, nome_usuario },
            )
            .unwrap_or_else(|e| {
                eprintln!("Erro ao listar usuários: {}", e);
                vec![]
            });
    }
    vec![]
}

pub fn remover_usuario(nome_usuario: &str) -> Result<(), ErroAplicacao> {
    if nome_usuario == "admin" {
        return Err(ErroAplicacao::NaoPodeRemoverAdmin);
    }
    let mut conn = obter_conexao()?;
    let resultado = conn
        .exec_iter(
            "DELETE FROM users WHERE username = :username",
            params! { "username" => nome_usuario },
        )
        .map_err(|e| ErroAplicacao::BancoDeDadosQuery(e.to_string()))?;
    if resultado.affected_rows() > 0 {
        Ok(())
    } else {
        Err(ErroAplicacao::UsuarioNaoEncontrado)
    }
}
