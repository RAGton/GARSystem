// src/empresa/repository.rs
//
// Camada SQL pura para Empresa. Sem regras de negócio.

use super::models::{Empresa, EmpresaConfiguracao, Plano};
use crate::servicos::ErroAplicacao;
use mysql::{params, prelude::Queryable, PooledConn};

use crate::banco_de_dados::conexao::obter_conexao;

/// Verifica se existe ao menos uma empresa. Usado pelo bootstrap.
pub fn empresa_existe() -> Result<bool, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<(i32,)> = conn.query_first("SELECT id FROM empresa LIMIT 1")?;
    Ok(row.is_some())
}

pub fn criar(
    uuid: &str,
    nome: &str,
    razao_social: &str,
    cnpj: Option<&str>,
    email: Option<&str>,
    telefone: Option<&str>,
    plano: Plano,
) -> Result<i32, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        "INSERT INTO empresa (uuid, nome, razao_social, cnpj, email, telefone, ativa, plano)
         VALUES (:uuid, :nome, :razao, :cnpj, :email, :tel, TRUE, :plano)",
        params! {
            "uuid" => uuid,
            "nome" => nome,
            "razao" => razao_social,
            "cnpj" => cnpj,
            "email" => email,
            "tel" => telefone,
            "plano" => plano.as_str(),
        },
    )?;
    Ok(conn.last_insert_id() as i32)
}

pub fn criar_configuracao_padrao(empresa_id: i32) -> Result<(), ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        "INSERT IGNORE INTO empresa_configuracao (empresa_id, timezone, moeda, idioma, tema, bootstrap_done)
         VALUES (:id, 'America/Sao_Paulo', 'BRL', 'pt-BR', 'light', FALSE)",
        params! { "id" => empresa_id },
    )?;
    Ok(())
}

pub fn obter_por_id(id: i32) -> Result<Option<Empresa>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<(
        i32,
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        i8,
        String,
    )> = conn.exec_first(
        "SELECT id, uuid, nome, razao_social, cnpj, email, telefone, ativa, plano
             FROM empresa WHERE id = :id",
        params! { "id" => id },
    )?;
    Ok(row.map(
        |(id, uuid, nome, razao_social, cnpj, email, telefone, ativa, plano)| Empresa {
            id,
            uuid,
            nome,
            razao_social,
            cnpj,
            email,
            telefone,
            ativa: ativa != 0,
            plano: Plano::from_string(&plano).unwrap_or(Plano::Free),
        },
    ))
}

pub fn obter_por_uuid(uuid: &str) -> Result<Option<Empresa>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<(
        i32,
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        i8,
        String,
    )> = conn.exec_first(
        "SELECT id, uuid, nome, razao_social, cnpj, email, telefone, ativa, plano
             FROM empresa WHERE uuid = :uuid",
        params! { "uuid" => uuid },
    )?;
    Ok(row.map(
        |(id, uuid, nome, razao_social, cnpj, email, telefone, ativa, plano)| Empresa {
            id,
            uuid,
            nome,
            razao_social,
            cnpj,
            email,
            telefone,
            ativa: ativa != 0,
            plano: Plano::from_string(&plano).unwrap_or(Plano::Free),
        },
    ))
}

pub fn listar() -> Result<Vec<Empresa>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let rows: Vec<(
        i32,
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        i8,
        String,
    )> = conn.query(
        "SELECT id, uuid, nome, razao_social, cnpj, email, telefone, ativa, plano
             FROM empresa ORDER BY id ASC",
    )?;
    Ok(rows
        .into_iter()
        .map(
            |(id, uuid, nome, razao_social, cnpj, email, telefone, ativa, plano)| Empresa {
                id,
                uuid,
                nome,
                razao_social,
                cnpj,
                email,
                telefone,
                ativa: ativa != 0,
                plano: Plano::from_string(&plano).unwrap_or(Plano::Free),
            },
        )
        .collect())
}

