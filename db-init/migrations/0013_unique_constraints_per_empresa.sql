-- ============================================================================
-- Migration 0013: Unique constraints compostos por empresa (Sprint P2.6.1)
-- ----------------------------------------------------------------------------
-- Migra UNIQUE constraints que são globais para compostos com `tenant_id`:
--   - users.username         → UNIQUE (tenant_id, username)
--   - clientes.cpf_cnpj      → UNIQUE (tenant_id, cpf_cnpj)
--   - fornecedores.cnpj      → UNIQUE (tenant_id, cnpj)
--   - equipamentos.numero_serie → UNIQUE (tenant_id, numero_serie)
--
-- IMPORTANTE:
--   * O runner trata "check that column/key exists" e "Duplicate key name"
--     como idempotente (atualizado em P2.6.1).
--   * Defaults cpf_cnpj/email NULL continuam permitidos (NULL não viola UNIQUE).
-- ============================================================================

-- ----------------------------------------------------------------------------
-- users.username — composto
-- ----------------------------------------------------------------------------
-- 0001 criou uk_users_username (UNIQUE global). Dropamos e criamos composto.
ALTER TABLE users DROP INDEX uk_users_username;
ALTER TABLE users ADD UNIQUE KEY uk_users_username_tenant (tenant_id, username);

-- ----------------------------------------------------------------------------
-- clientes.cpf_cnpj — composto
-- ----------------------------------------------------------------------------
-- 0001 criou apenas idx_clientes_cpf_cnpj (não UNIQUE). Agora composto UNIQUE.
ALTER TABLE clientes DROP INDEX idx_clientes_cpf_cnpj;
ALTER TABLE clientes ADD UNIQUE KEY uk_clientes_cpfcnpj_tenant (tenant_id, cpf_cnpj);

-- ----------------------------------------------------------------------------
-- clientes.email — busca por tenant
-- ----------------------------------------------------------------------------
ALTER TABLE clientes ADD INDEX idx_clientes_email_tenant (tenant_id, email);

-- ----------------------------------------------------------------------------
-- fornecedores.cnpj — composto
-- ----------------------------------------------------------------------------
-- 0001 criou uk_fornecedores_cnpj (UNIQUE global). Dropamos e criamos composto.
ALTER TABLE fornecedores DROP INDEX uk_fornecedores_cnpj;
ALTER TABLE fornecedores ADD UNIQUE KEY uk_fornecedores_cnpj_tenant (tenant_id, cnpj);

-- ----------------------------------------------------------------------------
-- equipamentos.numero_serie — composto
-- ----------------------------------------------------------------------------
-- 0005 criou uk_equipamentos_numero_serie (UNIQUE global). Dropamos e criamos composto.
ALTER TABLE equipamentos DROP INDEX uk_equipamentos_numero_serie;
ALTER TABLE equipamentos ADD UNIQUE KEY uk_equipamentos_serie_tenant (tenant_id, numero_serie);
