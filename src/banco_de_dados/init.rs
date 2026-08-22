// src/banco_de_dados/init.rs
//
// Inicialização do schema da tabela `users`.
//
// IMPORTANTE — POLÍTICA DE ADMIN:
//
// A partir da v1.9.0, este módulo NÃO cria mais um usuário admin
// automaticamente. Para criar o primeiro administrador, use o binário CLI:
//
//   senior-system-admin create-admin --username admin --password-env ADMIN_PASSWORD
//
// Este módulo mantém a migration da coluna `role` para compatibilidade
// com bancos legados (que tinham roles antigos como 'ADM', 'Vendedor',
// 'Gerente').

use super::conexao::obter_conexao;
use mysql::prelude::Queryable;
use mysql::PooledConn;

pub fn inicializar() {
    println!("Inicializando banco de dados (schema de usuários)...");
    let mut conn =
        obter_conexao().expect("Falha fatal ao obter conexão com o DB na inicialização.");

    let tabela_existe: bool = conn
        .query_first::<(String,), _>("SHOW TABLES LIKE 'users'")
        .unwrap_or(None)
        .is_some();

    if !tabela_existe {
        println!("Tabela 'users' não encontrada. Criando nova tabela...");
        criar_tabela_users(&mut conn).expect("Falha ao criar a tabela inicial.");
        println!("Tabela 'users' criada com sucesso.");
    } else {
        println!("Tabela 'users' encontrada. Verificando necessidade de migração...");
        if migrar_role_se_necessario(&mut conn).expect("Falha durante o processo de migração.") {
            println!("Migração do esquema da tabela 'users' concluída com sucesso.");
        } else {
            println!("Esquema da tabela 'users' já está atualizado.");
        }
    }

    // SEM criação automática de admin.
    // Use o CLI `senior-system-admin create-admin` para criar o primeiro.
    println!(
        "ℹ️  Nenhum usuário admin criado automaticamente. \
         Use `senior-system-admin create-admin` se o banco estiver vazio."
    );
}

fn criar_tabela_users(conn: &mut PooledConn) -> Result<(), mysql::Error> {
    conn.query_drop(
        r"CREATE TABLE IF NOT EXISTS users (
            id            INT AUTO_INCREMENT PRIMARY KEY,
            username      VARCHAR(64)  NOT NULL,
            password_hash VARCHAR(255) NOT NULL,
            role          ENUM('Administrador', 'Gerencia', 'Tecnico', 'Financeiro', 'Comercial', 'Estoquista')
                          NOT NULL DEFAULT 'Comercial',
            tenant_id     INT          NOT NULL DEFAULT 0,
            created_at    TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at    TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
            last_login_at TIMESTAMP    NULL,
            UNIQUE KEY uk_users_username (username)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci",
    )?;
    Ok(())
}

fn migrar_role_se_necessario(conn: &mut PooledConn) -> Result<bool, mysql::Error> {
    let definicao_coluna_role: Option<String> = conn.exec_first(
        "SELECT COLUMN_TYPE FROM INFORMATION_SCHEMA.COLUMNS
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'users' AND COLUMN_NAME = 'role'",
        (),
    )?;
    if let Some(def) = definicao_coluna_role {
        if def.contains("'ADM'") || def.contains("'Vendedor'") || def.contains("'Gerente'") {
            println!("Schema antigo detectado. Iniciando migração...");
            let mut tx = conn.start_transaction(mysql::TxOpts::default())?;
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let nome_tabela_backup = format!("users_backup_{}", ts);
            tx.query_drop(format!("CREATE TABLE `{}` LIKE `users`", nome_tabela_backup))?;
            tx.query_drop(format!(
                "INSERT INTO `{}` SELECT * FROM `users`",
                nome_tabela_backup
            ))?;
            println!("Backup criado: '{}'.", nome_tabela_backup);
            tx.exec_drop("UPDATE users SET role = 'Administrador' WHERE role = 'ADM'", ())?;
            tx.exec_drop(
                "UPDATE users SET role = 'Comercial' WHERE role IN ('Vendedor', 'Atendente')",
                (),
            )?;
            tx.exec_drop("UPDATE users SET role = 'Gerencia' WHERE role = 'Gerente'", ())?;
            tx.exec_drop(
                "ALTER TABLE users MODIFY COLUMN role ENUM('Administrador', 'Gerencia', 'Tecnico', 'Financeiro', 'Comercial', 'Estoquista') NOT NULL DEFAULT 'Comercial'",
                (),
            )?;
            tx.commit()?;
            return Ok(true);
        }
    }
    Ok(false)
}
