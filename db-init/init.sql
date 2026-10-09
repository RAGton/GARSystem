-- ============================================================================
-- Bootstrap do banco GAR System
-- ----------------------------------------------------------------------------
-- Este arquivo roda automaticamente na primeira inicialização do container
-- MySQL (docker-entrypoint-initdb.d). Em reinicializações do container com
-- volume persistente, ele é IGNORADO — por isso TODA mudança de schema
-- precisa entrar em `db-init/migrations/NNNN_*.sql` e ser aplicada pelo
-- runner Rust em `src/banco_de_dados/migrations.rs`.
--
-- O que este arquivo faz:
--   1. Cria o banco se não existir.
--   2. Cria a tabela de controle de migrations (`schema_migrations`).
--   3. Aplica TODAS as migrations versionadas via SOURCE, em ordem.
--
-- IMPORTANTE: este arquivo SÓ roda no primeiro start (volume vazio). As
-- migrations são idempotentes (CREATE TABLE IF NOT EXISTS, INSERT IGNORE),
-- então é seguro re-executar.
--
-- ⚠️ NÃO crie tabelas de negócio aqui. Use migrations.
-- ============================================================================

CREATE DATABASE IF NOT EXISTS gar_system
  CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
USE gar_system;

-- ----------------------------------------------------------------------------
-- Controle de migrations
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS schema_migrations (
    versao      VARCHAR(32)  NOT NULL PRIMARY KEY,
    aplicada_em TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    descricao   VARCHAR(255)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE utf8mb4_unicode_ci;

-- ----------------------------------------------------------------------------
-- Aplica TODAS as migrations versionadas em ordem.
-- SOURCE executa cada arquivo .sql dentro do contexto do mysql client.
-- O compose monta ./db-init em /docker-entrypoint-initdb.d, então os
-- arquivos em ./db-init/migrations/ ficam acessíveis via
-- /docker-entrypoint-initdb.d/migrations/.
-- ----------------------------------------------------------------------------

SOURCE /docker-entrypoint-initdb.d/migrations/0001_initial_schema.sql;
SOURCE /docker-entrypoint-initdb.d/migrations/0002_movimentacoes_clientes.sql;
SOURCE /docker-entrypoint-initdb.d/migrations/0003_audit_log.sql;
SOURCE /docker-entrypoint-initdb.d/migrations/0004_crm.sql;
SOURCE /docker-entrypoint-initdb.d/migrations/0005_equipamentos_expand.sql;
SOURCE /docker-entrypoint-initdb.d/migrations/0006_quote_order.sql;
SOURCE /docker-entrypoint-initdb.d/migrations/0007_files.sql;
SOURCE /docker-entrypoint-initdb.d/migrations/0008_os_mobile.sql;
SOURCE /docker-entrypoint-initdb.d/migrations/0009_operations.sql;
SOURCE /docker-entrypoint-initdb.d/migrations/0010_financial.sql;
SOURCE /docker-entrypoint-initdb.d/migrations/0011_empresa_rbac.sql;
SOURCE /docker-entrypoint-initdb.d/migrations/0012_tenant_id.sql;
SOURCE /docker-entrypoint-initdb.d/migrations/0013_unique_constraints_per_empresa.sql;
SOURCE /docker-entrypoint-initdb.d/migrations/0014_indexes_performance.sql;
SOURCE /docker-entrypoint-initdb.d/migrations/0015_tenant_performance_indexes.sql;
SOURCE /docker-entrypoint-initdb.d/migrations/0016_financeiro_tenant_id.sql;
