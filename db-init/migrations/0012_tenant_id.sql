-- ============================================================================
-- Migration 0012: tenant_id em todas as tabelas de negócio (Sprint P2.6.1)
-- ----------------------------------------------------------------------------
-- Adiciona a coluna `tenant_id INT NOT NULL DEFAULT 0` + índice em todas as
-- tabelas de negócio. Faz backfill para `tenant_id = 1` (empresa padrão).
--
-- IMPORTANTE:
--   * Idempotente: o runner trata "Duplicate column name" e
--     "Duplicate key name" como "já aplicado".
--   * DEFAULT 0 é temporário para que o ADD COLUMN não falhe em linhas
--     existentes. Imediatamente após, o UPDATE zera tudo para 1.
--   * NÃO mexe em: schema_migrations, empresa, empresa_configuracao,
--     permissions, roles, role_permissions (são sistema-wide).
-- ============================================================================

-- ----------------------------------------------------------------------------
-- AUDITORIA / LOG
-- ----------------------------------------------------------------------------
ALTER TABLE audit_log ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE audit_log ADD INDEX idx_audit_log_tenant (tenant_id);

-- ----------------------------------------------------------------------------
-- CRM (já tem em clientes/equipamentos/ordens_servico — rest ainda falta)
-- ----------------------------------------------------------------------------
ALTER TABLE cliente_contatos ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE cliente_contatos ADD INDEX idx_cliente_contatos_tenant (tenant_id);

ALTER TABLE cliente_anexos ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE cliente_anexos ADD INDEX idx_cliente_anexos_tenant (tenant_id);

ALTER TABLE cliente_observacoes ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE cliente_observacoes ADD INDEX idx_cliente_observacoes_tenant (tenant_id);

ALTER TABLE cliente_observacao_edicoes ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE cliente_observacao_edicoes ADD INDEX idx_cliente_observ_edicoes_tenant (tenant_id);

ALTER TABLE cliente_tags ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE cliente_tags ADD INDEX idx_cliente_tags_tenant (tenant_id);

ALTER TABLE cliente_tag_atribuicoes ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE cliente_tag_atribuicoes ADD INDEX idx_cliente_tag_atr_tenant (tenant_id);

ALTER TABLE cliente_timeline_eventos ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE cliente_timeline_eventos ADD INDEX idx_cliente_timeline_tenant (tenant_id);

-- ----------------------------------------------------------------------------
-- OS (já tem em ordens_servico — rest ainda falta)
-- ----------------------------------------------------------------------------
ALTER TABLE historico_edicoes ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE historico_edicoes ADD INDEX idx_historico_edicoes_tenant (tenant_id);

ALTER TABLE ordem_servico_pecas ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE ordem_servico_pecas ADD INDEX idx_os_pecas_tenant (tenant_id);

ALTER TABLE ordem_servico_servicos ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE ordem_servico_servicos ADD INDEX idx_os_servicos_tenant (tenant_id);

ALTER TABLE os_checklist_templates ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE os_checklist_templates ADD INDEX idx_os_checklist_tpl_tenant (tenant_id);

ALTER TABLE os_checklist_template_itens ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE os_checklist_template_itens ADD INDEX idx_os_checklist_tpl_itens_tenant (tenant_id);

ALTER TABLE os_checklists ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE os_checklists ADD INDEX idx_os_checklists_tenant (tenant_id);

ALTER TABLE os_checklist_itens ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE os_checklist_itens ADD INDEX idx_os_checklist_itens_tenant (tenant_id);

ALTER TABLE os_evolucoes ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE os_evolucoes ADD INDEX idx_os_evolucoes_tenant (tenant_id);

ALTER TABLE os_auditoria_campo ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE os_auditoria_campo ADD INDEX idx_os_auditoria_tenant (tenant_id);

ALTER TABLE os_assinaturas ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE os_assinaturas ADD INDEX idx_os_assinaturas_tenant (tenant_id);

-- ----------------------------------------------------------------------------
-- ESTOQUE / NF
-- ----------------------------------------------------------------------------
ALTER TABLE pecas ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE pecas ADD INDEX idx_pecas_tenant (tenant_id);

