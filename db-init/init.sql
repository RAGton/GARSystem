-- db-init/init.sql
-- Este arquivo será executado automaticamente pelo entrypoint do MySQL

CREATE DATABASE IF NOT EXISTS senior_system CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci;
USE senior_system;

-- Tabela de Usuários atualizada com o novo papel 'Estoquista'
CREATE TABLE IF NOT EXISTS users (
    id INT AUTO_INCREMENT PRIMARY KEY,
    username VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    role ENUM('Administrador', 'Gerencia', 'Tecnico', 'Financeiro', 'Comercial', 'Estoquista') NOT NULL DEFAULT 'Comercial'
);

-- Inserir usuário admin padrão
INSERT IGNORE INTO users (username, password_hash, role)
VALUES ('admin', '$2b$12$AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA', 'Administrador');


-- Tabelas existentes de Clientes e Equipamentos
CREATE TABLE IF NOT EXISTS clientes (
    id INT AUTO_INCREMENT PRIMARY KEY,
    nome VARCHAR(255) NOT NULL,
    telefone VARCHAR(20),
    email VARCHAR(255),
    endereco TEXT
);

CREATE TABLE IF NOT EXISTS equipamentos (
    id INT AUTO_INCREMENT PRIMARY KEY,
    cliente_id INT NOT NULL,
    descricao VARCHAR(255) NOT NULL,
    marca VARCHAR(100),
    modelo VARCHAR(100),
    numero_serie VARCHAR(255) UNIQUE,
    FOREIGN KEY (cliente_id) REFERENCES clientes(id)
);

-- Tabela de Ordens de Serviço
CREATE TABLE IF NOT EXISTS ordens_servico (
    id INT AUTO_INCREMENT PRIMARY KEY,
    cliente_id INT NOT NULL,
    equipamento_id INT NOT NULL,
    defeito_relatado TEXT NOT NULL,
    observacoes TEXT,
    parecer_tecnico TEXT,
    status ENUM('Aberta', 'EmAndamento', 'AguardandoPeca', 'Finalizada', 'Cancelada') NOT NULL DEFAULT 'Aberta',
    situacao ENUM('Orcamento', 'Aprovado', 'EmAndamento', 'AutorizadoAguardandoPeca', 'ServicoConcluido', 'AguardandoAutorizacao', 'AguardandoRetirada', 'Reprovado', 'AguardandoFaturar', 'Faturado') NOT NULL DEFAULT 'Orcamento',
    atendente VARCHAR(255),
    tecnico_responsavel VARCHAR(255),
    data_chegada DATETIME DEFAULT CURRENT_TIMESTAMP,
    prazo_entrega DATETIME,
    FOREIGN KEY (cliente_id) REFERENCES clientes(id),
    FOREIGN KEY (equipamento_id) REFERENCES equipamentos(id)
);

-- Tabela de Histórico de Edições da OS
CREATE TABLE IF NOT EXISTS historico_edicoes (
    id INT AUTO_INCREMENT PRIMARY KEY,
    ordem_servico_id INT NOT NULL,
    usuario VARCHAR(255) NOT NULL,
    data_hora DATETIME DEFAULT CURRENT_TIMESTAMP,
    campo_alterado VARCHAR(255) NOT NULL,
    valor_antigo TEXT,
    valor_novo TEXT,
    FOREIGN KEY (ordem_servico_id) REFERENCES ordens_servico(id) ON DELETE CASCADE
);

-- --- [NOVO] Tabelas do Módulo de Estoque ---

CREATE TABLE IF NOT EXISTS fornecedores (
    id INT AUTO_INCREMENT PRIMARY KEY,
    nome VARCHAR(255) NOT NULL UNIQUE,
    cnpj VARCHAR(20) UNIQUE,
    contato VARCHAR(255),
    telefone VARCHAR(25),
    email VARCHAR(255)
);

