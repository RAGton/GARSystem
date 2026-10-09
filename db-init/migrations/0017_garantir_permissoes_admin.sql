-- ============================================================================
-- Migration 0017: Garantir que role ADMIN tenha TODAS as permissões não-SAAS
-- ----------------------------------------------------------------------------
-- Bug encontrado em 2026-10-09 (ciclo 3 da execução autônoma):
-- A migration 0011_empresa_rbac.sql insere permissões pra ADMIN usando
-- `p.categoria != 'SAAS'`, MAS isso só funciona se TODAS as permissões
-- não-SAAS existirem na hora da execução. Como permissions são inseridas
-- dinamicamente (codigo Rust, ver empresa/service.rs), ao aplicar 0011
-- podem existir só 7 permissions não-SAAS (empresa.config + empresa.usuario.*).
-- Resultado: ADMIN fica sem crm.cliente.view, os.view, etc.
--
-- Esta migration 0017 é idempotente: INSERT IGNORE não duplica, e só adiciona
-- permissões que ainda não estão no role_permissions pra ADMIN.
-- ============================================================================

INSERT IGNORE INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r, permissions p
WHERE r.codigo = 'ADMIN'
  AND p.categoria != 'SAAS';
