// src/gui_services/mod.rs
//
// Camada de serviço para a GUI.
//
// As telas NÃO devem chamar `http_client` diretamente. Devem passar por
// estas funções, que:
//   * adicionam o `Authorization: Bearer ...` automaticamente
//   * padronizam o tratamento de erro (ErroHttp → mensagem amigável)
//   * centralizam o ponto de evolução para retry/cache/telemetria
//
// Cada função recebe:
//   - `base`: BaseHandle (host:porta do servidor)
//   - `token`: Arc<Mutex<Option<String>>> (token JWT atual)
//
// E retorna Result<T, ErroServico> com mensagem pronta pra UI.

use crate::http_client::{self, ErroHttp};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::{Arc, Mutex};

/// Estado de autenticação compartilhado entre todas as telas.
/// `None` significa que o usuário não está logado.
pub type TokenArc = Arc<Mutex<Option<String>>>;

/// Erro de serviço — pronto para exibir na UI.
#[derive(Debug, Clone)]
pub struct ErroServico {
    pub mensagem: String,
    pub http_status: Option<u16>,
}

impl std::fmt::Display for ErroServico {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.mensagem)
    }
}

impl std::error::Error for ErroServico {}

impl From<ErroHttp> for ErroServico {
    fn from(e: ErroHttp) -> Self {
        match e {
            ErroHttp::Rede(e) => ErroServico {
                mensagem: format!("Erro de rede: {}", e),
                http_status: None,
            },
            ErroHttp::Serializacao(e) => ErroServico {
                mensagem: format!("Resposta inválida do servidor: {}", e),
                http_status: None,
            },
            ErroHttp::Api { status, body } => {
                let mensagem = extrair_mensagem_erro(&body).unwrap_or_else(|| match status {
                    401 => "Não autorizado. Faça login novamente.".to_string(),
                    403 => "Permissão insuficiente.".to_string(),
                    404 => "Recurso não encontrado.".to_string(),
                    409 => "Conflito — registro já existe.".to_string(),
                    422 => "Dados inválidos.".to_string(),
                    429 => "Muitas requisições. Aguarde um momento.".to_string(),
                    500..=599 => "Erro interno do servidor.".to_string(),
                    _ => format!("HTTP {}", status),
                });
                ErroServico {
                    mensagem,
                    http_status: Some(status),
                }
            }
        }
    }
}

fn extrair_mensagem_erro(body: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(body).ok()?;
    v.get("error")
        .and_then(|e| e.get("message"))
        .and_then(|m| m.as_str())
        .map(|s| s.to_string())
}

fn ler_token(token: &TokenArc) -> Result<String, ErroServico> {
    token
        .lock()
        .ok()
        .and_then(|g| g.clone())
        .ok_or_else(|| ErroServico {
            mensagem: "Sessão expirada. Faça login novamente.".to_string(),
            http_status: Some(401),
        })
}

/// Handle para a URL base do servidor. Compartilhado entre as telas.
#[derive(Clone)]
pub struct BaseHandle {
    pub base: Arc<Mutex<String>>,
}

// =============================================================================
// Paginação
// =============================================================================

/// Resposta paginada genérica. Reflete a struct do backend.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Pagina<T> {
    pub items: Vec<T>,
    pub page: u32,
    pub limit: u32,
    pub total: u64,
    pub total_paginas: u32,
}

impl<T> Pagina<T> {
    pub fn vazia() -> Self {
        Self {
            items: Vec::new(),
            page: 1,
            limit: 50,
            total: 0,
            total_paginas: 0,
        }
    }
}

impl BaseHandle {
    pub fn new(base: Arc<Mutex<String>>) -> Self {
        Self { base }
    }

    pub fn url(&self, path: &str) -> String {
        let b = self.base.lock().map(|g| g.clone()).unwrap_or_default();
        if path.starts_with('/') {
            format!("{}{}", b, path)
        } else {
            format!("{}/{}", b, path)
        }
    }
}

// =============================================================================
// USUÁRIOS
// =============================================================================

pub fn listar_usuarios<T: DeserializeOwned>(
    base: &BaseHandle,
    token: &TokenArc,
) -> Result<T, ErroServico> {
    http_client::get_autenticado(&base.url("/usuarios"), Some(&ler_token(token)?))
        .map_err(Into::into)
}

pub fn criar_usuario<B: Serialize>(
    base: &BaseHandle,
    token: &TokenArc,
    body: &B,
) -> Result<(), ErroServico> {
    let _resp: serde_json::Value =
        http_client::post_autenticado(&base.url("/usuarios"), body, Some(&ler_token(token)?))?;
    Ok(())
}

// =============================================================================
// CLIENTES
// =============================================================================

pub fn listar_clientes<T: DeserializeOwned>(
    base: &BaseHandle,
    token: &TokenArc,
) -> Result<T, ErroServico> {
    http_client::get_autenticado(&base.url("/clientes"), Some(&ler_token(token)?))
        .map_err(Into::into)
}

/// Versão paginada — chama `/clientes?page=N&limit=M`.
pub fn listar_clientes_paginado(
    base: &BaseHandle,
    token: &TokenArc,
    page: u32,
    limit: u32,
) -> Result<Pagina<serde_json::Value>, ErroServico> {
    let url = format!("/clientes?page={}&limit={}", page, limit);
    http_client::get_autenticado(&base.url(&url), Some(&ler_token(token)?)).map_err(Into::into)
}

pub fn criar_cliente<B: Serialize>(
    base: &BaseHandle,
    token: &TokenArc,
    body: &B,
) -> Result<(), ErroServico> {
    let _resp: serde_json::Value =
        http_client::post_autenticado(&base.url("/clientes"), body, Some(&ler_token(token)?))?;
    Ok(())
}

