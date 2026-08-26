-- ============================================================================
-- Migration 0006: Módulo Cotação e Orçamento
-- ----------------------------------------------------------------------------
-- Sprint P2.2 — segundo módulo de produto do ERP.
--
-- Cria:
--   1. cotacoes                  (cabeçalho)
--   2. cotacao_itens             (itens da cotação)
--   3. cotacao_anexos            (fotos, PDFs — placeholder para áudio/vídeo)
--   4. orcamento_aprovacoes      (audit de aprovações)
--   5. orcamento_historico       (mudanças de status)
--
-- Expande:
--   - orcamentos (status, desconto, impostos_estimado, observacoes, ...)
--   - orcamento_items (categoria: PECA | SERVICO | OUTRO)
--
-- Idempotente: usa `IF NOT EXISTS` em todas as tabelas e aproveita a
-- tolerância do runner a "Duplicate column name" para os ALTERs.
-- ============================================================================

-- ----------------------------------------------------------------------------
-- 1. Cotações
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS cotacoes (
    id              INT AUTO_INCREMENT PRIMARY KEY,
    os_id           INT,  -- NULL se cotação avulsa (sem OS)
    cliente_id      INT NOT NULL,
    descricao       VARCHAR(500) NOT NULL,
    observacoes     TEXT,
    status          ENUM(
        'RASCUNHO',
        'AGUARDANDO_COTACAO',
        'COTADO',
        'AGUARDANDO_APROVACAO',
        'APROVADO',
        'REJEITADO',
        'FINALIZADO'
    ) NOT NULL DEFAULT 'RASCUNHO',
    -- usuário que criou
    criado_por      INT,
    criado_por_username VARCHAR(64),
    -- aprovado / rejeitado por
    decidido_por    INT,
    decidido_por_username VARCHAR(64),
    decidido_em     TIMESTAMP NULL,
    decisao_observacao TEXT,
    -- quando virou orçamento (FK para orcamentos)
    orcamento_id    INT,
    created_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_cotacoes_status (status, created_at DESC),
    INDEX idx_cotacoes_cliente (cliente_id, created_at DESC),
    INDEX idx_cotacoes_os (os_id),
    INDEX idx_cotacoes_orcamento (orcamento_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ----------------------------------------------------------------------------
-- 2. Itens da cotação
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS cotacao_itens (
    id              INT AUTO_INCREMENT PRIMARY KEY,
    cotacao_id      INT NOT NULL,
    nome            VARCHAR(255) NOT NULL,
    quantidade      DECIMAL(10,2) NOT NULL DEFAULT 1.00,
    valor_estimado  DECIMAL(12,2) NOT NULL DEFAULT 0.00,
    -- observações livres (ex: "link do fornecedor X")
    observacao      VARCHAR(500),
    -- ordenação dentro da cotação
    ordem           INT NOT NULL DEFAULT 0,
    FOREIGN KEY (cotacao_id) REFERENCES cotacoes(id) ON DELETE CASCADE,
    INDEX idx_cotacao_itens_cotacao (cotacao_id, ordem)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ----------------------------------------------------------------------------
-- 3. Anexos da cotação (fotos, PDFs — preparado para áudio/vídeo)
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS cotacao_anexos (
    id              INT AUTO_INCREMENT PRIMARY KEY,
    cotacao_id      INT NOT NULL,
    -- tipo: imagem, PDF (placeholder para áudio/vídeo no P2.2.1)
    tipo            ENUM('IMAGEM', 'PDF', 'AUDIO', 'VIDEO', 'OUTRO') NOT NULL,
    -- nome original do arquivo
    nome            VARCHAR(255) NOT NULL,
    -- caminho no disco / S3 / etc
    arquivo_path    VARCHAR(500) NOT NULL,
    -- tamanho em bytes
    tamanho_bytes   BIGINT,
    -- mime type (ex: image/jpeg, application/pdf)
    mime_type       VARCHAR(100),
    -- SHA-256 do arquivo (para integridade)
    hash_sha256     VARCHAR(64),
    -- quem subiu
    usuario_id      INT,
    username        VARCHAR(64),
    data_upload     TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (cotacao_id) REFERENCES cotacoes(id) ON DELETE CASCADE,
    INDEX idx_cotacao_anexos_cotacao (cotacao_id, data_upload DESC),
    INDEX idx_cotacao_anexos_tipo (tipo)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ----------------------------------------------------------------------------
-- 4. Expansão de orcamentos
-- ----------------------------------------------------------------------------
ALTER TABLE orcamentos ADD COLUMN status ENUM('RASCUNHO', 'AGUARDANDO_APROVACAO', 'APROVADO', 'REJEITADO', 'FINALIZADO') NOT NULL DEFAULT 'RASCUNHO';
ALTER TABLE orcamentos ADD COLUMN desconto DECIMAL(12,2) NOT NULL DEFAULT 0.00;
ALTER TABLE orcamentos ADD COLUMN subtotal DECIMAL(12,2) NOT NULL DEFAULT 0.00;
ALTER TABLE orcamentos ADD COLUMN impostos_estimado DECIMAL(12,2) NOT NULL DEFAULT 0.00;
ALTER TABLE orcamentos ADD COLUMN observacoes TEXT;
ALTER TABLE orcamentos ADD COLUMN cotacao_origem_id INT;
ALTER TABLE orcamentos ADD COLUMN criado_por INT;
ALTER TABLE orcamentos ADD COLUMN criado_por_username VARCHAR(64);
ALTER TABLE orcamentos ADD COLUMN decidido_por INT;
ALTER TABLE orcamentos ADD COLUMN decidido_por_username VARCHAR(64);
ALTER TABLE orcamentos ADD COLUMN decidido_em TIMESTAMP NULL;
ALTER TABLE orcamentos ADD COLUMN decisao_observacao TEXT;
ALTER TABLE orcamentos ADD COLUMN updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP;

CREATE INDEX idx_orcamentos_status ON orcamentos(status, created_at DESC);
CREATE INDEX idx_orcamentos_cotacao ON orcamentos(cotacao_origem_id);

-- ----------------------------------------------------------------------------
-- 5. Categoria nos itens do orçamento (PECA | SERVICO | OUTRO)
-- ----------------------------------------------------------------------------
ALTER TABLE orcamento_items ADD COLUMN categoria ENUM('PECA', 'SERVICO', 'OUTRO') NOT NULL DEFAULT 'OUTRO';
ALTER TABLE orcamento_items ADD COLUMN peca_id INT;
ALTER TABLE orcamento_items ADD COLUMN servico_id INT;
ALTER TABLE orcamento_items ADD COLUMN observacao VARCHAR(500);

CREATE INDEX idx_orcamento_items_categoria ON orcamento_items(categoria);

-- ----------------------------------------------------------------------------
-- 6. Histórico de aprovações (audit trail)
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS orcamento_aprovacoes (
    id              BIGINT AUTO_INCREMENT PRIMARY KEY,
    orcamento_id    INT NOT NULL,
    -- decisão
    decisao         ENUM('APROVADO', 'REJEITADO') NOT NULL,
    observacao      TEXT,
    decidido_por    INT,
    decidido_por_username VARCHAR(64),
    data_hora       TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (orcamento_id) REFERENCES orcamentos(id) ON DELETE CASCADE,
    INDEX idx_aprovacoes_orcamento (orcamento_id, data_hora DESC)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ----------------------------------------------------------------------------
-- 7. Histórico de status (todas as transições, append-only)
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS orcamento_historico (
    id              BIGINT AUTO_INCREMENT PRIMARY KEY,
    orcamento_id    INT NOT NULL,
    status_anterior ENUM('RASCUNHO', 'AGUARDANDO_APROVACAO', 'APROVADO', 'REJEITADO', 'FINALIZADO'),
    status_novo     ENUM('RASCUNHO', 'AGUARDANDO_APROVACAO', 'APROVADO', 'REJEITADO', 'FINALIZADO') NOT NULL,
    observacao      TEXT,
    usuario_id      INT,
    username        VARCHAR(64),
    data_hora       TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (orcamento_id) REFERENCES orcamentos(id) ON DELETE CASCADE,
    INDEX idx_historico_orcamento (orcamento_id, data_hora DESC)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
