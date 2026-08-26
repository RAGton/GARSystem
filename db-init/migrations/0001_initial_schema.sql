-- ============================================================================
-- Migration 0001: schema inicial limpo (Senior System v1.9.0)
-- ----------------------------------------------------------------------------
-- Este arquivo é a FONTE DE VERDADE do schema. Substitui o `init.sql` antigo
-- (que estava com tabelas duplicadas, sem colunas `credito`, sem tabela
-- `movimentacoes` e sem índices em FKs).
--
-- Idempotente: usa `IF NOT EXISTS`. Aplicável em banco novo ou existente.
-- ============================================================================

-- ----------------------------------------------------------------------------
-- 1. Tabela de Usuários
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS users (
    id            INT AUTO_INCREMENT PRIMARY KEY,
    username      VARCHAR(64)  NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role          ENUM('Administrador', 'Gerencia', 'Tecnico', 'Financeiro', 'Comercial', 'Estoquista')
                  NOT NULL DEFAULT 'Comercial',
    -- Para multi-tenant futuro. 0 = sem tenant (single-tenant).
    tenant_id     INT          NOT NULL DEFAULT 0,
    created_at    TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at    TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    last_login_at TIMESTAMP    NULL,
    UNIQUE KEY uk_users_username (username),
    INDEX idx_users_role (role),
    INDEX idx_users_tenant (tenant_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ----------------------------------------------------------------------------
-- 2. Clientes (com credito agora)
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS clientes (
    id                  INT AUTO_INCREMENT PRIMARY KEY,
    tenant_id           INT          NOT NULL DEFAULT 0,
    nome                VARCHAR(255) NOT NULL,
    telefone            VARCHAR(20),
    email               VARCHAR(255),
    endereco            TEXT,
    inscricao_estadual  VARCHAR(30),
    cpf_cnpj            VARCHAR(20),
    -- saldo de crédito disponível para o cliente (em reais).
    -- Adicionado na auditoria (estava só no struct Rust).
    credito             DECIMAL(12,2) NOT NULL DEFAULT 0.00,
    created_at          TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at          TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_clientes_nome (nome),
    INDEX idx_clientes_cpf_cnpj (cpf_cnpj),
    INDEX idx_clientes_tenant (tenant_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ----------------------------------------------------------------------------
-- 3. Equipamentos
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS equipamentos (
    id            INT AUTO_INCREMENT PRIMARY KEY,
    cliente_id    INT NOT NULL,
    descricao     VARCHAR(255) NOT NULL,
    marca         VARCHAR(100),
    modelo        VARCHAR(100),
    numero_serie  VARCHAR(255),
    created_at    TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (cliente_id) REFERENCES clientes(id) ON DELETE CASCADE,
    UNIQUE KEY uk_equipamentos_numero_serie (numero_serie),
    INDEX idx_equipamentos_cliente (cliente_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ----------------------------------------------------------------------------
-- 4. Ordens de Serviço
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS ordens_servico (
    id                  INT AUTO_INCREMENT PRIMARY KEY,
    tenant_id           INT NOT NULL DEFAULT 0,
    cliente_id          INT NOT NULL,
    equipamento_id      INT NOT NULL,
    defeito_relatado    TEXT NOT NULL,
    observacoes         TEXT,
    parecer_tecnico     TEXT,
    status              ENUM('Aberta', 'EmAndamento', 'AguardandoPeca', 'Finalizada', 'Cancelada')
                        NOT NULL DEFAULT 'Aberta',
    situacao            ENUM('Orcamento', 'Aprovado', 'EmAndamento', 'AutorizadoAguardandoPeca',
                             'ServicoConcluido', 'AguardandoAutorizacao', 'AguardandoRetirada',
                             'Reprovado', 'AguardandoFaturar', 'Faturado')
                        NOT NULL DEFAULT 'Orcamento',
    atendente           VARCHAR(255),
    tecnico_responsavel VARCHAR(255),
    data_chegada        DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    prazo_entrega       DATETIME,
    created_at          TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at          TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (cliente_id) REFERENCES clientes(id),
    FOREIGN KEY (equipamento_id) REFERENCES equipamentos(id),
    INDEX idx_os_cliente (cliente_id),
    INDEX idx_os_equipamento (equipamento_id),
    INDEX idx_os_status (status),
    INDEX idx_os_situacao (situacao),
    INDEX idx_os_data_chegada (data_chegada),
    INDEX idx_os_tenant (tenant_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ----------------------------------------------------------------------------
-- 5. Histórico de Edições da OS
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS historico_edicoes (
    id               INT AUTO_INCREMENT PRIMARY KEY,
    ordem_servico_id INT NOT NULL,
    usuario          VARCHAR(255) NOT NULL,
    data_hora        DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    campo_alterado   VARCHAR(255) NOT NULL,
    valor_antigo     TEXT,
    valor_novo       TEXT,
    FOREIGN KEY (ordem_servico_id) REFERENCES ordens_servico(id) ON DELETE CASCADE,
    INDEX idx_hist_os (ordem_servico_id, data_hora)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ----------------------------------------------------------------------------
-- 6. Fornecedores
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS fornecedores (
    id        INT AUTO_INCREMENT PRIMARY KEY,
    nome      VARCHAR(255) NOT NULL,
    cnpj      VARCHAR(20),
    contato   VARCHAR(255),
    telefone  VARCHAR(25),
    email     VARCHAR(255),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE KEY uk_fornecedores_nome (nome),
    UNIQUE KEY uk_fornecedores_cnpj (cnpj),
    INDEX idx_fornecedores_nome (nome)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ----------------------------------------------------------------------------
-- 7. Peças (estoque)
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS pecas (
    id                INT AUTO_INCREMENT PRIMARY KEY,
    codigo_interno    VARCHAR(50)  NOT NULL,
    part_number       VARCHAR(100),
    descricao         VARCHAR(255) NOT NULL,
    fabricante        VARCHAR(100),
    localizacao       VARCHAR(100),
    estoque_atual     INT NOT NULL DEFAULT 0,
    estoque_minimo    INT NOT NULL DEFAULT 1,
    preco_custo       DECIMAL(10,2) NOT NULL DEFAULT 0.00,
    preco_venda       DECIMAL(10,2) NOT NULL DEFAULT 0.00,
    ultima_atualizacao TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    created_at        TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE KEY uk_pecas_codigo_interno (codigo_interno),
    INDEX idx_pecas_descricao (descricao)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ----------------------------------------------------------------------------
-- 8. Notas Fiscais de Entrada
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS notas_fiscais_entrada (
    id            INT AUTO_INCREMENT PRIMARY KEY,
    numero_nf     VARCHAR(50) NOT NULL,
    fornecedor_id INT NOT NULL,
    data_emissao  DATE,
    data_entrada  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    valor_total   DECIMAL(10,2),
    observacoes   TEXT,
    FOREIGN KEY (fornecedor_id) REFERENCES fornecedores(id),
    INDEX idx_nf_fornecedor (fornecedor_id),
    INDEX idx_nf_numero (numero_nf)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ----------------------------------------------------------------------------
-- 9. Pivot: NF ↔ Peças
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS nf_entrada_pecas (
    id                     INT AUTO_INCREMENT PRIMARY KEY,
    nota_fiscal_id         INT NOT NULL,
    peca_id                INT NOT NULL,
    quantidade             INT NOT NULL,
    preco_custo_unitario   DECIMAL(10,2) NOT NULL,
    FOREIGN KEY (nota_fiscal_id) REFERENCES notas_fiscais_entrada(id) ON DELETE CASCADE,
    FOREIGN KEY (peca_id)        REFERENCES pecas(id),
    INDEX idx_nfpec_nota (nota_fiscal_id),
    INDEX idx_nfpec_peca (peca_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ----------------------------------------------------------------------------
-- 10. Pivot: OS ↔ Peças
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS ordem_servico_pecas (
    id                     INT AUTO_INCREMENT PRIMARY KEY,
    ordem_servico_id       INT NOT NULL,
    peca_id                INT NOT NULL,
    quantidade             INT NOT NULL,
    preco_venda_unitario   DECIMAL(10,2) NOT NULL,
    FOREIGN KEY (ordem_servico_id) REFERENCES ordens_servico(id) ON DELETE CASCADE,
    FOREIGN KEY (peca_id)          REFERENCES pecas(id),
    INDEX idx_ospec_os (ordem_servico_id),
    INDEX idx_ospec_peca (peca_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ----------------------------------------------------------------------------
-- 11. Orçamentos
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS orcamentos (
    id           INT AUTO_INCREMENT PRIMARY KEY,
    cliente_id   INT NOT NULL,
    total        DECIMAL(12,2) NOT NULL DEFAULT 0.00,
    data_criacao DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (cliente_id) REFERENCES clientes(id),
    INDEX idx_orc_cliente (cliente_id),
    INDEX idx_orc_data (data_criacao)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE IF NOT EXISTS orcamento_items (
    id            INT AUTO_INCREMENT PRIMARY KEY,
    orcamento_id  INT NOT NULL,
    descricao     VARCHAR(255) NOT NULL,
    quantidade    INT NOT NULL,
    preco_unitario DECIMAL(12,2) NOT NULL,
    preco_total   DECIMAL(12,2) NOT NULL,
    FOREIGN KEY (orcamento_id) REFERENCES orcamentos(id) ON DELETE CASCADE,
    INDEX idx_orcitem_orc (orcamento_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ----------------------------------------------------------------------------
-- 12. Movimentações de Estoque (auditoria)
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS movimentos_estoque (
    id                     INT AUTO_INCREMENT PRIMARY KEY,
    peca_id                INT NOT NULL,
    tipo_movimento         ENUM('Entrada NF', 'Saída OS', 'Ajuste Manual', 'Edicao OS') NOT NULL,
    referencia_id          INT,
    quantidade_movimentada INT NOT NULL, -- delta (positivo = entrada, negativo = saída)
    estoque_anterior       INT NOT NULL,
    estoque_novo           INT NOT NULL,
    usuario                VARCHAR(255) NOT NULL,
    motivo                 VARCHAR(255),
    data_movimento         DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (peca_id) REFERENCES pecas(id),
    INDEX idx_mov_peca (peca_id, data_movimento),
    INDEX idx_mov_tipo (tipo_movimento),
    INDEX idx_mov_referencia (referencia_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ----------------------------------------------------------------------------
-- 13. Serviços (mão de obra)
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS servicos (
    id        INT AUTO_INCREMENT PRIMARY KEY,
    nome      VARCHAR(255) NOT NULL,
    descricao TEXT,
    preco     DECIMAL(10,2) NOT NULL DEFAULT 0.00,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_servicos_nome (nome)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ----------------------------------------------------------------------------
-- 14. Pivot: OS ↔ Serviços
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS ordem_servico_servicos (
    id               INT AUTO_INCREMENT PRIMARY KEY,
    ordem_servico_id INT NOT NULL,
    servico_id       INT NOT NULL,
    quantidade       INT NOT NULL DEFAULT 1,
    preco_unitario   DECIMAL(10,2) NOT NULL DEFAULT 0.00,
    FOREIGN KEY (ordem_servico_id) REFERENCES ordens_servico(id) ON DELETE CASCADE,
    FOREIGN KEY (servico_id)       REFERENCES servicos(id),
    INDEX idx_osserv_os (ordem_servico_id),
    INDEX idx_osserv_serv (servico_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
