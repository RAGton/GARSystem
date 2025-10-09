// src/server.rs

mod banco_de_dados;
mod servicos;

use axum::{
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use servicos::{ErroAplicacao, InfoUsuario, PapelUsuario}; // Importar InfoUsuario
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
// não é mais necessário estado em memória para orçamentos

#[derive(Deserialize)]
struct LoginPayload {
    usuario: String,
    senha: String,
}

#[derive(Serialize)]
struct LoginResponse {
    papel: PapelUsuario,
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

#[tokio::main]
async fn main() {
    // Inicializa serviços e o pool de conexões com o banco de dados.
    servicos::inicializar();
    if let Err(e) = banco_de_dados::conexao::inicializar_pool() {
        eprintln!("Falha ao inicializar pool do banco de dados: {:?}", e);
        std::process::exit(1);
    }

    let cors = CorsLayer::new().allow_origin(Any);

    let app = Router::new()
        .route("/", get(|| async { "Servidor Senior System no ar!" }))
        .route("/login", post(handler_login))
        // [NOVO] Rota para buscar a lista de todos os usuários.
        .route("/usuarios", get(handler_listar_usuarios))
        // Clientes
        .route("/clientes", get(handler_listar_clientes))
        .route("/clientes", post(handler_criar_cliente))
        // Orçamentos (persistidos)
        .route("/orcamentos", post(handler_criar_orcamento))
        .route("/orcamentos/:id", get(handler_obter_orcamento))
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Servidor escutando em {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handler_login(
    Json(payload): Json<LoginPayload>,
) -> Result<Json<LoginResponse>, StatusCode> {
    match servicos::verificar_login(&payload.usuario, &payload.senha) {
        Ok(papel) => Ok(Json(LoginResponse { papel })),
        Err(e) => {
            eprintln!("Falha no login para usuário '{}': {:?}", payload.usuario, e);
            match e {
                ErroAplicacao::UsuarioNaoEncontrado | ErroAplicacao::SenhaInvalida => {
                    Err(StatusCode::UNAUTHORIZED)
                }
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            }
        }
    }
}

// [NOVA FUNÇÃO] Handler que responde à requisição GET /usuarios.
async fn handler_listar_usuarios() -> Result<Json<Vec<InfoUsuario>>, StatusCode> {
    // A chamada ao `servicos` aqui funciona, pois está sendo executada no servidor.
    let usuarios = servicos::listar_usuarios();
    Ok(Json(usuarios))
}

// Handler para listar clientes
async fn handler_listar_clientes() -> Result<Json<Vec<servicos::Cliente>>, StatusCode> {
    match servicos::listar_clientes() {
        Ok(list) => Ok(Json(list)),
        Err(e) => {
            eprintln!("Erro ao listar clientes: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Deserialize)]
struct ClientePayload {
    nome: String,
    email: String,
    telefone: String,
    endereco: Option<String>,
    cpf_cnpj: Option<String>,
}

// Handler para criar/atualizar cliente
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
    match servicos::criar_ou_atualizar_cliente(&cliente) {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(e) => {
            eprintln!("Erro ao criar cliente: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

use axum::extract::Path;
use axum::extract::State;

async fn handler_criar_orcamento(
    Json(payload): Json<Orcamento>,
) -> Result<Json<Orcamento>, StatusCode> {
    // Converte payload para o tipo de serviço e persiste
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
    match servicos::criar_orcamento(&o) {
        Ok(id) => {
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
        Err(e) => {
            eprintln!("Erro ao criar orcamento: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn handler_obter_orcamento(Path(id): Path<u32>) -> Result<Json<Orcamento>, StatusCode> {
    match servicos::obter_orcamento(id) {
        Ok(o) => Ok(Json(Orcamento {
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
        Err(e) => {
            eprintln!("Erro ao obter orcamento: {:?}", e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}
