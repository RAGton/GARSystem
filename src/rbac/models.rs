// src/rbac/models.rs

use serde::{Deserialize, Serialize};

/// Role (papel) — sistema-wide, catálogo imutável.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: i32,
    pub codigo: String,
    pub nome: String,
    pub descricao: String,
    pub nivel: i32,
}

/// Permission (permissão granular) — sistema-wide.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub id: i32,
    pub codigo: String,
    pub descricao: String,
    pub categoria: String,
}

/// Atribuição user × role × empresa. **Escopo multi-tenant**.
///
/// Mesmo usuário pode ser ADMIN na empresa A e TECNICO na empresa B.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRole {
    pub id: i32,
    pub user_id: i32,
    pub role_id: i32,
    pub empresa_id: i32,
}

/// Contexto de execução para o service layer de RBAC.
#[derive(Debug, Clone)]
pub struct ContextoRbac {
    pub usuario_id: Option<u32>,
    pub username: Option<String>,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<String>,
    /// Empresa ativa no contexto. 0 = SUPER_ADMIN cross-tenant.
    pub tenant_id: i32,
}

impl ContextoRbac {
    pub fn system() -> Self {
        Self {
            usuario_id: None,
            username: Some("system".into()),
            ip: None,
            user_agent: None,
            request_id: None,
            tenant_id: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_serialize_inclui_codigo() {
        let r = Role {
            id: 1,
            codigo: "ADMIN".into(),
            nome: "Administrador".into(),
            descricao: "X".into(),
            nivel: 100,
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains("\"codigo\":\"ADMIN\""));
        assert!(json.contains("\"nivel\":100"));
    }

    #[test]
    fn userrole_serialize_inclui_empresa() {
        let ur = UserRole {
            id: 1,
            user_id: 5,
            role_id: 2,
            empresa_id: 1,
        };
        let json = serde_json::to_string(&ur).unwrap();
        assert!(json.contains("\"empresa_id\":1"));
    }

    #[test]
    fn contexto_system_tenant_zero() {
        let c = ContextoRbac::system();
        assert_eq!(c.tenant_id, 0);
    }
}
