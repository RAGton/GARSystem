// src/banco_de_dados/init.rs

use super::conexao::obter_conexao; // Usa a função do módulo irmão `conexao.rs`
use mysql::{params, prelude::Queryable, PooledConn, TxOpts};
use std::time::SystemTime;

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
    let senha = "admin"; // Senha padrão para o usuário admin.
    let hash_senha = bcrypt::hash(senha, bcrypt::DEFAULT_COST).expect("Falha no hash.");

    let admin_existe: Option<i32> =
        conn.query_first("SELECT id FROM users WHERE username = 'admin'")?;

    if admin_existe.is_some() {
        conn.exec_drop(
            "UPDATE users SET password_hash = :password_hash, role = 'Administrador' WHERE username = 'admin'",
            params! { "password_hash" => &hash_senha },
        )?;
        println!("Senha e papel do usuário 'admin' foram sincronizados.");
    } else {
        conn.exec_drop(
            "INSERT INTO users (username, password_hash, role) VALUES ('admin', :password_hash, 'Administrador')",
            params! { "password_hash" => &hash_senha },
        )?;
        println!("Usuário 'admin' criado com a senha padrão.");
    }
    Ok(())
}
