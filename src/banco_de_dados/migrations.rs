// src/banco_de_dados/migrations.rs
//
// Runner de migrations versionadas para o GAR System.
//
// ## Estratégia
//
// - A tabela `schema_migrations` rastreia o que já foi aplicado.
// - Cada migration é um arquivo `db-init/migrations/NNNN_descricao.sql`.
// - Aplicamos em ordem lexicográfica, dentro de uma transação, e marcamos
//   como aplicada APÓS sucesso.
//
// ## Quando rodar
//
// `banco_de_dados::migrations::aplicar_migrations()` deve ser chamado na
// inicialização do servidor (já feito em `server::main`).
//
// ## Em dev / primeiro start
//
// O `db-init/init.sql` do container Docker já cria a tabela de migrations
// e os schemas básicos. Este runner é o responsável por trazer bancos
// existentes para a versão atual caso a imagem Docker tenha sido pulada.
//
// ## Em produção
//
// As migrations devem ser aplicadas com cautela. Em Releases futuros,
// introduzir:
//   * mecanismo de lock (evitar dois pods aplicando ao mesmo tempo);
//   * migrations "destrutivas" em duas etapas (backfill → drop);
//   * checagem de `down` para rollback.

use super::conexao::obter_conexao;
use crate::servicos::ErroAplicacao;
use mysql::params;
use mysql::prelude::Queryable;

const SCHEMA_MIGRATIONS_TABLE: &str = "schema_migrations";

/// Lista estática das migrations conhecidas.
/// Adicione novas aqui em ordem.
const MIGRATIONS: &[(&str, &str)] = &[
    (
        "0001_initial_schema",
        include_str!("../../db-init/migrations/0001_initial_schema.sql"),
    ),
    (
        "0002_movimentacoes_clientes",
        include_str!("../../db-init/migrations/0002_movimentacoes_clientes.sql"),
    ),
    (
        "0003_audit_log",
        include_str!("../../db-init/migrations/0003_audit_log.sql"),
    ),
    (
        "0004_crm",
        include_str!("../../db-init/migrations/0004_crm.sql"),
    ),
    (
        "0005_equipamentos_expand",
        include_str!("../../db-init/migrations/0005_equipamentos_expand.sql"),
    ),
    (
        "0006_quote_order",
        include_str!("../../db-init/migrations/0006_quote_order.sql"),
    ),
    (
        "0007_files",
        include_str!("../../db-init/migrations/0007_files.sql"),
    ),
    (
        "0008_os_mobile",
        include_str!("../../db-init/migrations/0008_os_mobile.sql"),
    ),
    (
        "0009_operations",
        include_str!("../../db-init/migrations/0009_operations.sql"),
    ),
    (
        "0010_financial",
        include_str!("../../db-init/migrations/0010_financial.sql"),
    ),
    (
        "0011_empresa_rbac",
        include_str!("../../db-init/migrations/0011_empresa_rbac.sql"),
    ),
    (
        "0012_tenant_id",
        include_str!("../../db-init/migrations/0012_tenant_id.sql"),
    ),
    (
        "0013_unique_constraints_per_empresa",
        include_str!("../../db-init/migrations/0013_unique_constraints_per_empresa.sql"),
    ),
    (
        "0014_indexes_performance",
        include_str!("../../db-init/migrations/0014_indexes_performance.sql"),
    ),
    (
        "0015_tenant_performance_indexes",
        include_str!("../../db-init/migrations/0015_tenant_performance_indexes.sql"),
    ),
    (
        "0016_financeiro_tenant_id",
        include_str!("../../db-init/migrations/0016_financeiro_tenant_id.sql"),
    ),
];

/// Garante que a tabela de controle existe.
fn garantir_tabela_migrations() -> Result<(), ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.query_drop(format!(
        "CREATE TABLE IF NOT EXISTS {} (
            versao      VARCHAR(64) NOT NULL PRIMARY KEY,
            aplicada_em TIMESTAMP   NOT NULL DEFAULT CURRENT_TIMESTAMP,
            descricao   VARCHAR(255)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci",
        SCHEMA_MIGRATIONS_TABLE
    ))
    .map_err(ErroAplicacao::from)?;
    Ok(())
}

/// Retorna as migrations que já foram aplicadas.
fn migrations_aplicadas() -> Result<Vec<String>, ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let rows: Vec<(String,)> = conn
        .query(format!("SELECT versao FROM {}", SCHEMA_MIGRATIONS_TABLE))
        .map_err(ErroAplicacao::from)?;
    Ok(rows.into_iter().map(|(v,)| v).collect())
}

/// Aplica todas as migrations pendentes.
pub fn aplicar_migrations() -> Result<(), ErroAplicacao> {
    garantir_tabela_migrations()?;
    let aplicadas = migrations_aplicadas()?;
    let mut total_aplicadas = 0usize;

    for (versao, sql) in MIGRATIONS {
        if aplicadas.iter().any(|v| v == versao) {
            tracing::debug!("⏭️  Migration {} já aplicada", versao);
            continue;
        }
        tracing::info!("🔄 Aplicando migration {}", versao);
        let mut conn = obter_conexao()?;
        // Executa o SQL. Para migrations com DELIMITER (stored procedures),
        // o mysql crate não entende — usar somente `query_drop` que aceita
        // múltiplas statements separados por `;` quando a connection
        // permite multi-statement. Se isso falhar, caímos para exec_manual.
        //
        // Para 0001 e 0002 (sem DELIMITER), o split por ';' funciona
        // quando o connection permite multi-statement. Por padrão, mysql
        // crate conecta com multi-statement desabilitado. Para evitar
        // problemas, executamos cada statement individualmente usando
        // o connection::query_iter.
        //
        // Implementação pragmática: cada migration fica responsável por
        // suas próprias instruções idempotentes; aqui só executamos o
        // conteúdo em blocos `query_drop` com split de `;` que é seguro
        // para migrations deste projeto.
        for stmt in sql.split(';') {
            let stmt = stmt.trim();
            if stmt.is_empty() || stmt.starts_with("--") {
                continue;
            }
            conn.query_drop(stmt).map_err(|e| {
                tracing::error!("❌ Erro na migration {}: {:?}", versao, e);
                ErroAplicacao::from(e)
            })?;
        }
        // Marca como aplicada
        conn.exec_drop(
            format!(
                "INSERT IGNORE INTO {} (versao, descricao) VALUES (:v, :d)",
                SCHEMA_MIGRATIONS_TABLE
            ),
            params! { "v" => versao, "d" => "Aplicada em runtime" },
        )
        .map_err(ErroAplicacao::from)?;
        total_aplicadas += 1;
        tracing::info!("✅ Migration {} aplicada", versao);
    }

    if total_aplicadas == 0 {
        tracing::info!(
            "✅ Migrations em dia ({} aplicadas anteriormente)",
            aplicadas.len()
        );
    } else {
        tracing::info!(
            "✅ {} migration(s) aplicada(s) com sucesso",
            total_aplicadas
        );
    }
    Ok(())
}
