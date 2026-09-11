-- ============================================================================
-- Migration 0002: campos/tabelas que faltavam no schema antigo
-- ----------------------------------------------------------------------------
-- Idempotente — pode rodar várias vezes sem efeito.
-- Cobre:
--   1. Tabela `movimentacoes` (consultada em cliente.rs mas não existia).
--   2. Coluna `credito` em `clientes` (consultada em cliente.rs).
--   3. Coluna `tenant_id` em tabelas relevantes (preparação multi-tenant).
-- ============================================================================

-- ----------------------------------------------------------------------------
-- 1. Tabela `movimentacoes` (ledger de gastos por cliente)
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS movimentacoes (
    id          INT AUTO_INCREMENT PRIMARY KEY,
    cliente_id  INT NOT NULL,
    os_id       INT,
    tipo        ENUM('OS', 'Pagamento', 'Credito', 'Estorno') NOT NULL DEFAULT 'OS',
    valor       DECIMAL(12,2) NOT NULL,
    descricao   VARCHAR(255),
    usuario     VARCHAR(255) NOT NULL,
    data_movimento DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (cliente_id) REFERENCES clientes(id),
    FOREIGN KEY (os_id)      REFERENCES ordens_servico(id),
    INDEX idx_movcli_cliente (cliente_id, data_movimento),
    INDEX idx_movcli_os (os_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ----------------------------------------------------------------------------
-- 2. Coluna `credito` em `clientes`
-- ----------------------------------------------------------------------------
-- Para MySQL 8, ADD COLUMN IF NOT EXISTS não existe até 8.0.29.
-- Usamos INFORMATION_SCHEMA para checar antes.
SET @col_exists = (
    SELECT COUNT(*)
    FROM INFORMATION_SCHEMA.COLUMNS
    WHERE TABLE_SCHEMA = DATABASE()
      AND TABLE_NAME = 'clientes'
      AND COLUMN_NAME = 'credito'
);
SET @sql = IF(
    @col_exists = 0,
    "ALTER TABLE clientes ADD COLUMN credito DECIMAL(12,2) NOT NULL DEFAULT 0.00 AFTER cpf_cnpj",
    "SELECT 'coluna credito já existe' AS info"
);
PREPARE stmt FROM @sql;
EXECUTE stmt;
DEALLOCATE PREPARE stmt;

-- ----------------------------------------------------------------------------
-- 3. Coluna `tenant_id` (preparação multi-tenant)
-- ----------------------------------------------------------------------------
SET @col_users = (SELECT COUNT(*) FROM INFORMATION_SCHEMA.COLUMNS WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'users' AND COLUMN_NAME = 'tenant_id');
SET @sql_users = IF(@col_users = 0, "ALTER TABLE users ADD COLUMN tenant_id INT NOT NULL DEFAULT 0 AFTER id", "SELECT 'users tenant_id ok'");
PREPARE stmt_u FROM @sql_users;
EXECUTE stmt_u;
DEALLOCATE PREPARE stmt_u;

SET @col_clientes = (SELECT COUNT(*) FROM INFORMATION_SCHEMA.COLUMNS WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'clientes' AND COLUMN_NAME = 'tenant_id');
SET @sql_clientes = IF(@col_clientes = 0, "ALTER TABLE clientes ADD COLUMN tenant_id INT NOT NULL DEFAULT 0 AFTER id", "SELECT 'clientes tenant_id ok'");
PREPARE stmt_c FROM @sql_clientes;
EXECUTE stmt_c;
DEALLOCATE PREPARE stmt_c;

SET @col_os = (SELECT COUNT(*) FROM INFORMATION_SCHEMA.COLUMNS WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'ordens_servico' AND COLUMN_NAME = 'tenant_id');
SET @sql_os = IF(@col_os = 0, "ALTER TABLE ordens_servico ADD COLUMN tenant_id INT NOT NULL DEFAULT 0 AFTER id", "SELECT 'os tenant_id ok'");
PREPARE stmt_os FROM @sql_os;
EXECUTE stmt_os;
DEALLOCATE PREPARE stmt_os;