pub fn listar_ativas() -> Result<Vec<Empresa>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let rows: Vec<(
        i32,
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        i8,
        String,
    )> = conn.query(
        "SELECT id, uuid, nome, razao_social, cnpj, email, telefone, ativa, plano
             FROM empresa WHERE ativa = TRUE ORDER BY id ASC",
    )?;
    Ok(rows
        .into_iter()
        .map(
            |(id, uuid, nome, razao_social, cnpj, email, telefone, ativa, plano)| Empresa {
                id,
                uuid,
                nome,
                razao_social,
                cnpj,
                email,
                telefone,
                ativa: ativa != 0,
                plano: Plano::from_string(&plano).unwrap_or(Plano::Free),
            },
        )
        .collect())
}

pub fn desativar(id: i32) -> Result<bool, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        "UPDATE empresa SET ativa = FALSE WHERE id = :id",
        params! { "id" => id },
    )?;
    Ok(conn.affected_rows() > 0)
}

pub fn atualizar_status(id: i32, ativa: bool) -> Result<bool, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        "UPDATE empresa SET ativa = :ativa WHERE id = :id",
        params! { "id" => id, "ativa" => ativa },
    )?;
    Ok(conn.affected_rows() > 0)
}

pub fn atualizar_configuracao(
    empresa_id: i32,
    timezone: Option<&str>,
    moeda: Option<&str>,
    idioma: Option<&str>,
    tema: Option<&str>,
    logo_url: Option<&str>,
    cor_primaria: Option<&str>,
) -> Result<bool, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    // Update only non-None fields; use a flexible update
    let mut sets: Vec<&str> = vec![];
    if let Some(_tz) = timezone {
        sets.push("timezone = :tz");
    }
    // Construir dinamicamente é verboso; abordagem simples: UPDATE full
    conn.exec_drop(
        "UPDATE empresa_configuracao SET
            timezone = COALESCE(:tz, timezone),
            moeda = COALESCE(:moeda, moeda),
            idioma = COALESCE(:idioma, idioma),
            tema = COALESCE(:tema, tema),
            logo_url = COALESCE(:logo, logo_url),
            cor_primaria = COALESCE(:cor, cor_primaria)
         WHERE empresa_id = :id",
        params! {
            "id" => empresa_id,
            "tz" => timezone,
            "moeda" => moeda,
            "idioma" => idioma,
            "tema" => tema,
            "logo" => logo_url,
            "cor" => cor_primaria,
        },
    )?;
    Ok(conn.affected_rows() > 0)
}

pub fn obter_configuracao(empresa_id: i32) -> Result<Option<EmpresaConfiguracao>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<(
        i32,
        String,
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        i8,
    )> = conn.exec_first(
        "SELECT empresa_id, timezone, moeda, idioma, tema, logo_url, cor_primaria, bootstrap_done
             FROM empresa_configuracao WHERE empresa_id = :id",
        params! { "id" => empresa_id },
    )?;
    Ok(row.map(
        |(empresa_id, timezone, moeda, idioma, tema, logo_url, cor_primaria, bootstrap_done)| {
            EmpresaConfiguracao {
                empresa_id,
                timezone,
                moeda,
                idioma,
                tema,
                logo_url,
                cor_primaria,
                bootstrap_done: bootstrap_done != 0,
            }
        },
    ))
}

pub fn marcar_bootstrap_done(empresa_id: i32) -> Result<bool, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        "UPDATE empresa_configuracao SET bootstrap_done = TRUE WHERE empresa_id = :id",
        params! { "id" => empresa_id },
    )?;
    Ok(conn.affected_rows() > 0)
}

pub fn contar_usuarios(empresa_id: i32) -> Result<u32, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<(i64,)> = conn.exec_first(
        "SELECT COUNT(*) FROM users WHERE empresa_id = :id",
        params! { "id" => empresa_id },
    )?;
    Ok(row.map(|(c,)| c as u32).unwrap_or(0))
}