CREATE TABLE IF NOT EXISTS pecas (
    id INT AUTO_INCREMENT PRIMARY KEY,
    codigo_interno VARCHAR(50) UNIQUE NOT NULL,
    part_number VARCHAR(100),
    descricao VARCHAR(255) NOT NULL,
    fabricante VARCHAR(100),
    localizacao VARCHAR(100), -- Ex: "Prateleira A-15, Gaveta 3"
    estoque_atual INT NOT NULL DEFAULT 0,
    estoque_minimo INT NOT NULL DEFAULT 1,
    preco_custo DECIMAL(10, 2) NOT NULL DEFAULT 0.00,
    preco_venda DECIMAL(10, 2) NOT NULL DEFAULT 0.00,
    ultima_atualizacao TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS notas_fiscais_entrada (
    id INT AUTO_INCREMENT PRIMARY KEY,
    numero_nf VARCHAR(50) NOT NULL,
    fornecedor_id INT NOT NULL,
    data_emissao DATE,
    data_entrada DATETIME DEFAULT CURRENT_TIMESTAMP,
    valor_total DECIMAL(10, 2),
    observacoes TEXT,
    FOREIGN KEY (fornecedor_id) REFERENCES fornecedores(id)
);

-- Tabela pivot para relacionar peças e notas fiscais (muitos-para-muitos)
CREATE TABLE IF NOT EXISTS nf_entrada_pecas (
    id INT AUTO_INCREMENT PRIMARY KEY,
    nota_fiscal_id INT NOT NULL,
    peca_id INT NOT NULL,
    quantidade INT NOT NULL,
    preco_custo_unitario DECIMAL(10, 2) NOT NULL,
    FOREIGN KEY (nota_fiscal_id) REFERENCES notas_fiscais_entrada(id) ON DELETE CASCADE,
    FOREIGN KEY (peca_id) REFERENCES pecas(id)
);

-- Tabela pivot para relacionar peças e Ordens de Serviço
CREATE TABLE IF NOT EXISTS ordem_servico_pecas (
    id INT AUTO_INCREMENT PRIMARY KEY,
    ordem_servico_id INT NOT NULL,
    peca_id INT NOT NULL,
    quantidade INT NOT NULL,
    preco_venda_unitario DECIMAL(10, 2) NOT NULL,
    FOREIGN KEY (ordem_servico_id) REFERENCES ordens_servico(id) ON DELETE CASCADE,
    FOREIGN KEY (peca_id) REFERENCES pecas(id)
);

-- Tabelas para Orçamentos
CREATE TABLE IF NOT EXISTS orcamentos (
    id INT AUTO_INCREMENT PRIMARY KEY,
    cliente_id INT NOT NULL,
    total DECIMAL(12,2) NOT NULL DEFAULT 0.00,
    data_criacao DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (cliente_id) REFERENCES clientes(id)
);

CREATE TABLE IF NOT EXISTS orcamento_items (
    id INT AUTO_INCREMENT PRIMARY KEY,
    orcamento_id INT NOT NULL,
    descricao VARCHAR(255) NOT NULL,
    quantidade INT NOT NULL,
    preco_unitario DECIMAL(12,2) NOT NULL,
    preco_total DECIMAL(12,2) NOT NULL,
    FOREIGN KEY (orcamento_id) REFERENCES orcamentos(id) ON DELETE CASCADE
);

-- Tabela para rastrear TODAS as movimentações de estoque (AUDITORIA)
CREATE TABLE IF NOT EXISTS movimentos_estoque (
    id INT AUTO_INCREMENT PRIMARY KEY,
    peca_id INT NOT NULL,
    tipo_movimento ENUM('Entrada NF', 'Saída OS', 'Ajuste Manual') NOT NULL,
    referencia_id INT, -- ID da NF ou da OS
    quantidade_movimentada INT NOT NULL, -- Positivo para entrada, negativo para saída
    estoque_anterior INT NOT NULL,
    estoque_novo INT NOT NULL,
    usuario VARCHAR(255),
    data_movimento DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (peca_id) REFERENCES pecas(id)
);

-- --- [NOVO] Tabelas do Módulo de Estoque ---

CREATE TABLE IF NOT EXISTS fornecedores (
    id INT AUTO_INCREMENT PRIMARY KEY,
    nome VARCHAR(255) NOT NULL UNIQUE,
    cnpj VARCHAR(20) UNIQUE,
    contato VARCHAR(255),
    telefone VARCHAR(25),
    email VARCHAR(255)
);

CREATE TABLE IF NOT EXISTS pecas (
    id INT AUTO_INCREMENT PRIMARY KEY,
    codigo_interno VARCHAR(50) UNIQUE NOT NULL,
    part_number VARCHAR(100),
    descricao VARCHAR(255) NOT NULL,
    fabricante VARCHAR(100),
    localizacao VARCHAR(100), -- Ex: "Prateleira A-15, Gaveta 3"
    estoque_atual INT NOT NULL DEFAULT 0,
    estoque_minimo INT NOT NULL DEFAULT 1,
    preco_custo DECIMAL(10, 2) NOT NULL DEFAULT 0.00,
    preco_venda DECIMAL(10, 2) NOT NULL DEFAULT 0.00,
    ultima_atualizacao TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS notas_fiscais_entrada (
    id INT AUTO_INCREMENT PRIMARY KEY,
    numero_nf VARCHAR(50) NOT NULL,
    fornecedor_id INT NOT NULL,
    data_emissao DATE,
    data_entrada DATETIME DEFAULT CURRENT_TIMESTAMP,
    valor_total DECIMAL(10, 2),
    observacoes TEXT,
    FOREIGN KEY (fornecedor_id) REFERENCES fornecedores(id)
);

-- Tabela pivot para relacionar peças e notas fiscais (muitos-para-muitos)
CREATE TABLE IF NOT EXISTS nf_entrada_pecas (
    id INT AUTO_INCREMENT PRIMARY KEY,
    nota_fiscal_id INT NOT NULL,
    peca_id INT NOT NULL,
    quantidade INT NOT NULL,
    preco_custo_unitario DECIMAL(10, 2) NOT NULL,
    FOREIGN KEY (nota_fiscal_id) REFERENCES notas_fiscais_entrada(id) ON DELETE CASCADE,
    FOREIGN KEY (peca_id) REFERENCES pecas(id)
);

-- Tabela pivot para relacionar peças e Ordens de Serviço
CREATE TABLE IF NOT EXISTS ordem_servico_pecas (
    id INT AUTO_INCREMENT PRIMARY KEY,
    ordem_servico_id INT NOT NULL,
    peca_id INT NOT NULL,
    quantidade INT NOT NULL,
    preco_venda_unitario DECIMAL(10, 2) NOT NULL,
    FOREIGN KEY (ordem_servico_id) REFERENCES ordens_servico(id) ON DELETE CASCADE,
    FOREIGN KEY (peca_id) REFERENCES pecas(id)
);

-- Tabela para rastrear TODAS as movimentações de estoque (AUDITORIA)
CREATE TABLE IF NOT EXISTS movimentos_estoque (
    id INT AUTO_INCREMENT PRIMARY KEY,
    peca_id INT NOT NULL,
    tipo_movimento ENUM('Entrada NF', 'Saída OS', 'Ajuste Manual') NOT NULL,
    referencia_id INT, -- ID da NF ou da OS
    quantidade_movimentada INT NOT NULL, -- Positivo para entrada, negativo para saída
    estoque_anterior INT NOT NULL,
    estoque_novo INT NOT NULL,
    usuario VARCHAR(255),
    data_movimento DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (peca_id) REFERENCES pecas(id)
);
