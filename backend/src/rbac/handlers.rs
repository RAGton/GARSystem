// src/rbac/handlers.rs
//! Handlers HTTP do módulo RBAC (roles, permissions, atribuição user × role).
//!
//! Compilado apenas pelo binário `senior-system-server` (declarado em
//! `server.rs` via `#[path]`).
//!
//! ## Segurança
//!
//! - `roles` e `permissions` são catálogos **globais** (sistema-wide), sem
//!   `tenant_id` — por isso essas duas leituras não filtram por tenant.
//! - A atribuição `user_roles` é **por empresa**: o `empresa_id` vem sempre de
//!   `auth::tenant_do_usuario(&claims)`, nunca do path/body. Um admin do tenant
//!   A não consegue conceder role no tenant B.
//! - Escrita passa por `rbac::service::{atribuir,remover}_role`, que invalida o
//!   cache de permissões do usuário afetado.

use axum::{extract::Path, http::StatusCode, response::Json as AxJson};

use crate::auth::{self, Claims};
use crate::check_perm;
use crate::rbac::models::{ContextoRbac, Permission, Role, UserRole};
use crate::rbac::{repository, service};
use crate::servicos::ErroAplicacao;
use crate::{erro_padrao, map_erro, ErroApi};

type Resposta<T> = Result<AxJson<T>, (StatusCode, AxJson<ErroApi>)>;
type RespostaStatus = Result<StatusCode, (StatusCode, AxJson<ErroApi>)>;

fn contexto(claims: &Claims) -> ContextoRbac {
    ContextoRbac {
        usuario_id: if claims.uid > 0 {
            Some(claims.uid as u32)
        } else {
            None
        },
        username: Some(claims.sub.clone()),
        ip: None,
        user_agent: None,
        request_id: None,
        tenant_id: auth::tenant_do_usuario(claims),
    }
}

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

/// Resolve o código de uma role pelo id numérico (o path usa id).
fn codigo_da_role(role_id: i32) -> Result<String, ErroAplicacao> {
    repository::listar_roles()?
        .into_iter()
        .find(|r| r.id == role_id)
        .map(|r| r.codigo)
        .ok_or(ErroAplicacao::UsuarioNaoEncontrado)
}

// =============================================================================
// Catálogos
// =============================================================================

/// `GET /rbac/roles` — perm `empresa.usuario.list`.
pub async fn listar_roles(claims: Claims) -> Resposta<Vec<Role>> {
    check_perm(&claims, "empresa.usuario.list")?;
    executar(repository::listar_roles, "Erro ao listar roles").await
}

/// `GET /rbac/permissions` — perm `empresa.usuario.list`.
pub async fn listar_permissions(claims: Claims) -> Resposta<Vec<Permission>> {
    check_perm(&claims, "empresa.usuario.list")?;
    executar(repository::listar_permissions, "Erro ao listar permissions").await
}

// =============================================================================
// Atribuição user × role
// =============================================================================

/// `GET /rbac/users/{user_id}/roles` — perm `empresa.usuario.list`.
///
/// Escopo: apenas as roles do usuário **no tenant do chamador**.
pub async fn listar_roles_usuario(
    claims: Claims,
    Path(user_id): Path<i32>,
) -> Resposta<Vec<UserRole>> {
    check_perm(&claims, "empresa.usuario.list")?;
    let empresa = auth::tenant_do_usuario(&claims);
    executar(
        move || repository::listar_roles_usuario(user_id, empresa),
        "Erro ao listar roles do usuário",
    )
    .await
}

/// `POST /rbac/users/{user_id}/roles/{role_id}` — perm `empresa.usuario.assign_role`.
///
/// Idempotente: se o vínculo já existir, responde 204 igual.
pub async fn atribuir_role(
    claims: Claims,
    Path((user_id, role_id)): Path<(i32, i32)>,
) -> RespostaStatus {
    check_perm(&claims, "empresa.usuario.assign_role")?;
    let empresa = auth::tenant_do_usuario(&claims);
    let ctx = contexto(&claims);
    let resultado = tokio::task::spawn_blocking(move || {
        let codigo = codigo_da_role(role_id)?;
        service::atribuir_role(user_id, &codigo, empresa, &ctx)
    })
    .await;
    match resultado {
        Ok(Ok(_)) => Ok(StatusCode::NO_CONTENT),
        Ok(Err(ErroAplicacao::UsuarioNaoEncontrado)) => {
            Err(erro_padrao("NOT_FOUND", "Role não encontrada", None))
        }
        Ok(Err(e)) => Err(map_erro(e, None)),
        Err(_) => Err(erro_padrao("INTERNAL", "Erro ao atribuir role", None)),
    }
}

/// `DELETE /rbac/users/{user_id}/roles/{role_id}` — perm `empresa.usuario.assign_role`.
pub async fn remover_role(
    claims: Claims,
    Path((user_id, role_id)): Path<(i32, i32)>,
) -> RespostaStatus {
    check_perm(&claims, "empresa.usuario.assign_role")?;
    let empresa = auth::tenant_do_usuario(&claims);
    let ctx = contexto(&claims);
    let resultado = tokio::task::spawn_blocking(move || {
        let codigo = codigo_da_role(role_id)?;
        service::remover_role(user_id, &codigo, empresa, &ctx)
    })
    .await;
    match resultado {
        Ok(Ok(true)) => Ok(StatusCode::NO_CONTENT),
        Ok(Ok(false)) => Err(erro_padrao("NOT_FOUND", "Vínculo não encontrado", None)),
        Ok(Err(ErroAplicacao::UsuarioNaoEncontrado)) => {
            Err(erro_padrao("NOT_FOUND", "Role não encontrada", None))
        }
        Ok(Err(e)) => Err(map_erro(e, None)),
        Err(_) => Err(erro_padrao("INTERNAL", "Erro ao remover role", None)),
    }
}
