Com certeza\! Baseado em todos os arquivos do projeto, preparei uma nova documentação para a versão 1.4.0 do **Senior System**, incorporando as mais recentes funcionalidades e melhorias.

-----

# Documentação do Projeto: Senior System (v1.4.0)

## 1\. Visão Geral

O **Senior System** é uma aplicação de desktop desenvolvida em Rust, utilizando a biblioteca `eframe` (com `egui`) para a interface gráfica e MySQL para a persistência de dados. Projetado como uma ferramenta de gestão interna, o sistema oferece múltiplos níveis de acesso baseados em papéis de usuário (Administrador, Gerencia, Tecnico, Financeiro, Comercial), garantindo que cada usuário tenha acesso apenas às funcionalidades pertinentes à sua função.

A arquitetura do projeto enfatiza uma clara separação de responsabilidades, modularizando a interface do usuário (UI), as regras de negócio e o acesso ao banco de dados. Essa abordagem não só facilita a manutenção e a escalabilidade, como também promove um desenvolvimento mais organizado e seguro.

## 2\. Principais Funcionalidades Implementadas

A versão 1.4.0 consolida as funcionalidades existentes e introduz novos módulos cruciais para a operação:

  * **Autenticação e Segurança**:

      * Tela de login com validação de credenciais assíncrona para não bloquear a UI.
      * Armazenamento seguro de senhas com hashing `bcrypt`.
      * Logout seguro, que redefine o estado da aplicação e retorna à tela de login.

  * **Gerenciamento de Usuários (Painel de Administração)**:

      * Interface completa para criar, listar e remover usuários do sistema.
      * Modal de confirmação para prevenir a remoção acidental de usuários.
      * Proteção contra a remoção do usuário `admin` padrão.

  * **Painel Técnico e Ordens de Serviço**:

      * **Nova Tela de Painel Técnico**: Uma interface dedicada para técnicos, focada na gestão de Ordens de Serviço (OS).
      * **Visualização e Filtragem de OS**: Exibe uma lista de ordens de serviço com informações como ID, cliente, equipamento e status. Inclui uma funcionalidade de busca para filtrar OS por nome do cliente.
      * **Detalhes da OS**: Um modal exibe informações detalhadas de uma OS selecionada, incluindo o defeito relatado.
      * **Dados Mockados**: Atualmente, a tela utiliza dados de exemplo (`mock`) para simular o fluxo de trabalho, que futuramente serão integrados ao banco de dados.

  * **Interface Dinâmica e Navegação por Papel**:

      * A janela da aplicação ajusta seu tamanho e capacidade de redimensionamento dinamicamente após o login.
      * Uma `sidebar` e um `dashboard` central apresentam atalhos de navegação que mudam conforme o `PapelUsuario` logado, restringindo o acesso a telas não autorizadas.
      * Suporte a temas claro e escuro, permitindo personalização da experiência do usuário.

## 3\. Arquitetura do Projeto

O código-fonte é organizado em módulos que separam as responsabilidades de forma clara:

  * **`main.rs`**: Ponto de entrada da aplicação. Inicializa o pool de conexões com o banco de dados, configura os canais de comunicação assíncrona (`mpsc`) entre a UI e a thread de banco de dados, e inicia a aplicação `eframe`.
  * **`aplicacao.rs`**: O coração do sistema. A `struct AplicativoPrincipal` gerencia o estado global (`EstadoTela`), a navegação entre as telas (Login, Dashboard, Admin, Tecnico, etc.), e ações globais como logout e troca de tema.
  * **`servicos.rs`**: Camada de lógica de negócio. Contém as regras de validação (ex: login, criação de usuário) e as estruturas de dados principais, como `OrdemServico` e `PapelUsuario`. Isola a lógica de negócio da UI e do acesso direto ao banco.
  * **`banco_de_dados/mod.rs`**: Módulo de acesso a dados. Centraliza toda a comunicação com o MySQL, utilizando um pool de conexões (`once_cell`) para otimizar a performance. Define as queries SQL para todas as operações de persistência e inclui um sistema de migração de schema para atualizar a tabela `users` de versões antigas.
  * **`telas/`**: Diretório que contém os módulos de cada tela da aplicação (`login.rs`, `painel_adm.rs`, `painel_tecnico.rs`, etc.). Cada módulo é autônomo, gerenciando seu próprio estado e renderizando seus componentes visuais.
  * **`telas/componentes/`**: Abriga componentes de UI reutilizáveis, como a `sidebar.rs`, que é usada em todas as telas principais após o login.

## 4\. Guia de Instalação e Execução

Siga os passos abaixo para compilar e executar o projeto localmente.

### 4.1. Pré-requisitos

  * **Rust**: Instale o compilador Rust e o gerenciador de pacotes Cargo através do `rustup`.
  * **Podman** (ou Docker): Necessário para executar o contêiner do banco de dados MySQL.
  * **Dependências de Sistema (Exemplo para Arch Linux)**:
    ```bash
    sudo pacman -Syu base-devel openssl pkg-config
    ```

### 4.2. Configuração do Banco de Dados

1.  **Baixe a imagem do MySQL 8**:
    ```bash
    podman pull mysql:8
    ```
2.  **Inicie o contêiner do banco de dados**: O comando a seguir cria um contêiner nomeado `mysql-seniorsystem` com as credenciais e o banco de dados que a aplicação espera encontrar.
    ```bash
    podman run --name mysql-seniorsystem -p 3306:3306 -e MYSQL_DATABASE=senior_system -e MYSQL_USER=rocha -e MYSQL_PASSWORD=200519 -e MYSQL_ROOT_PASSWORD=root_strong_password -d mysql:8
    ```
3.  **Verifique se o contêiner está em execução**:
    ```bash
    podman ps
    ```
    Caso o contêiner não esteja rodando (status `Exited`), inicie-o com `podman start mysql-seniorsystem`.

### 4.3. Executando a Aplicação

Com o banco de dados ativo, navegue até o diretório raiz do projeto e execute:

```bash
cargo run
```

A aplicação será compilada e iniciada. Na primeira execução, ela criará a tabela `users` e o usuário `admin` (senha: `12345678`), conectando-se automaticamente ao banco de dados.
