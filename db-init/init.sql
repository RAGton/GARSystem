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


