-- ============================================================================
-- Migration 0015: Índices compostos tenant-aware (Sprint P2.6.2a — Hardening)
-- ----------------------------------------------------------------------------
-- Complementa os índices da P2.6.1 (0014) com compostos que faltaram.
-- Foco em queries que ficaram lentas após P2.6.1.
--
-- Cria índices em:
--   - cliente_timeline_eventos
--   - cliente_observacoes
--   - workflow_movimentacoes
--   - lancamentos
--   - ordem_servico_pecas
--   - ordem_servico_servicos
--   - historico_edicoes
--   - cliente_contatos
--   - cliente_tag_atribuicoes
-- ============================================================================

-- Cliente timeline (dashboard 360° + listar_timeline)
CREATE INDEX IF NOT EXISTS idx_timeline_tenant_cliente_data
    ON cliente_timeline_eventos (tenant_id, cliente_id, data_evento DESC);

-- Observações por cliente (listar_observacoes)
CREATE INDEX IF NOT EXISTS idx_obs_tenant_cliente_data
    ON cliente_observacoes (tenant_id, cliente_id, data_criacao DESC);

-- Histórico de workflow (historico_workflow)
CREATE INDEX IF NOT EXISTS idx_wf_mov_tenant_os_data
    ON workflow_movimentacoes (tenant_id, ordem_servico_id, data_movimentacao DESC);

-- Lançamentos por conta
CREATE INDEX IF NOT EXISTS idx_lanc_tenant_cr
    ON lancamentos (tenant_id, conta_receber_id);
CREATE INDEX IF NOT EXISTS idx_lanc_tenant_cp
    ON lancamentos (tenant_id, conta_pagar_id);
CREATE INDEX IF NOT EXISTS idx_lanc_tenant_competencia
    ON lancamentos (tenant_id, data_competencia);

-- OS pivot tables
CREATE INDEX IF NOT EXISTS idx_os_pecas_tenant_os
    ON ordem_servico_pecas (tenant_id, ordem_servico_id);
CREATE INDEX IF NOT EXISTS idx_os_serv_tenant_os
    ON ordem_servico_servicos (tenant_id, ordem_servico_id);

-- Histórico de edições da OS
CREATE INDEX IF NOT EXISTS idx_hist_tenant_os
    ON historico_edicoes (tenant_id, ordem_servico_id);

-- Contatos por cliente
CREATE INDEX IF NOT EXISTS idx_contatos_tenant_cliente
    ON cliente_contatos (tenant_id, cliente_id);

-- Tags (pivot)
CREATE INDEX IF NOT EXISTS idx_tag_atr_tenant_tag
    ON cliente_tag_atribuicoes (tenant_id, tag_id);

-- Workflows
CREATE INDEX IF NOT EXISTS idx_wf_def_tenant
    ON workflow_definicoes (tenant_id);
CREATE INDEX IF NOT EXISTS idx_wf_estados_tenant_def
    ON workflow_estados (tenant_id, workflow_definicao_id);

-- SLA + Alertas
CREATE INDEX IF NOT EXISTS idx_sla_config_tenant
    ON sla_config (tenant_id);
CREATE INDEX IF NOT EXISTS idx_alertas_tenant
    ON alertas (tenant_id);
