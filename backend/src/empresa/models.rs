// src/empresa/models.rs
//
// Structs, enums e DTOs do módulo Empresa.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Plano SaaS. Espelha o ENUM da tabela `empresa`.
///
/// **P2.6.1**: catálogo estático. Em P2.6.2 vira tabela
/// `empresa_plano` com preços + features por plano.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Plano {
    Free,
    Starter,
    Business,
    Enterprise,
}

impl Plano {
    pub fn as_str(&self) -> &'static str {
        match self {
            Plano::Free => "FREE",
            Plano::Starter => "STARTER",
            Plano::Business => "BUSINESS",
            Plano::Enterprise => "ENTERPRISE",
        }
    }

    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "FREE" => Some(Plano::Free),
            "STARTER" => Some(Plano::Starter),
            "BUSINESS" => Some(Plano::Business),
            "ENTERPRISE" => Some(Plano::Enterprise),
            _ => None,
        }
    }

    /// Limites sugeridos (P2.6.1: apenas referência; enforcement em P2.6.2).
    pub fn limite_usuarios_sugerido(&self) -> u32 {
        match self {
            Plano::Free => 3,
            Plano::Starter => 10,
            Plano::Business => 50,
            Plano::Enterprise => u32::MAX,
        }
    }
}

impl fmt::Display for Plano {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Empresa (tenant raiz).
///
/// `id` é interno (FK em outras tabelas). `uuid` é o identificador
/// público para URLs externas (`/empresas/{uuid}`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Empresa {
    pub id: i32,
    pub uuid: String,
    pub nome: String,
    pub razao_social: String,
    pub cnpj: Option<String>,
    pub email: Option<String>,
    pub telefone: Option<String>,
    pub ativa: bool,
    pub plano: Plano,
}

/// Configuração visual / locale por empresa.
///
/// Defaults: timezone America/Sao_Paulo, moeda BRL, idioma pt-BR,
/// tema light.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmpresaConfiguracao {
    pub empresa_id: i32,
    pub timezone: String,
    pub moeda: String,
    pub idioma: String,
    pub tema: String,
    pub logo_url: Option<String>,
    pub cor_primaria: Option<String>,
    pub bootstrap_done: bool,
}

/// Resultado de bootstrap seguro — gerado UMA VEZ por empresa.
///
/// O caller deve imprimir a senha via `tracing::warn!` (NÃO `info!`) e
/// sinalizar ao operador que ela não será exibida novamente.
#[derive(Debug)]
pub struct EmpresaBootstrap {
    pub empresa_id: i32,
    pub admin_username: String,
    pub admin_senha_gerada: String,
    pub origem: OrigemBootstrap,
}

/// Origem da senha no bootstrap.
#[derive(Debug, PartialEq, Eq)]
pub enum OrigemBootstrap {
    /// Senha lida da env `SENIOR_BOOTSTRAP_PASSWORD`.
    EnvVar,
    /// Senha gerada aleatoriamente (recomendado).
    Aleatoria,
}

/// Contexto de execução para o service layer.
#[derive(Debug, Clone)]
pub struct ContextoEmpresa {
    pub usuario_id: Option<u32>,
    pub username: Option<String>,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<String>,
    /// Empresa ativa no contexto (pode ser 0 para SUPER_ADMIN cross-tenant).
    pub tenant_id: i32,
}

impl ContextoEmpresa {
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

/// Preferências personalizadas por usuário e tenant (ERP SaaS Enterprise).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserPreferences {
    pub id: i64,
    pub tenant_id: i64,
    pub user_id: i64,
    pub theme: Option<String>,
    pub language: Option<String>,
    pub dashboard_default: Option<String>,
    pub sidebar_collapsed: bool,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// Registro de auditoria de autenticação e acessos (ERP SaaS Enterprise).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LoginAudit {
    pub id: i64,
    pub tenant_id: i64,
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub success: bool,
    pub login_at: Option<String>,
}

/// Customização e identidade visual corporativa por tenant (ERP SaaS Enterprise).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TenantBranding {
    pub tenant_id: i64,
    pub company_name: Option<String>,
    pub logo_url: Option<String>,
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub accent_color: Option<String>,
    pub welcome_message: Option<String>,
}

