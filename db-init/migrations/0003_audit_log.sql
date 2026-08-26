-- 0003_audit_log.sql
-- Sprint P1 - Fase 4: Audit trail padronizado
--
-- Cria a tabela `audit_log` para registrar todas as ações sensíveis
-- realizadas no sistema. INSERTs nesta tabela são best-effort: uma falha
-- no audit NÃO deve impedir a operação principal.

CREATE TABLE IF NOT EXISTS audit_log (
    id              BIGINT AUTO_INCREMENT PRIMARY KEY,
    -- quem fez a ação (NULL se for anônimo/system)
    usuario_id      INT,
    -- duplicado para histórico (usuário pode ser deletado depois)
    username        VARCHAR(64),
    -- tipo de ação
    acao            ENUM('CREATE', 'READ', 'UPDATE', 'DELETE', 'LOGIN', 'LOGOUT', 'OTHER') NOT NULL,
    -- entidade afetada ('usuarios', 'clientes', 'ordens_servico', etc.)
    entidade        VARCHAR(64) NOT NULL,
    -- id do registro afetado (NULL para ações em coleção)
    entity_id       VARCHAR(64),
    -- snapshot do estado anterior/posterior (JSON, opcional)
    antes           JSON,
    depois          JSON,
    -- contexto de origem
    ip              VARCHAR(45),
    user_agent      VARCHAR(255),
    request_id      VARCHAR(64),
    -- timestamp com precisão de ms para ordenação estável
    data_hora       TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    INDEX idx_audit_usuario   (usuario_id, data_hora),
    INDEX idx_audit_entidade  (entidade, entity_id, data_hora),
    INDEX idx_audit_acao       (acao, data_hora),
    INDEX idx_audit_data       (data_hora)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