/// Conta registros em uma tabela filtrando por tenant_id.
///
/// Helper usado em **testes de isolamento multi-tenant** (tests/integration/tenants.rs).
///
/// **P2.6.1**: API intencionalmente exposta como `pub` para que os testes
/// possam validar o defense in depth sem precisar importar o pool.
pub fn ping_count(pool: &mysql::Pool, tabela: &str, tenant_id: i32) -> Result<u32, ErroAplicacao> {
    use mysql::prelude::Queryable;
    let mut conn = pool.get_conn()?;
    let query = format!("SELECT COUNT(*) FROM {} WHERE tenant_id = ?", tabela);
    let row: Option<(i64,)> = conn.exec_first(query, (tenant_id,))?;
    Ok(row.map(|(c,)| c as u32).unwrap_or(0))
}

#[allow(dead_code)]
pub fn pingar_conexao(conn: &mut PooledConn) -> Result<(), ErroAplicacao> {
    conn.query_drop("SELECT 1")?;
    Ok(())
}

use super::models::{
    DashboardLayout, LoginAudit, TenantBranding, UserPreferences, WorkspacePreferences,
};

pub fn obter_preferencias(
    tenant_id: i64,
    user_id: i64,
) -> Result<Option<UserPreferences>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<(
        i64,
        i64,
        i64,
        Option<String>,
        Option<String>,
        Option<String>,
        bool,
        Option<String>,
        Option<String>,
    )> = conn.exec_first(
        r#"SELECT id, tenant_id, user_id, theme, language, dashboard_default,
                  sidebar_collapsed, DATE_FORMAT(created_at, '%Y-%m-%d %H:%i:%s'),
                  DATE_FORMAT(updated_at, '%Y-%m-%d %H:%i:%s')
           FROM user_preferences
           WHERE tenant_id = :tid AND user_id = :uid"#,
        params! {
            "tid" => tenant_id,
            "uid" => user_id,
        },
    )?;

    Ok(row.map(
        |(id, tid, uid, theme, lang, dash, col, created, updated)| UserPreferences {
            id,
            tenant_id: tid,
            user_id: uid,
            theme,
            language: lang,
            dashboard_default: dash,
            sidebar_collapsed: col,
            created_at: created,
            updated_at: updated,
        },
    ))
}

pub fn salvar_preferencias(
    tenant_id: i64,
    user_id: i64,
    theme: Option<&str>,
    language: Option<&str>,
    dashboard_default: Option<&str>,
    sidebar_collapsed: bool,
) -> Result<(), ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        r#"INSERT INTO user_preferences
           (tenant_id, user_id, theme, language, dashboard_default, sidebar_collapsed)
           VALUES (:tid, :uid, :theme, :lang, :dash, :col)
           ON DUPLICATE KEY UPDATE
             theme = VALUES(theme),
             language = VALUES(language),
             dashboard_default = VALUES(dashboard_default),
             sidebar_collapsed = VALUES(sidebar_collapsed)"#,
        params! {
            "tid" => tenant_id,
            "uid" => user_id,
            "theme" => theme,
            "lang" => language,
            "dash" => dashboard_default,
            "col" => sidebar_collapsed,
        },
    )?;
    Ok(())
}

pub fn registrar_login_audit(
    tenant_id: i64,
    user_id: Option<i64>,
    username: Option<&str>,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
    success: bool,
) -> Result<(), ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        r#"INSERT INTO login_audit
           (tenant_id, user_id, username, ip_address, user_agent, success)
           VALUES (:tid, :uid, :user, :ip, :ua, :ok)"#,
        params! {
            "tid" => tenant_id,
            "uid" => user_id,
            "user" => username,
            "ip" => ip_address,
            "ua" => user_agent,
            "ok" => success,
        },
    )?;
    Ok(())
}

pub fn obter_tenant_branding(tenant_id: i64) -> Result<Option<TenantBranding>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<(
        i64,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = conn.exec_first(
        r#"SELECT tenant_id, company_name, logo_url, primary_color, secondary_color,
                  accent_color, welcome_message
           FROM tenant_branding
           WHERE tenant_id = :tid"#,
        params! { "tid" => tenant_id },
    )?;

    Ok(
        row.map(|(tid, name, logo, c1, c2, c3, msg)| TenantBranding {
            tenant_id: tid,
            company_name: name,
            logo_url: logo,
            primary_color: c1,
            secondary_color: c2,
            accent_color: c3,
            welcome_message: msg,
        }),
    )
}

