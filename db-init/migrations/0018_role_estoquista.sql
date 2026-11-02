-- ============================================================================
-- Migration 0018: Inserir role ESTOQUISTA e vincular permissões de estoque
-- ----------------------------------------------------------------------------
-- Fase 0 (Bug B8):
-- Cria a role ESTOQUISTA no catálogo de roles e vincula todas as permissões
-- da categoria ESTOQUE, além das permissões de arquivos e timeline.
-- Idempotente (INSERT IGNORE).
-- ============================================================================

INSERT IGNORE INTO roles (codigo, nome, descricao, nivel) VALUES
    ('ESTOQUISTA', 'Estoquista', 'Gerenciamento de peças, estoque, fornecedores e movimentações.', 20);

INSERT IGNORE INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.codigo = 'ESTOQUISTA'
  AND (p.categoria = 'ESTOQUE' OR p.codigo IN ('arquivo.view', 'arquivo.upload', 'arquivo.download', 'audit.timeline.view'));