pub fn resumo_cliente<T: DeserializeOwned>(
    base: &BaseHandle,
    token: &TokenArc,
    id: u32,
) -> Result<T, ErroServico> {
    http_client::get_autenticado(
        &base.url(&format!("/clientes/{}/resumo", id)),
        Some(&ler_token(token)?),
    )
    .map_err(Into::into)
}

// =============================================================================
// ESTOQUE
// =============================================================================

pub fn listar_pecas<T: DeserializeOwned>(
    base: &BaseHandle,
    token: &TokenArc,
) -> Result<T, ErroServico> {
    http_client::get_autenticado(&base.url("/estoque/pecas"), Some(&ler_token(token)?))
        .map_err(Into::into)
}

/// Versão paginada — chama `/estoque/pecas?page=N&limit=M`.
pub fn listar_pecas_paginado(
    base: &BaseHandle,
    token: &TokenArc,
    page: u32,
    limit: u32,
) -> Result<Pagina<serde_json::Value>, ErroServico> {
    let url = format!("/estoque/pecas?page={}&limit={}", page, limit);
    http_client::get_autenticado(&base.url(&url), Some(&ler_token(token)?)).map_err(Into::into)
}

// =============================================================================
// SERVIÇOS (mão de obra)
// =============================================================================

pub fn listar_servicos<T: DeserializeOwned>(
    base: &BaseHandle,
    token: &TokenArc,
) -> Result<T, ErroServico> {
    http_client::get_autenticado(&base.url("/servicos"), Some(&ler_token(token)?))
        .map_err(Into::into)
}

pub fn criar_servico<B: Serialize>(
    base: &BaseHandle,
    token: &TokenArc,
    body: &B,
) -> Result<serde_json::Value, ErroServico> {
    http_client::post_autenticado(&base.url("/servicos"), body, Some(&ler_token(token)?))
        .map_err(Into::into)
}

// =============================================================================
// ORDENS DE SERVIÇO
// =============================================================================

pub fn listar_ordens<T: DeserializeOwned>(
    base: &BaseHandle,
    token: &TokenArc,
) -> Result<T, ErroServico> {
    http_client::get_autenticado(&base.url("/ordens"), Some(&ler_token(token)?)).map_err(Into::into)
}

/// Versão paginada — chama `/ordens?page=N&limit=M`.
pub fn listar_ordens_paginado(
    base: &BaseHandle,
    token: &TokenArc,
    page: u32,
    limit: u32,
) -> Result<Pagina<serde_json::Value>, ErroServico> {
    let url = format!("/ordens?page={}&limit={}", page, limit);
    http_client::get_autenticado(&base.url(&url), Some(&ler_token(token)?)).map_err(Into::into)
}

pub fn obter_ordem<T: DeserializeOwned>(
    base: &BaseHandle,
    token: &TokenArc,
    id: u32,
) -> Result<T, ErroServico> {
    http_client::get_autenticado(
        &base.url(&format!("/ordens/{}", id)),
        Some(&ler_token(token)?),
    )
    .map_err(Into::into)
}

pub fn criar_ordem<T: DeserializeOwned, B: Serialize>(
    base: &BaseHandle,
    token: &TokenArc,
    body: &B,
) -> Result<T, ErroServico> {
    http_client::post_autenticado(&base.url("/ordens"), body, Some(&ler_token(token)?))
        .map_err(Into::into)
}

pub fn atualizar_ordem<B: Serialize>(
    base: &BaseHandle,
    token: &TokenArc,
    id: u32,
    body: &B,
) -> Result<(), ErroServico> {
    let _resp: serde_json::Value = http_client::put_autenticado(
        &base.url(&format!("/ordens/{}", id)),
        body,
        Some(&ler_token(token)?),
    )?;
    Ok(())
}

// =============================================================================
// ORÇAMENTOS
// =============================================================================

pub fn criar_orcamento<T: DeserializeOwned, B: Serialize>(
    base: &BaseHandle,
    token: &TokenArc,
    body: &B,
) -> Result<T, ErroServico> {
    http_client::post_autenticado(&base.url("/orcamentos"), body, Some(&ler_token(token)?))
        .map_err(Into::into)
}

pub fn obter_orcamento<T: DeserializeOwned>(
    base: &BaseHandle,
    token: &TokenArc,
    id: u32,
) -> Result<T, ErroServico> {
    http_client::get_autenticado(
        &base.url(&format!("/orcamentos/{}", id)),
        Some(&ler_token(token)?),
    )
    .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn erro_http_api_para_servico() {
        let e: ErroServico = ErroHttp::Api {
            status: 401,
            body: r#"{"error":{"code":"AUTH_INVALID","message":"Credenciais inválidas"}}"#.into(),
        }
        .into();
        assert_eq!(e.mensagem, "Credenciais inválidas");
        assert_eq!(e.http_status, Some(401));
    }

    #[test]
    fn erro_api_sem_json_e_amigavel() {
        let e: ErroServico = ErroHttp::Api {
            status: 500,
            body: "Internal Server Error".into(),
        }
        .into();
        assert_eq!(e.mensagem, "Erro interno do servidor.");
        assert_eq!(e.http_status, Some(500));
    }

    #[test]
    fn base_handle_url() {
        let base = Arc::new(Mutex::new("http://localhost:3000".to_string()));
        let h = BaseHandle::new(base);
        assert_eq!(h.url("/usuarios"), "http://localhost:3000/usuarios");
        assert_eq!(h.url("usuarios"), "http://localhost:3000/usuarios");
    }
}
