# Copilot Instructions for GAR System

## Visão Geral
- **GAR System** é um app desktop em Rust usando `eframe`/`egui` para UI e MySQL para persistência.
- O projeto é modular: separa interface, lógica de negócio e acesso a dados.
- Papéis de usuário (admin, técnico, vendedor, etc.) controlam o acesso a funcionalidades.

## Estrutura e Fluxo
- **src/main.rs**: inicializa banco, canais assíncronos (mpsc) e a aplicação principal.
- **src/aplicacao.rs**: gerencia estado global, navegação entre telas, controle dinâmico da janela (ex: redimensionamento pós-login).
- **src/servicos.rs**: lógica de negócio (ex: validação de login, criação de usuário, regras de senha). Não acessa UI diretamente.
- **src/banco_de_dados/mod.rs**: acesso ao MySQL, pool de conexões, queries SQL. Toda persistência passa por aqui.
- **src/telas/**: cada tela (login, painel_adm, etc.) é um módulo independente, responsável por seu estado e renderização.
- **src/telas/componentes/**: componentes reutilizáveis de UI (ex: sidebar).

## Convenções e Padrões
- **Comunicação UI ↔ Banco**: sempre via canais mpsc, nunca acesso direto ao banco na UI.
- **Enums**: `EstadoTela` controla navegação global; `PapelUsuario` define permissões.
- **Proteção de Usuário**: o usuário `admin` não pode ser removido.
- **Controle de Janela**: use `ViewportCommand` para alterar tamanho e redimensionamento da janela, sempre via `egui::Context`.
- **Senhas**: sempre hash com `bcrypt` antes de persistir.
- **Temas**: alternância claro/escuro implementada no núcleo da aplicação.

## Workflows de Desenvolvimento
- **Build**: `cargo run` (pré-requisito: banco MySQL rodando, ver README).
- **Banco de Dados**: use Podman/Docker para subir o MySQL (veja README para comandos e credenciais).
- **Primeiro uso**: ao rodar, cria tabela `users` e usuário `admin` (senha: `1234`).
- **Testes**: (não documentado, adicionar se houver scripts ou padrões).

## Integrações e Dependências
- **eframe/egui**: UI.
- **mysql**: acesso a banco.
- **bcrypt**: hash de senha.
- **once_cell**: pool de conexões.
- **std::sync::mpsc**: comunicação assíncrona.

## Exemplos de Padrões
- Para adicionar nova tela: crie módulo em `src/telas/`, adicione ao enum de estado e à navegação em `AplicativoPrincipal`.
- Para nova regra de negócio: implemente em `servicos.rs`, chame via canal a partir da UI.
- Para queries SQL: defina em `banco_de_dados/mod.rs`, nunca em tela ou serviço.

Consulte o [README.md](../README.md) para detalhes de setup e comandos de ambiente.
