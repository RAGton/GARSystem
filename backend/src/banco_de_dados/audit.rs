// src/banco_de_dados/audit.rs
//
// Audit trail padronizado (Sprint P1 - Fase 4).
//
// ## Filosofia
//
// O audit é **best-effort**: uma falha em registrar uma ação NÃO deve
// impedir a operação principal (criar cliente, criar OS, etc.). Por isso,
// `registrar` SEMPRE retorna `Ok(())` mesmo se a query falhar — apenas
// loga um warning.
//
// ## Uso típico
//
// ```ignore
// use crate::banco_de_dados::audit::{self, Acao};
//
// audit::registrar_de_claims(
//     &claims,
//     Acao::Create,
//     "clientes",
//     Some(&novo_id.to_string()),
//     None,
//     Some(serde_json::to_value(&cliente).ok()),
//     Some(request_id.as_deref()),
// )?;
// ```

use mysql::params;
use mysql::prelude::Queryable;
use serde::Serialize;
use tracing::warn;

use super::conexao::obter_conexao;
use crate::servicos::ErroAplicacao;

/// Tipo de ação auditada. Mapeia para o `ENUM` do MySQL.
#[derive(Debug, Clone, Copy, Serialize)]
pub enum Acao {
    Create,
    Read,
    Update,
    Delete,
    Login,
    Logout,
    Other,
}

impl Acao {
    /// Serialização como string esperada pelo MySQL ENUM.
    pub fn as_str(&self) -> &'static str {
        match self {
            Acao::Create => "CREATE",
            Acao::Read => "READ",
            Acao::Update => "UPDATE",
            Acao::Delete => "DELETE",
            Acao::Login => "LOGIN",
            Acao::Logout => "LOGOUT",
            Acao::Other => "OTHER",
        }
    }
}

/// Snapshot de uma ação a ser registrada.
#[derive(Debug, Clone)]
pub struct AuditEntry<'a> {
    pub usuario_id: Option<u32>,
    pub username: Option<&'a str>,
    pub acao: Acao,
    pub entidade: &'a str,
    pub entity_id: Option<&'a str>,
    pub antes: Option<serde_json::Value>,
    pub depois: Option<serde_json::Value>,
    pub ip: Option<&'a str>,
    pub user_agent: Option<&'a str>,
    pub request_id: Option<&'a str>,
}

/// Registra uma entrada no `audit_log`.
///
/// Best-effort: nunca retorna erro ao caller. Falhas são logadas como
/// warning mas não interrompem o fluxo principal.
pub fn registrar(entry: AuditEntry<'_>) -> Result<(), ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let res = conn.exec_drop(
        r#"INSERT INTO audit_log
           (usuario_id, username, acao, entidade, entity_id, antes, depois, ip, user_agent, request_id)
           VALUES
           (:usuario_id, :username, :acao, :entidade, :entity_id, :antes, :depois, :ip, :ua, :rid)"#,
        params! {
            "usuario_id" => entry.usuario_id,
            "username" => entry.username,
            "acao" => entry.acao.as_str(),
            "entidade" => entry.entidade,
            "entity_id" => entry.entity_id,
            "antes" => entry.antes.as_ref().map(|v| v.to_string()),
            "depois" => entry.depois.as_ref().map(|v| v.to_string()),
            "ip" => entry.ip,
            "ua" => entry.user_agent,
            "rid" => entry.request_id,
        },
    );

    match res {
        Ok(()) => Ok(()),
        Err(e) => {
            // Audit é best-effort: loga warning, não propaga erro
            warn!(
                erro = %e,
                entidade = entry.entidade,
                acao = entry.acao.as_str(),
                "falha ao registrar audit log (operação principal NÃO foi afetada)"
            );
            Ok(())
        }
    }
}

/// Helper que recebe uid/username em vez de `Claims` (para evitar
/// acoplamento entre a lib e o bin que define o tipo `Claims`).
#[allow(clippy::too_many_arguments)]
pub fn registrar_para(
    usuario_id: Option<u32>,
    username: Option<&str>,
    acao: Acao,
    entidade: &str,
    entity_id: Option<&str>,
    antes: Option<serde_json::Value>,
    depois: Option<serde_json::Value>,
    ip: Option<&str>,
    user_agent: Option<&str>,
    request_id: Option<&str>,
) -> Result<(), ErroAplicacao> {
    let entry = AuditEntry {
        usuario_id,
        username,
        acao,
        entidade,
        entity_id,
        antes,
        depois,
        ip,
        user_agent,
        request_id,
    };
    registrar(entry)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acao_as_str_mapeia_corretamente() {
        assert_eq!(Acao::Create.as_str(), "CREATE");
        assert_eq!(Acao::Read.as_str(), "READ");
        assert_eq!(Acao::Update.as_str(), "UPDATE");
        assert_eq!(Acao::Delete.as_str(), "DELETE");
        assert_eq!(Acao::Login.as_str(), "LOGIN");
        assert_eq!(Acao::Logout.as_str(), "LOGOUT");
        assert_eq!(Acao::Other.as_str(), "OTHER");
    }

    #[test]
    fn audit_entry_constroi_sem_panicar() {
        let entry = AuditEntry {
            usuario_id: Some(1),
            username: Some("admin"),
            acao: Acao::Create,
            entidade: "clientes",
            entity_id: Some("42"),
            antes: None,
            depois: Some(serde_json::json!({"id": 42, "nome": "X"})),
            ip: Some("127.0.0.1"),
            user_agent: Some("curl/8.0"),
            request_id: Some("req-abc"),
        };
        assert_eq!(entry.entidade, "clientes");
        assert_eq!(entry.acao.as_str(), "CREATE");
    }

    #[test]
    fn audit_update_captura_antes_e_depois() {
        // Simula o padrão usado em UPDATE cliente
        let antes: serde_json::Value = serde_json::json!({
            "id": 1, "nome": "Old", "email": "old@x.com", "credito": 0.0
        });
        let depois: serde_json::Value = serde_json::json!({
            "id": 1, "nome": "New", "email": "new@x.com", "credito": 100.0
        });

        let entry = AuditEntry {
            usuario_id: Some(1),
            username: Some("admin"),
            acao: Acao::Update,
            entidade: "clientes",
            entity_id: Some("1"),
            antes: Some(antes.clone()),
            depois: Some(depois.clone()),
            ip: Some("127.0.0.1"),
            user_agent: None,
            request_id: Some("req-xyz"),
        };

        // Verifica que antes/depois são preservados fielmente
        assert_eq!(entry.antes.unwrap()["nome"], "Old");
        assert_eq!(entry.depois.unwrap()["nome"], "New");
    }

    #[test]
    fn audit_delete_tem_antes_mas_nao_depois() {
        // Em DELETE, temos o `antes` mas não `depois` (registro foi removido)
        let antes: serde_json::Value = serde_json::json!({
            "id": 1, "nome": "X"
        });

        let entry = AuditEntry {
            usuario_id: Some(1),
            username: None,
            acao: Acao::Delete,
            entidade: "clientes",
            entity_id: Some("1"),
            antes: Some(antes),
            depois: None,
            ip: None,
            user_agent: None,
            request_id: None,
        };

        assert!(entry.antes.is_some());
        assert!(entry.depois.is_none());
    }

    #[test]
    fn acao_delete_serializa_corretamente() {
        assert_eq!(Acao::Delete.as_str(), "DELETE");
    }
}
