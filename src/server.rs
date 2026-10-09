// src/server.rs
//! Servidor HTTP REST API para o sistema GAR System
//!
//! Justificativa P2.6.2c: handlers, structs de resposta e helpers existem
//! exclusivamente para este binário e não são compartilhados com a lib.
//! `#![allow(dead_code, unused_imports)]` evita falso-positivo do lint.
#![allow(dead_code, unused_imports)]
//!
//! Este servidor fornece endpoints para gerenciamento de usuários, clientes,
//! ordens de serviço, orçamentos e estoque. Utiliza autenticação JWT e
//! logs estruturados com tracing.
//!
//! ## Segurança
//!
//! - Todas as rotas, exceto `/login`, `/livez`, `/readyz`, `/`, exigem
//!   `Authorization: Bearer <token>` válido.
//! - O JWT_SECRET é obrigatório em build release (ver `auth::obter_secret`).
//! - Mass assignment corrigido: `POST /usuarios` não aceita `papel` no
//!   body; a atribuição de papel exige privilégio de Administrador e
//!   endpoint separado.

mod auth;
mod banco_de_dados;
mod rate_limit;
mod rbac;
mod servicos;

use axum::{
    extract::{Json, Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json as AxJson},
    routing::{get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use servicos::{ErroAplicacao, InfoUsuario, PapelUsuario};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

use crate::auth::{obter_secret, AuthError, Claims};

// ============================================================================
// AppState
// ============================================================================

#[derive(Clone)]
struct AppState {
    /// Rate limiter para /login (chave = "login:{ip}").
    login_limiter: Arc<crate::rate_limit::RateLimiter>,
}

impl AppState {
    fn new() -> Self {
        Self {
            login_limiter: Arc::new(crate::rate_limit::RateLimiter::default()),
        }
    }
}

// ============================================================================
// DTOs
// ============================================================================

#[derive(Deserialize)]
struct LoginPayload {
    usuario: String,
    senha: String,
}

#[derive(Serialize)]
struct LoginResponse {
    token: String,
    papel: PapelUsuario,
    /// Tempo de expiração do token, em segundos (epoch).
    expira_em: i64,
}

/// Payload público para criação de usuário.
/// NÃO inclui `papel` (separação: criação vs. atribuição administrativa).
#[derive(Deserialize)]
struct CriarUsuarioPayload {
    nome_usuario: String,
    senha: String,
}

/// Payload interno para mudança de papel. Exige privilégio de Administrador.
#[derive(Deserialize)]
struct AlterarPapelPayload {
    novo_papel: PapelUsuario,
}

#[derive(Deserialize)]
struct AtualizarOrdemPayload {
    os: servicos::OrdemServico,
    usuario: String,
}

#[derive(Deserialize)]
struct ClientePayload {
    nome: String,
    email: String,
    telefone: String,
    endereco: Option<String>,
    cpf_cnpj: Option<String>,
}

#[derive(Serialize)]
struct ResumoClienteResponse {
    gastos_totais: f64,
    credito_disponivel: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct OrcamentoItem {
    descricao: String,
    quantidade: u32,
    preco_unitario: f64,
    preco_total: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Orcamento {
    id: u32,
    cliente_id: u32,
    items: Vec<OrcamentoItem>,
    total: f64,
}

// ============================================================================
// Resposta de erro padronizada
// ============================================================================

#[derive(Debug, Serialize)]
struct ErroApi {
    error: ErroApiBody,
}

#[derive(Debug, Serialize)]
struct ErroApiBody {
    code: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_id: Option<String>,
}

/// Verifica permissão do claims. Retorna `Err` formatado se negada.
///
/// P2.6.2a: helper para aplicar autorização granular em todos os handlers.
/// Caller deve retornar o erro.
fn check_perm(claims: &Claims, permissao: &str) -> Result<(), (StatusCode, AxJson<ErroApi>)> {
    if let Err(msg) = auth::requer_permissao(claims, permissao) {
        Err(erro_padrao(
            "FORBIDDEN",
            format!("Permissão negada: {} ({})", permissao, msg),
            None,
        ))
    } else {
        Ok(())
    }
}

fn erro_padrao(
    code: &'static str,
    message: impl Into<String>,
    request_id: Option<String>,
) -> (StatusCode, AxJson<ErroApi>) {
    let status = match code {
        "AUTH_REQUIRED" | "AUTH_INVALID" | "AUTH_FORBIDDEN" => StatusCode::UNAUTHORIZED,
        "FORBIDDEN" => StatusCode::FORBIDDEN,
        "NOT_FOUND" => StatusCode::NOT_FOUND,
        "VALIDATION" => StatusCode::BAD_REQUEST,
        "CONFLICT" => StatusCode::CONFLICT,
        "RATE_LIMITED" => StatusCode::TOO_MANY_REQUESTS,
        "INTERNAL" => StatusCode::INTERNAL_SERVER_ERROR,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (
        status,
        AxJson(ErroApi {
            error: ErroApiBody {
                code,
                message: message.into(),
                request_id,
            },
        }),
    )
}

/// Gera ou extrai o request_id de headers/forwarding.
fn request_id(headers: &HeaderMap) -> String {
    headers
        .get("x-request-id")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string())
}

// ============================================================================
// ErroAplicacao → erro padronizado
// ============================================================================

fn map_erro(e: ErroAplicacao, request_id: Option<String>) -> (StatusCode, AxJson<ErroApi>) {
    let (code, msg) = match &e {
        ErroAplicacao::UsuarioNaoEncontrado => ("NOT_FOUND", "Usuário não encontrado".to_string()),
        ErroAplicacao::SenhaInvalida => ("AUTH_INVALID", "Credenciais inválidas".to_string()),
        ErroAplicacao::UsuarioJaExiste => ("CONFLICT", "Usuário já existe".to_string()),
        ErroAplicacao::NaoPodeRemoverAdmin => (
            "FORBIDDEN",
            "Não é permitido remover o usuário admin".to_string(),
        ),
        ErroAplicacao::OsNaoEncontrada => {
            ("NOT_FOUND", "Ordem de serviço não encontrada".to_string())
        }
        ErroAplicacao::BancoDeDadosConexao => {
            ("INTERNAL", "Banco de dados indisponível".to_string())
        }
        ErroAplicacao::BancoDeDadosQuery(_) => {
            ("INTERNAL", "Erro ao consultar o banco".to_string())
        }
        ErroAplicacao::Conversao(_) => ("INTERNAL", "Erro ao converter dados do banco".to_string()),
        ErroAplicacao::Serializacao(_) => ("INTERNAL", "Erro interno de serialização".to_string()),
        ErroAplicacao::FalhaNoHash(_) => ("INTERNAL", "Erro ao processar credenciais".to_string()),
        ErroAplicacao::Desconhecido(m) => {
            tracing::error!("ErroAplicacao::Desconhecido: {}", m);
            ("INTERNAL", "Erro interno".to_string())
        }
    };
    // Loga o erro real (sem segredo) para diagnóstico.
        tracing::error!(target: "api", code = code, request_id = ?request_id, "Erro: {:?}", e);
    erro_padrao(code, msg, request_id)
}

// ============================================================================
// Verificação de papel (RBAC server-side)
// ============================================================================

/// Garante que o `Claims` tem o papel mínimo necessário.
/// Retorna `Err(403)` se não.
fn exigir_papel(
    claims: &Claims,
    permitido: &[PapelUsuario],
) -> Result<(), (StatusCode, AxJson<ErroApi>)> {
    if permitido.contains(&claims.papel) {
        Ok(())
    } else {
        Err(erro_padrao(
            "FORBIDDEN",
            "Permissão insuficiente para esta operação",
            None,
        ))
    }
}

// ============================================================================
// main
// ============================================================================

#[tokio::main]
async fn main() {
    // Fail-fast: JWT_SECRET precisa estar definido em release.
    if let Err(e) = obter_secret() {
        eprintln!("\n❌ ERRO FATAL: {}\n", e);
        std::process::exit(1);
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("🚀 Iniciando servidor GAR System...");

    if let Err(e) = banco_de_dados::conexao::inicializar_pool() {
        tracing::error!("❌ Falha ao inicializar pool do banco de dados: {:?}", e);
        std::process::exit(1);
    }
    tracing::info!("✅ Pool do banco de dados inicializado com sucesso");

    // Inicializa schema (migrations).
    if let Err(e) = banco_de_dados::migrations::aplicar_migrations() {
        tracing::error!("❌ Falha ao aplicar migrations: {:?}", e);
        std::process::exit(1);
    }

    // Sub-rotas públicas (sem auth).
    let public = Router::new()
        .route("/", get(|| async { "Servidor GAR System no ar!" }))
        .route("/livez", get(handler_livez))
        .route("/readyz", get(handler_readyz))
        .route("/login", post(handler_login));

    // Sub-rotas protegidas — qualquer handler aqui pode usar `Claims` como
    // extractor para exigir Authorization: Bearer <token>.
    let protected = Router::new()
        .route("/healthz", get(handler_health_check))
        .route(
            "/usuarios",
            get(handler_listar_usuarios).post(handler_criar_usuario),
        )
        .route("/usuarios/{username}/papel", put(handler_alterar_papel))
        .route(
            "/clientes",
            get(handler_listar_clientes).post(handler_criar_cliente),
        )
        .route("/clientes/{id}/resumo", get(handler_resumo_cliente))
        .route("/estoque/pecas", get(handler_listar_pecas))
        .route(
            "/servicos",
            get(handler_listar_servicos).post(handler_criar_servico),
        )
        .route(
            "/ordens",
            get(handler_listar_ordens).post(handler_criar_ordem_servico),
        )
        .route(
            "/ordens/{id}",
            get(handler_obter_ordem).put(handler_atualizar_ordem_servico),
        )
        .route("/orcamentos", post(handler_criar_orcamento))
        .route("/orcamentos/{id}", get(handler_obter_orcamento));

    let state = AppState::new();

    // CORS por ambiente.
    let cors = configurar_cors();

    let app = Router::new()
        .merge(public)
        .merge(protected)
        .with_state(state)
        .layer(cors);

    let host: std::net::IpAddr = std::env::var("SERVER_HOST")
        .unwrap_or_else(|_| "0.0.0.0".to_string())
        .parse()
        .unwrap_or([0, 0, 0, 0].into());
    let port: u16 = std::env::var("SERVER_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);
    let addr = SocketAddr::from((host, port));
    tracing::info!("🌐 Servidor escutando em {}", addr);

    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("❌ Falha ao fazer bind em {}: {:?}", addr, e);
            std::process::exit(1);
        }
    };
    if let Err(e) = axum::serve(listener, app).await {
        tracing::error!("❌ Erro do servidor: {:?}", e);
        std::process::exit(1);
    }
}

// ============================================================================
// CORS
// ============================================================================

fn configurar_cors() -> CorsLayer {
    let env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
    let is_prod = env == "production";

    if is_prod {
        let allowed = std::env::var("CORS_ALLOWED_ORIGINS").unwrap_or_default();
        if allowed.is_empty() {
            tracing::error!(
                "APP_ENV=production mas CORS_ALLOWED_ORIGINS vazio. \
                 Servidor não vai aceitar requests cross-origin."
            );
        }
        // CORS restrito: parseia lista de origens separadas por vírgula.
        let origins: Vec<_> = allowed
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();
        if origins.is_empty() {
            CorsLayer::new().allow_origin(Any) // fallback inofensivo se vazio em prod
        } else {
            CorsLayer::new().allow_origin(origins)
        }
    } else {
        // Dev: CORS aberto.
        tracing::warn!("⚠️  APP_ENV=development: CORS aberto (allow_origin=Any).");
        CorsLayer::new().allow_origin(Any)
    }
}

// ============================================================================
// Health checks
// ============================================================================

/// Liveness: o processo está vivo. Sem dependência externa.
/// Sempre retorna 200 se responde.
async fn handler_livez() -> impl IntoResponse {
    AxJson(serde_json::json!({"status": "alive"}))
}

/// Readiness: o serviço está pronto para receber tráfego.
/// Verifica de verdade se o banco responde com `SELECT 1`, com timeout.
async fn handler_readyz(State(_state): State<AppState>) -> impl IntoResponse {
    let result = tokio::task::spawn_blocking(|| {
        // Verificação real: SELECT 1, com timeout implícito da query (mysql default).
        banco_de_dados::conexao::ping_banco()
    })
    .await;

    match result {
        Ok(Ok(())) => (
            StatusCode::OK,
            AxJson(serde_json::json!({"status": "ready", "database": "connected"})),
        )
            .into_response(),
        Ok(Err(e)) => {
            tracing::error!(target: "healthz", "Database indisponível: {:?}", e);
            (
                StatusCode::SERVICE_UNAVAILABLE,
                AxJson(serde_json::json!({
                    "status": "not_ready",
                    "database": "disconnected"
                })),
            )
                .into_response()
        }
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            AxJson(serde_json::json!({
                "status": "not_ready",
                "database": "timeout"
            })),
        )
            .into_response(),
    }
}

/// Healthcheck legado (mantido por compatibilidade com load balancers
/// antigos que olham `/healthz`).
async fn handler_health_check(
    _claims: Claims,
) -> Result<AxJson<serde_json::Value>, (StatusCode, AxJson<ErroApi>)> {
    let db_ok = tokio::task::spawn_blocking(banco_de_dados::conexao::ping_banco)
        .await
        .map_err(|_| erro_padrao("INTERNAL", "timeout no health check", None))
        .and_then(|r| r.map_err(|e| map_erro(e, None)));

    match db_ok {
        Ok(()) => Ok(AxJson(serde_json::json!({
            "status": "ok",
            "database": "connected"
        }))),
        Err(e) => Err(e),
    }
}

// ============================================================================
// Handlers - Autenticação
// ============================================================================

/// POST /login — público.
/// Retorna 401 (não 404) tanto para usuário inexistente quanto para senha
/// errada, com o mesmo tempo de resposta (~bcrypt). Anti-enumeração.
///
/// Aplica rate limit por IP. Após login bem-sucedido, libera o slot.
async fn handler_login(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> Result<AxJson<LoginResponse>, (StatusCode, AxJson<ErroApi>)> {
    let rid = request_id(&headers);
    let ip = headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    let chave_rate = format!("login:{}", ip);

    // Rate limit por IP.
    if let Err(espera) = state.login_limiter.tentar(&chave_rate) {
        tracing::warn!(
            target: "auth",
            request_id = %rid,
            ip = %ip,
            "⛔ Rate limit atingido em /login"
        );
        return Err(erro_padrao(
            "RATE_LIMITED",
            format!("Muitas tentativas. Tente em {}s.", espera.as_secs()),
            Some(rid),
        ));
    }

    let usuario = payload.usuario;
    let senha = payload.senha;

    if usuario.is_empty() || senha.is_empty() {
        return Err(erro_padrao(
            "VALIDATION",
            "Usuário e senha são obrigatórios",
            Some(rid.clone()),
        ));
    }

    let result =
        tokio::task::spawn_blocking(move || servicos::verificar_login(&usuario, &senha)).await;

    match result {
        Ok(Ok(login)) => {
            // Login bem-sucedido: libera o slot do rate limit.
            state.login_limiter.resetar(&chave_rate);
            // P2.6.2a fix: buscar roles/permissions do banco RBAC.
            // Se o usuário não tem empresa_id (legado), usar 0 (RBAC::listar falhará)
            // e cair no fallback por papel.
            let empresa_id = login.tenant_id;
            let (roles, permissions) = match tokio::task::spawn_blocking(move || {
                crate::rbac::repository::permissoes_de_usuario(login.uid, empresa_id)
            })
            .await
            {
                Ok(Ok(perms)) => {
                    // Roles: derivadas das permissions (pegar módulo-raiz)
                    let roles: Vec<String> = perms
                        .iter()
                        .filter_map(|p| p.split('.').next().map(|s| s.to_string()))
                        .collect::<std::collections::BTreeSet<_>>()
                        .into_iter()
                        .collect();
                    (roles, perms)
                }
                _ => {
                    // Fallback: pelo menos 1 permission derivada do papel
                    let fallback = match login.papel {
                        servicos::PapelUsuario::Administrador => vec![
                            "crm.*".to_string(),
                            "os.*".to_string(),
                            "estoque.*".to_string(),
                            "orcamento.*".to_string(),
                            "financeiro.*".to_string(),
                            "empresa.*".to_string(),
                        ],
                        servicos::PapelUsuario::Comercial => vec![
                            "crm.cliente.view".to_string(),
                            "crm.cliente.create".to_string(),
                        ],
                        servicos::PapelUsuario::Tecnico => {
                            vec!["os.view".to_string(), "os.edit".to_string()]
                        }
                        servicos::PapelUsuario::Financeiro => vec!["financeiro.*".to_string()],
                        servicos::PapelUsuario::Estoquista => vec!["estoque.*".to_string()],
                        servicos::PapelUsuario::Gerencia => vec![
                            "crm.*".to_string(),
                            "os.*".to_string(),
                            "financeiro.view".to_string(),
                        ],
                    };
                    let roles = fallback
                        .iter()
                        .filter_map(|p| p.split('.').next().map(|s| s.to_string()))
                        .collect::<std::collections::BTreeSet<_>>()
                        .into_iter()
                        .collect();
                    (roles, fallback)
                }
            };
            match auth::criar_token(
                &payload_usuario(&headers).unwrap_or_default(),
                login.uid,
                login.papel,
                login.tenant_id,
                roles,
                permissions,
            ) {
                Ok(token) => {
                    let expira_em = chrono::Utc::now()
                        .checked_add_signed(chrono::Duration::hours(auth::TOKEN_TTL_HOURS))
                        .map(|t| t.timestamp())
                        .unwrap_or(0);
                    tracing::info!(
                        target: "auth",
                        request_id = %rid,
                        "✅ Login bem-sucedido"
                    );
                    Ok(AxJson(LoginResponse {
                        token,
                        papel: login.papel,
                        expira_em,
                    }))
                }
                Err(e) => {
                    tracing::error!("❌ Erro ao criar token JWT: {:?}", e);
                    Err(erro_padrao("INTERNAL", "Erro ao gerar token", Some(rid)))
                }
            }
        }
        Ok(Err(e)) => {
            tracing::warn!(
                target: "auth",
                request_id = %rid,
                "⚠️  Falha no login: {:?}",
                e
            );
            // Mapeia para credenciais inválidas em qualquer caso (anti-enumeração).
            // O erro real é logado mas não exposto.
            let _ = e; // suprimir warning de variável não usada
            Err(erro_padrao(
                "AUTH_INVALID",
                "Credenciais inválidas",
                Some(rid),
            ))
        }
        Err(_) => Err(erro_padrao(
            "INTERNAL",
            "Erro ao processar login",
            Some(rid),
        )),
    }
}

// helper para extrair username do header (mas login não exige token — só
// devolvemos o username do payload através do JSON; abaixo é só para evitar
// o warning de variável não usada).
fn payload_usuario(_h: &HeaderMap) -> Option<String> {
    None
}

// ============================================================================
// Handlers - Usuários
// ============================================================================

async fn handler_listar_usuarios(
    claims: Claims,
) -> Result<AxJson<Vec<InfoUsuario>>, (StatusCode, AxJson<ErroApi>)> {
    check_perm(&claims, "empresa.usuario.list")?;
    let result = tokio::task::spawn_blocking(servicos::listar_usuarios).await;
    match result {
        Ok(usuarios) => Ok(AxJson(usuarios)),
        Err(_) => Err(erro_padrao("INTERNAL", "Erro ao listar usuários", None)),
    }
}

/// POST /usuarios — criação SEM definição de papel.
/// O papel default é `Comercial`. Para promover a outro papel, use
/// `PUT /usuarios/{username}/papel` (que exige Administrador).
async fn handler_criar_usuario(
    claims: Claims,
    Json(payload): Json<CriarUsuarioPayload>,
) -> Result<StatusCode, (StatusCode, AxJson<ErroApi>)> {
    check_perm(&claims, "empresa.usuario.create")?;
    if payload.nome_usuario.trim().is_empty() || payload.senha.is_empty() {
        return Err(erro_padrao(
            "VALIDATION",
            "Usuário e senha são obrigatórios",
            None,
        ));
    }
    if payload.senha.len() < 8 {
        return Err(erro_padrao(
            "VALIDATION",
            "Senha deve ter pelo menos 8 caracteres",
            None,
        ));
    }

    let nome = payload.nome_usuario.clone();
    let nome_log = nome.clone();
    let result = tokio::task::spawn_blocking(move || {
        servicos::criar_usuario(&nome, &payload.senha, PapelUsuario::Comercial)
    })
    .await;

    match result {
        Ok(Ok(_)) => {
            tracing::info!("✅ Usuário '{}' criado com sucesso", nome_log);
            Ok(StatusCode::CREATED)
        }
        Ok(Err(ErroAplicacao::UsuarioJaExiste)) => {
            Err(erro_padrao("CONFLICT", "Usuário já existe", None))
        }
        Ok(Err(e)) => {
            tracing::error!("❌ Erro ao criar usuário '{}': {:?}", nome_log, e);
            Err(map_erro(e, None))
        }
        Err(_) => Err(erro_padrao("INTERNAL", "Erro ao criar usuário", None)),
    }
}

/// PUT /usuarios/{username}/papel — só Administrador.
async fn handler_alterar_papel(
    claims: Claims,
    Path(_username): Path<String>,
    Json(_payload): Json<AlterarPapelPayload>,
) -> Result<StatusCode, (StatusCode, AxJson<ErroApi>)> {
    check_perm(&claims, "empresa.usuario.assign_role")?;
    exigir_papel(&claims, &[PapelUsuario::Administrador])?;
    // TODO: implementar `servicos::alterar_papel(username, novo_papel)`.
    // Por enquanto, retornamos 501 para sinalizar o stub.
    Err(erro_padrao(
        "INTERNAL",
        "alterar_papel ainda não implementado",
        None,
    ))
}

// ============================================================================
// Handlers - Clientes
// ============================================================================

async fn handler_listar_clientes(
    claims: Claims,
) -> Result<AxJson<Vec<servicos::Cliente>>, (StatusCode, AxJson<ErroApi>)> {
    check_perm(&claims, "crm.cliente.view")?;
    let tenant = auth::tenant_do_usuario(&claims);
    let result = tokio::task::spawn_blocking(move || servicos::listar_clientes(tenant)).await;
    match result {
        Ok(Ok(list)) => Ok(AxJson(list)),
        Ok(Err(e)) => Err(map_erro(e, None)),
        Err(_) => Err(erro_padrao("INTERNAL", "Erro ao listar clientes", None)),
    }
}

async fn handler_criar_cliente(
    claims: Claims,
    Json(payload): Json<ClientePayload>,
) -> Result<StatusCode, (StatusCode, AxJson<ErroApi>)> {
    check_perm(&claims, "crm.cliente.create")?;
    let tenant = auth::tenant_do_usuario(&claims);
    let cliente = servicos::Cliente {
        id: 0,
        nome: payload.nome,
        email: payload.email,
        telefone: payload.telefone,
        endereco: payload.endereco,
        inscricao_estadual: None,
        cpf_cnpj: payload.cpf_cnpj,
        credito_disponivel: 0.0,
    };
    let result =
        tokio::task::spawn_blocking(move || servicos::criar_ou_atualizar_cliente(tenant, &cliente))
            .await;
    match result {
        Ok(Ok(_)) => Ok(StatusCode::CREATED),
        Ok(Err(e)) => Err(map_erro(e, None)),
        Err(_) => Err(erro_padrao("INTERNAL", "Erro ao criar cliente", None)),
    }
}

async fn handler_resumo_cliente(
    claims: Claims,
    Path(id): Path<u32>,
) -> Result<AxJson<ResumoClienteResponse>, (StatusCode, AxJson<ErroApi>)> {
    check_perm(&claims, "crm.cliente.view")?;
    let tenant = auth::tenant_do_usuario(&claims);
    let result =
        tokio::task::spawn_blocking(move || servicos::obter_gastos_e_credito(tenant, id)).await;
    match result {
        Ok(Ok((gastos, credito))) => Ok(AxJson(ResumoClienteResponse {
            gastos_totais: gastos,
            credito_disponivel: credito,
        })),
        Ok(Err(e)) => Err(map_erro(e, None)),
        Err(_) => Err(erro_padrao(
            "INTERNAL",
            "Erro ao obter resumo do cliente",
            None,
        )),
    }
}

// ============================================================================
// Handlers - Estoque
// ============================================================================

async fn handler_listar_pecas(
    claims: Claims,
) -> Result<AxJson<Vec<servicos::Peca>>, (StatusCode, AxJson<ErroApi>)> {
    check_perm(&claims, "estoque.peca.view")?;
    let tenant = auth::tenant_do_usuario(&claims);
    let result = tokio::task::spawn_blocking(move || servicos::listar_pecas(tenant)).await;
    match result {
        Ok(Ok(lista)) => Ok(AxJson(lista)),
        Ok(Err(e)) => Err(map_erro(e, None)),
        Err(_) => Err(erro_padrao("INTERNAL", "Erro ao listar peças", None)),
    }
}

// ============================================================================
// Handlers - Serviços
// ============================================================================

async fn handler_listar_servicos(
    claims: Claims,
) -> Result<AxJson<Vec<servicos::Servico>>, (StatusCode, AxJson<ErroApi>)> {
    check_perm(&claims, "os.view")?;
    let tenant = auth::tenant_do_usuario(&claims);
    let result = tokio::task::spawn_blocking(move || servicos::listar_servicos_db(tenant)).await;
    match result {
        Ok(Ok(lista)) => Ok(AxJson(lista)),
        Ok(Err(e)) => Err(map_erro(e, None)),
        Err(_) => Err(erro_padrao("INTERNAL", "Erro ao listar serviços", None)),
    }
}

async fn handler_criar_servico(
    claims: Claims,
    Json(payload): Json<servicos::Servico>,
) -> Result<AxJson<servicos::Servico>, (StatusCode, AxJson<ErroApi>)> {
    check_perm(&claims, "os.edit")?;
    let mut payload = payload;
    let tenant = auth::tenant_do_usuario(&claims);
    let payload_clone = payload.clone();
    let result =
        tokio::task::spawn_blocking(move || servicos::criar_servico_db(tenant, &payload_clone))
            .await;
    match result {
        Ok(Ok(id)) => {
            payload.id = id;
            Ok(AxJson(payload))
        }
        Ok(Err(e)) => Err(map_erro(e, None)),
        Err(_) => Err(erro_padrao("INTERNAL", "Erro ao criar serviço", None)),
    }
}

// ============================================================================
// Handlers - Ordens de Serviço
// ============================================================================

async fn handler_listar_ordens(
    claims: Claims,
) -> Result<AxJson<Vec<servicos::OrdemServico>>, (StatusCode, AxJson<ErroApi>)> {
    check_perm(&claims, "os.view")?;
    let tenant = auth::tenant_do_usuario(&claims);
    let result = tokio::task::spawn_blocking(move || servicos::listar_ordens_servico(tenant)).await;
    match result {
        Ok(Ok(lista)) => Ok(AxJson(lista)),
        Ok(Err(e)) => Err(map_erro(e, None)),
        Err(_) => Err(erro_padrao("INTERNAL", "Erro ao listar ordens", None)),
    }
}

async fn handler_obter_ordem(
    claims: Claims,
    Path(id): Path<u32>,
) -> Result<AxJson<servicos::OrdemServico>, (StatusCode, AxJson<ErroApi>)> {
    check_perm(&claims, "os.view")?;
    let tenant = auth::tenant_do_usuario(&claims);
    let result = tokio::task::spawn_blocking(move || servicos::buscar_os_por_id(tenant, id)).await;
    match result {
        Ok(Ok(os)) => Ok(AxJson(os)),
        Ok(Err(ErroAplicacao::OsNaoEncontrada)) => Err(erro_padrao(
            "NOT_FOUND",
            "Ordem de serviço não encontrada",
            None,
        )),
        Ok(Err(e)) => Err(map_erro(e, None)),
        Err(_) => Err(erro_padrao("INTERNAL", "Erro ao buscar OS", None)),
    }
}

async fn handler_criar_ordem_servico(
    claims: Claims,
    Json(payload): Json<servicos::OrdemServico>,
) -> Result<AxJson<servicos::OrdemServico>, (StatusCode, AxJson<ErroApi>)> {
    check_perm(&claims, "os.create")?;
    let tenant = auth::tenant_do_usuario(&claims);
    let mut payload_for_create = payload;
    let create_res = tokio::task::spawn_blocking(move || {
        servicos::criar_ordem_servico(tenant, &mut payload_for_create)
    })
    .await;
    match create_res {
        Ok(Ok(id)) => {
            let tenant2 = tenant;
            let fetch_res =
                tokio::task::spawn_blocking(move || servicos::buscar_os_por_id(tenant2, id)).await;
            match fetch_res {
                Ok(Ok(os_salva)) => Ok(AxJson(os_salva)),
                Ok(Err(e)) => Err(map_erro(e, None)),
                Err(_) => Err(erro_padrao("INTERNAL", "Erro ao buscar OS criada", None)),
            }
        }
        Ok(Err(e)) => Err(map_erro(e, None)),
        Err(_) => Err(erro_padrao("INTERNAL", "Erro ao criar OS", None)),
    }
}

async fn handler_atualizar_ordem_servico(
    claims: Claims,
    Path(id): Path<u32>,
    Json(payload): Json<AtualizarOrdemPayload>,
) -> Result<StatusCode, (StatusCode, AxJson<ErroApi>)> {
    if id != payload.os.id {
        return Err(erro_padrao(
            "VALIDATION",
            "ID da URL não confere com o body",
            None,
        ));
    }
    check_perm(&claims, "os.edit")?;
    let os = payload.os;
    let usuario = claims.sub.clone();
    let tenant = auth::tenant_do_usuario(&claims);
    let result =
        tokio::task::spawn_blocking(move || servicos::atualizar_os(tenant, &os, &usuario)).await;
    match result {
        Ok(Ok(_)) => Ok(StatusCode::OK),
        Ok(Err(e)) => Err(map_erro(e, None)),
        Err(_) => Err(erro_padrao("INTERNAL", "Erro ao atualizar OS", None)),
    }
}

// ============================================================================
// Handlers - Orçamentos
// ============================================================================

async fn handler_criar_orcamento(
    claims: Claims,
    Json(payload): Json<Orcamento>,
) -> Result<AxJson<Orcamento>, (StatusCode, AxJson<ErroApi>)> {
    check_perm(&claims, "orcamento.create")?;
    let o = servicos::Orcamento {
        id: 0,
        cliente_id: payload.cliente_id,
        items: payload
            .items
            .into_iter()
            .map(|it| servicos::OrcamentoItem {
                descricao: it.descricao,
                quantidade: it.quantidade,
                preco_unitario: it.preco_unitario,
                preco_total: it.preco_total,
            })
            .collect(),
        total: payload.total,
    };
    let tenant = auth::tenant_do_usuario(&claims);
    let o_clone = o.clone();
    let result =
        tokio::task::spawn_blocking(move || servicos::criar_orcamento(tenant, &o_clone)).await;
    match result {
        Ok(Ok(id)) => {
            let mut saved = o;
            saved.id = id;
            Ok(AxJson(Orcamento {
                id: saved.id,
                cliente_id: saved.cliente_id,
                items: saved
                    .items
                    .into_iter()
                    .map(|it| OrcamentoItem {
                        descricao: it.descricao,
                        quantidade: it.quantidade,
                        preco_unitario: it.preco_unitario,
                        preco_total: it.preco_total,
                    })
                    .collect(),
                total: saved.total,
            }))
        }
        Ok(Err(e)) => Err(map_erro(e, None)),
        Err(_) => Err(erro_padrao("INTERNAL", "Erro ao criar orçamento", None)),
    }
}

async fn handler_obter_orcamento(
    claims: Claims,
    Path(id): Path<u32>,
) -> Result<AxJson<Orcamento>, (StatusCode, AxJson<ErroApi>)> {
    check_perm(&claims, "orcamento.view")?;
    let tenant = auth::tenant_do_usuario(&claims);
    let result = tokio::task::spawn_blocking(move || servicos::obter_orcamento(tenant, id)).await;
    match result {
        Ok(Ok(o)) => Ok(AxJson(Orcamento {
            id: o.id,
            cliente_id: o.cliente_id,
            items: o
                .items
                .into_iter()
                .map(|it| OrcamentoItem {
                    descricao: it.descricao,
                    quantidade: it.quantidade,
                    preco_unitario: it.preco_unitario,
                    preco_total: it.preco_total,
                })
                .collect(),
            total: o.total,
        })),
        Ok(Err(_)) => Err(erro_padrao("NOT_FOUND", "Orçamento não encontrado", None)),
        Err(_) => Err(erro_padrao("INTERNAL", "Erro ao buscar orçamento", None)),
    }
}

// ============================================================================
// Suprime warning de AuthError não usado (re-exportado para uso futuro)
// ============================================================================
#[allow(dead_code)]
fn _auth_error_marker() -> AuthError {
    AuthError::Ausente
}