ALTER TABLE fornecedores ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE fornecedores ADD INDEX idx_fornecedores_tenant (tenant_id);

ALTER TABLE notas_fiscais_entrada ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE notas_fiscais_entrada ADD INDEX idx_nf_entrada_tenant (tenant_id);

ALTER TABLE nf_entrada_pecas ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE nf_entrada_pecas ADD INDEX idx_nf_entrada_pecas_tenant (tenant_id);

ALTER TABLE movimentos_estoque ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE movimentos_estoque ADD INDEX idx_movimentos_estoque_tenant (tenant_id);

-- ----------------------------------------------------------------------------
-- COTAÇÃO / ORÇAMENTO
-- ----------------------------------------------------------------------------
ALTER TABLE cotacoes ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE cotacoes ADD INDEX idx_cotacoes_tenant (tenant_id);

ALTER TABLE cotacao_itens ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE cotacao_itens ADD INDEX idx_cotacao_itens_tenant (tenant_id);

ALTER TABLE cotacao_anexos ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE cotacao_anexos ADD INDEX idx_cotacao_anexos_tenant (tenant_id);

ALTER TABLE orcamentos ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE orcamentos ADD INDEX idx_orcamentos_tenant (tenant_id);

ALTER TABLE orcamento_items ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE orcamento_items ADD INDEX idx_orcamento_items_tenant (tenant_id);

ALTER TABLE orcamento_aprovacoes ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE orcamento_aprovacoes ADD INDEX idx_orcamento_aprovacoes_tenant (tenant_id);

ALTER TABLE orcamento_historico ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE orcamento_historico ADD INDEX idx_orcamento_historico_tenant (tenant_id);

ALTER TABLE servicos ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE servicos ADD INDEX idx_servicos_tenant (tenant_id);

-- ----------------------------------------------------------------------------
-- ARQUIVOS / TRANSCRIÇÃO
-- ----------------------------------------------------------------------------
ALTER TABLE arquivos ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE arquivos ADD INDEX idx_arquivos_tenant (tenant_id);

ALTER TABLE arquivo_thumbnails ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE arquivo_thumbnails ADD INDEX idx_arquivo_thumbnails_tenant (tenant_id);

ALTER TABLE arquivo_vinculos ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE arquivo_vinculos ADD INDEX idx_arquivo_vinculos_tenant (tenant_id);

ALTER TABLE transcricoes ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE transcricoes ADD INDEX idx_transcricoes_tenant (tenant_id);

-- ----------------------------------------------------------------------------
-- OPERAÇÕES
-- ----------------------------------------------------------------------------
ALTER TABLE workflow_definicoes ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE workflow_definicoes ADD INDEX idx_workflow_def_tenant (tenant_id);

ALTER TABLE workflow_estados ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE workflow_estados ADD INDEX idx_workflow_estados_tenant (tenant_id);

ALTER TABLE workflow_transicoes ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE workflow_transicoes ADD INDEX idx_workflow_trans_tenant (tenant_id);

ALTER TABLE workflow_movimentacoes ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE workflow_movimentacoes ADD INDEX idx_workflow_mov_tenant (tenant_id);

ALTER TABLE agendas ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE agendas ADD INDEX idx_agendas_tenant (tenant_id);

ALTER TABLE eventos_agenda ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE eventos_agenda ADD INDEX idx_eventos_agenda_tenant (tenant_id);

ALTER TABLE sla_config ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE sla_config ADD INDEX idx_sla_config_tenant (tenant_id);

ALTER TABLE sla_calculos ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE sla_calculos ADD INDEX idx_sla_calculos_tenant (tenant_id);

ALTER TABLE sla_eventos ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE sla_eventos ADD INDEX idx_sla_eventos_tenant (tenant_id);

ALTER TABLE alertas ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE alertas ADD INDEX idx_alertas_tenant (tenant_id);

ALTER TABLE alertas_ocorrencias ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE alertas_ocorrencias ADD INDEX idx_alertas_ocorr_tenant (tenant_id);

-- ----------------------------------------------------------------------------
-- FINANCEIRO
-- ----------------------------------------------------------------------------
ALTER TABLE plano_contas ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE plano_contas ADD INDEX idx_plano_contas_tenant (tenant_id);

