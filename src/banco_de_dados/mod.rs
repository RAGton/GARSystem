// src/banco_de_dados/mod.rs

use crate::servicos::{ErroAplicacao, InfoUsuario, PapelUsuario};
use dotenvy::dotenv;
use mysql::{params, prelude::Queryable, OptsBuilder, Pool, PooledConn, TxOpts};
use once_cell::sync::Lazy;
use std::env;
use std::sync::Mutex;
use std::time::SystemTime;

// CORRIGIDO: Adicionado #[allow(dead_code)] para silenciar o aviso sobre campos não lidos.
#[derive(Debug)]
#[allow(dead_code)]
struct Usuario {
    id: i32,
    nome_usuario: String,
    hash_senha: String,
    papel: String,
}

static POOL_DB: Lazy<Mutex<Pool>> = Lazy::new(|| {
    dotenv().ok();
    let url_db =
        env::var("DATABASE_URL").expect("A variável de ambiente DATABASE_URL não foi definida.");
    let opts = OptsBuilder::from_opts(mysql::Opts::from_url(&url_db).expect("URL do DB inválida"));
    let pool = Pool::new(opts).expect("Não foi possível criar o pool de conexões.");
    Mutex::new(pool)
});

// ... o resto do arquivo permanece exatamente o mesmo ...
fn obter_conexao() -> Result<PooledConn, ErroAplicacao> {
    POOL_DB
        .lock()
        .unwrap()
        .get_conn()
        .map_err(|_| ErroAplicacao::BancoDeDadosConexao)
}

pub fn inicializar() {
    println!("Inicializando banco de dados...");
    let mut conn =
        obter_conexao().expect("Falha fatal ao obter conexão com o DB na inicialização.");

    let tabela_existe: bool = conn
        .query_first::<(String,), _>("SHOW TABLES LIKE 'users'")
        .unwrap_or(None)
        .is_some();

    if !tabela_existe {
        println!("Tabela 'users' não encontrada. Criando nova tabela...");
        criar_tabela_e_admin_padrao(&mut conn).expect("Falha ao criar a tabela inicial.");
        println!("Tabela 'users' criada com sucesso.");
    } else {
        println!("Tabela 'users' encontrada. Verificando necessidade de migração...");
        if verificar_e_migrar_se_necessario(&mut conn)
            .expect("Falha durante o processo de migração.")
        {
            println!("Migração do esquema da tabela 'users' concluída com sucesso.");
        } else {
            println!("Esquema da tabela 'users' já está atualizado.");
        }
    }

    garantir_admin(&mut conn).expect("Falha ao garantir a existência do usuário 'admin'.");
    println!("Verificação do usuário 'admin' concluída.");
}

fn verificar_e_migrar_se_necessario(conn: &mut PooledConn) -> Result<bool, mysql::Error> {
    let definicao_coluna_role: Option<String> = conn.exec_first(
        "SELECT COLUMN_TYPE FROM INFORMATION_SCHEMA.COLUMNS 
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'users' AND COLUMN_NAME = 'role'",
        (),
    )?;

    if let Some(def) = definicao_coluna_role {
        if def.contains("'ADM'") || def.contains("'Vendedor'") || def.contains("'Gerente'") {
            println!("Schema antigo detectado. Iniciando migração...");
            let mut tx = conn.start_transaction(TxOpts::default())?;
            let ts = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let nome_tabela_backup = format!("users_backup_{}", ts);
            tx.query_drop(format!(
                "CREATE TABLE `{}` LIKE `users`",
                nome_tabela_backup
            ))?;
            tx.query_drop(format!(
                "INSERT INTO `{}` SELECT * FROM `users`",
                nome_tabela_backup
            ))?;
            println!("Backup criado: '{}'.", nome_tabela_backup);
            tx.exec_drop(
                "UPDATE users SET role = 'Administrador' WHERE role = 'ADM'",
                (),
            )?;
            tx.exec_drop(
                "UPDATE users SET role = 'Comercial' WHERE role IN ('Vendedor', 'Atendente')",
                (),
            )?;
            tx.exec_drop(
                "UPDATE users SET role = 'Gerencia' WHERE role = 'Gerente'",
                (),
            )?;
            tx.exec_drop(
                "ALTER TABLE users MODIFY COLUMN role ENUM('Administrador', 'Gerencia', 'Tecnico', 'Financeiro', 'Comercial') NOT NULL DEFAULT 'Comercial'",
                (),
            )?;
            tx.commit()?;
            return Ok(true);
        }
    }
    Ok(false)
}

