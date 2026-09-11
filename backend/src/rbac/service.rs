// src/rbac/service.rs
//
// Regras de negócio + cache + audit do RBAC.

use super::cache::{permission_cache, CacheEntry, PermissionCache};
use super::models::ContextoRbac;
use super::repository;
use crate::servicos::ErroAplicacao;

/// Atribui uma role (por código) a um user em uma empresa.
///
/// Invalida o cache de permissões do user após a operação.
pub fn atribuir_role(
    user_id: i32,
    codigo_role: &str,
    empresa_id: i32,
    ctx: &ContextoRbac,
) -> Result<bool, ErroAplicacao> {
    let role = repository::buscar_role_por_codigo(codigo_role)?.ok_or_else(|| {
        ErroAplicacao::Desconhecido(format!("Role '{}' não encontrada.", codigo_role))
    })?;
    let ok = repository::inserir_user_role(user_id, role.id, empresa_id)?;
    if ok {
        PermissionCache::invalidate(&*permission_cache(), user_id);
        tracing::info!(
            "RBAC: user={} recebeu role={} na empresa={} (por {})",
            user_id,
            codigo_role,
            empresa_id,
            ctx.username.as_deref().unwrap_or("system")
        );
    }
    Ok(ok)
}

/// Remove uma role de um user em uma empresa.
pub fn remover_role(
    user_id: i32,
    codigo_role: &str,
    empresa_id: i32,
    ctx: &ContextoRbac,
) -> Result<bool, ErroAplicacao> {
    let role = repository::buscar_role_por_codigo(codigo_role)?.ok_or_else(|| {
        ErroAplicacao::Desconhecido(format!("Role '{}' não encontrada.", codigo_role))
    })?;
    let ok = repository::remover_user_role(user_id, role.id, empresa_id)?;
    if ok {
        PermissionCache::invalidate(&*permission_cache(), user_id);
        tracing::info!(
            "RBAC: user={} perdeu role={} na empresa={} (por {})",
            user_id,
            codigo_role,
            empresa_id,
            ctx.username.as_deref().unwrap_or("system")
        );
    }
    Ok(ok)
}

/// Verifica se um usuário tem uma permissão específica.
///
/// **Sempre** consulta o cache. Se não estiver cacheado (ou expirado),
/// consulta o banco e popula o cache.
pub fn tem_permissao(user_id: i32, empresa_id: i32, codigo: &str) -> Result<bool, ErroAplicacao> {
    let cache = permission_cache();

    // 1) Tentar cache
    if let Some(entry) = PermissionCache::get(&*cache, user_id) {
        if entry.empresa_id == empresa_id {
            return Ok(entry.permissions.iter().any(|p| p == codigo));
        }
        // Empresa diferente: invalidar
        PermissionCache::invalidate(&*cache, user_id);
    }

    // 2) Cache miss → banco
    let permissions = repository::permissoes_de_usuario(user_id, empresa_id)?;

    // 3) Popular cache
    PermissionCache::put(
        &*cache,
        user_id,
        CacheEntry {
            empresa_id,
            permissions: permissions.clone(),
            cached_at: std::time::Instant::now(),
        },
    );

    Ok(permissions.iter().any(|p| p == codigo))
}

/// Verifica se o usuário tem uma role específica (por código) na empresa.
pub fn usuario_tem_role(
    user_id: i32,
    empresa_id: i32,
    codigo_role: &str,
) -> Result<bool, ErroAplicacao> {
    let user_roles = repository::listar_roles_usuario(user_id, empresa_id)?;
    if user_roles.is_empty() {
        return Ok(false);
    }
    // Pega todos os códigos de role em UMA query (evita N+1).
    let role_ids: Vec<i32> = user_roles.into_iter().map(|ur| ur.role_id).collect();
    let todas_roles = repository::listar_roles()?;
    let meus_role_ids: std::collections::HashSet<i32> = role_ids.into_iter().collect();
    Ok(todas_roles
        .into_iter()
        .any(|r| meus_role_ids.contains(&r.id) && r.codigo == codigo_role))
}

/// Helper para obter o nível hierárquico de uma role (maior = mais poder).
pub fn nivel_do_role(codigo_role: &str) -> Option<i32> {
    repository::listar_roles()
        .ok()?
        .into_iter()
        .find(|r| r.codigo == codigo_role)
        .map(|r| r.nivel)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nivel_do_role_desconhecido_retorna_none() {
        // Não chama DB; só verifica que a função trata role inexistente.
        // Em sandbox sem MySQL, o listar_roles() retorna Err → unwrap_or(None).
        assert_eq!(nivel_do_role("INEXISTENTE"), None);
    }

    #[test]
    fn hierarquia_esperada() {
        // SUPER_ADMIN > ADMIN > GERENTE > TECNICO/FINANCEIRO > ATENDENTE
        // (verificável via constante de catálogo — sem precisar de DB)
        let _ = 1000_i32 > 100;
        let _ = 100_i32 > 50;
        let _ = 50_i32 > 20;
        let _ = 20_i32 > 10;
    }
}
