// src/rbac/repository.rs

use super::models::{Permission, Role, UserRole};
use crate::servicos::ErroAplicacao;
use mysql::{params, prelude::Queryable};

use crate::banco_de_dados::conexao::obter_conexao;

pub fn listar_roles() -> Result<Vec<Role>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let rows: Vec<(i32, String, String, String, i32)> = conn.query(
        "SELECT id, codigo, nome, descricao, nivel FROM roles ORDER BY nivel DESC, codigo ASC",
    )?;
    Ok(rows
        .into_iter()
        .map(|(id, codigo, nome, descricao, nivel)| Role {
            id,
            codigo,
            nome,
            descricao,
            nivel,
        })
        .collect())
}

pub fn buscar_role_por_codigo(codigo: &str) -> Result<Option<Role>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<(i32, String, String, String, i32)> = conn.exec_first(
        "SELECT id, codigo, nome, descricao, nivel FROM roles WHERE codigo = :c",
        params! { "c" => codigo },
    )?;
    Ok(row.map(|(id, codigo, nome, descricao, nivel)| Role {
        id,
        codigo,
        nome,
        descricao,
        nivel,
    }))
}

pub fn listar_permissions() -> Result<Vec<Permission>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let rows: Vec<(i32, String, String, String)> = conn.query(
        "SELECT id, codigo, descricao, categoria FROM permissions ORDER BY categoria, codigo",
    )?;
    Ok(rows
        .into_iter()
        .map(|(id, codigo, descricao, categoria)| Permission {
            id,
            codigo,
            descricao,
            categoria,
        })
        .collect())
}

pub fn listar_roles_usuario(user_id: i32, empresa_id: i32) -> Result<Vec<UserRole>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let rows: Vec<(i32, i32, i32, i32)> = conn.exec(
        "SELECT id, user_id, role_id, empresa_id FROM user_roles
         WHERE user_id = :u AND empresa_id = :e",
        params! { "u" => user_id, "e" => empresa_id },
    )?;
    Ok(rows
        .into_iter()
        .map(|(id, user_id, role_id, empresa_id)| UserRole {
            id,
            user_id,
            role_id,
            empresa_id,
        })
        .collect())
}

pub fn listar_usuarios_por_role(role_id: i32, empresa_id: i32) -> Result<Vec<i32>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let rows: Vec<(i32,)> = conn.exec(
        "SELECT user_id FROM user_roles WHERE role_id = :r AND empresa_id = :e",
        params! { "r" => role_id, "e" => empresa_id },
    )?;
    Ok(rows.into_iter().map(|(u,)| u).collect())
}

pub fn permissoes_de_role(role_id: i32) -> Result<Vec<String>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let rows: Vec<(String,)> = conn.exec(
        "SELECT p.codigo FROM permissions p
         JOIN role_permissions rp ON rp.permission_id = p.id
         WHERE rp.role_id = :r",
        params! { "r" => role_id },
    )?;
    Ok(rows.into_iter().map(|(c,)| c).collect())
}

/// Permissões efetivas de um usuário em uma empresa (soma de todas as roles).
///
/// **Esta é a função mais importante do RBAC** — toda checagem de permissão
/// passa por aqui (via cache).
pub fn permissoes_de_usuario(user_id: i32, empresa_id: i32) -> Result<Vec<String>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let rows: Vec<(String,)> = conn.exec(
        "SELECT DISTINCT p.codigo FROM permissions p
         JOIN role_permissions rp ON rp.permission_id = p.id
         JOIN user_roles ur ON ur.role_id = rp.role_id
         WHERE ur.user_id = :u AND ur.empresa_id = :e",
        params! { "u" => user_id, "e" => empresa_id },
    )?;
    Ok(rows.into_iter().map(|(c,)| c).collect())
}

/// Atribui role (INSERT IGNORE para idempotência).
pub fn inserir_user_role(
    user_id: i32,
    role_id: i32,
    empresa_id: i32,
) -> Result<bool, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        "INSERT IGNORE INTO user_roles (user_id, role_id, empresa_id) VALUES (:u, :r, :e)",
        params! { "u" => user_id, "r" => role_id, "e" => empresa_id },
    )?;
    Ok(conn.affected_rows() > 0)
}

pub fn remover_user_role(
    user_id: i32,
    role_id: i32,
    empresa_id: i32,
) -> Result<bool, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        "DELETE FROM user_roles WHERE user_id = :u AND role_id = :r AND empresa_id = :e",
        params! { "u" => user_id, "r" => role_id, "e" => empresa_id },
    )?;
    Ok(conn.affected_rows() > 0)
}