ALTER TABLE centros_custo ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE centros_custo ADD INDEX idx_centros_custo_tenant (tenant_id);

ALTER TABLE contas_receber ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE contas_receber ADD INDEX idx_contas_receber_tenant (tenant_id);

ALTER TABLE contas_pagar ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE contas_pagar ADD INDEX idx_contas_pagar_tenant (tenant_id);

ALTER TABLE lancamentos ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE lancamentos ADD INDEX idx_lancamentos_tenant (tenant_id);

ALTER TABLE alertas_financeiros ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE alertas_financeiros ADD INDEX idx_alertas_fin_tenant (tenant_id);

ALTER TABLE alertas_financeiros_ocorrencias ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE alertas_financeiros_ocorrencias ADD INDEX idx_alertas_fin_ocorr_tenant (tenant_id);

-- configuracao_financeira é SINGLETON hoje (id=1). Será refatorada em
-- P2.6.2 para empresa_configuracao_financeira. Por ora, apenas adiciona
-- tenant_id.
ALTER TABLE configuracao_financeira ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE configuracao_financeira ADD INDEX idx_config_financeira_tenant (tenant_id);

-- ----------------------------------------------------------------------------
-- LEGADO (movimentacoes — crédito de cliente, P0)
-- ----------------------------------------------------------------------------
ALTER TABLE movimentacoes ADD COLUMN tenant_id INT NOT NULL DEFAULT 0;
ALTER TABLE movimentacoes ADD INDEX idx_movimentacoes_tenant (tenant_id);

-- ----------------------------------------------------------------------------
-- BACKFILL — todos os dados legados vão para tenant 1 (empresa padrão)
-- ----------------------------------------------------------------------------
-- Esta migration roda APÓS a 0011 que cria empresa 1.
-- Para cada tabela, se tenant_id = 0, setar para 1.
-- Tabelas com `tenant_id` preexistente (clientes, users, ordens_servico,
-- equipamentos) também passam pelo backfill.
-- ----------------------------------------------------------------------------

UPDATE audit_log SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE cliente_contatos SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE cliente_anexos SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE cliente_observacoes SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE cliente_observacao_edicoes SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE cliente_tags SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE cliente_tag_atribuicoes SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE cliente_timeline_eventos SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE clientes SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE equipamentos SET tenant_id = 1 WHERE tenant_id = 0;

UPDATE historico_edicoes SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE ordem_servico_pecas SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE ordem_servico_servicos SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE os_checklist_templates SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE os_checklist_template_itens SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE os_checklists SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE os_checklist_itens SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE os_evolucoes SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE os_auditoria_campo SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE os_assinaturas SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE ordens_servico SET tenant_id = 1 WHERE tenant_id = 0;

UPDATE pecas SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE fornecedores SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE notas_fiscais_entrada SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE nf_entrada_pecas SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE movimentos_estoque SET tenant_id = 1 WHERE tenant_id = 0;

UPDATE cotacoes SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE cotacao_itens SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE cotacao_anexos SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE orcamentos SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE orcamento_items SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE orcamento_aprovacoes SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE orcamento_historico SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE servicos SET tenant_id = 1 WHERE tenant_id = 0;

UPDATE arquivos SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE arquivo_thumbnails SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE arquivo_vinculos SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE transcricoes SET tenant_id = 1 WHERE tenant_id = 0;

UPDATE workflow_definicoes SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE workflow_estados SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE workflow_transicoes SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE workflow_movimentacoes SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE agendas SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE eventos_agenda SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE sla_config SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE sla_calculos SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE sla_eventos SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE alertas SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE alertas_ocorrencias SET tenant_id = 1 WHERE tenant_id = 0;

UPDATE plano_contas SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE centros_custo SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE contas_receber SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE contas_pagar SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE lancamentos SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE alertas_financeiros SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE alertas_financeiros_ocorrencias SET tenant_id = 1 WHERE tenant_id = 0;
UPDATE configuracao_financeira SET tenant_id = 1 WHERE tenant_id = 0;

UPDATE movimentacoes SET tenant_id = 1 WHERE tenant_id = 0;
