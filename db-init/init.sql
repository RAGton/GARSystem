-- init.sql
-- Este arquivo será executado automaticamente pelo entrypoint do MySQL
-- Cria a base e a tabela users com os papéis esperados pela aplicação.

CREATE DATABASE IF NOT EXISTS senior_system CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci;
USE senior_system;

CREATE TABLE IF NOT EXISTS users (
    id INT AUTO_INCREMENT PRIMARY KEY,
    username VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    role ENUM('Administrador', 'Gerencia', 'Tecnico', 'Financeiro', 'Comercial') NOT NULL DEFAULT 'Comercial'
);

-- NOTA: não insira um hash placeholder aqui. Gere um bcrypt válido ou crie o usuário admin depois.
-- Exemplo (opcional): inserir admin só se não existir (sem senha em texto)
INSERT IGNORE INTO users (username, password_hash, role)
VALUES ('admin', '$2b$12$AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA', 'Administrador');
-- O hash acima é um placeholder. Substitua por um hash bcrypt real ou crie o admin manualmente depois.