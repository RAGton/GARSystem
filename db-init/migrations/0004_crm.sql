-- ============================================================================
-- Migration 0004: CRM — Timeline, Observações, Tags, Contatos, Anexos
-- ----------------------------------------------------------------------------
-- Sprint P2.1 — primeiro módulo funcional do ERP.
--
-- Adiciona 6 novas tabelas:
--   1. cliente_timeline_eventos  (histórico unificado do cliente)
--   2. cliente_observacoes      (notas internas auditadas)
--   3. cliente_observacao_edicoes (audit trail de edições)
--   4. cliente_tags             (sistema extensível de tags)
--   5. cliente_tag_atribuicoes  (N:N cliente ↔ tag)
--   6. cliente_contatos         (telefone/whatsapp/email, com principal)
--   7. cliente_anexos           (placeholder para P2.2 — áudio/foto/arquivo)
--
-- A expansão da tabela `equipamentos` está em 0005_equipamentos_expand.sql
-- porque requer `PREPARE/EXECUTE` (não suportado pelo runner simples).
--
-- Idempotente: usa `IF NOT EXISTS` em todas as tabelas e `INSERT IGNORE`
-- nas seeds.
-- ============================================================================

-- ----------------------------------------------------------------------------
-- 1. Timeline unificada do cliente
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS cliente_timeline_eventos (
    id              BIGINT AUTO_INCREMENT PRIMARY KEY,
    cliente_id      INT NOT NULL,
    tipo            ENUM(
        'CLIENTE_CRIADO',
        'CLIENTE_ATUALIZADO',
        'CLIENTE_REMOVIDO',
        'OBSERVACAO_ADICIONADA',
        'OBSERVACAO_EDITADA',
        'TAG_ADICIONADA',
        'TAG_REMOVIDA',
        'CONTATO_ADICIONADO',
        'CONTATO_REMOVIDO',
        'EQUIPAMENTO_ADICIONADO',
        'EQUIPAMENTO_REMOVIDO',
        'ORCAMENTO_CRIADO',
        'ORCAMENTO_ATUALIZADO',
        'OS_CRIADA',
        'OS_ATUALIZADA',
        'OS_FINALIZADA',
        'ANEXO_ADICIONADO',
        'OUTRO'
    ) NOT NULL,
    descricao       VARCHAR(500) NOT NULL,
    payload         JSON,
    usuario_id      INT,
    username        VARCHAR(64),
    data_hora       TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    FOREIGN KEY (cliente_id) REFERENCES clientes(id) ON DELETE CASCADE,
    INDEX idx_timeline_cliente_data (cliente_id, data_hora DESC),
    INDEX idx_timeline_tipo (tipo, data_hora DESC),
    INDEX idx_timeline_usuario (usuario_id, data_hora DESC)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ----------------------------------------------------------------------------
-- 2. Observações internas
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS cliente_observacoes (
    id              INT AUTO_INCREMENT PRIMARY KEY,
    cliente_id      INT NOT NULL,
    conteudo        TEXT NOT NULL,
    autor_id        INT,
    autor_username  VARCHAR(64),
    editada         BOOLEAN NOT NULL DEFAULT FALSE,
    vezes_editada   INT NOT NULL DEFAULT 0,
    created_at      TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at      TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    FOREIGN KEY (cliente_id) REFERENCES clientes(id) ON DELETE CASCADE,
    INDEX idx_observacoes_cliente (cliente_id, created_at DESC),
    INDEX idx_observacoes_autor (autor_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS cliente_observacao_edicoes (
    id                  BIGINT AUTO_INCREMENT PRIMARY KEY,
    observacao_id       INT NOT NULL,
    conteudo_anterior   TEXT NOT NULL,
    conteudo_novo       TEXT NOT NULL,
    editor_id           INT,
    editor_username     VARCHAR(64),
    data_hora           TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    FOREIGN KEY (observacao_id) REFERENCES cliente_observacoes(id) ON DELETE CASCADE,
    INDEX idx_edicao_observacao (observacao_id, data_hora DESC)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ----------------------------------------------------------------------------
-- 3. Tags (sistema extensível)
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS cliente_tags (
    id              INT AUTO_INCREMENT PRIMARY KEY,
    nome            VARCHAR(32) NOT NULL,
    cor             VARCHAR(7) NOT NULL DEFAULT '#808080',
    descricao       VARCHAR(255),
    publica         BOOLEAN NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE KEY uk_tags_nome (nome)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Tags pré-definidas (seed)
INSERT IGNORE INTO cliente_tags (nome, cor, descricao, publica) VALUES
    ('VIP',          '#FFD700', 'Cliente VIP',                    TRUE),
    ('EMPRESA',      '#0066CC', 'Pessoa jurídica',                 TRUE),
    ('GARANTIA',     '#00AA00', 'Em garantia',                    TRUE),
    ('INADIMPLENTE', '#CC0000', 'Possui pendências financeiras',  TRUE),
    ('CONTRATO',     '#990099', 'Possui contrato de manutenção',  TRUE),
    ('RECORRENTE',   '#FF6600', 'Cliente que volta sempre',       TRUE);

CREATE TABLE IF NOT EXISTS cliente_tag_atribuicoes (
    id                          INT AUTO_INCREMENT PRIMARY KEY,
    cliente_id                  INT NOT NULL,
    tag_id                      INT NOT NULL,
    atribuido_por               INT,
    atribuido_por_username      VARCHAR(64),
    data_hora                   TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (cliente_id) REFERENCES clientes(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES cliente_tags(id) ON DELETE CASCADE,
    UNIQUE KEY uk_cliente_tag (cliente_id, tag_id),
    INDEX idx_tag_cliente (tag_id, cliente_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ----------------------------------------------------------------------------
-- 4. Contatos (múltiplos por cliente)
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS cliente_contatos (
    id              INT AUTO_INCREMENT PRIMARY KEY,
    cliente_id      INT NOT NULL,
    tipo            ENUM('TELEFONE', 'WHATSAPP', 'EMAIL', 'SITE', 'OUTRO') NOT NULL,
    valor           VARCHAR(255) NOT NULL,
    rotulo          VARCHAR(50),
    principal       BOOLEAN NOT NULL DEFAULT FALSE,
    observacao      VARCHAR(255),
    created_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (cliente_id) REFERENCES clientes(id) ON DELETE CASCADE,
    INDEX idx_contatos_cliente (cliente_id),
    INDEX idx_contatos_tipo (tipo)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ----------------------------------------------------------------------------
-- 5. Anexos (placeholder para P2.2 — áudio transcrito, foto, PDF)
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS cliente_anexos (
    id              INT AUTO_INCREMENT PRIMARY KEY,
    cliente_id      INT NOT NULL,
    tipo            ENUM('FOTO', 'AUDIO', 'DOCUMENTO', 'OUTRO') NOT NULL,
    arquivo_path    VARCHAR(500) NOT NULL,
    nome_original   VARCHAR(255),
    tamanho_bytes   BIGINT,
    mime_type       VARCHAR(100),
    transcricao     TEXT,
    usuario_id      INT,
    username        VARCHAR(64),
    created_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (cliente_id) REFERENCES clientes(id) ON DELETE CASCADE,
    INDEX idx_anexos_cliente (cliente_id, created_at DESC),
    INDEX idx_anexos_tipo (tipo)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
