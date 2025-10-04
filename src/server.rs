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
use servicos::{ErroAplicacao, PapelUsuario};
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
    servicos::inicializar();

    // [CORREÇÃO] A sintaxe correta é usar `Any` diretamente, sem `::new()`.
    let cors = CorsLayer::new().allow_origin(Any);

    let app = Router::new()
        .route("/login", post(handler_login))
        .route("/", get(|| async { "Servidor Senior System no ar!" }))
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
