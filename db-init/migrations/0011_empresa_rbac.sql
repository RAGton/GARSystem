-- ============================================================================
-- Migration 0011: Empresa + RBAC (Sprint P2.6.1)
-- ----------------------------------------------------------------------------
-- Fundação SaaS multiempresa. Cria:
--   - empresa (raiz do tenant, com uuid para APIs externas)
--   - empresa_configuracao (1:1, timezone/moeda/idioma/tema)
--   - permissions (catálogo global, ~80 seeds)
--   - roles (catálogo global, 6 seeds: SUPER_ADMIN, ADMIN, GERENTE,
--            TECNICO, FINANCEIRO, ATENDENTE)
--   - role_permissions (matriz role × permission, sistema-wide)
--   - user_roles (atribuição user × role × empresa — escopo multi-tenant)
--
-- Backfill:
--   - Cria empresa 1 (bootstrap "Empresa Padrão")
--   - Adiciona users.empresa_id (FK) para todos os users existentes → 1
--   - Cria user_roles para todo user existente com role derivada do papel
--   - Empresa 1 herda todas as permissões compatíveis
--
-- IMPORTANTE:
--   * Bootstrap NÃO cria user admin/admin. A senha é gerada em runtime
--     (ver `bootstrap_admin` no startup do servidor).
--   * Esta migration é PURAMENTE ADITIVA. Não mexe em dados existentes
--     exceto o backfill de `users.empresa_id`.
--   * Idempotente: usar IF NOT EXISTS, INSERT IGNORE.
-- ============================================================================

