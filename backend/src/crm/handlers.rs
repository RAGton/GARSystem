// src/crm/handlers.rs
//! Handlers HTTP do módulo CRM.
//!
//! Este arquivo é compilado **apenas** pelo binário `senior-system-server`
//! (declarado em `server.rs` via `#[path]`), porque depende de
//! `crate::auth::Claims` e dos helpers de resposta do servidor.
//!
//! ## Contrato comum
//!
//! - Toda rota exige `Claims` (JWT válido via header `Bearer` ou cookie
//!   `token`) e passa por `check_perm(&claims, "<permissao>")`.
//! - O `tenant_id` **sempre** vem de `auth::tenant_do_usuario(&claims)`, nunca
//!   do body/query. Isso impede que um usuário leia dados de outro tenant.
//! - Trabalho de banco roda em `spawn_blocking` (driver `mysql` é síncrono).

use axum::{
    extract::{Json, Path, Query},
    http::StatusCode,
    response::Json as AxJson,
};
use serde::Deserialize;

use crate::auth::{self, Claims};
use crate::crm::models::{Contato, DashboardCliente, Equipamento, Observacao, Tag, TimelineEvento};
use crate::crm::service::{self, Contexto};
use crate::servicos::ErroAplicacao;
use crate::{check_perm, erro_padrao, map_erro, ErroApi};

type Resposta<T> = Result<AxJson<T>, (StatusCode, AxJson<ErroApi>)>;
type RespostaStatus = Result<StatusCode, (StatusCode, AxJson<ErroApi>)>;

/// Monta o contexto de auditoria/timeline a partir das claims.
fn contexto(claims: &Claims) -> Contexto {
    Contexto {
        usuario_id: if claims.uid > 0 {
            Some(claims.uid as u32)
        } else {
            None
        },
        username: Some(claims.sub.clone()),
        ip: None,
        user_agent: None,
        request_id: None,
    }
}

/// Normaliza um limite vindo da query: 0/ausente → default, cap em 200.
fn limite_sanitizado(bruto: Option<u32>, default: u32) -> u32 {
    match bruto {
        Some(0) | None => default,
        Some(n) if n > 200 => 200,
        Some(n) => n,
    }
}

