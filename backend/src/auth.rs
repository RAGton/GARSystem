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
use jsonwebtoken::{
    decode, encode, errors::ErrorKind, DecodingKey, EncodingKey, Header, Validation,
};
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
    /// Roles do usuário no tenant (P2.6.1 RBAC). ex: ["ADMIN", "TECNICO"]
    #[serde(default)]
    pub roles: Vec<String>,
    /// Permissions concedidas ao usuário (P2.6.1 RBAC). ex: ["os.view", "crm.cliente.create"]
    #[serde(default)]
    pub permissions: Vec<String>,
}

/// Verifica permissão granular (P2.6.2a — RBAC real).
///
/// Cruzamento:
/// 1. SUPER_ADMIN bypass (qualquer permission passa).
/// 2. Permission presente em `claims.permissions` (carregado do token JWT).
/// 3. Wildcard match (ex: "financeiro.*" cobre "financeiro.conta_receber.pagar").
///
/// Em sprint futura: fallback para `PermissionCache` trait (P2.6.1).
pub fn requer_permissao(claims: &Claims, permissao: &str) -> Result<(), String> {
    if claims.uid <= 0 {
        return Err("Usuário inválido".to_string());
    }

    // 1. SUPER_ADMIN bypass
    if claims.roles.iter().any(|r| r == "SUPER_ADMIN") {
        return Ok(());
    }

    // 2. Verifica permission direta
    if claims.permissions.iter().any(|p| p == permissao) {
        return Ok(());
    }

    // 3. Wildcard match (defense in depth)
    let parts: Vec<&str> = permissao.split('.').collect();
    if parts.len() > 1 {
        let wildcard = format!("{}.*", parts[..parts.len() - 1].join("."));
        if claims.permissions.iter().any(|p| p == &wildcard) {
            return Ok(());
        }
        let prefix_wildcard = format!("{}.*", parts[0]);
        if claims.permissions.iter().any(|p| p == &prefix_wildcard) {
            return Ok(());
        }
    }

    Err(format!("permission '{}' não concedida", permissao))
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
                Err(
                    "JWT_SECRET igual ao placeholder de dev. Em produção use um segredo real."
                        .to_string(),
                )
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
///
/// **P2.6.2a fix (Achado #1)**: agora aceita `roles` e `permissions` populadas
/// do banco de dados. Tokens sem roles/permissions são inúteis para RBAC.
pub fn criar_token(
    usuario: &str,
    uid: i32,
    papel: PapelUsuario,
    tenant: i32,
    roles: Vec<String>,
    permissions: Vec<String>,
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
        roles,
        permissions,
        exp: expiracao as usize,
        iat: agora.timestamp() as usize,
    };

    let secret =
        obter_secret().map_err(|_| jsonwebtoken::errors::Error::from(ErrorKind::InvalidToken))?;

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
}

/// Valida e decodifica um token JWT
pub fn validar_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret =
        obter_secret().map_err(|_| jsonwebtoken::errors::Error::from(ErrorKind::InvalidToken))?;

    if blacklist_contem(token) {
        // Token foi revogado por logout. Mapeia para InvalidToken — o
        // middleware responde 401 com `code=AUTH_INVALID`, igual a um
        // token expirado.
        return Err(jsonwebtoken::errors::Error::from(ErrorKind::InvalidToken));
    }

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

/// Extrai o token do cookie `token` (setado por `POST /login`).
///
/// O login emite um cookie `HttpOnly; Secure; SameSite=Strict`, portanto o
/// front não tem acesso ao JWT em JS e não consegue montar o header
/// `Authorization`. O extractor aceita as duas formas; o cookie é protegido
/// contra CSRF por `SameSite=Strict` + allow-list de CORS.
pub fn extrair_cookie_token(headers: &axum::http::HeaderMap) -> Option<String> {
    let raw = headers
        .get(axum::http::header::COOKIE)
        .and_then(|h| h.to_str().ok())?;
    for parte in raw.split(';') {
        let parte = parte.trim();
        if let Some(valor) = parte.strip_prefix("token=") {
            let valor = valor.trim();
            if !valor.is_empty() {
                return Some(valor.to_string());
            }
        }
    }
    None
}