pub fn obter_dashboard_layout(
    tenant_id: i64,
    user_id: i64,
    dashboard_key: &str,
) -> Result<Option<DashboardLayout>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<(
        i64,
        i64,
        i64,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = conn.exec_first(
        r#"SELECT id, tenant_id, user_id, dashboard_key, layout_json,
                  DATE_FORMAT(created_at, '%Y-%m-%d %H:%i:%s'),
                  DATE_FORMAT(updated_at, '%Y-%m-%d %H:%i:%s')
           FROM dashboard_layouts
           WHERE tenant_id = :tid AND user_id = :uid AND dashboard_key = :k"#,
        params! {
            "tid" => tenant_id,
            "uid" => user_id,
            "k" => dashboard_key,
        },
    )?;

    Ok(row.map(
        |(id, tid, uid, key, json, created, updated)| DashboardLayout {
            id,
            tenant_id: tid,
            user_id: uid,
            dashboard_key: key,
            layout_json: json,
            created_at: created,
            updated_at: updated,
        },
    ))
}

pub fn salvar_dashboard_layout(
    tenant_id: i64,
    user_id: i64,
    dashboard_key: &str,
    layout_json: Option<&str>,
) -> Result<(), ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        r#"INSERT INTO dashboard_layouts
           (tenant_id, user_id, dashboard_key, layout_json)
           VALUES (:tid, :uid, :k, :json)
           ON DUPLICATE KEY UPDATE layout_json = VALUES(layout_json)"#,
        params! {
            "tid" => tenant_id,
            "uid" => user_id,
            "k" => dashboard_key,
            "json" => layout_json,
        },
    )?;
    Ok(())
}

pub fn obter_workspace_preferences(
    tenant_id: i64,
    user_id: i64,
) -> Result<Option<WorkspacePreferences>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<(
        i64,
        i64,
        i64,
        bool,
        bool,
        bool,
        String,
        Option<String>,
        Option<String>,
        Option<String>,
    )> = conn.exec_first(
        r#"SELECT id, tenant_id, user_id, dense_mode, animations_enabled,
                  notifications_sound, active_workspace, custom_css,
                  DATE_FORMAT(created_at, '%Y-%m-%d %H:%i:%s'),
                  DATE_FORMAT(updated_at, '%Y-%m-%d %H:%i:%s')
           FROM workspace_preferences
           WHERE tenant_id = :tid AND user_id = :uid"#,
        params! {
            "tid" => tenant_id,
            "uid" => user_id,
        },
    )?;

    Ok(row.map(
        |(id, tid, uid, dense, anim, sound, ws, css, created, updated)| WorkspacePreferences {
            id,
            tenant_id: tid,
            user_id: uid,
            dense_mode: dense,
            animations_enabled: anim,
            notifications_sound: sound,
            active_workspace: ws,
            custom_css: css,
            created_at: created,
            updated_at: updated,
        },
    ))
}

pub fn salvar_workspace_preferences(
    tenant_id: i64,
    user_id: i64,
    dense: bool,
    anim: bool,
    sound: bool,
    active_ws: &str,
    css: Option<&str>,
) -> Result<(), ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        r#"INSERT INTO workspace_preferences
           (tenant_id, user_id, dense_mode, animations_enabled, notifications_sound, active_workspace, custom_css)
           VALUES (:tid, :uid, :dense, :anim, :sound, :ws, :css)
           ON DUPLICATE KEY UPDATE
             dense_mode = VALUES(dense_mode),
             animations_enabled = VALUES(animations_enabled),
             notifications_sound = VALUES(notifications_sound),
             active_workspace = VALUES(active_workspace),
             custom_css = VALUES(custom_css)"#,
        params! {
            "tid" => tenant_id,
            "uid" => user_id,
            "dense" => dense,
            "anim" => anim,
            "sound" => sound,
            "ws" => active_ws,
            "css" => css,
        },
    )?;
    Ok(())
}
