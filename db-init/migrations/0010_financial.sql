-- db-init/migrations/0010_financial.sql
-- Sprint P2.5 — Financeiro Básico (fundação).
--
-- Tabelas:
--   configuracao_financeira   (singleton)
--   plano_contas              (categorias de receita/despesa)
--   centros_custo             (departamentos/projetos)
--   contas_receber            (cliente → nós)
--   contas_pagar              (nós → fornecedor)
--   lancamentos               (append-only, imutável)
--   alertas_financeiros       (regras)
--   alertas_financeiros_ocorrencias (log)
--
-- Decisões:
--   - Valores em BIGINT (centavos) para precisão monetária
--   - Lançamentos append-only (estorno = novo lançamento inverso)
--   - Centro de custo sempre obrigatório
--   - Multi-tenant virá em P2.6 (não implementado aqui)

CREATE TABLE IF NOT EXISTS configuracao_financeira (
    id INT UNSIGNED NOT NULL DEFAULT 1,
    dias_vencimento_padrao INT UNSIGNED NOT NULL DEFAULT 7,
    centro_custo_padrao_receber_id INT UNSIGNED NULL,
    plano_conta_padrao_receber_id INT UNSIGNED NULL,
    centro_custo_padrao_pagar_id INT UNSIGNED NULL,
    plano_conta_padrao_pagar_id INT UNSIGNED NULL,
    moeda_padrao CHAR(3) NOT NULL DEFAULT 'BRL',
    data_atualizacao DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Insere config default
INSERT IGNORE INTO configuracao_financeira (id) VALUES (1);

-- =============================================================================
-- Plano de contas (seed mínimo, expansível via API em P2.5.x)
-- =============================================================================
CREATE TABLE IF NOT EXISTS plano_contas (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    codigo VARCHAR(32) NOT NULL,
    nome VARCHAR(180) NOT NULL,
    tipo ENUM('RECEITA','DESPESA','INTERNO','IMOBILIZADO') NOT NULL,
    pai_id INT UNSIGNED NULL,
    ativo BOOLEAN NOT NULL DEFAULT TRUE,
    descricao VARCHAR(500) NULL,
    data_criacao DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    UNIQUE KEY uk_plano_codigo (codigo),
    KEY idx_plano_tipo (tipo),
    KEY idx_plano_pai (pai_id),
    KEY idx_plano_ativo (ativo)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

INSERT IGNORE INTO plano_contas (id, codigo, nome, tipo) VALUES
    (1, '1.1.1', 'Manutenção',         'RECEITA'),
    (2, '1.1.2', 'Instalação',         'RECEITA'),
    (3, '1.1.3', 'Consultoria',        'RECEITA'),
    (4, '1.2.1', 'Venda de Peças',     'RECEITA'),
    (5, '2.1.1', 'Compra de Peças',    'DESPESA'),
    (6, '2.1.2', 'Material',           'DESPESA'),
    (7, '2.1.3', 'Serviços Terceiros', 'DESPESA'),
    (8, '2.3.1', 'Aluguel',            'DESPESA'),
    (9, '2.3.2', 'Energia',            'DESPESA'),
    (10, '2.3.3', 'Internet',          'DESPESA'),
    (11, '2.3.4', 'Contabilidade',     'DESPESA'),
    (12, '2.4.1', 'ISS',               'DESPESA'),
    (13, '3.1',   'Transferências',    'INTERNO');

-- =============================================================================
-- Centro de Custo (5 seeds)
-- =============================================================================
CREATE TABLE IF NOT EXISTS centros_custo (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    codigo VARCHAR(32) NOT NULL,
    nome VARCHAR(180) NOT NULL,
    descricao VARCHAR(500) NULL,
    ativo BOOLEAN NOT NULL DEFAULT TRUE,
    data_criacao DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    UNIQUE KEY uk_centro_codigo (codigo),
    KEY idx_centro_ativo (ativo)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

INSERT IGNORE INTO centros_custo (id, codigo, nome, descricao) VALUES
    (1, 'LAB',   'LABORATORIO',     'Atendimento em bancada/laboratório'),
    (2, 'CAMPO', 'CAMPO',           'Atendimento externo / técnico em campo'),
    (3, 'COM',   'COMERCIAL',       'Vendas e prospecção'),
    (4, 'ADM',   'ADMINISTRATIVO',  'Administrativo, financeiro, RH'),
    (5, 'INF',   'INFRAESTRUTURA',  'TI, redes, infraestrutura física');

-- Atualiza config default com centro de custo e plano de conta
UPDATE configuracao_financeira
SET centro_custo_padrao_receber_id = 1,
    plano_conta_padrao_receber_id = 1,
    centro_custo_padrao_pagar_id = 4,
    plano_conta_padrao_pagar_id = 11
WHERE id = 1;

-- =============================================================================
-- Contas a Receber
-- =============================================================================
CREATE TABLE IF NOT EXISTS contas_receber (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    cliente_id INT UNSIGNED NOT NULL,
    origem_tipo ENUM('ORCAMENTO','OS','MANUAL','OUTRO') NOT NULL DEFAULT 'MANUAL',
    origem_id INT UNSIGNED NULL,
    descricao VARCHAR(255) NOT NULL,
    valor BIGINT NOT NULL,                          -- centavos
    valor_pago BIGINT NOT NULL DEFAULT 0,
    vencimento DATE NOT NULL,
    data_pagamento DATE NULL,
    data_competencia DATE NOT NULL,
    status ENUM('PENDENTE','PARCIAL','PAGO','CANCELADO') NOT NULL DEFAULT 'PENDENTE',
    centro_custo_id INT UNSIGNED NOT NULL,
    plano_conta_id INT UNSIGNED NOT NULL,
    observacao VARCHAR(500) NULL,
    usuario_criacao_id INT UNSIGNED NOT NULL,
    usuario_pagamento_id INT UNSIGNED NULL,
    data_criacao DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    data_atualizacao DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    KEY idx_cr_cliente (cliente_id, status),
    KEY idx_cr_status (status),
    KEY idx_cr_vencimento (vencimento),
    KEY idx_cr_competencia (data_competencia),
    KEY idx_cr_origem (origem_tipo, origem_id),
    KEY idx_cr_centro (centro_custo_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- =============================================================================
-- Contas a Pagar
-- =============================================================================
CREATE TABLE IF NOT EXISTS contas_pagar (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    fornecedor VARCHAR(255) NOT NULL,
    fornecedor_doc VARCHAR(32) NULL,
    descricao VARCHAR(255) NOT NULL,
    valor BIGINT NOT NULL,
    valor_pago BIGINT NOT NULL DEFAULT 0,
    vencimento DATE NOT NULL,
    data_pagamento DATE NULL,
    data_competencia DATE NOT NULL,
    status ENUM('PENDENTE','PARCIAL','PAGO','CANCELADO') NOT NULL DEFAULT 'PENDENTE',
    centro_custo_id INT UNSIGNED NOT NULL,
    plano_conta_id INT UNSIGNED NOT NULL,
    origem_tipo ENUM('COTACAO','MANUAL','OUTRO') NOT NULL DEFAULT 'MANUAL',
    origem_id INT UNSIGNED NULL,
    observacao VARCHAR(500) NULL,
    usuario_criacao_id INT UNSIGNED NOT NULL,
    usuario_pagamento_id INT UNSIGNED NULL,
    data_criacao DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    data_atualizacao DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    KEY idx_cp_fornecedor (fornecedor, status),
    KEY idx_cp_status (status),
    KEY idx_cp_vencimento (vencimento),
    KEY idx_cp_competencia (data_competencia),
    KEY idx_cp_origem (origem_tipo, origem_id),
    KEY idx_cp_centro (centro_custo_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- =============================================================================
-- Lançamentos (append-only)
-- =============================================================================
CREATE TABLE IF NOT EXISTS lancamentos (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    tipo ENUM('ENTRADA','SAIDA','TRANSFERENCIA','ESTORNO') NOT NULL,
    conta_receber_id INT UNSIGNED NULL,
    conta_pagar_id INT UNSIGNED NULL,
    valor BIGINT NOT NULL,                          -- sempre positivo; tipo define sinal
    data_lancamento DATE NOT NULL,
    data_competencia DATE NOT NULL,
    descricao VARCHAR(255) NOT NULL,
    plano_conta_id INT UNSIGNED NOT NULL,
    centro_custo_id INT UNSIGNED NOT NULL,
    forma_pagamento VARCHAR(32) NULL,                -- PIX, BOLETO, CARTAO, DINHEIRO, TRANSFERENCIA
    -- Estorno
    lancamento_estornado_id INT UNSIGNED NULL,
    lancamento_origem_id INT UNSIGNED NULL,
    -- Auditoria
    usuario_id INT UNSIGNED NOT NULL,
    data_criacao DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    KEY idx_lcto_data (data_competencia),
    KEY idx_lcto_tipo (tipo),
    KEY idx_lcto_cr (conta_receber_id),
    KEY idx_lcto_cp (conta_pagar_id),
    KEY idx_lcto_plano (plano_conta_id),
    KEY idx_lcto_centro (centro_custo_id),
    KEY idx_lcto_origem (lancamento_origem_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- =============================================================================
-- Alertas Financeiros
-- =============================================================================
CREATE TABLE IF NOT EXISTS alertas_financeiros (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    codigo VARCHAR(64) NOT NULL,
    titulo VARCHAR(180) NOT NULL,
    descricao TEXT NULL,
    severidade ENUM('INFO','WARNING','CRITICAL') NOT NULL DEFAULT 'WARNING',
    condicao_tipo VARCHAR(32) NOT NULL,
    condicao_threshold_dias INT UNSIGNED NULL,
    condicao_threshold_valor BIGINT NULL,
    ativo BOOLEAN NOT NULL DEFAULT TRUE,
    data_criacao DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    UNIQUE KEY uk_alerta_fin_codigo (codigo),
    KEY idx_alerta_fin_ativo (ativo)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

INSERT IGNORE INTO alertas_financeiros (id, codigo, titulo, descricao, severidade, condicao_tipo, condicao_threshold_dias) VALUES
    (1, 'CONTA_RECEBER_VENCENDO_3_DIAS', 'Conta a receber vencendo em 3 dias', 'Conta a receber vence em até 3 dias', 'INFO',     'VENCIMENTO_PROXIMO', 3),
    (2, 'CONTA_PAGAR_VENCENDO_3_DIAS',   'Conta a pagar vencendo em 3 dias',   'Conta a pagar vence em até 3 dias',   'INFO',     'VENCIMENTO_PROXIMO', 3),
    (3, 'CONTA_RECEBER_ATRASADA',         'Conta a receber atrasada',           'Conta a receber venceu e não foi paga', 'CRITICAL', 'VENCIMENTO_PASSADO',  NULL),
    (4, 'CONTA_PAGAR_ATRASADA',           'Conta a pagar atrasada',             'Conta a pagar venceu e não foi paga',   'CRITICAL', 'VENCIMENTO_PASSADO',  NULL),
    (5, 'FLUXO_NEGATIVO_PROJETADO',       'Fluxo de caixa projetado negativo',  'Saldo projetado nos próximos 30 dias é negativo', 'WARNING', 'FLUXO_NEGATIVO', NULL),
    (6, 'RECEBIMENTO_PENDENTE_ALTO',      'Alto volume a receber',              'Total a receber (pendente) acima de R$ 10.000',   'WARNING', 'RECEBIMENTO_PENDENTE', NULL);

UPDATE alertas_financeiros SET condicao_threshold_valor = 1000000 WHERE codigo = 'RECEBIMENTO_PENDENTE_ALTO';

CREATE TABLE IF NOT EXISTS alertas_financeiros_ocorrencias (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    alerta_id INT UNSIGNED NOT NULL,
    entidade_tipo ENUM('CONTA_RECEBER','CONTA_PAGAR','FLUXO') NOT NULL,
    entidade_id INT UNSIGNED NOT NULL,
    data_ocorrencia DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    severidade ENUM('INFO','WARNING','CRITICAL') NOT NULL,
    mensagem VARCHAR(500) NOT NULL,
    contexto_json JSON NULL,
    visualizado BOOLEAN NOT NULL DEFAULT FALSE,
    data_visualizacao DATETIME NULL,
    resolvido BOOLEAN NOT NULL DEFAULT FALSE,
    data_resolucao DATETIME NULL,
    PRIMARY KEY (id),
    KEY idx_alerta_fin_ocorrencia_alerta (alerta_id),
    KEY idx_alerta_fin_ocorrencia_entidade (entidade_tipo, entidade_id),
    KEY idx_alerta_fin_ocorrencia_pendentes (resolvido, visualizado, data_ocorrencia DESC)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