/// Encapsula o padrão `spawn_blocking` + tradução de erro.
async fn executar<T, F>(f: F, contexto_erro: &'static str) -> Resposta<T>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, ErroAplicacao> + Send + 'static,
{
    match tokio::task::spawn_blocking(f).await {
        Ok(Ok(v)) => Ok(AxJson(v)),
        Ok(Err(e)) => Err(map_erro(e, None)),
        Err(_) => Err(erro_padrao("INTERNAL", contexto_erro, None)),
    }
}

// =============================================================================
// Query/body DTOs
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct LimiteQuery {
    pub limite: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct BuscaQuery {
    pub q: Option<String>,
    pub limite: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct ClienteIdQuery {
    pub cliente_id: u32,
}

#[derive(Debug, Deserialize)]
pub struct ObservacaoPayload {
    pub conteudo: String,
}

#[derive(Debug, Deserialize)]
pub struct ContatoPayload {
    pub tipo: String,
    pub valor: String,
    pub rotulo: Option<String>,
    #[serde(default)]
    pub principal: bool,
    pub observacao: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EquipamentoPayload {
    pub descricao: String,
    pub marca: Option<String>,
    pub modelo: Option<String>,
    pub numero_serie: Option<String>,
    pub patrimonio: Option<String>,
    pub observacao: Option<String>,
    pub tipo: Option<String>,
}

// =============================================================================
// Dashboard / timeline / busca
// =============================================================================

/// `GET /crm/clientes/{id}/dashboard` — perm `crm.cliente.view`.
pub async fn dashboard_cliente(claims: Claims, Path(id): Path<u32>) -> Resposta<DashboardCliente> {
    check_perm(&claims, "crm.cliente.view")?;
    let tenant = auth::tenant_do_usuario(&claims);
    executar(
        move || service::dashboard_cliente(tenant, id),
        "Erro ao montar dashboard do cliente",
    )
    .await
}

/// `GET /crm/clientes/{id}/timeline?limite=50` — perm `crm.cliente.view`.
pub async fn listar_timeline(
    claims: Claims,
    Path(id): Path<u32>,
    Query(q): Query<LimiteQuery>,
) -> Resposta<Vec<TimelineEvento>> {
    check_perm(&claims, "crm.cliente.view")?;
    let tenant = auth::tenant_do_usuario(&claims);
    let limite = limite_sanitizado(q.limite, 50);
    executar(
        move || service::listar_timeline(tenant, id, limite),
        "Erro ao listar timeline",
    )
    .await
}

/// `GET /crm/buscar?q=...&limite=20` — perm `crm.cliente.view`.
pub async fn buscar_clientes(
    claims: Claims,
    Query(q): Query<BuscaQuery>,
) -> Resposta<Vec<serde_json::Value>> {
    check_perm(&claims, "crm.cliente.view")?;
    let tenant = auth::tenant_do_usuario(&claims);
    let termo = q.q.unwrap_or_default();
    let limite = limite_sanitizado(q.limite, 20);
    executar(
        move || service::buscar_clientes(tenant, &termo, limite),
        "Erro ao buscar clientes",
    )
    .await
}

// =============================================================================
// Observações
// =============================================================================

/// `GET /crm/clientes/{id}/observacoes?limite=20` — perm `crm.cliente.view`.
pub async fn listar_observacoes(
    claims: Claims,
    Path(id): Path<u32>,
    Query(q): Query<LimiteQuery>,
) -> Resposta<Vec<Observacao>> {
    check_perm(&claims, "crm.cliente.view")?;
    let tenant = auth::tenant_do_usuario(&claims);
    let limite = limite_sanitizado(q.limite, 20);
    executar(
        move || service::listar_observacoes(tenant, id, limite),
        "Erro ao listar observações",
    )
    .await
}

/// `POST /crm/clientes/{id}/observacoes` — perm `crm.cliente.create`.
pub async fn adicionar_observacao(
    claims: Claims,
    Path(id): Path<u32>,
    Json(payload): Json<ObservacaoPayload>,
) -> Resposta<Observacao> {
    check_perm(&claims, "crm.cliente.create")?;
    if payload.conteudo.trim().is_empty() {
        return Err(erro_padrao("VALIDATION", "conteudo é obrigatório", None));
    }
    let tenant = auth::tenant_do_usuario(&claims);
    let ctx = contexto(&claims);
    executar(
        move || service::adicionar_observacao(tenant, id, &payload.conteudo, &ctx),
        "Erro ao adicionar observação",
    )
    .await
}

/// `PUT /crm/observacoes/{id}` — perm `crm.cliente.edit`.
pub async fn editar_observacao(
    claims: Claims,
    Path(id): Path<u32>,
    Json(payload): Json<ObservacaoPayload>,
) -> Resposta<Observacao> {
    check_perm(&claims, "crm.cliente.edit")?;
    if payload.conteudo.trim().is_empty() {
        return Err(erro_padrao("VALIDATION", "conteudo é obrigatório", None));
    }
    let tenant = auth::tenant_do_usuario(&claims);
    let ctx = contexto(&claims);
    executar(
        move || service::editar_observacao(tenant, id, &payload.conteudo, &ctx),
        "Erro ao editar observação",
    )
    .await
}

// =============================================================================
// Tags
// =============================================================================

/// `GET /crm/tags` — perm `crm.tag.view`.
pub async fn listar_tags(claims: Claims) -> Resposta<Vec<Tag>> {
    check_perm(&claims, "crm.tag.view")?;
    let tenant = auth::tenant_do_usuario(&claims);
    executar(move || service::listar_tags(tenant), "Erro ao listar tags").await
}

/// `POST /crm/clientes/{id}/tags/{tag_id}` — perm `crm.cliente.edit`.
pub async fn atribuir_tag(claims: Claims, Path((id, tag_id)): Path<(u32, u32)>) -> RespostaStatus {
    check_perm(&claims, "crm.cliente.edit")?;
    let tenant = auth::tenant_do_usuario(&claims);
    let ctx = contexto(&claims);
    match tokio::task::spawn_blocking(move || service::atribuir_tag(tenant, id, tag_id, &ctx)).await
    {
        Ok(Ok(())) => Ok(StatusCode::NO_CONTENT),
        Ok(Err(e)) => Err(map_erro(e, None)),
        Err(_) => Err(erro_padrao("INTERNAL", "Erro ao atribuir tag", None)),
    }
}

/// `DELETE /crm/clientes/{id}/tags/{tag_id}` — perm `crm.cliente.edit`.
pub async fn remover_tag(claims: Claims, Path((id, tag_id)): Path<(u32, u32)>) -> RespostaStatus {
    check_perm(&claims, "crm.cliente.edit")?;
    let tenant = auth::tenant_do_usuario(&claims);
    let ctx = contexto(&claims);
    match tokio::task::spawn_blocking(move || service::remover_tag(tenant, id, tag_id, &ctx)).await {
        Ok(Ok(true)) => Ok(StatusCode::NO_CONTENT),
        Ok(Ok(false)) => Err(erro_padrao("NOT_FOUND", "Tag não vinculada", None)),
        Ok(Err(e)) => Err(map_erro(e, None)),
        Err(_) => Err(erro_padrao("INTERNAL", "Erro ao remover tag", None)),
    }
}

// =============================================================================
// Contatos
// =============================================================================

/// `GET /crm/clientes/{id}/contatos` — perm `crm.cliente.view`.
pub async fn listar_contatos(claims: Claims, Path(id): Path<u32>) -> Resposta<Vec<Contato>> {
    check_perm(&claims, "crm.cliente.view")?;
    let tenant = auth::tenant_do_usuario(&claims);
    executar(
        move || service::listar_contatos(tenant, id),
        "Erro ao listar contatos",
    )
    .await
}

/// `POST /crm/clientes/{id}/contatos` — perm `crm.cliente.create`.
pub async fn adicionar_contato(
    claims: Claims,
    Path(id): Path<u32>,
    Json(p): Json<ContatoPayload>,
) -> Resposta<Contato> {
    check_perm(&claims, "crm.cliente.create")?;
    if p.valor.trim().is_empty() {
        return Err(erro_padrao("VALIDATION", "valor é obrigatório", None));
    }
    let tenant = auth::tenant_do_usuario(&claims);
    let ctx = contexto(&claims);
    executar(
        move || {
            service::adicionar_contato(
                tenant,
                id,
                &p.tipo,
                &p.valor,
                p.rotulo.as_deref(),
                p.principal,
                p.observacao.as_deref(),
                &ctx,
            )
        },
        "Erro ao adicionar contato",
    )
    .await
}

/// `DELETE /crm/contatos/{id}?cliente_id=N` — perm `crm.cliente.edit`.
///
/// `cliente_id` é obrigatório porque a timeline é escrita no cliente dono do
/// contato; o `DELETE` em si já é filtrado por tenant no repositório.
pub async fn remover_contato(
    claims: Claims,
    Path(id): Path<u32>,
    Query(q): Query<ClienteIdQuery>,
) -> RespostaStatus {
    check_perm(&claims, "crm.cliente.edit")?;
    let tenant = auth::tenant_do_usuario(&claims);
    let ctx = contexto(&claims);
    match tokio::task::spawn_blocking(move || {
        service::remover_contato(tenant, id, q.cliente_id, &ctx)
    })
    .await
    {
        Ok(Ok(true)) => Ok(StatusCode::NO_CONTENT),
        Ok(Ok(false)) => Err(erro_padrao("NOT_FOUND", "Contato não encontrado", None)),
        Ok(Err(e)) => Err(map_erro(e, None)),
        Err(_) => Err(erro_padrao("INTERNAL", "Erro ao remover contato", None)),
    }
}

// =============================================================================
// Equipamentos
// =============================================================================

/// `GET /crm/clientes/{id}/equipamentos` — perm `crm.cliente.view`.
pub async fn listar_equipamentos(
    claims: Claims,
    Path(id): Path<u32>,
) -> Resposta<Vec<Equipamento>> {
    check_perm(&claims, "crm.cliente.view")?;
    let tenant = auth::tenant_do_usuario(&claims);
    executar(
        move || service::listar_equipamentos(tenant, id),
        "Erro ao listar equipamentos",
    )
    .await
}

/// `POST /crm/clientes/{id}/equipamentos` — perm `crm.cliente.create`.
pub async fn adicionar_equipamento(
    claims: Claims,
    Path(id): Path<u32>,
    Json(p): Json<EquipamentoPayload>,
) -> Resposta<Equipamento> {
    check_perm(&claims, "crm.cliente.create")?;
    if p.descricao.trim().is_empty() {
        return Err(erro_padrao("VALIDATION", "descricao é obrigatória", None));
    }
    let tenant = auth::tenant_do_usuario(&claims);
    let ctx = contexto(&claims);
    let tipo = p.tipo.clone().unwrap_or_else(|| "OUTRO".to_string());
    executar(
        move || {
            service::adicionar_equipamento(
                tenant,
                id,
                &p.descricao,
                p.marca.as_deref(),
                p.modelo.as_deref(),
                p.numero_serie.as_deref(),
                p.patrimonio.as_deref(),
                p.observacao.as_deref(),
                &tipo,
                &ctx,
            )
        },
        "Erro ao adicionar equipamento",
    )
    .await
}

/// `DELETE /crm/equipamentos/{id}?cliente_id=N` — perm `crm.cliente.edit`.
pub async fn remover_equipamento(
    claims: Claims,
    Path(id): Path<u32>,
    Query(q): Query<ClienteIdQuery>,
) -> RespostaStatus {
    check_perm(&claims, "crm.cliente.edit")?;
    let tenant = auth::tenant_do_usuario(&claims);
    let ctx = contexto(&claims);
    match tokio::task::spawn_blocking(move || {
        service::remover_equipamento(tenant, id, q.cliente_id, &ctx)
    })
    .await
    {
        Ok(Ok(true)) => Ok(StatusCode::NO_CONTENT),
        Ok(Ok(false)) => Err(erro_padrao("NOT_FOUND", "Equipamento não encontrado", None)),
        Ok(Err(e)) => Err(map_erro(e, None)),
        Err(_) => Err(erro_padrao("INTERNAL", "Erro ao remover equipamento", None)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limite_zero_vira_default() {
        assert_eq!(limite_sanitizado(Some(0), 50), 50);
        assert_eq!(limite_sanitizado(None, 20), 20);
    }

    #[test]
    fn limite_e_capado_em_200() {
        assert_eq!(limite_sanitizado(Some(9999), 50), 200);
        assert_eq!(limite_sanitizado(Some(30), 50), 30);
    }
}
