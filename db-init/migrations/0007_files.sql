-- db-init/migrations/0007_files.sql
-- Sprint P2.2.1 — Infraestrutura de Anexos e Mídia.
--
-- Tabelas:
--   arquivos            — entidade universal (1 linha por arquivo físico)
--   arquivo_vinculos    — N:M entre arquivo e entidades do domínio
--   transcricoes        — texto/idioma/confiança de áudios transcritos
--   arquivos_metricas   — views materializadas? (opcional, deixamos para depois)
--
-- Design:
--   - Hash SHA256 é a chave de deduplicação. Mesmo conteúdo → mesmo arquivo.
--   - `arquivo_vinculos` permite o mesmo arquivo ser reutilizado em várias
--     OS / clientes / cotações sem duplicar bytes no storage.
--   - Storage backend agnóstico: `storage_backend` + `storage_chave`.

CREATE TABLE IF NOT EXISTS arquivos (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,

    -- Identidade
    nome_original VARCHAR(255) NOT NULL,
    nome_armazenado VARCHAR(255) NOT NULL,  -- geralmente {uuid}.{ext}
    extensao VARCHAR(16) NOT NULL,
    mime_type VARCHAR(127) NOT NULL,

    -- Conteúdo
    tamanho BIGINT UNSIGNED NOT NULL DEFAULT 0,
    hash_sha256 CHAR(64) NOT NULL,

    -- Classificação (semântica de negócio)
    tipo ENUM('IMAGEM','PDF','AUDIO','VIDEO','OUTRO') NOT NULL DEFAULT 'OUTRO',

    -- Storage (agnóstico)
    storage_backend VARCHAR(32) NOT NULL DEFAULT 'local',
    storage_chave VARCHAR(512) NOT NULL,

    -- Áudio (opcional)
    duracao_segundos INT UNSIGNED NULL,
    codec VARCHAR(32) NULL,
    taxa_amostral_hz INT UNSIGNED NULL,

    -- Vídeo (opcional)
    largura INT UNSIGNED NULL,
    altura INT UNSIGNED NULL,
    duracao_video_segundos INT UNSIGNED NULL,

    -- Auditoria
    usuario_upload_id INT UNSIGNED NOT NULL,
    data_upload DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    removido_em DATETIME NULL,

    PRIMARY KEY (id),
    UNIQUE KEY uk_arquivos_hash (hash_sha256, removido_em),
    KEY idx_arquivos_tipo (tipo),
    KEY idx_arquivos_usuario (usuario_upload_id),
    KEY idx_arquivos_data (data_upload)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Tabela de vínculos (N:M genérica)
-- Suporta: os_id, cliente_id, cotacao_id, orcamento_id, financeiro_id, fiscal_id,
--          equipamento_id, observacao_id, contato_id, etc.
CREATE TABLE IF NOT EXISTS arquivo_vinculos (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    arquivo_id INT UNSIGNED NOT NULL,

    -- Tipo da entidade alvo
    entidade_tipo ENUM(
        'OS','CLIENTE','COTACAO','ORCAMENTO','EQUIPAMENTO',
        'OBSERVACAO','CONTATO','TIMELINE_EVENTO','FINANCEIRO','FISCAL','OUTRO'
    ) NOT NULL,

    -- ID da entidade alvo (não FK por design — pode apontar para qualquer tabela)
    entidade_id INT UNSIGNED NOT NULL,

    -- Quem vinculou
    usuario_id INT UNSIGNED NOT NULL,
    data_vinculo DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    -- Metadados do vínculo (legenda, ordem em galeria, papel do arquivo)
    papel VARCHAR(32) NULL,    -- ex: 'foto_antes', 'foto_depois', 'comprovante', 'nota_fiscal'
    observacao VARCHAR(500) NULL,

    PRIMARY KEY (id),
    UNIQUE KEY uk_vinculo (arquivo_id, entidade_tipo, entidade_id, papel),
    KEY idx_vinculo_entidade (entidade_tipo, entidade_id),
    KEY idx_vinculo_usuario (usuario_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Transcrições (preparada para Whisper/OpenAI/etc)
CREATE TABLE IF NOT EXISTS transcricoes (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    arquivo_id INT UNSIGNED NOT NULL,

    -- Engine
    engine VARCHAR(64) NOT NULL DEFAULT 'pendente',  -- 'whisper-local', 'openai-whisper-1', 'minimax', 'deepgram'
    engine_modelo VARCHAR(64) NULL,

    -- Resultado
    texto LONGTEXT NULL,
    idioma VARCHAR(8) NULL,                  -- ISO 639-1: 'pt', 'en', etc
    confianca DECIMAL(4,3) NULL,            -- 0.000 a 1.000

    -- Processamento
    tempo_processamento_ms INT UNSIGNED NULL,
    data_inicio DATETIME NULL,
    data_fim DATETIME NULL,

    -- Status
    status ENUM('PENDENTE','EM_ANDAMENTO','CONCLUIDA','FALHA','CANCELADA') NOT NULL DEFAULT 'PENDENTE',
    erro_mensagem VARCHAR(500) NULL,

    PRIMARY KEY (id),
    UNIQUE KEY uk_transcricao_arquivo (arquivo_id),
    KEY idx_transcricoes_status (status),
    KEY idx_transcricoes_engine (engine)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- Thumbnails (1 arquivo pode ter N tamanhos: 128, 256, 512)
CREATE TABLE IF NOT EXISTS arquivo_thumbnails (
    id INT UNSIGNED NOT NULL AUTO_INCREMENT,
    arquivo_id INT UNSIGNED NOT NULL,

    tamanho ENUM('THUMB_128','THUMB_256','THUMB_512') NOT NULL,
    largura INT UNSIGNED NOT NULL,
    altura INT UNSIGNED NOT NULL,
    tamanho_bytes BIGINT UNSIGNED NOT NULL,
    storage_chave VARCHAR(512) NOT NULL,

    data_geracao DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (id),
    UNIQUE KEY uk_thumb (arquivo_id, tamanho),
    KEY idx_thumb_data (data_geracao)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
