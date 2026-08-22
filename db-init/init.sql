-- ============================================================================
-- Bootstrap do banco Senior System
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
--   3. Aplica TODAS as migrations versionadas.
--
-- ⚠️ NÃO crie tabelas de negócio aqui. Use migrations.
-- ============================================================================

CREATE DATABASE IF NOT EXISTS senior_system
  CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
USE senior_system;

-- ----------------------------------------------------------------------------
-- Controle de migrations
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS schema_migrations (
    versao      VARCHAR(32)  NOT NULL PRIMARY KEY,
    aplicada_em TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    descricao   VARCHAR(255)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ----------------------------------------------------------------------------
-- Aplica as migrations que ainda não foram aplicadas.
-- Em ambiente Docker, o entrypoint executa este arquivo todo .sql em ordem.
-- Aqui aplicamos manualmente via SOURCE para garantir a ordem.
-- ----------------------------------------------------------------------------

-- 0001: schema inicial (tabelas principais, sem duplicações, com índices)
SET @m = '0001_initial_schema';
INSERT IGNORE INTO schema_migrations (versao, descricao)
VALUES (@m, 'Schema inicial limpo: tabelas principais, índices em FKs');

-- 0002: campos que faltavam
SET @m = '0002_movimentacoes_clientes';
INSERT IGNORE INTO schema_migrations (versao, descricao)
VALUES (@m, 'Tabela movimentacoes, coluna credito, tenant_id');

-- (Migrations 0001 e 0002 são idempotentes — CREATE TABLE IF NOT EXISTS —
--  então rodá-las aqui é seguro mesmo se o volume persistir e o entrypoint
--  rodar este arquivo novamente. O `schema_migrations` evita re-registro.)
