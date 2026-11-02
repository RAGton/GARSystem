-- ============================================================================
-- Migration 0019: Unificação do ciclo de vida da OS (etapa) e vínculo com Orçamento
-- ----------------------------------------------------------------------------
-- 1. Adiciona coluna `etapa` na tabela `ordens_servico` com máquina de estados única.
-- 2. Backfill inicial a partir de `status` e `situacao`.
-- 3. Adiciona rastreamento de baixa atômica de estoque em `ordem_servico_pecas`.
-- 4. Adiciona `ordem_servico_id` em `orcamentos`.
-- ============================================================================

ALTER TABLE ordens_servico
    ADD COLUMN etapa ENUM(
        'RECEBIDO',
        'EM_DIAGNOSTICO',
        'ORCAMENTO_GERADO',
        'APROVADO',
        'EM_EXECUCAO',
        'AGUARDANDO_PECA',
        'TESTES_CONTROLE_QUALIDADE',
        'PRONTO_RETIRADA',
        'FINALIZADO',
        'CANCELADO',
        'REPROVADO'
    ) NOT NULL DEFAULT 'RECEBIDO' AFTER situacao,
    ADD INDEX idx_ordens_servico_etapa (tenant_id, etapa);

UPDATE ordens_servico SET etapa = 'CANCELADO' WHERE status = 'Cancelada';
UPDATE ordens_servico SET etapa = 'FINALIZADO' WHERE status = 'Finalizada';
UPDATE ordens_servico SET etapa = 'REPROVADO' WHERE situacao = 'Reprovado' AND status NOT IN ('Finalizada', 'Cancelada');
UPDATE ordens_servico SET etapa = 'APROVADO' WHERE situacao = 'Aprovado' AND status = 'Aberta';
UPDATE ordens_servico SET etapa = 'EM_EXECUCAO' WHERE (situacao = 'EmAndamento' OR status = 'EmAndamento') AND status NOT IN ('Finalizada', 'Cancelada');
UPDATE ordens_servico SET etapa = 'AGUARDANDO_PECA' WHERE (situacao = 'AutorizadoAguardandoPeca' OR status = 'AguardandoPeca') AND status NOT IN ('Finalizada', 'Cancelada');
UPDATE ordens_servico SET etapa = 'TESTES_CONTROLE_QUALIDADE' WHERE situacao = 'ServicoConcluido' AND status NOT IN ('Finalizada', 'Cancelada');
UPDATE ordens_servico SET etapa = 'PRONTO_RETIRADA' WHERE situacao = 'AguardandoRetirada' AND status NOT IN ('Finalizada', 'Cancelada');
UPDATE ordens_servico SET etapa = 'ORCAMENTO_GERADO' WHERE situacao = 'AguardandoAutorizacao' AND status NOT IN ('Finalizada', 'Cancelada');
UPDATE ordens_servico SET etapa = 'EM_DIAGNOSTICO' WHERE situacao = 'Orcamento' AND status = 'Aberta';

ALTER TABLE ordem_servico_pecas
    ADD COLUMN baixado BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN baixado_em DATETIME NULL,
    ADD INDEX idx_os_pecas_baixado (tenant_id, baixado);

ALTER TABLE orcamentos
    ADD COLUMN ordem_servico_id INT NULL AFTER cotacao_origem_id,
    ADD INDEX idx_orcamentos_ordem_servico (tenant_id, ordem_servico_id);
