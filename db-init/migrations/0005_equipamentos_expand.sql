-- ============================================================================
-- Migration 0005: Expansão da tabela `equipamentos`
-- ----------------------------------------------------------------------------
-- Adiciona colunas à tabela `equipamentos` criada em 0001:
--   - patrimonio: número de patrimônio interno
--   - observacao: texto livre (ex: "carregador com defeito")
--   - updated_at: para auditoria de alterações
--   - tenant_id:  preparado para multi-tenant
--   - tipo: ENUM (NOTEBOOK, DESKTOP, IMPRESSORA, CELULAR, SERVIDOR, MONITOR, TABLET, OUTRO)
--
-- MySQL 8 não suporta `ADD COLUMN IF NOT EXISTS` nativamente.
-- Usamos uma estratégia compatível com o runner de migrations:
-- ALTERs são idempotentes porque a operação `ADD COLUMN` em uma coluna
-- existente retorna erro específico que o app trata como "já existe".
--
-- IMPORTANTE: esta migration é executada APÓS a 0004 e assume que a
-- tabela `equipamentos` (de 0001) já existe.
-- ============================================================================

ALTER TABLE equipamentos ADD COLUMN patrimonio VARCHAR(100);
ALTER TABLE equipamentos ADD COLUMN observacao TEXT;
ALTER TABLE equipamentos ADD COLUMN updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP;
ALTER TABLE equipamentos ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE equipamentos ADD COLUMN tipo ENUM('NOTEBOOK', 'DESKTOP', 'IMPRESSORA', 'CELULAR', 'SERVIDOR', 'MONITOR', 'TABLET', 'OUTRO') NOT NULL DEFAULT 'OUTRO';

CREATE INDEX idx_equipamentos_tipo ON equipamentos(tipo);
CREATE INDEX idx_equipamentos_patrimonio ON equipamentos(patrimonio);
CREATE INDEX idx_equipamentos_tenant ON equipamentos(tenant_id);