fn criar_tabela_e_admin_padrao(conn: &mut PooledConn) -> Result<(), mysql::Error> {
    conn.query_drop(
        r"CREATE TABLE users (
            id INT AUTO_INCREMENT PRIMARY KEY,
            username VARCHAR(255) NOT NULL UNIQUE,
            password_hash VARCHAR(255) NOT NULL,
            role ENUM('Administrador', 'Gerencia', 'Tecnico', 'Financeiro', 'Comercial') NOT NULL DEFAULT 'Comercial'
        )",
    )?;
    Ok(())
}

fn garantir_admin(conn: &mut PooledConn) -> Result<(), mysql::Error> {
    let senha = "12345678";
    let hash_senha = bcrypt::hash(senha, bcrypt::DEFAULT_COST).expect("Falha no hash.");

    let admin_existe: Option<i32> =
        conn.query_first("SELECT id FROM users WHERE username = 'admin'")?;

    if let Some(_) = admin_existe {
        // Admin existe, atualiza a senha e o papel para garantir que estejam corretos.
        conn.exec_drop(
            "UPDATE users SET password_hash = :password_hash, role = 'Administrador' WHERE username = 'admin'",
            params! { "password_hash" => &hash_senha },
        )?;
        println!("Senha e papel do usuário 'admin' foram sincronizados.");
    } else {
        // Admin não existe, cria um novo.
        conn.exec_drop(
            "INSERT INTO users (username, password_hash, role) VALUES ('admin', :password_hash, 'Administrador')",
            params! { "password_hash" => &hash_senha },
        )?;
        println!("Usuário 'admin' criado com a senha padrão.");
    }
    Ok(())
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
                    "Administrador" => Ok(PapelUsuario::Administrador),
                    "Gerencia" => Ok(PapelUsuario::Gerencia),
                    "Tecnico" => Ok(PapelUsuario::Tecnico),
                    "Financeiro" => Ok(PapelUsuario::Financeiro),
                    "Comercial" => Ok(PapelUsuario::Comercial),
                    _ => Err(ErroAplicacao::Desconhecido(
                        "Papel de usuário inválido.".into(),
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
        .map_err(|e| ErroAplicacao::FalhaNoHash(e.to_string()))?;

    let papel_str = match papel {
        PapelUsuario::Administrador => "Administrador",
        PapelUsuario::Gerencia => "Gerencia",
        PapelUsuario::Tecnico => "Tecnico",
        PapelUsuario::Financeiro => "Financeiro",
        PapelUsuario::Comercial => "Comercial",
    };

    conn.exec_drop(
        "INSERT INTO users (username, password_hash, role) VALUES (:username, :password_hash, :role)",
        params! { "username" => nome_usuario, "password_hash" => hash_senha, "role" => papel_str },
    )?;
    Ok(())
}

pub fn listar_todos_usuarios() -> Vec<InfoUsuario> {
    obter_conexao().map_or_else(
        |_| vec![],
        |mut conn| {
            conn.query_map(
                "SELECT id, username FROM users ORDER BY username ASC",
                |(id, nome_usuario)| InfoUsuario { id, nome_usuario },
            )
            .unwrap_or_else(|e| {
                eprintln!("Erro ao listar usuários: {}", e);
                vec![]
            })
        },
    )
}

pub fn remover_usuario(nome_usuario: &str) -> Result<(), ErroAplicacao> {
    if nome_usuario == "admin" {
        return Err(ErroAplicacao::NaoPodeRemoverAdmin);
    }
    let mut conn = obter_conexao()?;
    let resultado = conn.exec_iter(
        "DELETE FROM users WHERE username = :username",
        params! { "username" => nome_usuario },
    )?;
    if resultado.affected_rows() > 0 {
        Ok(())
    } else {
        Err(ErroAplicacao::UsuarioNaoEncontrado)
    }
}
