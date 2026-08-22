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
}

/// Verifica credenciais e retorna o papel.
///
/// **Anti-enumeração:** quando o usuário não existe, ainda assim executa
/// `bcrypt::verify` contra um hash dummy, de modo que o tempo de resposta
/// seja indistinguível do caso "usuário existe, senha errada".
///
/// Retorna `Err(SenhaInvalida)` em ambos os casos — quem chama não deve
/// diferenciar. A função de login do servidor mapeia para 401 com mensagem
/// genérica.
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
                    "Estoquista" => Ok(PapelUsuario::Estoquista),
                    _ => Err(ErroAplicacao::Desconhecido(
                        "Papel de usuário inválido.".into(),
                    )),
                }
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