/// Configuração de layout e widgets de dashboard customizados por usuário (ERP SaaS Enterprise).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DashboardLayout {
    pub id: i64,
    pub tenant_id: i64,
    pub user_id: i64,
    pub dashboard_key: String,
    pub layout_json: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// Preferências avançadas de workspace e experiência de usuário (ERP SaaS Enterprise).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspacePreferences {
    pub id: i64,
    pub tenant_id: i64,
    pub user_id: i64,
    pub dense_mode: bool,
    pub animations_enabled: bool,
    pub notifications_sound: bool,
    pub active_workspace: String,
    pub custom_css: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plano_roundtrip() {
        for p in [
            Plano::Free,
            Plano::Starter,
            Plano::Business,
            Plano::Enterprise,
        ] {
            assert_eq!(Plano::from_string(p.as_str()), Some(p));
        }
    }

    #[test]
    fn plano_invalido_retorna_none() {
        assert_eq!(Plano::from_string("PREMIUM"), None);
        assert_eq!(Plano::from_string(""), None);
    }

    #[test]
    fn plano_free_limite_3() {
        assert_eq!(Plano::Free.limite_usuarios_sugerido(), 3);
    }

    #[test]
    fn plano_enterprise_limite_max() {
        assert_eq!(Plano::Enterprise.limite_usuarios_sugerido(), u32::MAX);
    }

    #[test]
    fn contexto_system_tenant_zero() {
        let ctx = ContextoEmpresa::system();
        assert_eq!(ctx.tenant_id, 0);
        assert_eq!(ctx.username.as_deref(), Some("system"));
    }

    #[test]
    fn empresa_serialize_tem_uuid_string() {
        let emp = Empresa {
            id: 1,
            uuid: "abc-123".into(),
            nome: "Teste".into(),
            razao_social: "Teste LTDA".into(),
            cnpj: None,
            email: None,
            telefone: None,
            ativa: true,
            plano: Plano::Free,
        };
        let json = serde_json::to_string(&emp).unwrap();
        assert!(json.contains("\"uuid\":\"abc-123\""));
        assert!(json.contains("\"plano\":\"FREE\""));
    }

    #[test]
    fn saas_models_roundtrip_test() {
        let pref = UserPreferences {
            id: 1,
            tenant_id: 10,
            user_id: 42,
            theme: Some("dark".into()),
            language: Some("pt-BR".into()),
            dashboard_default: Some("executivo".into()),
            sidebar_collapsed: false,
            created_at: None,
            updated_at: None,
        };
        let pref_json = serde_json::to_string(&pref).unwrap();
        assert!(pref_json.contains("\"tenant_id\":10"));

        let audit = LoginAudit {
            id: 1,
            tenant_id: 10,
            user_id: Some(42),
            username: Some("admin".into()),
            ip_address: Some("127.0.0.1".into()),
            user_agent: Some("SeniorSystem/2.0".into()),
            success: true,
            login_at: None,
        };
        let audit_json = serde_json::to_string(&audit).unwrap();
        assert!(audit_json.contains("\"success\":true"));

        let branding = TenantBranding {
            tenant_id: 10,
            company_name: Some("Empresa Exemplo".into()),
            logo_url: None,
            primary_color: Some("#07142B".into()),
            secondary_color: Some("#0E2F5A".into()),
            accent_color: Some("#1B4F9C".into()),
            welcome_message: Some("Bem-vindo ao SeniorSystem Enterprise".into()),
        };
        let branding_json = serde_json::to_string(&branding).unwrap();
        assert!(branding_json.contains("\"company_name\":\"Empresa Exemplo\""));

        let layout = DashboardLayout {
            id: 1,
            tenant_id: 10,
            user_id: 42,
            dashboard_key: "executivo".into(),
            layout_json: Some("{\"widgets\":[\"kpi_cr\",\"chart_os\"]}".into()),
            created_at: None,
            updated_at: None,
        };
        let layout_json = serde_json::to_string(&layout).unwrap();
        assert!(layout_json.contains("\"dashboard_key\":\"executivo\""));

        let ws = WorkspacePreferences {
            id: 1,
            tenant_id: 10,
            user_id: 42,
            dense_mode: true,
            animations_enabled: true,
            notifications_sound: false,
            active_workspace: "comercial".into(),
            custom_css: None,
            created_at: None,
            updated_at: None,
        };
        let ws_json = serde_json::to_string(&ws).unwrap();
        assert!(ws_json.contains("\"active_workspace\":\"comercial\""));
    }
}
