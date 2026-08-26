-- db-init/migrations/0008_os_mobile.sql
-- Sprint P2.3 — OS Mobile + Captura de Campo.
--
-- Tabelas novas:
--   os_checklist_templates  — templates reutilizáveis (ex: "Manutenção padrão")
--   os_checklist_template_itens — itens de um template
--   os_checklist            — checklist aplicado a uma OS
--   os_checklist_itens      — itens do checklist (respostas)
--   os_evolucoes            — entradas de evolução/histórico (manual)
--   os_assinaturas          — assinatura do cliente (placeholder — captura real em P2.3.x)
--   os_auditoria_campo      — log de ações offline-first (created_on, synced_em)
--
-- ALTERs:
--   ordens_servico  + latitude_inicio, + longitude_inicio, + latitude_fim, + longitude_fim,
--                     + iniciado_em, + concluido_em
--   (Já tem status, cliente_id, equipamento_id, etc)
--
-- Tudo desenhado pensando em mobile-first:
--   - colunas nullable para compat com OS legadas
--   - timestamps para auditoria offline
--   - assinaturas: arquivo em si fica no storage (vinculado via arquivo_vinculos)

-- =============================================================================
-- Templates de checklist (reutilizáveis)
-- =============================================================================
CREATE TABLE IF NOT EXISTS os_checklist_templates (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    nome VARCHAR(120) NOT NULL,
    descricao VARCHAR(500) NULL,
    ativo BOOLEAN NOT NULL DEFAULT TRUE,
    data_criacao DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    usuario_criacao_id INT UNSIGNED NOT NULL,

    PRIMARY KEY (id),
    KEY idx_template_ativo (ativo),
    KEY idx_template_nome (nome)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS os_checklist_template_itens (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    template_id INT UNSIGNED NOT NULL,
    ordem INT UNSIGNED NOT NULL DEFAULT 0,
    texto VARCHAR(255) NOT NULL,
    obrigatorio BOOLEAN NOT NULL DEFAULT FALSE,

    PRIMARY KEY (id),
    KEY idx_template_item (template_id, ordem)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- =============================================================================
-- Checklists aplicados a uma OS
-- =============================================================================
CREATE TABLE IF NOT EXISTS os_checklists (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    os_id INT UNSIGNED NOT NULL,
    template_id INT UNSIGNED NULL,    -- NULL = checklist ad-hoc
    titulo VARCHAR(120) NOT NULL,
    data_inicio DATETIME NULL,
    data_conclusao DATETIME NULL,
    total_itens INT UNSIGNED NOT NULL DEFAULT 0,
    concluidos INT UNSIGNED NOT NULL DEFAULT 0,

    PRIMARY KEY (id),
    KEY idx_checklist_os (os_id),
    KEY idx_checklist_template (template_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS os_checklist_itens (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    checklist_id INT UNSIGNED NOT NULL,
    template_item_id INT UNSIGNED NULL,  -- pode ser null se ad-hoc
    ordem INT UNSIGNED NOT NULL DEFAULT 0,
    texto VARCHAR(255) NOT NULL,
    obrigatorio BOOLEAN NOT NULL DEFAULT FALSE,
    concluido BOOLEAN NOT NULL DEFAULT FALSE,
    data_conclusao DATETIME NULL,
    observacao VARCHAR(500) NULL,

    PRIMARY KEY (id),
    KEY idx_checklist_item (checklist_id, ordem),
    KEY idx_checklist_item_concluido (checklist_id, concluido)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- =============================================================================
-- Evoluções de OS (entradas manuais de status/histórico)
-- =============================================================================
CREATE TABLE IF NOT EXISTS os_evolucoes (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    os_id INT UNSIGNED NOT NULL,
    usuario_id INT UNSIGNED NOT NULL,
    texto TEXT NOT NULL,
    tipo ENUM('STATUS','OBSERVACAO','PROBLEMA','SOLUCAO','OUTRO') NOT NULL DEFAULT 'OBSERVACAO',
    data_evolucao DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- dados de localização (opcional, vem do app mobile)
    latitude DECIMAL(10, 7) NULL,
    longitude DECIMAL(10, 7) NULL,

    PRIMARY KEY (id),
    KEY idx_evolucao_os (os_id, data_evolucao DESC),
    KEY idx_evolucao_tipo (tipo),
    KEY idx_evolucao_usuario (usuario_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- =============================================================================
-- Assinaturas do cliente (placeholder para P2.3.x implementar captura real)
-- =============================================================================
CREATE TABLE IF NOT EXISTS os_assinaturas (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    os_id INT UNSIGNED NOT NULL,
    -- O arquivo (imagem PNG do canvas) fica em `arquivos` + vínculo
    arquivo_id INT UNSIGNED NOT NULL,

    -- Metadados da assinatura
    nome_assinante VARCHAR(255) NOT NULL,
    documento_assinante VARCHAR(32) NULL,    -- CPF/CNPJ (opcional)
    data_assinatura DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ip_assinatura VARCHAR(64) NULL,           -- se foi via web
    latitude DECIMAL(10, 7) NULL,
    longitude DECIMAL(10, 7) NULL,

    observacao VARCHAR(500) NULL,

    PRIMARY KEY (id),
    UNIQUE KEY uk_assinatura_os (os_id),
    KEY idx_assinatura_data (data_assinatura)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- =============================================================================
-- Auditoria offline-first
-- Log de mutações feitas enquanto offline; quando reconecta, sincroniza.
-- =============================================================================
CREATE TABLE IF NOT EXISTS os_auditoria_campo (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    -- Identificação
    cliente_device_id VARCHAR(128) NULL,    -- UUID do device (permite dedup)
    usuario_id INT UNSIGNED NOT NULL,

    -- O que aconteceu
    tipo_acao ENUM(
        'OS_CRIADA','OS_ATUALIZADA','STATUS_ALTERADO',
        'CHECKLIST_ITEM_CONCLUIDO','CHECKLIST_CONCLUIDO',
        'ANEXO_ENVIADO','EVOLUCAO_REGISTRADA',
        'ASSINATURA_REGISTRADA'
    ) NOT NULL,

    -- Referência (polimórfico)
    os_id INT UNSIGNED NULL,
    checklist_id INT UNSIGNED NULL,
    checklist_item_id INT UNSIGNED NULL,
    evolucao_id INT UNSIGNED NULL,
    arquivo_id INT UNSIGNED NULL,
    assinatura_id INT UNSIGNED NULL,

    -- Quando aconteceu no device
    data_evento_device DATETIME NOT NULL,

    -- Quando chegou no servidor
    data_sincronizacao DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- Sincronização
    processado BOOLEAN NOT NULL DEFAULT TRUE,
    payload_json JSON NULL,

    PRIMARY KEY (id),
    UNIQUE KEY uk_audit_device_evento (cliente_device_id, data_evento_device, tipo_acao, os_id),
    KEY idx_audit_os (os_id),
    KEY idx_audit_data (data_evento_device),
    KEY idx_audit_processado (processado)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- =============================================================================
-- ALTERs em ordens_servico (mobile-first fields)
-- =============================================================================
-- Adiciona colunas geolocalização e timestamps de início/conclusão.
-- São idempotentes graças ao suporte do migration runner.

-- (verificação prévia + ALTER condicional feito pelo runner)

-- =============================================================================
-- Seed: template "Manutenção Padrão" para uso imediato
-- =============================================================================
INSERT IGNORE INTO os_checklist_templates (id, nome, descricao, ativo, usuario_criacao_id)
VALUES (1, 'Manutenção Padrão', 'Checklist básico para manutenção preventiva', TRUE, 1);

INSERT IGNORE INTO os_checklist_template_itens (template_id, ordem, texto, obrigatorio) VALUES
    (1, 1, 'Equipamento recebido', TRUE),
    (1, 2, 'Diagnóstico concluído', TRUE),
    (1, 3, 'Orçamento apresentado', FALSE),
    (1, 4, 'Peça solicitada', FALSE),
    (1, 5, 'Reparo concluído', TRUE),
    (1, 6, 'Testes realizados', TRUE),
    (1, 7, 'Equipamento limpo', FALSE),
    (1, 8, 'Cliente notificado', FALSE);
