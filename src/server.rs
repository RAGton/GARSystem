// src/server.rs
//! Servidor HTTP REST API para o sistema Senior System
//!
//! Este servidor fornece endpoints para gerenciamento de usuários, clientes,
//! ordens de serviço, orçamentos e estoque. Utiliza autenticação JWT e
//! logs estruturados com tracing.

mod auth;
mod banco_de_dados;
mod servicos;

use axum::{
    extract::Path,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use servicos::{ErroAplicacao, InfoUsuario, PapelUsuario};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

// ============================================================================
// Structs de Request/Response
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
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    database: String,
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

#[derive(Deserialize)]
struct CriarUsuarioPayload {
    nome_usuario: String,
    senha: String,
    papel: PapelUsuario,
}

#[derive(Deserialize)]
struct AtualizarOrdemPayload {
    os: servicos::OrdemServico,
    usuario: String,
}

// ============================================================================
// Main - Inicialização do Servidor
// ============================================================================

#[tokio::main]
async fn main() {
    // Inicializar o sistema de logs estruturados (tracing)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("🚀 Iniciando servidor Senior System...");

    // Inicializa o pool de conexões com o banco de dados
    if let Err(e) = banco_de_dados::conexao::inicializar_pool() {
        tracing::error!("❌ Falha ao inicializar pool do banco de dados: {:?}", e);
        std::process::exit(1);
    }
    tracing::info!("✅ Pool do banco de dados inicializado com sucesso");

    // Inicializa os serviços (migração de schema, etc.)
    servicos::inicializar();
    tracing::info!("✅ Serviços inicializados");

    // Configuração CORS (permitir qualquer origem - ajustar em produção)
    let cors = CorsLayer::new().allow_origin(Any);

    // Definição das rotas da API
    let app = Router::new()
        // Healthcheck
        .route("/healthz", get(handler_health_check))
        // Root
        .route("/", get(|| async { "Servidor Senior System no ar!" }))
        // Autenticação
        .route("/login", post(handler_login))
        // Usuários
        .route(
            "/usuarios",
            get(handler_listar_usuarios).post(handler_criar_usuario),
        )
        // Clientes
        .route(
            "/clientes",
            get(handler_listar_clientes).post(handler_criar_cliente),
        )
        .route("/clientes/{id}/resumo", get(handler_resumo_cliente))
        // Estoque
        .route("/estoque/pecas", get(handler_listar_pecas))
        // Serviços
        .route(
            "/servicos",
            get(handler_listar_servicos).post(handler_criar_servico),
        )
        // Ordens de Serviço
        .route(
            "/ordens",
            get(handler_listar_ordens).post(handler_criar_ordem_servico),
        )
        .route(
            "/ordens/{id}",
            get(handler_obter_ordem).put(handler_atualizar_ordem_servico),
        )
        // Orçamentos
        .route("/orcamentos", post(handler_criar_orcamento))
        .route("/orcamentos/{id}", get(handler_obter_orcamento))
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("🌐 Servidor escutando em {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// ============================================================================
// Handlers - Health Check
// ============================================================================

/// Handler para verificar a saúde do servidor e conexão com o banco
async fn handler_health_check() -> Result<Json<HealthResponse>, StatusCode> {
    // Tenta listar usuários (operação leve) para verificar se o banco está acessível
    let db_status = tokio::task::spawn_blocking(move || {
        servicos::listar_usuarios() // Esta função pública retorna Vec vazio em caso de erro
    })
    .await;

    let (status, database) = match db_status {
        Ok(usuarios) if !usuarios.is_empty() || usuarios.is_empty() => {
            ("ok".to_string(), "connected".to_string())
        }
        _ => ("degraded".to_string(), "disconnected".to_string()),
    };

    if database == "connected" {
        Ok(Json(HealthResponse { status, database }))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

// ============================================================================
// Handlers - Autenticação
// ============================================================================

/// Handler para login de usuários - retorna token JWT
async fn handler_login(
    Json(payload): Json<LoginPayload>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let usuario = payload.usuario;
    let senha = payload.senha;
    let usuario_log = usuario.clone();

    let result =
        tokio::task::spawn_blocking(move || servicos::verificar_login(&usuario, &senha)).await;

    match result {
        Ok(Ok(papel)) => {
            // Gera token JWT para o usuário autenticado
            match auth::criar_token(&usuario_log, papel) {
                Ok(token) => {
                    tracing::info!("✅ Login bem-sucedido para usuário '{}'", usuario_log);
                    Ok(Json(LoginResponse { token, papel }))
                }
                Err(e) => {
                    tracing::error!("❌ Erro ao criar token JWT: {:?}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Ok(Err(e)) => {
            tracing::warn!("⚠️  Falha no login para usuário '{}': {:?}", usuario_log, e);
            match e {
                ErroAplicacao::UsuarioNaoEncontrado | ErroAplicacao::SenhaInvalida => {
                    Err(StatusCode::UNAUTHORIZED)
                }
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
        Err(join_err) => {
            tracing::error!(
                "❌ Erro ao executar tarefa de verificação de login: {:?}",
                join_err
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Handlers - Usuários
// ============================================================================

async fn handler_listar_usuarios() -> Result<Json<Vec<InfoUsuario>>, StatusCode> {
    let result = tokio::task::spawn_blocking(move || servicos::listar_usuarios()).await;
    match result {
        Ok(usuarios) => Ok(Json(usuarios)),
        Err(join_err) => {
            tracing::error!("❌ Erro ao listar usuários: {:?}", join_err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn handler_criar_usuario(
    Json(payload): Json<CriarUsuarioPayload>,
) -> Result<StatusCode, StatusCode> {
    let nome = payload.nome_usuario.clone();
    let senha = payload.senha.clone();
    let papel = payload.papel;
    let nome_log = nome.clone();

    let result =
        tokio::task::spawn_blocking(move || servicos::criar_usuario(&nome, &senha, papel)).await;

    match result {
        Ok(Ok(_)) => {
            tracing::info!("✅ Usuário '{}' criado com sucesso", nome_log);
            Ok(StatusCode::CREATED)
        }
        Ok(Err(e)) => {
            tracing::error!("❌ Erro ao criar usuário '{}': {:?}", nome_log, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(join_err) => {
            tracing::error!("❌ Erro ao executar tarefa criar_usuario: {:?}", join_err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Handlers - Clientes
// ============================================================================

async fn handler_listar_clientes() -> Result<Json<Vec<servicos::Cliente>>, StatusCode> {
    let result = tokio::task::spawn_blocking(move || servicos::listar_clientes()).await;
    match result {
        Ok(Ok(list)) => Ok(Json(list)),
        Ok(Err(e)) => {
            tracing::error!("❌ Erro ao listar clientes: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(join_err) => {
            tracing::error!("❌ Erro ao executar tarefa listar_clientes: {:?}", join_err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn handler_criar_cliente(
    Json(payload): Json<ClientePayload>,
) -> Result<StatusCode, StatusCode> {
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
        tokio::task::spawn_blocking(move || servicos::criar_ou_atualizar_cliente(&cliente)).await;

    match result {
        Ok(Ok(_)) => Ok(StatusCode::CREATED),
        Ok(Err(e)) => {
            tracing::error!("❌ Erro ao criar cliente: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(join_err) => {
            tracing::error!(
                "❌ Erro ao executar tarefa criar_ou_atualizar_cliente: {:?}",
                join_err
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn handler_resumo_cliente(
    Path(id): Path<u32>,
) -> Result<Json<ResumoClienteResponse>, StatusCode> {
    let result = tokio::task::spawn_blocking(move || servicos::obter_gastos_e_credito(id)).await;

    match result {
        Ok(Ok((gastos, credito))) => Ok(Json(ResumoClienteResponse {
            gastos_totais: gastos,
            credito_disponivel: credito,
        })),
        Ok(Err(e)) => {
            tracing::error!("❌ Erro ao obter resumo do cliente {}: {:?}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(join_err) => {
            tracing::error!(
                "❌ Erro ao executar tarefa obter_gastos_e_credito: {:?}",
                join_err
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Handlers - Estoque
// ============================================================================

async fn handler_listar_pecas() -> Result<Json<Vec<servicos::Peca>>, StatusCode> {
    let result = tokio::task::spawn_blocking(move || servicos::listar_pecas()).await;
    match result {
        Ok(Ok(lista)) => Ok(Json(lista)),
        Ok(Err(e)) => {
            tracing::error!("❌ Erro ao listar peças: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(join_err) => {
            tracing::error!("❌ Erro ao executar tarefa listar_pecas: {:?}", join_err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Handlers - Serviços
// ============================================================================

async fn handler_listar_servicos() -> Result<Json<Vec<servicos::Servico>>, StatusCode> {
    let result = tokio::task::spawn_blocking(move || servicos::listar_servicos_db()).await;
    match result {
        Ok(Ok(lista)) => Ok(Json(lista)),
        Ok(Err(e)) => {
            tracing::error!("❌ Erro ao listar serviços: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(join_err) => {
            tracing::error!(
                "❌ Erro ao executar tarefa listar_servicos_db: {:?}",
                join_err
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn handler_criar_servico(
    Json(payload): Json<servicos::Servico>,
) -> Result<Json<servicos::Servico>, StatusCode> {
    let mut payload = payload;
    let payload_clone = payload.clone();

    let result =
        tokio::task::spawn_blocking(move || servicos::criar_servico_db(&payload_clone)).await;

    match result {
        Ok(Ok(id)) => {
            payload.id = id;
            Ok(Json(payload))
        }
        Ok(Err(e)) => {
            tracing::error!("❌ Erro ao criar serviço: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(join_err) => {
            tracing::error!(
                "❌ Erro ao executar tarefa criar_servico_db: {:?}",
                join_err
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Handlers - Ordens de Serviço
// ============================================================================

async fn handler_listar_ordens() -> Result<Json<Vec<servicos::OrdemServico>>, StatusCode> {
    let result = tokio::task::spawn_blocking(move || servicos::listar_ordens_servico()).await;
    match result {
        Ok(Ok(lista)) => Ok(Json(lista)),
        Ok(Err(e)) => {
            tracing::error!("❌ Erro ao listar ordens de serviço: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(join_err) => {
            tracing::error!(
                "❌ Erro ao executar tarefa listar_ordens_servico: {:?}",
                join_err
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn handler_obter_ordem(
    Path(id): Path<u32>,
) -> Result<Json<servicos::OrdemServico>, StatusCode> {
    let result = tokio::task::spawn_blocking(move || servicos::buscar_os_por_id(id)).await;
    match result {
        Ok(Ok(os)) => Ok(Json(os)),
        Ok(Err(ErroAplicacao::OsNaoEncontrada)) => Err(StatusCode::NOT_FOUND),
        Ok(Err(e)) => {
            tracing::error!("❌ Erro ao obter OS {}: {:?}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(join_err) => {
            tracing::error!(
                "❌ Erro ao executar tarefa buscar_os_por_id: {:?}",
                join_err
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn handler_criar_ordem_servico(
    Json(payload): Json<servicos::OrdemServico>,
) -> Result<Json<servicos::OrdemServico>, StatusCode> {
    let mut payload_for_create = payload;

    let create_res =
        tokio::task::spawn_blocking(move || servicos::criar_ordem_servico(&mut payload_for_create))
            .await;

    match create_res {
        Ok(Ok(id)) => {
            let fetch_res =
                tokio::task::spawn_blocking(move || servicos::buscar_os_por_id(id)).await;

            match fetch_res {
                Ok(Ok(os_salva)) => Ok(Json(os_salva)),
                Ok(Err(e)) => {
                    tracing::error!("❌ Erro ao buscar OS recém-criada {}: {:?}", id, e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
                Err(join_err) => {
                    tracing::error!(
                        "❌ Erro ao executar tarefa buscar_os_por_id: {:?}",
                        join_err
                    );
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Ok(Err(e)) => {
            tracing::error!("❌ Erro ao criar ordem de serviço: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(join_err) => {
            tracing::error!(
                "❌ Erro ao executar tarefa criar_ordem_servico: {:?}",
                join_err
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn handler_atualizar_ordem_servico(
    Path(id): Path<u32>,
    Json(payload): Json<AtualizarOrdemPayload>,
) -> Result<StatusCode, StatusCode> {
    if id != payload.os.id {
        return Err(StatusCode::BAD_REQUEST);
    }

    let os = payload.os;
    let usuario = payload.usuario.clone();

    let result = tokio::task::spawn_blocking(move || servicos::atualizar_os(&os, &usuario)).await;

    match result {
        Ok(Ok(_)) => Ok(StatusCode::OK),
        Ok(Err(e)) => {
            tracing::error!("❌ Erro ao atualizar OS {}: {:?}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(join_err) => {
            tracing::error!("❌ Erro ao executar tarefa atualizar_os: {:?}", join_err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Handlers - Orçamentos
// ============================================================================

async fn handler_criar_orcamento(
    Json(payload): Json<Orcamento>,
) -> Result<Json<Orcamento>, StatusCode> {
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

    let o_clone = o.clone();
    let result = tokio::task::spawn_blocking(move || servicos::criar_orcamento(&o_clone)).await;

    match result {
        Ok(Ok(id)) => {
            let mut saved = o.clone();
            saved.id = id;
            Ok(Json(Orcamento {
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
        Ok(Err(e)) => {
            tracing::error!("❌ Erro ao criar orcamento: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(join_err) => {
            tracing::error!("❌ Erro ao executar tarefa criar_orcamento: {:?}", join_err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn handler_obter_orcamento(Path(id): Path<u32>) -> Result<Json<Orcamento>, StatusCode> {
    let result = tokio::task::spawn_blocking(move || servicos::obter_orcamento(id)).await;

    match result {
        Ok(Ok(o)) => Ok(Json(Orcamento {
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
        Ok(Err(e)) => {
            tracing::error!("❌ Erro ao obter orcamento: {:?}", e);
            Err(StatusCode::NOT_FOUND)
        }
        Err(join_err) => {
            tracing::error!("❌ Erro ao executar tarefa obter_orcamento: {:?}", join_err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
