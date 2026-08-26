-- db-init/migrations/0009_operations.sql
-- Sprint P2.4 — Plataforma de Gestão Operacional.
--
-- Tabelas:
--   workflow_definicoes    — pipelines configuráveis (um por tipo de entidade)
--   workflow_estados       — colunas do Kanban
--   workflow_transicoes    — transições válidas (grafo)
--   workflow_movimentacoes — log de cada movimentação de status
--
--   agendas                — calendários (1 por técnico, ou 1 geral)
--   eventos_agenda         — entradas do calendário
--
--   sla_config             — configuração de SLA por tipo de OS
--   sla_eventos            — log de tempos
--   sla_calculos           — métricas calculadas por OS
--
--   alertas                — alertas operacionais (INFO/WARNING/CRITICAL)
--   alertas_ocorrencias    — log de quando um alerta disparou
--
--   ordens_servico ALTER   — +workflow_estado_id, +responsavel_id, +prioridade

-- =============================================================================
-- Workflow Engine
-- =============================================================================
CREATE TABLE IF NOT EXISTS workflow_definicoes (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    nome VARCHAR(120) NOT NULL,
    descricao VARCHAR(500) NULL,
    entidade_tipo VARCHAR(32) NOT NULL DEFAULT 'OS',
    ativo BOOLEAN NOT NULL DEFAULT TRUE,
    data_criacao DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    usuario_criacao_id INT UNSIGNED NOT NULL,

    PRIMARY KEY (id),
    UNIQUE KEY uk_workflow_nome (nome, entidade_tipo),
    KEY idx_workflow_ativo (ativo)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS workflow_estados (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    workflow_id INT UNSIGNED NOT NULL,
    nome VARCHAR(64) NOT NULL,
    slug VARCHAR(64) NOT NULL,
    descricao VARCHAR(255) NULL,
    cor VARCHAR(16) NULL,                    -- hex (#RRGGBB)
    ordem INT UNSIGNED NOT NULL DEFAULT 0,
    -- Flags
    eh_inicial BOOLEAN NOT NULL DEFAULT FALSE,
    eh_final BOOLEAN NOT NULL DEFAULT FALSE,
    requer_responsavel BOOLEAN NOT NULL DEFAULT FALSE,
    bloqueia_sla BOOLEAN NOT NULL DEFAULT FALSE,

    PRIMARY KEY (id),
    UNIQUE KEY uk_estado (workflow_id, slug),
    KEY idx_estado_ordem (workflow_id, ordem)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS workflow_transicoes (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    workflow_id INT UNSIGNED NOT NULL,
    estado_origem_id INT UNSIGNED NOT NULL,
    estado_destino_id INT UNSIGNED NOT NULL,
    -- Regras opcionais
    requer_papel VARCHAR(64) NULL,          -- role necessária
    exige_motivo BOOLEAN NOT NULL DEFAULT FALSE,
    exige_arquivo BOOLEAN NOT NULL DEFAULT FALSE,

    PRIMARY KEY (id),
    UNIQUE KEY uk_transicao (workflow_id, estado_origem_id, estado_destino_id),
    KEY idx_transicao_origem (estado_origem_id),
    KEY idx_transicao_destino (estado_destino_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS workflow_movimentacoes (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    workflow_id INT UNSIGNED NOT NULL,
    entidade_id INT UNSIGNED NOT NULL,
    estado_origem_id INT UNSIGNED NULL,
    estado_destino_id INT UNSIGNED NOT NULL,
    usuario_id INT UNSIGNED NULL,
    username VARCHAR(120) NULL,
    motivo TEXT NULL,
    data_movimentacao DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    duracao_no_estado_anterior_segundos INT UNSIGNED NULL,
    ip_origem VARCHAR(64) NULL,

    PRIMARY KEY (id),
    KEY idx_mov_workflow (workflow_id),
    KEY idx_mov_entidade (entidade_id, data_movimentacao DESC),
    KEY idx_mov_data (data_movimentacao),
    KEY idx_mov_usuario (usuario_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- =============================================================================
-- Agenda
-- =============================================================================
CREATE TABLE IF NOT EXISTS agendas (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    nome VARCHAR(120) NOT NULL,
    tecnico_id INT UNSIGNED NULL,            -- NULL = agenda geral
    cor VARCHAR(16) NULL,
    ativo BOOLEAN NOT NULL DEFAULT TRUE,
    data_criacao DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (id),
    UNIQUE KEY uk_agenda_tecnico (tecnico_id),
    KEY idx_agenda_ativo (ativo)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS eventos_agenda (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    agenda_id INT UNSIGNED NOT NULL,
    tecnico_id INT UNSIGNED NOT NULL,
    os_id INT UNSIGNED NULL,
    cliente_id INT UNSIGNED NULL,
    titulo VARCHAR(180) NOT NULL,
    descricao TEXT NULL,
    tipo ENUM('VISITA','COLETA','ENTREGA','MANUTENCAO','RETORNO','OUTRO') NOT NULL DEFAULT 'OUTRO',
    status ENUM('AGENDADO','CONFIRMADO','EM_ANDAMENTO','CONCLUIDO','CANCELADO','FALTOU') NOT NULL DEFAULT 'AGENDADO',
    inicio DATETIME NOT NULL,
    fim DATETIME NOT NULL,
    -- geolocalização
    latitude DECIMAL(10, 7) NULL,
    longitude DECIMAL(10, 7) NULL,
    endereco VARCHAR(500) NULL,
    -- controle
    usuario_criacao_id INT UNSIGNED NOT NULL,
    data_criacao DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    data_atualizacao DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,

    PRIMARY KEY (id),
    KEY idx_evento_tecnico (tecnico_id, inicio),
    KEY idx_evento_agenda (agenda_id, inicio),
    KEY idx_evento_os (os_id),
    KEY idx_evento_cliente (cliente_id, inicio),
    KEY idx_evento_periodo (inicio, fim),
    KEY idx_evento_status (status)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- =============================================================================
-- SLA
-- =============================================================================
CREATE TABLE IF NOT EXISTS sla_config (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    nome VARCHAR(120) NOT NULL,
    descricao VARCHAR(500) NULL,
    ativo BOOLEAN NOT NULL DEFAULT TRUE,

    -- Limites em horas
    max_horas_diagnostico INT UNSIGNED NULL,
    max_horas_execucao INT UNSIGNED NULL,
    max_horas_total INT UNSIGNED NULL,
    max_horas_espera_peca INT UNSIGNED NULL,
    max_horas_espera_aprovacao INT UNSIGNED NULL,

    data_criacao DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (id),
    UNIQUE KEY uk_sla_nome (nome),
    KEY idx_sla_ativo (ativo)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS sla_eventos (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    os_id INT UNSIGNED NOT NULL,
    tipo ENUM('RECEBIDO','DIAGNOSTICO_INICIO','DIAGNOSTICO_FIM','PECA_SOLICITADA','PECA_RECEBIDA','APROVACAO_SOLICITADA','APROVACAO_RECEBIDA','EXECUCAO_INICIO','EXECUCAO_FIM','TESTE_INICIO','TESTE_FIM','FINALIZADO','ENTREGUE','CANCELADO') NOT NULL,
    data_evento DATETIME NOT NULL,
    usuario_id INT UNSIGNED NULL,
    observacao VARCHAR(500) NULL,

    PRIMARY KEY (id),
    KEY idx_sla_evento_os (os_id, data_evento),
    KEY idx_sla_evento_tipo (tipo)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS sla_calculos (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    os_id INT UNSIGNED NOT NULL,
    sla_config_id INT UNSIGNED NULL,

    -- Tempos (em segundos, NULL se não aplicável)
    tempo_diagnostico_segundos INT UNSIGNED NULL,
    tempo_espera_peca_segundos INT UNSIGNED NULL,
    tempo_espera_aprovacao_segundos INT UNSIGNED NULL,
    tempo_execucao_segundos INT UNSIGNED NULL,
    tempo_total_segundos INT UNSIGNED NULL,

    -- Avaliação
    sla_violado BOOLEAN NOT NULL DEFAULT FALSE,
    motivo_violacao VARCHAR(500) NULL,

    data_calculo DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (id),
    UNIQUE KEY uk_sla_calc_os (os_id),
    KEY idx_sla_calc_violado (sla_violado)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- =============================================================================
-- Alertas Operacionais
-- =============================================================================
CREATE TABLE IF NOT EXISTS alertas (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    codigo VARCHAR(64) NOT NULL,            -- ex: 'OS_PARADA_3_DIAS'
    titulo VARCHAR(180) NOT NULL,
    descricao TEXT NULL,
    severidade ENUM('INFO','WARNING','CRITICAL') NOT NULL DEFAULT 'WARNING',
    ativo BOOLEAN NOT NULL DEFAULT TRUE,

    -- Configuração
    condicao_tipo VARCHAR(32) NOT NULL,     -- ex: 'TEMPO_NO_ESTADO', 'TEMPO_NA_FILA'
    condicao_threshold_horas INT UNSIGNED NULL,
    condicao_estado_slug VARCHAR(64) NULL,

    -- Alvo
    entidade_tipo VARCHAR(32) NOT NULL DEFAULT 'OS',

    data_criacao DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (id),
    UNIQUE KEY uk_alerta_codigo (codigo),
    KEY idx_alerta_ativo (ativo)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS alertas_ocorrencias (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    alerta_id INT UNSIGNED NOT NULL,
    entidade_id INT UNSIGNED NOT NULL,        -- ex: os_id
    data_ocorrencia DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    severidade ENUM('INFO','WARNING','CRITICAL') NOT NULL,
    mensagem VARCHAR(500) NOT NULL,
    contexto_json JSON NULL,
    -- Controle
    visualizado BOOLEAN NOT NULL DEFAULT FALSE,
    data_visualizacao DATETIME NULL,
    usuario_visualizacao_id INT UNSIGNED NULL,
    resolvido BOOLEAN NOT NULL DEFAULT FALSE,
    data_resolucao DATETIME NULL,

    PRIMARY KEY (id),
    KEY idx_ocorrencia_alerta (alerta_id),
    KEY idx_ocorrencia_entidade (entidade_id, data_ocorrencia),
    KEY idx_ocorrencia_pendentes (resolvido, visualizado, data_ocorrencia DESC)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- =============================================================================
-- ALTERs em ordens_servico
-- =============================================================================
-- Idempotente: migration runner tolera "Duplicate column name"

-- =============================================================================
-- SEED: workflow padrão "OS Padrão" com 10 estados
-- =============================================================================
INSERT IGNORE INTO workflow_definicoes (id, nome, descricao, usuario_criacao_id)
VALUES (1, 'OS Padrão', 'Pipeline padrão para Ordens de Serviço', 1);

INSERT IGNORE INTO workflow_estados (id, workflow_id, nome, slug, descricao, cor, ordem, eh_inicial, eh_final, bloqueia_sla) VALUES
    (1, 1, 'Recebido',        'RECEBIDO',              'OS aberta, ainda não diagnosticada',         '#9E9E9E', 1,  TRUE,  FALSE, FALSE),
    (2, 1, 'Diagnóstico',     'DIAGNOSTICO',           'Técnico avaliando o problema',               '#2196F3', 2,  FALSE, FALSE, FALSE),
    (3, 1, 'Aguardando Peça', 'AGUARDANDO_PECA',       'Solicitada peça ao fornecedor',              '#FF9800', 3,  FALSE, FALSE, TRUE),
    (4, 1, 'Orçamento',       'ORCAMENTO',             'Elaborando orçamento',                       '#FFC107', 4,  FALSE, FALSE, FALSE),
    (5, 1, 'Aguard. Aprovação','AGUARDANDO_APROVACAO', 'Aguardando aprovação do cliente',            '#FF5722', 5,  FALSE, FALSE, TRUE),
    (6, 1, 'Aprovado',        'APROVADO',              'Cliente aprovou, pronto para executar',      '#4CAF50', 6,  FALSE, FALSE, FALSE),
    (7, 1, 'Execução',        'EXECUCAO',              'Em reparo',                                  '#3F51B5', 7,  FALSE, FALSE, FALSE),
    (8, 1, 'Teste',           'TESTE',                 'Testes finais',                              '#00BCD4', 8,  FALSE, FALSE, FALSE),
    (9, 1, 'Finalizado',      'FINALIZADO',            'OS finalizada, pendente entrega',            '#8BC34A', 9,  FALSE, FALSE, FALSE),
    (10, 1, 'Entregue',       'ENTREGUE',              'Equipamento entregue ao cliente',            '#1B5E20', 10, FALSE, TRUE,  FALSE);

-- Transições válidas (grafo)
INSERT IGNORE INTO workflow_transicoes (workflow_id, estado_origem_id, estado_destino_id, exige_motivo) VALUES
    (1, 1, 2,  FALSE),  -- Recebido → Diagnóstico
    (1, 1, 3,  FALSE),  -- Recebido → Aguardando Peça
    (1, 2, 1,  FALSE),  -- Diagnóstico → Recebido (volta)
    (1, 2, 3,  FALSE),  -- Diagnóstico → Aguardando Peça
    (1, 2, 4,  FALSE),  -- Diagnóstico → Orçamento
    (1, 2, 9,  TRUE),   -- Diagnóstico → Finalizado (não tem conserto)
    (1, 3, 2,  FALSE),  -- Aguardando Peça → Diagnóstico (peça chegou)
    (1, 3, 4,  FALSE),  -- Aguardando Peça → Orçamento
    (1, 4, 5,  FALSE),  -- Orçamento → Aguardando Aprovação
    (1, 4, 9,  TRUE),   -- Orçamento → Finalizado (cliente desistiu)
    (1, 5, 6,  FALSE),  -- Aguardando Aprovação → Aprovado
    (1, 5, 4,  TRUE),   -- Aguardando Aprovação → Orçamento (revisar)
    (1, 5, 9,  TRUE),   -- Aguardando Aprovação → Finalizado (rejeitado)
    (1, 6, 7,  FALSE),  -- Aprovado → Execução
    (1, 6, 4,  FALSE),  -- Aprovado → Orçamento (revisar valores)
    (1, 7, 8,  FALSE),  -- Execução → Teste
    (1, 7, 3,  FALSE),  -- Execução → Aguardando Peça (descobriu nova peça)
    (1, 8, 9,  FALSE),  -- Teste → Finalizado
    (1, 8, 7,  FALSE),  -- Teste → Execução (falhou, refazer)
    (1, 9, 10, FALSE);  -- Finalizado → Entregue

-- =============================================================================
-- SEED: SLA padrão
-- =============================================================================
INSERT IGNORE INTO sla_config (id, nome, descricao, max_horas_diagnostico, max_horas_execucao, max_horas_total, max_horas_espera_peca, max_horas_espera_aprovacao)
VALUES
    (1, 'SLA Padrão', 'SLA padrão para todas as OS', 24, 72, 240, 168, 72);

-- =============================================================================
-- SEED: Alertas padrão
-- =============================================================================
INSERT IGNORE INTO alertas (id, codigo, titulo, descricao, severidade, condicao_tipo, condicao_threshold_horas, condicao_estado_slug, entidade_tipo) VALUES
    (1, 'OS_PARADA_3_DIAS', 'OS parada há mais de 3 dias', 'OS sem movimentação há mais de 3 dias no mesmo estado', 'WARNING',   'TEMPO_NO_ESTADO', 72,  NULL, 'OS'),
    (2, 'OS_aguardando_peca_7_DIAS', 'OS aguardando peça há 7 dias', 'OS parada no estado Aguardando Peça', 'CRITICAL', 'TEMPO_NO_ESTADO', 168, 'AGUARDANDO_PECA', 'OS'),
    (3, 'OS_aguardando_aprovacao_5_DIAS', 'OS aguardando aprovação há 5 dias', 'OS parada no estado Aguardando Aprovação', 'WARNING', 'TEMPO_NO_ESTADO', 120, 'AGUARDANDO_APROVACAO', 'OS'),
    (4, 'CHECKLIST_INCOMPLETO', 'Checklist incompleto há 24h', 'OS com checklist iniciado e não concluído', 'INFO',     'CHECKLIST_PENDENTE', 24, NULL, 'OS');
