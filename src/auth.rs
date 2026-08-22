// src/auth.rs
//! Módulo de autenticação JWT para o servidor Senior System
//!
//! Este módulo gerencia a criação e validação de tokens JWT,
//! garantindo que apenas usuários autenticados possam acessar
//! recursos protegidos da API.
//!
//! ## Política de segredo
//!
//! O `JWT_SECRET` é OBRIGATÓRIO. Em build release (`--release`), o servidor
//! PANICA na inicialização se a variável estiver vazia ou ausente. Em dev, é
//! permitido um placeholder, com aviso nos logs.
//!
//! ## Claims emitidas
//!
//! - `sub`     : username
//! - `uid`     : id numérico do usuário
//! - `papel`   : papel (Administrador, Gerencia, ...)
//! - `tenant`  : id do tenant (0 = sistema single-tenant)
//! - `exp`     : expiração (24h por padrão)
//! - `iat`     : emissão

use crate::servicos::PapelUsuario;
use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, errors::ErrorKind, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

/// Tempo de expiração do token.
pub const TOKEN_TTL_HOURS: i64 = 24;

/// Segredo placeholder aceito APENAS em dev. Em release o servidor panic.
const DEV_PLACEHOLDER_SECRET: &str = "dev-only-not-for-production-rotate-me";

/// Comprimento mínimo aceitável de um JWT_SECRET em produção.
const MIN_PROD_SECRET_LEN: usize = 32;

/// Claims (reivindicações) contidas no token JWT
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject - identificador do usuário (username)
    pub sub: String,
    /// ID numérico do usuário (para evitar lookup por username)
    pub uid: i32,
    /// Papel/Role do usuário no sistema
    pub papel: PapelUsuario,
    /// Tenant (multi-tenant). 0 = single-tenant.
    #[serde(default)]
    pub tenant: i32,
    /// Expiration time - timestamp Unix de quando o token expira
    pub exp: usize,
    /// Issued at - timestamp Unix de quando o token foi criado
    pub iat: usize,
}

/// Resolvedor único do segredo JWT. Aplica a política de fail-fast em release.
pub fn obter_secret() -> Result<String, String> {
    let raw = std::env::var("JWT_SECRET").ok();

    match raw {
        // Em release: segredo vazio ou placeholder é proibido.
        None => {
            if cfg!(debug_assertions) {
                tracing::warn!(
                    "⚠️  JWT_SECRET não definido; usando placeholder de DESENVOLVIMENTO. \
                     Isso é proibido em produção."
                );
                Ok(DEV_PLACEHOLDER_SECRET.to_string())
            } else {
                Err(
                    "JWT_SECRET não definido. Em produção, esta variável é OBRIGATÓRIA. \
                     Gere uma com `openssl rand -base64 64`."
                        .to_string(),
                )
            }
        }
        Some(s) if s.is_empty() => {
            if cfg!(debug_assertions) {
                tracing::warn!("⚠️  JWT_SECRET vazio; usando placeholder de dev.");
                Ok(DEV_PLACEHOLDER_SECRET.to_string())
            } else {
                Err("JWT_SECRET vazio. Defina um valor forte.".to_string())
            }
        }
        Some(s) if s == DEV_PLACEHOLDER_SECRET => {
            if cfg!(debug_assertions) {
                tracing::warn!("⚠️  Usando JWT_SECRET de dev explícito.");
                Ok(s)
            } else {
                Err("JWT_SECRET igual ao placeholder de dev. Em produção use um segredo real.".to_string())
            }
        }
        Some(s) => {
            if !cfg!(debug_assertions) && s.len() < MIN_PROD_SECRET_LEN {
                return Err(format!(
                    "JWT_SECRET muito curto ({} < {} bytes). Use pelo menos 64 bytes em produção.",
                    s.len(),
                    MIN_PROD_SECRET_LEN
                ));
            }
            Ok(s)
        }
    }
}

/// Cria um novo token JWT para um usuário autenticado
pub fn criar_token(
    usuario: &str,
    uid: i32,
    papel: PapelUsuario,
    tenant: i32,
) -> Result<String, jsonwebtoken::errors::Error> {
    let agora = Utc::now();
    let expiracao = agora
        .checked_add_signed(Duration::hours(TOKEN_TTL_HOURS))
        .expect("timestamp válido")
        .timestamp();

    let claims = Claims {
        sub: usuario.to_owned(),
        uid,
        papel,
        tenant,
        exp: expiracao as usize,
        iat: agora.timestamp() as usize,
    };

    let secret = obter_secret().map_err(|_| jsonwebtoken::errors::Error::from(ErrorKind::InvalidToken))?;

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
}

