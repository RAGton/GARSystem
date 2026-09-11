// src/rbac/mod.rs
//
// Módulo RBAC — Sprint P2.6.1.
//
// Implementa roles, permissions, user_roles (escopo multi-tenant) e cache
// em memória de permissões por usuário (preparado para Redis no futuro).
//
// Hierarquia:
//   - SUPER_ADMIN:  nível 1000, bypassa tenant_id
//   - ADMIN:        nível 100, tudo dentro da empresa
//   - GERENTE:      nível 50, dashboards + relatórios
//   - TECNICO:      nível 20, execução de OS
//   - FINANCEIRO:   nível 20, módulo financeiro
//   - ATENDENTE:    nível 10, cadastro básico
//
// Permission cache:
//   - In-memory (Arc<RwLock<HashMap<user_id, (Vec<String>, Instant)>>>)
//   - TTL configurável (default 5 minutos)
//   - Invalidation no logout e em mudanças de role
//
// Refs:
//   - db-init/migrations/0011_empresa_rbac.sql

pub mod cache;
pub mod models;
pub mod repository;
pub mod service;

// Re-exports
pub use cache::{invalidar_cache, permission_cache, PermissionCache};
pub use models::{ContextoRbac, Permission, Role, UserRole};
pub use repository::{
    buscar_role_por_codigo, listar_permissions, listar_roles, listar_roles_usuario,
    listar_usuarios_por_role, permissoes_de_role, permissoes_de_usuario,
};
pub use service::{atribuir_role, nivel_do_role, remover_role, tem_permissao, usuario_tem_role};
