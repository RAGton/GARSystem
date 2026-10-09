-----

# Documentação do Projeto: GAR System (v1.9.0 - Fundação Segura)

## 📌 Status & Documentação

| Documento | Conteúdo |
|---|---|
| **`AUDITORIA.md`** | Reconhecimento completo do projeto (estado, P0/P1/P2, riscos). |
| **`P0-REPORT.md`** | Relatório da Sprint P0 (correções de segurança, testes, rollback). |
| **`CONTEXT.md`** | Memória técnica (ADRs, domínio, deploy, limitações). |
| **`CHANGELOG.md`** | Histórico de versões, com breaking changes e security summary. |
| **`README.md`** | Este arquivo. Visão geral, instalação, arquitetura. |

**Versão atual:** 1.9.0 — Fundação P0 (segurança, autenticação, integridade de dados).
**Versão documentada no README legado:** 1.5.0 (substituído pelo conjunto acima).

> ⚠️ **Se você está deployando em produção pela primeira vez:**
> 1. Leia `P0-REPORT.md` §6 (Riscos restantes) — há ações obrigatórias suas.
> 2. Leia `CONTEXT.md` §9 (Deploy) — passos completos.
> 3. **Rotacione `MYSQL_PASSWORD` e `JWT_SECRET`** antes do primeiro start.
> 4. Crie o primeiro admin com `cargo run --bin gar-system-admin -- create-admin`.

---

# Documentação do Projeto: GAR System (v1.5.0 - Arquitetura Cliente-Servidor)

## 1\. Visão Geral

O **GAR System** é um sistema de gestão projetado com uma arquitetura moderna cliente-servidor, utilizando Rust tanto no backend quanto no frontend. A aplicação visa fornecer uma ferramenta de gestão interna robusta, segura e escalável, com múltiplos níveis de acesso baseados em papéis de usuário (Administrador, Gerencia, Tecnico, Financeiro, Comercial).

A arquitetura atual é composta por três componentes principais:

1.  **Backend (Servidor):** Uma API RESTful construída em Rust com o framework **Axum**. É responsável por toda a lógica de negócio, validações e comunicação direta com o banco de dados.
2.  **Frontend (Cliente):** Uma aplicação desktop nativa construída em Rust com a biblioteca **eframe/egui**. É responsável por toda a interface do usuário e pela interação com o usuário final.
3.  **Banco de Dados:** Um servidor **MySQL 8** para persistência de dados.

Todo o ambiente de backend e banco de dados é orquestrado por contêineres gerenciados com **Podman Compose**, garantindo um setup de desenvolvimento rápido, consistente e isolado do sistema operacional do desenvolvedor.

## 2\. Principais Funcionalidades Implementadas

Até o momento, o sistema conta com as seguintes funcionalidades consolidadas:

  * **Arquitetura Cliente-Servidor:**

      * O cliente (GUI) foi completamente desacoplado do banco de dados. Toda a comunicação de dados é feita através de requisições HTTP para a API do backend.
      * A lógica de negócio (`servicos`) e o acesso a dados (`banco_de_dados`) residem exclusivamente no servidor, garantindo segurança e centralização.

  * **Autenticação de Usuário via API:**

      * Tela de login que envia as credenciais para o endpoint `/login` do servidor de forma assíncrona, usando uma thread separada para não travar a interface.
      * O servidor valida as credenciais contra o banco de dados e retorna o papel do usuário (`PapelUsuario`) em formato JSON.

  * **Gestão de Estado e Navegação por Eventos:**

      * A navegação da interface é controlada por um sistema de eventos central (`AppEvent`), permitindo que as telas solicitem ações (como navegar ou abrir um editor) sem conhecerem umas às outras.
      * Uma `sidebar` dinâmica exibe as opções de navegação com base nas permissões do usuário autenticado.

  * **Painel de Administração (Usuários):**

      * A tela de administração agora busca a lista de usuários de forma assíncrona a partir do endpoint `/usuarios` do servidor.
      * Exibe uma mensagem de "Carregando..." enquanto os dados são buscados, garantindo uma experiência de usuário fluida.
      * Interface para criar novos usuários e um modal para alterar senhas (UI implementada, lógica de API pendente).

  * **Interface do Usuário (UI):**

      * Carregamento de fontes customizadas (`JetBrains Mono` e `Noto Color Emoji`) para garantir a correta exibição de ícones e símbolos.
      * Suporte a temas claro e escuro.
      * O logo da aplicação é carregado de forma centralizada e segura, evitando travamentos (*deadlocks*).

## 3\. Arquitetura do Projeto

O código está modularizado para garantir a separação de responsabilidades. O projeto agora é um workspace Cargo que produz dois binários distintos.

  * **`src/server.rs`**: Ponto de entrada do **Backend**. Inicia o servidor Axum, define as rotas da API (`/login`, `/usuarios`) e lida com as requisições HTTP. É o único que interage com os módulos `servicos` e `banco_de_dados`.

  * **`src/main.rs`**: Ponto de entrada do **Cliente GUI**. Inicia a aplicação `eframe`, configura as fontes e o tamanho da janela.

  * **`src/aplicacao.rs`**: O núcleo do **Cliente GUI**. Gerencia a máquina de estados das telas (`EstadoTela`), processa os eventos (`AppEvent`) e controla o estado global da UI (usuário logado, tema, etc.).

  * **`src/servicos.rs`**: Camada de lógica de negócio (usada **apenas pelo servidor**). Define as regras e as estruturas de dados principais (`PapelUsuario`, `InfoUsuario`, `OrdemServico`).

  * **`src/banco_de_dados/`**: Módulo de acesso a dados (usado **apenas pelo servidor**). Foi refatorado para ter responsabilidades separadas:

      * `conexao.rs`: Gerencia o pool de conexões com o MySQL.
      * `init.rs`: Lógica de inicialização e migração do banco.
      * `usuario_repo.rs`: Funções específicas para a tabela `users`.

  * **`src/telas/`**: Diretório que contém os módulos de cada tela do **Cliente GUI**. Cada tela é responsável por sua própria UI e por emitir eventos para o `aplicacao.rs`.

      * `login.rs` e `painel_adm.rs` foram refatorados para buscar dados de forma assíncrona usando `reqwest` e `thread::spawn`.

