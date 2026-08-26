-- ============================================================================
-- Migration 0016: tenant_id em todas as tabelas financeiras (P2.6.2a)
-- ----------------------------------------------------------------------------
-- Garante 100% cobertura de tenant_id no módulo financeiro.
-- Defense in depth — toda query de repository deve filtrar por tenant_id.
-- ============================================================================

-- configuracao_financeira (singleton por tenant)
ALTER TABLE configuracao_financeira
    ADD COLUMN tenant_id INT NOT NULL DEFAULT 1 AFTER id,
    ADD INDEX idx_cfgfin_tenant (tenant_id),
    ADD UNIQUE KEY uk_cfgfin_tenant (tenant_id);

-- plano_contas
ALTER TABLE plano_contas
    ADD COLUMN tenant_id INT NOT NULL DEFAULT 1,
    ADD INDEX idx_plano_contas_tenant (tenant_id),
    ADD INDEX idx_plano_contas_tenant_codigo (tenant_id, codigo);

-- centros_custo
ALTER TABLE centros_custo
    ADD COLUMN tenant_id INT NOT NULL DEFAULT 1,
    ADD INDEX idx_centros_custo_tenant (tenant_id),
    ADD INDEX idx_centros_custo_tenant_codigo (tenant_id, codigo);

-- contas_receber
ALTER TABLE contas_receber
    ADD COLUMN tenant_id INT NOT NULL DEFAULT 1 AFTER id,
    ADD INDEX idx_contas_receber_tenant (tenant_id),
    ADD INDEX idx_contas_receber_tenant_cliente (tenant_id, cliente_id),
    ADD INDEX idx_contas_receber_tenant_status (tenant_id, status),
    ADD INDEX idx_contas_receber_tenant_vencimento (tenant_id, vencimento);

-- contas_pagar
ALTER TABLE contas_pagar
    ADD COLUMN tenant_id INT NOT NULL DEFAULT 1 AFTER id,
    ADD INDEX idx_contas_pagar_tenant (tenant_id),
    ADD INDEX idx_contas_pagar_tenant_status (tenant_id, status),
    ADD INDEX idx_contas_pagar_tenant_vencimento (tenant_id, vencimento);

-- lancamentos
ALTER TABLE lancamentos
    ADD COLUMN tenant_id INT NOT NULL DEFAULT 1,
    ADD INDEX idx_lancamentos_tenant (tenant_id),
    ADD INDEX idx_lancamentos_tenant_cr (tenant_id, conta_receber_id),
    ADD INDEX idx_lancamentos_tenant_cp (tenant_id, conta_pagar_id),
    ADD INDEX idx_lancamentos_tenant_data (tenant_id, data_lancamento);

-- alertas_financeiros
ALTER TABLE alertas_financeiros
    ADD COLUMN tenant_id INT NOT NULL DEFAULT 1,
    ADD INDEX idx_alertas_fin_tenant (tenant_id);

-- alertas_financeiros_ocorrencias
ALTER TABLE alertas_financeiros_ocorrencias
    ADD COLUMN tenant_id INT NOT NULL DEFAULT 1,
    ADD INDEX idx_alertas_fin_ocorr_tenant (tenant_id),
    ADD INDEX idx_alertas_fin_ocorr_tenant_resolvido (tenant_id, resolvido);
