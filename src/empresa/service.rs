// src/empresa/service.rs
//
// Regras de negócio + audit + integração com RBAC.

use super::models::{
    ContextoEmpresa, Empresa, EmpresaBootstrap, EmpresaConfiguracao, OrigemBootstrap, Plano,
};
use super::repository;
use crate::banco_de_dados::audit;
use crate::rbac;
use crate::servicos::ErroAplicacao;
use rand::{distributions::Alphanumeric, Rng};
use uuid::Uuid;

/// Gera um UUID v4 (string).
pub fn uuid_novo() -> String {
    Uuid::new_v4().to_string()
}

/// Gera uma senha aleatória forte (32 caracteres alfanuméricos).
///
/// **NÃO usar** para usuários finais — apenas bootstrap seguro.
pub fn gerar_senha_aleatoria() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

/// Cria uma nova empresa. Apenas SUPER_ADMIN pode chamar (validado no
/// handler — aqui assume que o caller já validou).
///
/// Fluxo:
///   1. Gerar UUID
///   2. INSERT empresa
///   3. INSERT empresa_configuracao (defaults)
///   4. (P2.6.2) Popular plano_contas, centros_custo, etc
///   5. audit log
pub fn criar_empresa(
    nome: &str,
    razao_social: &str,
    cnpj: Option<&str>,
    email: Option<&str>,
    telefone: Option<&str>,
    plano: Plano,
    ctx: &ContextoEmpresa,
) -> Result<i32, ErroAplicacao> {
    let uuid = uuid_novo();
    let id = repository::criar(&uuid, nome, razao_social, cnpj, email, telefone, plano)?;
    repository::criar_configuracao_padrao(id)?;
    registrar_audit(
        id,
        "EMPRESA_CRIADA",
        Some(&format!("{} ({})", nome, uuid)),
        ctx,
    );
    Ok(id)
}

/// Desativa uma empresa (soft delete).
///
/// Regras:
///   - Empresa 1 (bootstrap) não pode ser desativada
///   - Apenas SUPER_ADMIN
pub fn desativar_empresa(empresa_id: i32, ctx: &ContextoEmpresa) -> Result<(), ErroAplicacao> {
    if empresa_id == 1 {
        return Err(ErroAplicacao::Desconhecido(
            "Empresa bootstrap (id=1) não pode ser desativada.".into(),
        ));
    }
    let ok = repository::desativar(empresa_id)?;
    if !ok {
        return Err(ErroAplicacao::Desconhecido(
            "Empresa não encontrada.".into(),
        ));
    }
    registrar_audit(empresa_id, "EMPRESA_DESATIVADA", None, ctx);
    Ok(())
}

/// Obtém a configuração efetiva (com defaults aplicados se ausente).
///
/// **P2.6.1**: lida diretamente. Se não houver config, retorna defaults
/// (empresa pode estar em estado parcial).
pub fn obter_configuracao_efetiva(empresa_id: i32) -> Result<EmpresaConfiguracao, ErroAplicacao> {
    repository::obter_configuracao(empresa_id).map(|opt| {
        opt.unwrap_or_else(|| EmpresaConfiguracao {
            empresa_id,
            timezone: "America/Sao_Paulo".into(),
            moeda: "BRL".into(),
            idioma: "pt-BR".into(),
            tema: "light".into(),
            logo_url: None,
            cor_primaria: None,
            bootstrap_done: false,
        })
    })
}