## 4\. Guia de Instalação e Execução

O projeto agora é executado em duas partes independentes: o ambiente de servidor e o cliente GUI.

### 4.1. Pré-requisitos

  * **Rust**: Instale o Rust e o Cargo através do `rustup`.
  * **Podman** e **Podman Compose**: Para gerenciar os contêineres.

### 4.2. Configuração do Ambiente

1.  **Crie o arquivo de ambiente:** Na raiz do projeto, crie um arquivo chamado `.env` e preencha com as credenciais do banco de dados.

    ```env
    # .env
    MYSQL_DATABASE=gar_system
    MYSQL_USER=rocha
    MYSQL_PASSWORD=200519
    MYSQL_ROOT_PASSWORD=root_strong_password
    DATABASE_URL="mysql://${MYSQL_USER}:${MYSQL_PASSWORD}@db:3306/${MYSQL_DATABASE}"
    ```

2.  **Adicione `.env` ao `.gitignore`:** Para não enviar suas senhas para o repositório, certifique-se que a linha `.env` existe no seu arquivo `.gitignore`.

### 4.3. Executando a Aplicação

É necessário ter dois terminais abertos.

1.  **Terminal 1: Iniciar o Backend e o Banco de Dados**
    Na raiz do projeto, execute o comando:

    ```bash
    podman-compose up --build
    ```

    Este comando irá:

      * Construir a imagem do seu servidor a partir do `Dockerfile`.
      * Baixar a imagem do MySQL.
      * Iniciar ambos os contêineres.
      * Aguardar o banco de dados ficar "saudável" (`healthcheck`) antes de iniciar o servidor para evitar erros de conexão.
        Espere até ver a mensagem `Servidor escutando em 0.0.0.0:3000`.

2.  **Terminal 2: Iniciar o Cliente GUI**
    Enquanto o `podman-compose` estiver rodando, abra um novo terminal na raiz do projeto e execute:

    ```bash
    cargo run --bin gar-system-gui
    ```

    A interface gráfica será compilada e iniciada. Agora, ao fazer login, ela se comunicará com o servidor que está rodando no contêiner.

    ## 5. Novas funcionalidades adicionadas (Serviços / Impressão)

    Adicionei suporte a "Serviços" (mão de obra) com persistência no banco e integração com Ordens de Serviço. As principais funções e pontos de integração criados são:

    * Banco de dados (DDL):
        * `servicos` (id, nome, descricao, preco)
        * `ordem_servico_servicos` (pivot: ordem_servico_id, servico_id, quantidade, preco_unitario)

    * Módulo `src/banco_de_dados/servico.rs` (funções públicas):
        * `listar_servicos() -> Result<Vec<Servico>, ErroAplicacao>`: retorna todos os serviços cadastrados.
        * `criar_servico(s: &Servico) -> Result<u32, ErroAplicacao>`: insere um novo serviço e retorna o id criado.
        * `listar_servicos_da_os(os_id: u32) -> Result<Vec<ServicoOS>, ErroAplicacao>`: retorna os serviços vinculados a uma OS.
        * `gravar_servicos_na_os(os_id: u32, servicos: &Vec<ServicoOS>) -> Result<(), ErroAplicacao>`: grava/atualiza os serviços associados a uma OS.

    * Facade `src/servicos.rs` (funções expostas):
        * `listar_servicos() -> Vec<Servico>`: delega ao `banco_de_dados::servico::listar_servicos`.
        * `criar_servico(s: &Servico) -> u32`: delega ao `banco_de_dados::servico::criar_servico`.

    * Integração com Ordens de Serviço (`src/banco_de_dados/ordem_servico.rs`):
        * `buscar_os_por_id(id)` agora carrega também os serviços associados (via `listar_servicos_da_os`).
        * `criar_os` e `atualizar_os` gravam os serviços associados à OS (via `gravar_servicos_na_os`).

    * Visualização / Impressão na UI:
        * `src/telas/painel_ordens.rs`: adicionei um botão "📄 Ver/Imprimir" na lista de ordens (visível a Comercial/Administrador) que abre um modal com a OS formatada.
        * O modal tem o botão "Exportar TXT" que salva um arquivo `os_<id>.txt` no diretório de execução com o conteúdo da OS (peças, serviços, totais e observações).

    Observações:
    * Se o seu banco ainda não tiver as tabelas `servicos` e `ordem_servico_servicos`, atualize `db-init/init.sql` (já incluí as DDLs no arquivo) e reinicie os containers para que o MySQL execute as criações.
    * A exportação para TXT é propositalmente simples e portátil; se quiser exportar para PDF ou enviar à impressora diretamente, posso adicionar essa opção.