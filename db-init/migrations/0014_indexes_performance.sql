-- ============================================================================
-- Migration 0014: Índices compostos para performance (Sprint P2.6.1)
-- ----------------------------------------------------------------------------
-- Cria índices compostos (tenant_id, ...) para as queries mais frequentes.
-- Sem isso, queries filtradas por tenant_id fariam scan + filtro.
--
-- Padrão: (tenant_id, coluna_mais_filtrada, ...)
-- ----------------------------------------------------------------------------

-- ----------------------------------------------------------------------------
-- CRM — clientes
-- ----------------------------------------------------------------------------
ALTER TABLE clientes ADD INDEX idx_clientes_tenant_nome (tenant_id, nome);
ALTER TABLE clientes ADD INDEX idx_clientes_tenant_created (tenant_id, created_at);

-- equipamentos
ALTER TABLE equipamentos ADD INDEX idx_equipamentos_tenant_cliente (tenant_id, cliente_id);

-- cliente_timeline_eventos
ALTER TABLE cliente_timeline_eventos ADD INDEX idx_cliente_timeline_tenant_cliente (tenant_id, cliente_id);
ALTER TABLE cliente_timeline_eventos ADD INDEX idx_cliente_timeline_tenant_created (tenant_id, data_evento);

-- ----------------------------------------------------------------------------
-- OS — ordens_servico
-- ----------------------------------------------------------------------------
ALTER TABLE ordens_servico ADD INDEX idx_os_tenant_status (tenant_id, status);
ALTER TABLE ordens_servico ADD INDEX idx_os_tenant_cliente (tenant_id, cliente_id);
ALTER TABLE ordens_servico ADD INDEX idx_os_tenant_data_chegada (tenant_id, data_chegada);

-- ----------------------------------------------------------------------------
-- Cotação / Orçamento
-- ----------------------------------------------------------------------------
ALTER TABLE orcamentos ADD INDEX idx_orcamentos_tenant_cliente (tenant_id, cliente_id);
ALTER TABLE orcamentos ADD INDEX idx_orcamentos_tenant_created (tenant_id, data_criacao);
ALTER TABLE cotacoes ADD INDEX idx_cotacoes_tenant_cliente (tenant_id, cliente_id);

-- ----------------------------------------------------------------------------
-- Financeiro
-- ----------------------------------------------------------------------------
ALTER TABLE contas_receber ADD INDEX idx_contas_receber_tenant_status (tenant_id, status);
ALTER TABLE contas_receber ADD INDEX idx_contas_receber_tenant_vencimento (tenant_id, vencimento);
ALTER TABLE contas_receber ADD INDEX idx_contas_receber_tenant_cliente (tenant_id, cliente_id);

ALTER TABLE contas_pagar ADD INDEX idx_contas_pagar_tenant_status (tenant_id, status);
ALTER TABLE contas_pagar ADD INDEX idx_contas_pagar_tenant_vencimento (tenant_id, vencimento);

ALTER TABLE lancamentos ADD INDEX idx_lancamentos_tenant_data (tenant_id, data_lancamento);
ALTER TABLE lancamentos ADD INDEX idx_lancamentos_tenant_tipo (tenant_id, tipo);

-- ----------------------------------------------------------------------------
-- Estoque
-- ----------------------------------------------------------------------------
ALTER TABLE pecas ADD INDEX idx_pecas_tenant_descricao (tenant_id, descricao);
ALTER TABLE movimentos_estoque ADD INDEX idx_mov_tenant_peca (tenant_id, peca_id);

-- ----------------------------------------------------------------------------
-- Operations
-- ----------------------------------------------------------------------------
ALTER TABLE workflow_movimentacoes ADD INDEX idx_wf_mov_tenant_os (tenant_id, ordem_servico_id);
ALTER TABLE eventos_agenda ADD INDEX idx_eventos_agenda_tenant_data (tenant_id, data_inicio);
ALTER TABLE alertas_ocorrencias ADD INDEX idx_alertas_ocorr_tenant_status (tenant_id, status);

-- ----------------------------------------------------------------------------
-- Arquivos
-- ----------------------------------------------------------------------------
ALTER TABLE arquivos ADD INDEX idx_arquivos_tenant_entidade (tenant_id, tipo_entidade, entidade_id);
ALTER TABLE arquivos ADD INDEX idx_arquivos_tenant_created (tenant_id, data_upload);

-- ----------------------------------------------------------------------------
-- Audit
-- ----------------------------------------------------------------------------
ALTER TABLE audit_log ADD INDEX idx_audit_tenant_usuario (tenant_id, usuario_id);
ALTER TABLE audit_log ADD INDEX idx_audit_tenant_acao (tenant_id, acao);
ALTER TABLE audit_log ADD INDEX idx_audit_tenant_data (tenant_id, data_hora);