/// Extrai o token da requisição: header `Authorization: Bearer ***` primeiro,
/// cookie `token` como fallback.
pub fn extrair_token(headers: &axum::http::HeaderMap) -> Option<String> {
    if let Some(t) = extrair_bearer(headers) {
        return Some(t.to_string());
    }
    extrair_cookie_token(headers)
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
        // Aceita `Authorization: Bearer *** ou o cookie HttpOnly `token`
        // emitido por `POST /login` (o front não consegue ler o cookie em JS).
        let token = extrair_token(headers).ok_or_else(|| {
            (
                AuthError::Ausente.status(),
                axum::Json(serde_json::json!({
                    "error": {
                        "code": AuthError::Ausente.code(),
                        "message": "Autenticação necessária. Envie 'Authorization: Bearer ***'.",
                    }
                })),
            )
        })?;

        let claims = validar_token(&token).map_err(|_| {
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

/// Resolve o tenant do usuário autenticado (P2.6.2a).
///
/// - Retorna `claims.tenant` se preenchido (multi-tenant ativo).
/// - Fallback para `1` (tenant legacy) se não preenchido (compat retroativa
///   com tokens antigos). Em produção, exige-se tenant preenchido.
pub fn tenant_do_usuario(claims: &Claims) -> i32 {
    if claims.tenant > 0 {
        claims.tenant
    } else {
        crate::servicos::TENANT_LEGACY
    }
}

// (Função `requer_permissao` foi movida para cima, com implementação real.)

// =============================================================================
// Blacklist in-memory de tokens (logout)
// =============================================================================
//
// Minimalista: Map<jti, exp> para tokens revogados. Limpeza lazy —
// entradas expiradas são removidas a cada `inserir`.
// Único-pod: suficiente. Multi-pod requer Redis (TODO: Sprint X).
//
// Não usamos JWT `jti` hoje (Claims não tem), então chaveamos pelo
// próprio token. Funciona para uma sessão única; invalidação só tem
// efeito até `exp`.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

static BLACKLIST: OnceLock<Mutex<HashMap<String, usize>>> = OnceLock::new();

fn blacklist() -> &'static Mutex<HashMap<String, usize>> {
    BLACKLIST.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Adiciona um token à blacklist até a data de expiração (epoch).
/// Remove entradas expiradas em background (cleanup lazy).
/// Se `exp_epoch` já está no passado, ignora silenciosamente — o
/// token seria rejeitado de qualquer forma via `exp` do JWT.
pub fn blacklist_inserir(token: &str, exp_epoch: usize) {
    let agora = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs() as usize)
        .unwrap_or(0);
    if exp_epoch <= agora {
        return;
    }
    let mut map = match blacklist().lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    };
    map.retain(|_, exp| *exp > agora);
    map.insert(token.to_string(), exp_epoch);
}

/// Retorna `true` se o token está revogado.
pub fn blacklist_contem(token: &str) -> bool {
    let map = match blacklist().lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    };
    map.contains_key(token)
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
        let token = criar_token(
            "usuario_teste",
            42,
            PapelUsuario::Tecnico,
            0,
            vec!["Tecnico".to_string()],
            vec!["os.view".to_string()],
        )
        .expect("Deveria criar token");
        let claims = validar_token(&token).expect("Deveria validar token");
        assert_eq!(claims.sub, "usuario_teste");
        assert_eq!(claims.uid, 42);
        assert_eq!(claims.papel, PapelUsuario::Tecnico);
        assert!(!claims.roles.is_empty());
        assert!(!claims.permissions.is_empty());
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

    // ========================================================================
    // SECURITY REVIEW ADVERSARIAL — Achado #1 — REFUTADO
    // Token agora sai com roles e permissions populadas
    // ========================================================================

    #[test]
    fn achado1_token_deve_ter_roles_e_permissions() {
        set_test_secret();
        // Simular o que o handler_login faz: roles + permissions reais
        let token = criar_token(
            "joao.comercial",
            42,
            PapelUsuario::Comercial,
            1,
            vec!["Comercial".to_string()],
            vec![
                "crm.cliente.view".to_string(),
                "crm.cliente.create".to_string(),
            ],
        )
        .expect("Deveria criar token");
        let claims = validar_token(&token).expect("Deveria validar token");
        eprintln!("=== CLAIMS DECODED (PÓS-CORREÇÃO) ===");
        eprintln!("uid:         {}", claims.uid);
        eprintln!("tenant:      {}", claims.tenant);
        eprintln!("roles:       {:?}", claims.roles);
        eprintln!("permissions: {:?}", claims.permissions);
        assert!(!claims.roles.is_empty(), "Token deve ter roles populadas");
        assert!(
            !claims.permissions.is_empty(),
            "Token deve ter permissions populadas"
        );
        assert!(claims.uid > 0, "uid deve ser > 0");
        assert!(claims.tenant > 0, "tenant deve ser > 0");
    }

    #[test]
    fn achado1_check_perm_deve_funcionar_para_usuario_comercial() {
        set_test_secret();
        let token = criar_token(
            "joao.comercial",
            42,
            PapelUsuario::Comercial,
            1,
            vec!["Comercial".to_string()],
            vec!["crm.cliente.view".to_string()],
        )
        .expect("Deveria criar token");
        let claims = validar_token(&token).expect("Deveria validar token");
        let result = requer_permissao(&claims, "crm.cliente.view");
        eprintln!("requer_permissao('crm.cliente.view') = {:?}", result);
        assert!(
            result.is_ok(),
            "Usuário 'Comercial' com permission 'crm.cliente.view' deve ter acesso"
        );
    }

    #[test]
    fn achado1_check_perm_deve_negar_sem_permissao() {
        set_test_secret();
        let token = criar_token(
            "joao.comercial",
            42,
            PapelUsuario::Comercial,
            1,
            vec!["Comercial".to_string()],
            vec!["crm.cliente.view".to_string()],
        )
        .expect("Deveria criar token");
        let claims = validar_token(&token).expect("Deveria validar token");
        let result = requer_permissao(&claims, "financeiro.conta_receber.pagar");
        eprintln!(
            "requer_permissao('financeiro.conta_receber.pagar') = {:?}",
            result
        );
        assert!(
            result.is_err(),
            "Comercial NÃO deve ter acesso a financeiro.*"
        );
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

    // ========================================================================
    // Blacklist (D1 — POST /logout)
    // ========================================================================

    #[test]
    fn achado1_blacklist_inserir_e_contem() {
        blacklist_inserir("token-fake-aaaaaaaaaa", 9_999_999_999);
        assert!(blacklist_contem("token-fake-aaaaaaaaaa"));
        assert!(!blacklist_contem("outro-token-bbbbbbbbbb"));
        // Inserção com `exp` no passado é no-op.
        blacklist_inserir("token-expirado", 1);
        assert!(!blacklist_contem("token-expirado"));
    }

    #[test]
    fn achado1_validar_token_recusa_blacklist() {
        set_test_secret();
        let token = criar_token(
            "u", 1, PapelUsuario::Tecnico, 1,
            vec!["T".into()], vec!["os.view".into()],
        ).unwrap();
        // Antes da blacklist: passa.
        assert!(validar_token(&token).is_ok());
        // Blacklista e tenta de novo.
        blacklist_inserir(&token, 9_999_999_999);
        let r = validar_token(&token);
        assert!(r.is_err(), "Token na blacklist deve falhar validação");
    }
}
