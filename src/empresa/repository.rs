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