/// Bootstrap seguro — gera um admin para a empresa 1 (ou para a empresa
/// recém-criada) com senha aleatória forte.
///
/// **Comportamento**:
///   - Se `SENIOR_BOOTSTRAP_PASSWORD` estiver setada, usa esse valor.
///     Caso contrário, gera uma senha aleatória de 32 chars alfanuméricos.
///   - Cria o user `admin@local` (NÃO `admin`!) com bcrypt hash.
///   - Atribui role ADMIN (não SUPER_ADMIN!) na empresa alvo.
///   - Marca `empresa_configuracao.bootstrap_done = TRUE`.
///   - Loga a senha UMA VEZ com `tracing::warn!`.
///
/// **PROIBIDO** criar `admin/admin` ou `admin/<vazio>`. Esta função é a
/// única via de criação de admin inicial e é safe-by-construction.
pub fn bootstrap_seguro(empresa_id: i32) -> Result<EmpresaBootstrap, ErroAplicacao> {
    // Idempotência: se já foi feito, não repetir.
    if let Some(cfg) = repository::obter_configuracao(empresa_id)? {
        if cfg.bootstrap_done {
            return Err(ErroAplicacao::Desconhecido(
                "Bootstrap já realizado para esta empresa.".into(),
            ));
        }
    }

    let username = "admin@local";

    // 1) Definir senha
    let (senha, origem) = match std::env::var("SENIOR_BOOTSTRAP_PASSWORD") {
        Ok(s) if !s.is_empty() && s.len() >= 16 => (s, OrigemBootstrap::EnvVar),
        _ => (gerar_senha_aleatoria(), OrigemBootstrap::Aleatoria),
    };

    // 2) Hash bcrypt
    let hash = bcrypt::hash(&senha, bcrypt::DEFAULT_COST)
        .map_err(|e| ErroAplicacao::FalhaNoHash(e.to_string()))?;

    // 3) Criar user (UPSERT para idempotência em re-tentativa)
    let mut conn = crate::banco_de_dados::conexao::obter_conexao()?;
    use mysql::params;
    use mysql::prelude::Queryable;

    conn.exec_drop(
        "INSERT INTO users (username, password_hash, role, tenant_id, empresa_id)
         VALUES (:u, :h, 'Administrador', :t, :e)
         ON DUPLICATE KEY UPDATE password_hash = VALUES(password_hash),
                                 tenant_id = VALUES(tenant_id),
                                 empresa_id = VALUES(empresa_id)",
        params! {
            "u" => username,
            "h" => &hash,
            "t" => empresa_id,
            "e" => empresa_id,
        },
    )?;

    let user_id: i32 = conn
        .query_first("SELECT id FROM users WHERE username = :u")?
        .ok_or_else(|| ErroAplicacao::Desconhecido("Falha ao obter id do admin criado".into()))?;

    // 4) Atribuir role ADMIN nesta empresa
    let rbac_ctx = crate::rbac::models::ContextoRbac {
        usuario_id: None,
        username: Some("system".into()),
        ip: None,
        user_agent: None,
        request_id: None,
        tenant_id: empresa_id,
    };
    rbac::service::atribuir_role(user_id, "ADMIN", empresa_id, &rbac_ctx)?;

    // 5) Marcar bootstrap_done
    repository::marcar_bootstrap_done(empresa_id)?;

    // 6) Logar senha (uma única vez)
    match origem {
        OrigemBootstrap::EnvVar => {
            tracing::warn!(
                "🔐 BOOTSTRAP — Empresa {}: admin criado com senha de SENIOR_BOOTSTRAP_PASSWORD. \
                 NÃO será exibida novamente.",
                empresa_id
            );
        }
        OrigemBootstrap::Aleatoria => {
            tracing::warn!(
                "🔐 BOOTSTRAP — Empresa {}: admin criado com senha ALEATÓRIA. \
                 Guarde em local seguro (NÃO será exibida novamente):\n  \
                 username: {}\n  \
                 senha: {}\n  \
                 origem: SENHA ALEATÓRIA (defina SENIOR_BOOTSTRAP_PASSWORD antes de subir para controlar)",
                empresa_id, username, senha
            );
        }
    }

    Ok(EmpresaBootstrap {
        empresa_id,
        admin_username: username.to_string(),
        admin_senha_gerada: senha,
        origem,
    })
}

/// Helper para gravar audit log de empresa.
///
/// O `acao` é uma string livre (ex.: "EMPRESA_CRIADA"). Vai para o campo
/// `entity_id` (codificado) e o detalhe vai para `depois` (JSON).
///
/// O audit ENUM do MySQL tem 7 valores fixos (CREATE, READ, UPDATE, etc.);
/// aqui usamos `OTHER` para qualquer ação específica de empresa, e
/// embutimos a string no detalhe.
pub fn registrar_audit(empresa_id: i32, acao: &str, detalhe: Option<&str>, ctx: &ContextoEmpresa) {
    let username = ctx.username.clone().unwrap_or_else(|| "system".into());
    let user_id = ctx.usuario_id;
    let ip = ctx.ip.as_deref();
    let ua = ctx.user_agent.as_deref();
    let rid = ctx.request_id.as_deref();
    let mut depois = serde_json::json!({ "acao": acao });
    if let Some(d) = detalhe {
        depois["detalhe"] = serde_json::Value::String(d.to_string());
    }
    let _ = audit::registrar_para(
        user_id,
        Some(&username),
        audit::Acao::Other,
        "empresa",
        Some(&empresa_id.to_string()),
        None,
        Some(depois),
        ip,
        ua,
        rid,
    );
}

/// Helper de leitura: lista empresas ativas (para dashboards SaaS).
pub fn listar_ativas() -> Result<Vec<Empresa>, ErroAplicacao> {
    repository::listar_ativas()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn senha_aleatoria_32_chars() {
        let s = gerar_senha_aleatoria();
        assert_eq!(s.len(), 32);
    }

    #[test]
    fn senhas_aleatorias_sao_diferentes() {
        let a = gerar_senha_aleatoria();
        let b = gerar_senha_aleatoria();
        assert_ne!(a, b);
    }

    #[test]
    fn uuid_novo_formato_valido() {
        let u = uuid_novo();
        // v4 tem forma 8-4-4-4-12
        assert_eq!(u.len(), 36);
        assert!(u.chars().filter(|c| *c == '-').count() == 4);
    }
}
