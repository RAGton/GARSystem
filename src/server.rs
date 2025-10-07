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

#[derive(Deserialize)]
struct LoginPayload {
    usuario: String,
    senha: String,
}

#[derive(Serialize)]
struct LoginResponse {
    papel: PapelUsuario,
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