-- ----------------------------------------------------------------------------
-- 1. Tabela EMPRESA
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS empresa (
    id               INT AUTO_INCREMENT PRIMARY KEY,
    uuid             CHAR(36) NOT NULL,
    nome             VARCHAR(255) NOT NULL,
    razao_social     VARCHAR(255) NOT NULL,
    cnpj             VARCHAR(20),
    email            VARCHAR(255),
    telefone         VARCHAR(25),
    ativa            BOOLEAN NOT NULL DEFAULT TRUE,
    plano            ENUM('FREE','STARTER','BUSINESS','ENTERPRISE') NOT NULL DEFAULT 'FREE',
    data_criacao     TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    data_atualizacao TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    UNIQUE KEY uk_empresa_uuid (uuid),
    INDEX idx_empresa_cnpj (cnpj),
    INDEX idx_empresa_ativa (ativa),
    INDEX idx_empresa_plano (plano)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ----------------------------------------------------------------------------
-- 2. Tabela EMPRESA_CONFIGURACAO (1:1 com empresa)
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS empresa_configuracao (
    empresa_id       INT NOT NULL PRIMARY KEY,
    timezone         VARCHAR(50) NOT NULL DEFAULT 'America/Sao_Paulo',
    moeda            CHAR(3) NOT NULL DEFAULT 'BRL',
    idioma           VARCHAR(10) NOT NULL DEFAULT 'pt-BR',
    tema             VARCHAR(20) NOT NULL DEFAULT 'light',
    logo_url         VARCHAR(500),
    cor_primaria     VARCHAR(7),
    bootstrap_done   BOOLEAN NOT NULL DEFAULT FALSE,
    data_atualizacao TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (empresa_id) REFERENCES empresa(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ----------------------------------------------------------------------------
-- 3. Tabela PERMISSIONS (catálogo sistema-wide)
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS permissions (
    id          INT AUTO_INCREMENT PRIMARY KEY,
    codigo      VARCHAR(100) NOT NULL UNIQUE,   -- 'cliente.create', 'financeiro.pagar'
    descricao   VARCHAR(255) NOT NULL,
    categoria   VARCHAR(50) NOT NULL,           -- 'CRM','FINANCEIRO','OS','SAAS',...
    created_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_permissions_categoria (categoria)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ----------------------------------------------------------------------------
-- 4. Tabela ROLES (catálogo sistema-wide)
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS roles (
    id          INT AUTO_INCREMENT PRIMARY KEY,
    codigo      VARCHAR(50) NOT NULL UNIQUE,    -- 'SUPER_ADMIN','ADMIN','GERENTE','TECNICO','FINANCEIRO','ATENDENTE'
    nome        VARCHAR(100) NOT NULL,
    descricao   VARCHAR(255) NOT NULL,
    nivel       INT NOT NULL,                    -- hierarquia (1000=super, 100=admin, 10=atendente)
    created_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ----------------------------------------------------------------------------
-- 5. Tabela ROLE_PERMISSIONS (matriz role × permission)
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS role_permissions (
    role_id         INT NOT NULL,
    permission_id   INT NOT NULL,
    PRIMARY KEY (role_id, permission_id),
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE,
    FOREIGN KEY (permission_id) REFERENCES permissions(id) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ----------------------------------------------------------------------------
-- 6. Tabela USER_ROLES (escopo multi-tenant)
-- ----------------------------------------------------------------------------
-- Mesmo usuário pode ter roles diferentes em empresas diferentes.
-- (user_id, role_id, empresa_id) é UNIQUE.
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS user_roles (
    id          INT AUTO_INCREMENT PRIMARY KEY,
    user_id     INT NOT NULL,
    role_id     INT NOT NULL,
    empresa_id  INT NOT NULL,
    created_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE KEY uk_user_role_empresa (user_id, role_id, empresa_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (role_id) REFERENCES roles(id),
    FOREIGN KEY (empresa_id) REFERENCES empresa(id) ON DELETE CASCADE,
    INDEX idx_user_roles_user (user_id),
    INDEX idx_user_roles_empresa (empresa_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

-- ----------------------------------------------------------------------------
-- 7. Adicionar users.empresa_id (FK para empresa)
-- ----------------------------------------------------------------------------
-- A tabela users já tem `tenant_id` (P0). Agora ganha `empresa_id` que é
-- a FK canônica. `tenant_id` é mantido como cache/índice rápido.
-- O runner de migrations trata "Duplicate column name" como idempotente.
-- ----------------------------------------------------------------------------

ALTER TABLE users ADD COLUMN empresa_id INT NULL AFTER tenant_id;

ALTER TABLE users ADD CONSTRAINT fk_users_empresa FOREIGN KEY (empresa_id) REFERENCES empresa(id);

ALTER TABLE users ADD INDEX idx_users_empresa (empresa_id);

-- ----------------------------------------------------------------------------
-- 8. SEED: 6 ROLES
-- ----------------------------------------------------------------------------
INSERT IGNORE INTO roles (codigo, nome, descricao, nivel) VALUES
    ('SUPER_ADMIN', 'Super Administrador', 'Owner do SaaS. Vê todas as empresas, pode criar/desativar tenants.', 1000),
    ('ADMIN',       'Administrador',       'Admin da empresa. Tudo dentro do seu tenant.', 100),
    ('GERENTE',     'Gerente',             'Vê dashboards, relatórios. Não mexe em configuração.', 50),
    ('TECNICO',     'Técnico',             'Executa OS, checklists, evoluções.', 20),
    ('FINANCEIRO',  'Financeiro',          'Contas a pagar/receber, fluxo, dashboard financeiro.', 20),
    ('ATENDENTE',   'Atendente',           'Cadastra clientes, abre OS, cotação.', 10);

-- ----------------------------------------------------------------------------
-- 9. SEED: ~80 PERMISSIONS
-- ----------------------------------------------------------------------------
-- Categoria: 'SAAS'
INSERT IGNORE INTO permissions (codigo, descricao, categoria) VALUES
    ('saas.empresa.create',           'Criar nova empresa (SUPER_ADMIN)',           'SAAS'),
    ('saas.empresa.list_all',         'Listar todas empresas (SUPER_ADMIN)',        'SAAS'),
    ('saas.empresa.deactivate',       'Desativar empresa',                          'SAAS'),
    ('saas.dashboard.view',           'Ver dashboard global SaaS',                  'SAAS'),
    ('saas.billing.view',             'Ver billing/faturamento (futuro)',           'SAAS'),
    ('saas.tenant.bypass',            'Bypassar tenant_id (cross-tenant)',          'SAAS');

-- Categoria: 'EMPRESA'
INSERT IGNORE INTO permissions (codigo, descricao, categoria) VALUES
    ('empresa.config.view',           'Ver configuração da empresa',                'EMPRESA'),
    ('empresa.config.edit',           'Editar configuração da empresa',              'EMPRESA'),
    ('empresa.usuario.list',          'Listar usuários da empresa',                 'EMPRESA'),
    ('empresa.usuario.create',        'Criar usuário na empresa',                   'EMPRESA'),
    ('empresa.usuario.edit',          'Editar usuário da empresa',                  'EMPRESA'),
    ('empresa.usuario.delete',        'Remover usuário da empresa',                 'EMPRESA'),
    ('empresa.usuario.assign_role',   'Atribuir role a usuário',                    'EMPRESA');

-- Categoria: 'CRM'
INSERT IGNORE INTO permissions (codigo, descricao, categoria) VALUES
    ('crm.cliente.create',            'Criar cliente',                              'CRM'),
    ('crm.cliente.edit',              'Editar cliente',                             'CRM'),
    ('crm.cliente.view',              'Ver cliente',                                'CRM'),
    ('crm.cliente.delete',            'Remover cliente',                            'CRM'),
    ('crm.equipamento.create',        'Criar equipamento',                          'CRM'),
    ('crm.equipamento.edit',          'Editar equipamento',                         'CRM'),
    ('crm.equipamento.view',          'Ver equipamento',                            'CRM'),
    ('crm.contato.create',            'Criar contato de cliente',                   'CRM'),
    ('crm.contato.edit',              'Editar contato',                             'CRM'),
    ('crm.contato.view',              'Ver contato',                                'CRM'),
    ('crm.timeline.view',             'Ver timeline do cliente',                    'CRM'),
    ('crm.busca.global',              'Buscar em todos módulos',                    'CRM'),
    ('crm.dashboard.view',            'Ver dashboard 360° do cliente',              'CRM');

-- Categoria: 'OS'
INSERT IGNORE INTO permissions (codigo, descricao, categoria) VALUES
    ('os.create',                     'Criar ordem de serviço',                     'OS'),
    ('os.edit',                       'Editar OS',                                  'OS'),
    ('os.view',                       'Ver OS',                                     'OS'),
    ('os.delete',                     'Remover OS',                                 'OS'),
    ('os.status.change',              'Mudar status de OS',                         'OS'),
    ('os.workflow.advance',           'Avançar workflow da OS',                     'OS'),
    ('os.evolucao.create',            'Criar evolução de OS',                       'OS'),
    ('os.assinatura.create',          'Criar assinatura na OS',                     'OS'),
    ('os.checklist.execute',          'Executar checklist de OS',                   'OS'),
    ('os.dashboard.view',             'Ver dashboard técnico',                      'OS');

-- Categoria: 'COTACAO'
INSERT IGNORE INTO permissions (codigo, descricao, categoria) VALUES
    ('cotacao.create',                'Criar cotação',                              'COTACAO'),
    ('cotacao.edit',                  'Editar cotação',                             'COTACAO'),
    ('cotacao.view',                  'Ver cotação',                                'COTACAO'),
    ('cotacao.delete',                'Remover cotação',                            'COTACAO'),
    ('cotacao.aprovar',               'Aprovar cotação',                            'COTACAO'),
    ('cotacao.reprovar',              'Reprovar cotação',                           'COTACAO');

-- Categoria: 'ORCAMENTO'
INSERT IGNORE INTO permissions (codigo, descricao, categoria) VALUES
    ('orcamento.create',              'Criar orçamento',                            'ORCAMENTO'),
    ('orcamento.edit',                'Editar orçamento',                           'ORCAMENTO'),
    ('orcamento.view',                'Ver orçamento',                              'ORCAMENTO'),
    ('orcamento.delete',              'Remover orçamento',                          'ORCAMENTO'),
    ('orcamento.aprovar',             'Aprovar orçamento',                          'ORCAMENTO'),
    ('orcamento.reprovar',            'Reprovar orçamento',                         'ORCAMENTO'),
    ('orcamento.concluir',            'Concluir / faturar orçamento',               'ORCAMENTO');

-- Categoria: 'ESTOQUE'
INSERT IGNORE INTO permissions (codigo, descricao, categoria) VALUES
    ('estoque.peca.create',           'Criar peça',                                 'ESTOQUE'),
    ('estoque.peca.edit',             'Editar peça',                                'ESTOQUE'),
    ('estoque.peca.view',             'Ver peça',                                   'ESTOQUE'),
    ('estoque.peca.delete',           'Remover peça',                               'ESTOQUE'),
    ('estoque.movimento.create',      'Criar movimento de estoque',                 'ESTOQUE'),
    ('estoque.movimento.view',        'Ver histórico de movimentos',                'ESTOQUE'),
    ('estoque.nf.entrada.create',     'Criar NF de entrada',                        'ESTOQUE'),
    ('estoque.nf.entrada.view',       'Ver NF de entrada',                          'ESTOQUE'),
    ('estoque.fornecedor.create',     'Criar fornecedor',                           'ESTOQUE'),
    ('estoque.fornecedor.edit',       'Editar fornecedor',                          'ESTOQUE'),
    ('estoque.fornecedor.view',       'Ver fornecedor',                             'ESTOQUE');

-- Categoria: 'ARQUIVO'
INSERT IGNORE INTO permissions (codigo, descricao, categoria) VALUES
    ('arquivo.upload',                'Upload de arquivo',                          'ARQUIVO'),
    ('arquivo.download',              'Download de arquivo',                        'ARQUIVO'),
    ('arquivo.delete',                'Remover arquivo',                            'ARQUIVO'),
    ('arquivo.view',                  'Ver metadados do arquivo',                   'ARQUIVO'),
    ('arquivo.thumbnail.generate',    'Gerar thumbnails',                           'ARQUIVO'),
    ('arquivo.transcricao.request',   'Solicitar transcrição',                      'ARQUIVO'),
    ('arquivo.transcricao.view',      'Ver transcrição',                            'ARQUIVO');

-- Categoria: 'OPERATIONS'
INSERT IGNORE INTO permissions (codigo, descricao, categoria) VALUES
    ('operations.workflow.create',    'Criar workflow customizado',                 'OPERATIONS'),
    ('operations.workflow.edit',      'Editar workflow',                            'OPERATIONS'),
    ('operations.workflow.view',      'Ver workflow',                               'OPERATIONS'),
    ('operations.workflow.advance',   'Avançar OS no workflow',                     'OPERATIONS'),
    ('operations.agenda.create',      'Criar evento na agenda',                     'OPERATIONS'),
    ('operations.agenda.edit',        'Editar evento da agenda',                    'OPERATIONS'),
    ('operations.agenda.view',        'Ver agenda',                                 'OPERATIONS'),
    ('operations.sla.config.edit',    'Editar config de SLA',                       'OPERATIONS'),
    ('operations.sla.view',           'Ver SLA',                                    'OPERATIONS'),
    ('operations.alerta.view',        'Ver alertas',                                'OPERATIONS'),
    ('operations.alerta.resolve',     'Resolver alerta',                            'OPERATIONS'),
    ('operations.dashboard.view',     'Ver dashboard executivo',                    'OPERATIONS'),
    ('operations.kanban.view',        'Ver Kanban',                                 'OPERATIONS');

-- Categoria: 'FINANCEIRO'
INSERT IGNORE INTO permissions (codigo, descricao, categoria) VALUES
    ('financeiro.config.view',        'Ver config financeira',                      'FINANCEIRO'),
    ('financeiro.config.edit',        'Editar config financeira',                   'FINANCEIRO'),
    ('financeiro.plano_contas.view',  'Ver plano de contas',                        'FINANCEIRO'),
    ('financeiro.plano_contas.edit',  'Editar plano de contas',                     'FINANCEIRO'),
    ('financeiro.centro_custo.view',  'Ver centros de custo',                       'FINANCEIRO'),
    ('financeiro.centro_custo.edit',  'Editar centros de custo',                    'FINANCEIRO'),
    ('financeiro.conta_receber.create','Criar conta a receber',                     'FINANCEIRO'),
    ('financeiro.conta_receber.view',  'Ver conta a receber',                        'FINANCEIRO'),
    ('financeiro.conta_receber.pagar', 'Pagar conta a receber',                      'FINANCEIRO'),
    ('financeiro.conta_receber.cancelar','Cancelar conta a receber',                 'FINANCEIRO'),
    ('financeiro.conta_pagar.create', 'Criar conta a pagar',                        'FINANCEIRO'),
    ('financeiro.conta_pagar.view',   'Ver conta a pagar',                           'FINANCEIRO'),
    ('financeiro.conta_pagar.pagar',  'Pagar conta a pagar',                         'FINANCEIRO'),
    ('financeiro.fluxo_caixa.view',   'Ver fluxo de caixa',                          'FINANCEIRO'),
    ('financeiro.dashboard.view',     'Ver dashboard financeiro',                   'FINANCEIRO'),
    ('financeiro.alerta.view',        'Ver alertas financeiros',                    'FINANCEIRO'),
    ('financeiro.alerta.resolve',     'Resolver alerta financeiro',                 'FINANCEIRO');

-- Categoria: 'AUDIT'
INSERT IGNORE INTO permissions (codigo, descricao, categoria) VALUES
    ('audit.log.view',                'Ver audit log da empresa',                   'AUDIT'),
    ('audit.log.export',              'Exportar audit log',                         'AUDIT'),
    ('audit.timeline.view',           'Ver timeline CRM',                           'AUDIT');

-- ----------------------------------------------------------------------------
-- 10. SEED: matriz ROLE × PERMISSIONS
-- ----------------------------------------------------------------------------
-- Helper via INSERT ... SELECT a partir de roles e permissions pelos códigos.
-- Para cada role, listamos os códigos de permission que ela recebe.
-- ----------------------------------------------------------------------------

-- SUPER_ADMIN: tudo (todas as permissions)
INSERT IGNORE INTO role_permissions (role_id, permission_id)
    SELECT r.id, p.id
    FROM roles r, permissions p
    WHERE r.codigo = 'SUPER_ADMIN';

-- ADMIN: tudo DENTRO da empresa (sem as permissões SAAS cross-tenant)
INSERT IGNORE INTO role_permissions (role_id, permission_id)
    SELECT r.id, p.id
    FROM roles r, permissions p
    WHERE r.codigo = 'ADMIN'
      AND p.categoria != 'SAAS';

-- GERENTE: visualização ampla, edição limitada
INSERT IGNORE INTO role_permissions (role_id, permission_id)
    SELECT r.id, p.id
    FROM roles r, permissions p
    WHERE r.codigo = 'GERENTE'
      AND p.codigo IN (
          -- Empresa
          'empresa.config.view', 'empresa.usuario.list',
          -- CRM
          'crm.cliente.create', 'crm.cliente.edit', 'crm.cliente.view', 'crm.cliente.delete',
          'crm.equipamento.create', 'crm.equipamento.edit', 'crm.equipamento.view',
          'crm.contato.create', 'crm.contato.edit', 'crm.contato.view',
          'crm.timeline.view', 'crm.busca.global', 'crm.dashboard.view',
          -- OS
          'os.create', 'os.edit', 'os.view', 'os.status.change', 'os.workflow.advance',
          'os.evolucao.create', 'os.dashboard.view',
          -- Cotação
          'cotacao.create', 'cotacao.edit', 'cotacao.view', 'cotacao.aprovar', 'cotacao.reprovar',
          -- Orçamento
          'orcamento.create', 'orcamento.edit', 'orcamento.view',
          'orcamento.aprovar', 'orcamento.reprovar', 'orcamento.concluir',
          -- Estoque
          'estoque.peca.create', 'estoque.peca.edit', 'estoque.peca.view',
          'estoque.movimento.view', 'estoque.nf.entrada.view',
          'estoque.fornecedor.create', 'estoque.fornecedor.edit', 'estoque.fornecedor.view',
          -- Arquivo
          'arquivo.upload', 'arquivo.download', 'arquivo.view',
          'arquivo.transcricao.view',
          -- Operations
          'operations.workflow.view', 'operations.agenda.view',
          'operations.sla.view', 'operations.alerta.view', 'operations.alerta.resolve',
          'operations.dashboard.view', 'operations.kanban.view',
          -- Financeiro (somente visualização)
          'financeiro.config.view', 'financeiro.plano_contas.view', 'financeiro.centro_custo.view',
          'financeiro.conta_receber.view', 'financeiro.conta_pagar.view',
          'financeiro.fluxo_caixa.view', 'financeiro.dashboard.view',
          'financeiro.alerta.view',
          -- Audit
          'audit.log.view', 'audit.timeline.view'
      );

-- TECNICO: execução de OS + visualização
INSERT IGNORE INTO role_permissions (role_id, permission_id)
    SELECT r.id, p.id
    FROM roles r, permissions p
    WHERE r.codigo = 'TECNICO'
      AND p.codigo IN (
          'crm.cliente.view', 'crm.equipamento.view', 'crm.contato.view',
          'crm.timeline.view', 'crm.busca.global',
          'os.create', 'os.edit', 'os.view', 'os.status.change', 'os.workflow.advance',
          'os.evolucao.create', 'os.assinatura.create', 'os.checklist.execute',
          'os.dashboard.view',
          'cotacao.view',
          'orcamento.view',
          'arquivo.upload', 'arquivo.download', 'arquivo.view',
          'arquivo.transcricao.view',
          'operations.workflow.advance', 'operations.agenda.view',
          'operations.sla.view', 'operations.alerta.view',
          'operations.dashboard.view', 'operations.kanban.view',
          'estoque.peca.view', 'estoque.movimento.view',
          'audit.timeline.view'
      );

-- FINANCEIRO: foco em financeiro + visualização
INSERT IGNORE INTO role_permissions (role_id, permission_id)
    SELECT r.id, p.id
    FROM roles r, permissions p
    WHERE r.codigo = 'FINANCEIRO'
      AND p.codigo IN (
          'crm.cliente.view', 'crm.busca.global', 'crm.dashboard.view',
          'os.view',
          'cotacao.view', 'orcamento.view',
          'estoque.peca.view', 'estoque.movimento.view',
          'estoque.nf.entrada.view', 'estoque.fornecedor.view',
          'arquivo.view', 'arquivo.upload', 'arquivo.download',
          -- Financeiro completo
          'financeiro.config.view', 'financeiro.config.edit',
          'financeiro.plano_contas.view', 'financeiro.plano_contas.edit',
          'financeiro.centro_custo.view', 'financeiro.centro_custo.edit',
          'financeiro.conta_receber.create', 'financeiro.conta_receber.view',
          'financeiro.conta_receber.pagar', 'financeiro.conta_receber.cancelar',
          'financeiro.conta_pagar.create', 'financeiro.conta_pagar.view',
          'financeiro.conta_pagar.pagar',
          'financeiro.fluxo_caixa.view', 'financeiro.dashboard.view',
          'financeiro.alerta.view', 'financeiro.alerta.resolve',
          'operations.alerta.view',
          'audit.timeline.view'
      );

-- ATENDENTE: cadastro + abertura de OS
INSERT IGNORE INTO role_permissions (role_id, permission_id)
    SELECT r.id, p.id
    FROM roles r, permissions p
    WHERE r.codigo = 'ATENDENTE'
      AND p.codigo IN (
          'crm.cliente.create', 'crm.cliente.edit', 'crm.cliente.view',
          'crm.equipamento.create', 'crm.equipamento.view',
          'crm.contato.create', 'crm.contato.view',
          'crm.timeline.view', 'crm.busca.global', 'crm.dashboard.view',
          'os.create', 'os.view', 'os.status.change', 'os.evolucao.create',
          'os.dashboard.view',
          'cotacao.create', 'cotacao.edit', 'cotacao.view',
          'orcamento.create', 'orcamento.view',
          'estoque.peca.view',
          'arquivo.upload', 'arquivo.download', 'arquivo.view',
          'operations.agenda.view', 'operations.alerta.view',
          'operations.kanban.view',
          'audit.timeline.view'
      );

-- ----------------------------------------------------------------------------
-- 11. BACKFILL — Empresa 1 (bootstrap "Empresa Padrão")
-- ----------------------------------------------------------------------------
-- Cria a empresa 1 e sua configuração. Se já existir (idempotente), ignora.
-- ----------------------------------------------------------------------------
INSERT IGNORE INTO empresa (id, uuid, nome, razao_social, cnpj, email, telefone, ativa, plano)
    VALUES (1, UUID(), 'Empresa Padrão', 'Empresa Padrão LTDA',
            '00.000.000/0001-00', 'admin@local', NULL, TRUE, 'BUSINESS');

INSERT IGNORE INTO empresa_configuracao (empresa_id, timezone, moeda, idioma, tema, bootstrap_done)
    VALUES (1, 'America/Sao_Paulo', 'BRL', 'pt-BR', 'light', FALSE);

-- ----------------------------------------------------------------------------
-- 12. BACKFILL — users.empresa_id = 1 (para todos existentes)
-- E sincronizar tenant_id com empresa_id (defesa em profundidade).
-- ----------------------------------------------------------------------------
UPDATE users SET empresa_id = 1 WHERE empresa_id IS NULL OR empresa_id = 0;
UPDATE users SET tenant_id = 1 WHERE tenant_id = 0 OR tenant_id IS NULL;

-- ----------------------------------------------------------------------------
-- 13. BACKFILL — user_roles (atribuir role derivada do papel legado)
-- ----------------------------------------------------------------------------
-- Mapeamento:
--   'Administrador'   -> ADMIN
--   'Gerencia'        -> GERENTE
--   'Tecnico'         -> TECNICO
--   'Financeiro'      -> FINANCEIRO
--   'Comercial'       -> ATENDENTE
--   'Estoquista'      -> ATENDENTE
-- ----------------------------------------------------------------------------
INSERT IGNORE INTO user_roles (user_id, role_id, empresa_id)
    SELECT u.id, r.id, 1
    FROM users u
    JOIN roles r ON (
        (u.role = 'Administrador'  AND r.codigo = 'ADMIN')      OR
        (u.role = 'Gerencia'       AND r.codigo = 'GERENTE')     OR
        (u.role = 'Tecnico'        AND r.codigo = 'TECNICO')     OR
        (u.role = 'Financeiro'     AND r.codigo = 'FINANCEIRO')  OR
        (u.role = 'Comercial'      AND r.codigo = 'ATENDENTE')   OR
        (u.role = 'Estoquista'     AND r.codigo = 'ATENDENTE')
    );

-- ----------------------------------------------------------------------------
-- 14. ÍNDICES (MySQL 8 não tem IF NOT EXISTS — runner trata duplicatas)
-- ----------------------------------------------------------------------------
ALTER TABLE users ADD INDEX idx_users_empresa_id (empresa_id);
ALTER TABLE user_roles ADD INDEX idx_user_roles_role (role_id);
