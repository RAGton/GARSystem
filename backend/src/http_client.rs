// src/http_client.rs
//
// Singleton de cliente HTTP + helpers para envio de `Authorization: Bearer ...`
// em todas as requests autenticadas.
//
// As telas devem usar `get_autenticado`, `post_autenticado`, etc., passando o
// token JWT obtido no login. O token fica armazenado no `eframe::Storage`
// (criptografia no SO é desejável mas não implementada ainda — ver P2 #29).

use once_cell::sync::Lazy;
use reqwest::blocking::{Client, RequestBuilder};
use serde::de::DeserializeOwned;
use std::time::Duration;

/// Singleton reqwest blocking client reutilizável por toda a aplicação GUI.
static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(8))
        .build()
        .expect("Falha ao criar Client HTTP global")
});

pub fn get_client() -> &'static Client {
    &HTTP_CLIENT
}

/// Chave usada para armazenar o token JWT no `eframe::Storage`.
pub const STORAGE_KEY_TOKEN: &str = "jwt_token";
/// Chave para armazenar o papel do usuário logado.
pub const STORAGE_KEY_PAPEL: &str = "papel_usuario";

/// Aplica o header `Authorization: Bearer <token>` se o token não for vazio.
/// Loga warning se for chamado sem token (provavelmente bug na tela).
fn apply_auth(builder: RequestBuilder, token: Option<&str>) -> RequestBuilder {
    match token {
        Some(t) if !t.is_empty() => builder.bearer_auth(t),
        _ => {
            tracing::warn!(
                "http_client: request feita SEM token. \
                 Provável falta de login ativo. A requisição será 401."
            );
            builder
        }
    }
}

/// GET autenticado.
pub fn get_autenticado<T: DeserializeOwned>(url: &str, token: Option<&str>) -> Result<T, ErroHttp> {
    let req = apply_auth(get_client().get(url), token);
    let resp = req.send().map_err(ErroHttp::Rede)?;
    parse(resp)
}

/// POST autenticado com body JSON.
pub fn post_autenticado<T: DeserializeOwned, B: serde::Serialize>(
    url: &str,
    body: &B,
    token: Option<&str>,
) -> Result<T, ErroHttp> {
    let req = apply_auth(get_client().post(url).json(body), token);
    let resp = req.send().map_err(ErroHttp::Rede)?;
    parse(resp)
}

/// PUT autenticado com body JSON.
pub fn put_autenticado<T: DeserializeOwned, B: serde::Serialize>(
    url: &str,
    body: &B,
    token: Option<&str>,
) -> Result<T, ErroHttp> {
    let req = apply_auth(get_client().put(url).json(body), token);
    let resp = req.send().map_err(ErroHttp::Rede)?;
    parse(resp)
}

fn parse<T: DeserializeOwned>(resp: reqwest::blocking::Response) -> Result<T, ErroHttp> {
    let status = resp.status();
    if status.is_success() {
        resp.json::<T>().map_err(ErroHttp::Serializacao)
    } else {
        // Tenta extrair a mensagem do corpo de erro padronizado do backend.
        let body = resp.text().unwrap_or_default();
        Err(ErroHttp::Api {
            status: status.as_u16(),
            body,
        })
    }
}

/// Erro genérico das chamadas HTTP da GUI.
#[derive(Debug)]
pub enum ErroHttp {
    /// Erro de rede / timeout / DNS.
    Rede(reqwest::Error),
    /// Erro de deserialização da resposta (resposta inesperada).
    Serializacao(reqwest::Error),
    /// Resposta de erro do backend (4xx/5xx) com corpo.
    Api { status: u16, body: String },
}

impl std::fmt::Display for ErroHttp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErroHttp::Rede(e) => write!(f, "Erro de rede: {}", e),
            ErroHttp::Serializacao(e) => write!(f, "Resposta inválida: {}", e),
            ErroHttp::Api { status, body } => write!(f, "HTTP {}: {}", status, body),
        }
    }
}

impl std::error::Error for ErroHttp {}