/// Valida e decodifica um token JWT
pub fn validar_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = obter_secret().map_err(|_| jsonwebtoken::errors::Error::from(ErrorKind::InvalidToken))?;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}

/// Extrai o token do cabeçalho Authorization no formato "Bearer <token>".
/// Centraliza a lógica para o middleware e o extractor.
pub fn extrair_bearer(headers: &axum::http::HeaderMap) -> Option<&str> {
    headers
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .map(|s| s.trim())
}

/// Erro de autenticação, mapeado para HTTP pelo middleware.
#[derive(Debug, Clone, Copy)]
pub enum AuthError {
    /// Sem header `Authorization: Bearer ...`
    Ausente,
    /// Token presente mas inválido (assinatura, expiração, formato)
    Invalido,
}

impl AuthError {
    pub fn status(self) -> StatusCode {
        match self {
            AuthError::Ausente => StatusCode::UNAUTHORIZED,
            AuthError::Invalido => StatusCode::UNAUTHORIZED,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            AuthError::Ausente => "AUTH_REQUIRED",
            AuthError::Invalido => "AUTH_INVALID",
        }
    }
}

/// Extractor de Claims para handlers protegidos.
///
/// Uso:
/// ```ignore
/// async fn handler(claims: Claims, ...) -> ... { ... }
/// ```
/// Se o token estiver ausente ou inválido, retorna 401 com
/// `{ "error": { "code": "AUTH_REQUIRED", ... } }`.
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, axum::Json<serde_json::Value>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let headers = &parts.headers;
        let token = extrair_bearer(headers).ok_or_else(|| {
            (
                AuthError::Ausente.status(),
                axum::Json(serde_json::json!({
                    "error": {
                        "code": AuthError::Ausente.code(),
                        "message": "Autenticação necessária. Envie 'Authorization: Bearer <token>'.",
                    }
                })),
            )
        })?;

        let claims = validar_token(token).map_err(|_| {
            (
                AuthError::Invalido.status(),
                axum::Json(serde_json::json!({
                    "error": {
                        "code": AuthError::Invalido.code(),
                        "message": "Token inválido ou expirado.",
                    }
                })),
            )
        })?;

        // Anexa o claims para handlers que precisem do `Extension` ou `Parts`.
        parts.extensions.insert(claims.clone());
        Ok(claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set_test_secret() {
        std::env::set_var("JWT_SECRET", "teste_secret_key_minimo_32_bytes_aaaaaaaa");
    }

    #[test]
    fn test_criar_e_validar_token() {
        set_test_secret();
        let token = criar_token("usuario_teste", 42, PapelUsuario::Tecnico, 0)
            .expect("Deveria criar token");
        let claims = validar_token(&token).expect("Deveria validar token");
        assert_eq!(claims.sub, "usuario_teste");
        assert_eq!(claims.uid, 42);
        assert_eq!(claims.papel, PapelUsuario::Tecnico);
    }

    #[test]
    fn test_token_invalido_nao_passa() {
        set_test_secret();
        let resultado = validar_token("isto.nao.e.um.jwt");
        assert!(resultado.is_err());
    }

    #[test]
    fn test_extrair_bearer_valido() {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert(AUTHORIZATION, "Bearer abc.def.ghi".parse().unwrap());
        assert_eq!(extrair_bearer(&headers), Some("abc.def.ghi"));
    }

    #[test]
    fn test_extrair_bearer_ausente() {
        let headers = axum::http::HeaderMap::new();
        assert_eq!(extrair_bearer(&headers), None);
    }

    #[test]
    fn test_extrair_bearer_formato_errado() {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert(AUTHORIZATION, "Token abc".parse().unwrap());
        assert_eq!(extrair_bearer(&headers), None);
    }

    #[test]
    fn test_secret_curto_em_release_e_rejeitado() {
        // Em release, segredo < 32 bytes é rejeitado.
        if !cfg!(debug_assertions) {
            std::env::set_var("JWT_SECRET", "curto");
            assert!(obter_secret().is_err());
        }
    }
}
