// src/auth.rs
//! Módulo de autenticação JWT para o servidor Senior System
//!
//! Este módulo gerencia a criação e validação de tokens JWT,
//! garantindo que apenas usuários autenticados possam acessar
//! recursos protegidos da API.

use crate::servicos::PapelUsuario;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

/// Claims (reivindicações) contidas no token JWT
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject - identificador do usuário (username)
    pub sub: String,
    /// Papel/Role do usuário no sistema
    pub papel: PapelUsuario,
    /// Expiration time - timestamp Unix de quando o token expira
    pub exp: usize,
    /// Issued at - timestamp Unix de quando o token foi criado
    pub iat: usize,
}

/// Cria um novo token JWT para um usuário autenticado
///
/// # Argumentos
/// * `usuario` - Nome de usuário
/// * `papel` - Papel/role do usuário no sistema
///
/// # Retorna
/// * `Ok(String)` - Token JWT codificado como string
/// * `Err(jsonwebtoken::errors::Error)` - Erro ao criar o token
///
/// # Exemplo
/// ```ignore
/// let token = criar_token("admin", PapelUsuario::Administrador)?;
/// ```
pub fn criar_token(
    usuario: &str,
    papel: PapelUsuario,
) -> Result<String, jsonwebtoken::errors::Error> {
    let agora = Utc::now();
    let expiracao = agora
        .checked_add_signed(Duration::hours(24)) // Token válido por 24 horas
        .expect("timestamp válido")
        .timestamp();

    let claims = Claims {
        sub: usuario.to_owned(),
        papel,
        exp: expiracao as usize,
        iat: agora.timestamp() as usize,
    };

    // Obtém a chave secreta das variáveis de ambiente
    // IMPORTANTE: Configure JWT_SECRET no arquivo .env
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| {
        tracing::warn!("JWT_SECRET não configurado! Usando chave padrão INSEGURA.");
        "chave_padrao_MUITO_INSEGURA_mude_isso_em_producao".to_string()
    });

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
}

/// Valida e decodifica um token JWT
///
/// # Argumentos
/// * `token` - String contendo o token JWT a ser validado
///
/// # Retorna
/// * `Ok(Claims)` - Claims extraídos do token válido
/// * `Err(jsonwebtoken::errors::Error)` - Erro de validação ou token inválido
///
/// # Exemplo
/// ```ignore
/// match validar_token(&token_string) {
///     Ok(claims) => println!("Usuário: {}, Papel: {:?}", claims.sub, claims.papel),
///     Err(_) => println!("Token inválido"),
/// }
/// ```
pub fn validar_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "chave_padrao_MUITO_INSEGURA_mude_isso_em_producao".to_string());

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}

/// Extrai o token do cabeçalho Authorization no formato "Bearer <token>"
///
/// # Argumentos
/// * `auth_header` - String do cabeçalho Authorization
///
/// # Retorna
/// * `Some(&str)` - Token extraído
/// * `None` - Formato inválido ou token ausente
pub fn extrair_token_bearer(auth_header: &str) -> Option<&str> {
    auth_header.strip_prefix("Bearer ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_criar_e_validar_token() {
        std::env::set_var("JWT_SECRET", "teste_secret_key_123");

        let token =
            criar_token("usuario_teste", PapelUsuario::Tecnico).expect("Deveria criar token");

        let claims = validar_token(&token).expect("Deveria validar token");

        assert_eq!(claims.sub, "usuario_teste");
        assert_eq!(claims.papel, PapelUsuario::Tecnico);
    }

    #[test]
    fn test_extrair_token_bearer() {
        let header = "Bearer meu_token_jwt_123";
        let token = extrair_token_bearer(header);
        assert_eq!(token, Some("meu_token_jwt_123"));

        let header_invalido = "Token meu_token";
        assert_eq!(extrair_token_bearer(header_invalido), None);
    }
}
