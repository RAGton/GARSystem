-----

# Documentação do Projeto: Senior System (v1.2.0)

## 1\. Visão Geral

O **Senior System** é uma aplicação de desktop desenvolvida em Rust, utilizando a biblioteca `eframe` (com o backend `egui`) para a interface gráfica e MySQL como banco de dados. O sistema foi projetado para ser uma ferramenta de gestão interna com múltiplos níveis de acesso baseados em papéis de usuário (Administrador, Técnico, Vendedor, etc.).

A arquitetura do projeto prioriza a separação de responsabilidades, garantindo que a lógica da interface do usuário, as regras de negócio e o acesso ao banco de dados sejam independentes, facilitando a manutenção e a escalabilidade.

## 2\. Principais Funcionalidades Implementadas

Até o momento, o sistema conta com as seguintes funcionalidades consolidadas:

  * **Autenticação de Usuário**:

      * Tela de login que valida as credenciais do usuário de forma assíncrona para não travar a interface.
      * Armazenamento seguro de senhas no banco de dados usando hashing com `bcrypt`.
      * Funcionalidade de "Deslogar" que retorna o usuário à tela de login.

  * **Gerenciamento de Estado e Navegação**:

      * Um sistema de máquina de estados (`enum EstadoTela`) controla a tela que está sendo exibida, permitindo a navegação entre a tela de Login, o Painel Principal e o Painel de Administração.
      * O Painel Principal (`TelaDashboard`) exibe botões de navegação diferentes com base no papel (`PapelUsuario`) do usuário autenticado.

  * **Painel de Administração**:

      * Interface para criar, listar e remover usuários do sistema.
      * Janela de confirmação (modal) para evitar a remoção acidental de usuários.
      * O usuário `admin` padrão é protegido contra remoção.

  * **Interface Dinâmica**:

      * Alternância entre tema claro e escuro.
      * **Controle Dinâmico da Janela**: A janela da aplicação inicia com um tamanho fixo e não redimensionável para a tela de login. Após o login, ela se torna redimensionável e se ajusta a um novo tamanho, ideal para os painéis internos. Isso é feito através de `ViewportCommand`s enviados ao `egui::Context`, resolvendo um `panic` que ocorria devido à ordem incorreta dos comandos.

## 3\. Arquitetura do Projeto

O código está modularizado da seguinte forma:

  * **`main.rs`**: Ponto de entrada. Responsável por inicializar o banco de dados, configurar a comunicação assíncrona entre a UI e a thread do banco de dados (usando `std::sync::mpsc::channel`), e iniciar a aplicação `eframe`.
  * **`aplicacao.rs`**: O núcleo da aplicação. A `struct AplicativoPrincipal` gerencia o estado atual (`EstadoTela`), a navegação entre telas e as ações globais (logout, tema). É aqui que a lógica de controle da janela é implementada dinamicamente no método `update`.
  * **`servicos.rs`**: Camada de lógica de negócio. Contém funções como `verificar_login` e `criar_usuario`, que aplicam regras (ex: tamanho mínimo de senha) e orquestram as chamadas ao banco de dados, mantendo essa lógica isolada da UI.
  * **`banco_de_dados/mod.rs`**: Módulo de acesso a dados. Toda a comunicação com o MySQL é encapsulada aqui. Utiliza um pool de conexões (`once_cell`) para performance e define as queries SQL para criar tabelas, inserir, buscar e deletar usuários.
  * **`telas/`**: Contém os módulos para cada tela da aplicação (`login.rs`, `painel_adm.rs`, etc.). Cada módulo é responsável por gerenciar seu próprio estado e desenhar seus componentes visuais.

## 4\. Guia de Instalação e Execução

Para compilar e executar o projeto, siga os passos abaixo.

### 4.1. Pré-requisitos

  * **Rust**: Instale o Rust e o Cargo através do `rustup`.
  * **Podman** (ou Docker): Para gerenciar o contêiner do banco de dados.
  * **Dependências de Sistema (Arch Linux)**:
    ```bash
    sudo pacman -Syu base-devel openssl pkg-config
    ```

### 4.2. Configuração do Banco de Dados

1.  **Baixe a imagem do MySQL**:
    ```bash
    podman pull mysql:8
    ```
2.  **Inicie o contêiner do banco de dados**: O comando abaixo cria um contêiner chamado `mysql-seniorsystem` com as credenciais e o banco de dados esperados pela aplicação.
    ```bash
    podman run --name mysql-seniorsystem -p 3306:3306 -e MYSQL_DATABASE=senior_system -e MYSQL_USER=rocha -e MYSQL_PASSWORD=200519 -e MYSQL_ROOT_PASSWORD=root_strong_password -d mysql:8
    ```
3.  **Verifique se o contêiner está em execução**:
    ```bash
    podman ps
    ```
    Se o contêiner estiver parado (`Exited`), inicie-o com `podman start mysql-seniorsystem`.

### 4.3. Executando a Aplicação

Com o banco de dados em execução, navegue até a pasta raiz do projeto e execute:

```bash
cargo run
```

A aplicação irá compilar e iniciar, conectando-se automaticamente ao banco de dados. Na primeira execução, a tabela `users` e o usuário `admin` (senha: `1234`) serão criados.
