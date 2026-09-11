// src/banco_de_dados/usuario.rs

use super::conexao::obter_conexao; // Usa a função do módulo irmão `conexao.rs`
use crate::servicos::{ErroAplicacao, InfoUsuario, PapelUsuario};
use mysql::{params, prelude::Queryable, PooledConn};

/// Hash bcrypt dummy, computado UMA VEZ em `lazy_static`. Usado para igualar
/// o tempo de resposta de `verificar_login` quando o usuário não existe
/// (anti-enumeração via timing).
///
/// IMPORTANTE: este hash é de uma senha fictícia, não corresponde a nenhum
/// usuário real. Serve apenas para que `bcrypt::verify` rode com custo
/// similar ao de uma verificação real.
fn dummy_hash() -> &'static str {
    use once_cell::sync::Lazy;
    static HASH: Lazy<String> = Lazy::new(|| {
        // senha dummy: "x" - nunca corresponde a nada.
        bcrypt::hash("x", bcrypt::DEFAULT_COST)
            .expect("Falha ao gerar hash dummy para anti-enumeration")
    });
    &HASH
}

// A struct que mapeia a tabela `users`. É privada para este módulo.
#[derive(Debug)]
struct Usuario {
    #[allow(dead_code)]
    id: i32,
    #[allow(dead_code)]
    nome_usuario: String,
    hash_senha: String,
    papel: String,
    /// P2.6.2a: tenant_id do usuário (necessário para criar token com tenant real)
    #[allow(dead_code)]
    tenant_id: i32,
}

/// Resultado do login (P2.6.2a — propagação de dados para o token JWT)
#[derive(Debug, Clone)]
pub struct LoginResult {
    pub uid: i32,
    pub papel: PapelUsuario,
    pub tenant_id: i32,
}

/// Verifica credenciais e retorna o papel.
///
/// **Anti-enumeração:** quando o usuário não existe, ainda assim executa
/// `bcrypt::verify` contra um hash dummy, de modo que o tempo de resposta
/// seja indistinguível do caso "usuário existe, senha errada".
///
/// Retorna `Err(SenhaInvalida)` em ambos os casos — quem chama não deve
/// Diferenciar. A função de login do servidor mapeia para 401 com mensagem
/// genérica.
///
/// **P2.6.2a fix (Achado #1)**: agora retorna `LoginResult` com
/// `(uid, papel, tenant_id)` para que o handler possa criar tokens
/// com `roles` e `permissions` reais do banco.
pub fn verificar_senha_e_obter_papel(
    nome_usuario: &str,
    senha: &str,
) -> Result<LoginResult, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    match encontrar_usuario_por_nome(&mut conn, nome_usuario) {
        Some(usuario) => {
            if bcrypt::verify(senha, &usuario.hash_senha).unwrap_or(false) {
                let papel = match usuario.papel.as_str() {
                    "Administrador" => PapelUsuario::Administrador,
                    "Gerencia" => PapelUsuario::Gerencia,
                    "Tecnico" => PapelUsuario::Tecnico,
                    "Financeiro" => PapelUsuario::Financeiro,
                    "Comercial" => PapelUsuario::Comercial,
                    "Estoquista" => PapelUsuario::Estoquista,
                    _ => {
                        return Err(ErroAplicacao::Desconhecido(
                            "Papel de usuário inválido.".into(),
                        ))
                    }
                };
                Ok(LoginResult {
                    uid: usuario.id,
                    papel,
                    tenant_id: usuario.tenant_id,
                })
            } else {
                Err(ErroAplicacao::SenhaInvalida)
            }
        }
        None => {
            // Anti-enumeração: roda bcrypt::verify contra hash dummy para
            // igualar o tempo de resposta.
            let _ = bcrypt::verify(senha, dummy_hash());
            Err(ErroAplicacao::SenhaInvalida)
        }
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
        PapelUsuario::Estoquista => "Estoquista",
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

/// Altera o papel (role) de um usuário existente.
///
/// Retorna `UsuarioNaoEncontrado` se não existir, ou erro de banco
/// caso falhe. Não permite rebaixar o último administrador para
/// evitar lockout (`NaoPodeRemoverAdmin`).
pub fn alterar_papel(
    nome_usuario: &str,
    novo_papel: PapelUsuario,
) -> Result<(), ErroAplicacao> {
    if nome_usuario == "admin" {
        return Err(ErroAplicacao::NaoPodeRemoverAdmin);
    }

    let papel_str = match novo_papel {
        PapelUsuario::Administrador => "Administrador",
        PapelUsuario::Gerencia => "Gerencia",
        PapelUsuario::Tecnico => "Tecnico",
        PapelUsuario::Financeiro => "Financeiro",
        PapelUsuario::Comercial => "Comercial",
        PapelUsuario::Estoquista => "Estoquista",
    };

    let mut conn = obter_conexao()?;

    // Primeiro: garante que o usuário existe. UPDATE cego retorna
    // `affected_rows = 0` sem distinguir "não existe" vs "sem mudança".
    let existe: Option<i32> = conn
        .exec_first(
            "SELECT id FROM users WHERE username = :username",
            params! { "username" => nome_usuario },
        )?;
    if existe.is_none() {
        return Err(ErroAplicacao::UsuarioNaoEncontrado);
    }

    // Regra: nunca deixar o sistema sem Administrador. Se o user alvo
    // é o último admin, e estamos rebaixando para papel != admin,
    // bloqueia.
    if novo_papel != PapelUsuario::Administrador {
        let papel_atual: Option<String> = conn.exec_first(
            "SELECT role FROM users WHERE username = :username",
            params! { "username" => nome_usuario },
        )?;
        if papel_atual.as_deref() == Some("Administrador") {
            let total_admins: i64 = conn
                .query_first("SELECT COUNT(*) FROM users WHERE role = 'Administrador'")
                .map_err(ErroAplicacao::from)?
                .unwrap_or(0);
            if total_admins <= 1 {
                return Err(ErroAplicacao::NaoPodeRemoverAdmin);
            }
        }
    }

    conn.exec_drop(
        "UPDATE users SET role = :role WHERE username = :username",
        params! { "username" => nome_usuario, "role" => papel_str },
    )?;
    Ok(())
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

// Função auxiliar, privada para este módulo.
fn encontrar_usuario_por_nome(conn: &mut PooledConn, nome_usuario: &str) -> Option<Usuario> {
    conn.exec_first(
        "SELECT id, username, password_hash, role, COALESCE(tenant_id, 1) FROM users WHERE username = :username",
        params! { "username" => nome_usuario },
    )
    .ok()
    .flatten()
    .map(|(id, nome_usuario, hash_senha, papel, tenant_id)| Usuario {
        id,
        nome_usuario,
        hash_senha,
        papel,
        tenant_id,
    })
}
