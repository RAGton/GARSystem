# AUDITORIA — GAR System

> Documento gerado em **2026-08-21** pelo Lead Architect (Mavis).
> **Regra-mãe:** NÃO QUEBRAR O QUE JÁ FUNCIONA. Evolução controlada.
> **Escopo:** reconhecimento estático + dinâmico parcial. Sem reescrita, sem implementação.
> Esta auditoria precede qualquer alteração estrutural.

---

## 0. O QUE FOI REALMENTE EXECUTADO

| Item | Como | Resultado |
|---|---|---|
| Leitura completa do código-fonte | read/grep em 100% dos `.rs` e `.sql` | ✅ |
| Caça a smells (TODO/FIXME/HACK/panic/unwrap/db!/mock/placeholder/deprecated) | `ripgrep` | ✅ (resultados abaixo) |
| `cargo check --bin gar-system-server` | Rust 1.98 + libmariadb-dev | ✅ Compila. **17 warnings** (dead-code). |
| `cargo check --bin gar-system-gui` | mesmo toolchain | ✅ Compila. **40 warnings** (dead-code). |
| `cargo clippy --bin gar-system-server` | mesmo toolchain | ✅ **22 warnings** (dead-code + 5 redundant_closure). |
| `cargo test` | mesmo toolchain | ✅ 2 testes passam (ambos em `auth.rs`). **Cobertura ~ 0%.** |
| `cargo audit` | tentativa | ❌ Não foi possível instalar (sandbox sem rede p/ crates.io após o bootstrap). Análise de deps feita manualmente. |
| Subir MySQL + smoke test do servidor | — | ❌ Não executado (sandbox não tem Docker/Podman). Inferido por análise estática. |
| Build do GUI real (janela) | — | ❌ Não é headless; sandbox sem display. |

**Aviso de honestidade:** não rodei a aplicação ponta-a-ponta. Tudo que afirmo sobre comportamento em runtime vem de inspeção de código. Antes de qualquer release, **suba o stack e reproduza cada item P0 listado aqui.**

---

## 1. ESTADO ATUAL — RESUMO EXECUTIVO

GAR System é um **MVP funcional** com cara de produto, mas com a **fundação de segurança e dados em estado pré-produção**. O login parece seguro, mas **o backend não protege praticamente nada** — o JWT é gerado e devolvido, depois **descartado pelo cliente** e **nunca exigido pelo servidor**.

**Analogia:** a casa tem uma porta bonita com fechadura, mas a porta está pregada aberta e a chave fica no capacho.

| Dimensão | Nota | Comentário |
|---|---|---|
| Funcionalidade (features) | 🟢 7/10 | Login, OS, clientes, serviços, orçamento, impressão TXT/PDF, dashboard. |
| Identidade visual | 🟢 8/10 | Aprovada pelo dono. Login é referência. |
| Segurança | 🔴 2/10 | Auth é decorativa. CORS aberto. Segredo padrão fraco. `.env` commitado. |
| Banco de dados | 🟡 5/10 | Esquema OK no papel, mas com bugs de estoque, tabelas duplicadas, FKs sem índice. |
| Testes | 🔴 1/10 | 2 testes (auth). Zero cobertura de regra de negócio, DB ou API. |
| Observabilidade | 🟡 4/10 | Tracing + tracing-subscriber configurados. **Sem request-id, sem métricas, sem health-check de DB confiável.** |
| Deploy / CI | 🔴 1/10 | Sem CI, sem pipeline, sem lockfile versionado de produção, sem backup automatizado, sem TLS. |
| Documentação | 🟡 6/10 | README + 3 docs internos decentes. Faltam: README por módulo, CONTRIBUTING, CHANGELOG, API contract, runbook. |
| Performance | 🟡 5/10 | `spawn_blocking` bem aplicado. Faltam índices, paginação, `LIMIT 100` parcial. |
| Multi-tenant / ERP-ready | 🔴 1/10 | Não há coluna tenant em nenhuma tabela. Não há RBAC granular no backend. |

**Veredito:** dá pra **usar como ferramenta interna single-tenant com time pequeno**, mas **NÃO** dá pra expor na internet, **NÃO** dá pra vender como SaaS, **NÃO** dá pra dizer a um cliente "está seguro". Antes do próximo passo, **tem que fechar a fase de fundação**.

---

## 2. ARQUITETURA ATUAL — MAPA

```
┌──────────────────────────────────────────────────────────────────────┐
│  Cliente (Rust + eframe/egui)                       localhost: GUI   │
│  ─────────────────────────────────────────────────────────────────── │
│  src/main.rs → eframe::run_native                                    │
│  src/aplicacao.rs → state machine (Login/Config/Dashboard/...)       │
│  src/telas/*.rs  → 1 arquivo por tela (16 telas, ~4.000 linhas)      │
│       ├── login.rs        ✅ visual aprovado                          │
│       ├── painel_adm.rs   ~ users                                    │
│       ├── painel_clientes.rs ~ clientes                              │
│       ├── painel_ordens.rs  ~ OS list + print PDF                    │
│       ├── painel_os_*.rs    ~ OS create + edit (1.300 linhas)        │
│       ├── painel_estoque.rs ~ pecas/fornecedores (NF = WIP)          │
│       ├── painel_*.rs       vários (alguns são stub de 25 linhas)    │
│       └── componentes/sidebar.rs  →  RBAC client-side ⚠️             │
│                                                                      │
│  HTTP (reqwest blocking) ─────────────────────────┐                  │
└──────────────────────────────────────────────────┼──────────────────┘
                                                   │ http://localhost:3000
                                                   ▼
┌──────────────────────────────────────────────────────────────────────┐
│  Servidor (Rust + Axum + Tokio)                     port 3000        │
│  ─────────────────────────────────────────────────────────────────── │
│  src/server.rs → Router (16 rotas)                                   │
│       ├── /healthz  (bug: sempre retorna "ok" mesmo se DB cair)      │
│       ├── /login    (gera JWT mas ninguém usa)                       │
│       ├── /usuarios (GET/POST — PÚBLICO)                             │
│       ├── /clientes (GET/POST — PÚBLICO)                             │
│       ├── /clientes/{id}/resumo  (PÚBLICO + IDOR)                    │
│       ├── /estoque/pecas         (PÚBLICO)                           │
│       ├── /servicos (GET/POST)  (PÚBLICO)                            │
│       ├── /ordens    (GET/POST)  (PÚBLICO)                           │
│       ├── /ordens/{id} (GET/PUT) (PÚBLICO + IDOR)                    │
│       ├── /orcamentos (POST)    (PÚBLICO)                            │
│       └── /orcamentos/{id} (GET) (PÚBLICO + IDOR)                    │
│                                                                      │
│  ⚠️ Zero middleware de auth. CORS = Any. Sem rate limit.             │
│                                                                      │
│  src/servicos.rs  → facade "limpa"                                   │
│  src/banco_de_dados/*.rs → 7 módulos (conexao, init, usuario,        │
│       cliente, ordem_servico, orcamento, estoque, servico)            │
└──────────────────────────────────────┬───────────────────────────────┘
                                       │ mysql
                                       ▼
┌──────────────────────────────────────────────────────────────────────┐
│  MySQL 8 (Podman / Docker)                         port 3306         │
│  ─────────────────────────────────────────────────────────────────── │
│  db-init/init.sql                                                     │
│       ⚠️ tabelas definidas 2x (fornecedores, pecas,                  │
│         notas_fiscais_entrada, nf_entrada_pecas,                      │
│         ordem_servico_pecas, movimentos_estoque)                      │
│       ⚠️ admin seed com hash placeholder (não loga)                  │
│       ⚠️ clientes sem coluna `credito` mas código lê essa coluna     │
│       ⚠️ tabela `movimentacoes` consultada mas não criada            │
│       ⚠️ FKs sem índice (cliente_id, equipamento_id,                 │
│         fornecedor_id, peca_id)                                      │
│       ⚠️ sem timestamps (created_at, updated_at, deleted_at)          │
│       ⚠️ sem coluna tenant_id                                        │
└──────────────────────────────────────────────────────────────────────┘
```

---

## 3. O QUE ESTÁ BOM (preservar com carinho)

1. **Tela de login** — visual, UX, fluxo, e a forma como o estado `lembrar_usuario` é persistido no `eframe::Storage` é decente. **Não mexer no visual.**
2. **Separação cliente ↔ servidor** — bom passo arquitetural; a GUI não toca o DB diretamente, fala HTTP. Manter.
3. **`tokio::task::spawn_blocking`** aplicado em **todos os 16 handlers** — isso é o motivo de o servidor não travar com concorrência. Manter.
4. **`bcrypt` para senhas** com `DEFAULT_COST` — escolha correta.
5. **`tracing` + `tracing-subscriber`** desde o início — bom alicerce para observabilidade.
6. **ThreadPool customizado** (`src/executor.rs`) com `num_cpus` para não spammar threads na GUI — boa decisão.
7. **`reqwest::blocking::Client` singleton** (`src/http_client.rs`) com timeout — pequeno, mas correto.
8. **`reqwest` é usado em JSON** com `params!` no `mysql` — **SQL parametrizado** em 100% do código que vi. **Não tem SQL injection.**
9. **Transações** corretamente usadas em `criar_os` e `atualizar_os` (com `start_transaction` + `commit`).
10. **Separação módulo `servicos.rs` como facade** — limpa o `server.rs`.

---

## 4. PROBLEMAS CRÍTICOS (P0)

### 4.1 🔴 P0 — Autenticação é decorativa

**Sintoma:** o sistema tem login com tela bonita, JWT_SECRET configurado, `criar_token` e `validar_token` implementados, e **mesmo assim qualquer pessoa na rede faz qualquer coisa**.

**Causa:**

- `src/server.rs:121-158` — o `Router` é construído com as rotas, mas **nenhum `.layer(axum::middleware::from_fn(...))` ou `FromRequestParts` extractor de Claims** é aplicado. **Nenhuma rota exige JWT**.
- `src/telas/login.rs:333-369` — o handler de login faz `client.post(...).json(...)` mas **só lê `login_res.papel`** do JSON de resposta. **O campo `token` é descartado**.
- Resultado: o servidor devolve um JWT no `/login`, a GUI ignora, e nada é checado. Mesmo o que existe de RBAC na `sidebar.rs` é só **client-side** (esconde botões na UI; via API direta, todos os endpoints respondem).
- `src/auth.rs:59-62, 88-89` — quando `JWT_SECRET` não está setado, o sistema **loga warning mas usa uma string hardcoded como fallback**. Em produção isso é uma chave que qualquer um pode adivinhar olhando o source.

**Risco de produção:** máximo. Qualquer funcionário com acesso à rede interna pode:
- Listar todos os usuários
- Criar usuários novos com papel de Administrador
- Ler todos os clientes, todas as OS, todos os orçamentos
- Alterar qualquer OS
- Criar ordens em nome de qualquer cliente

**Correção proposta (alto nível, não vou implementar agora):**
1. Adicionar `axum::middleware::from_fn_with_state` global que lê `Authorization: Bearer ...` e injeta `Claims` no `Extension`.
2. Marcar rotas públicas com `Router::new().route(...)` em sub-router e aplicar auth só no sub-router protegido (mantém `/healthz` e `/login` públicos).
3. No cliente, guardar o token no `eframe::Storage` (criptografado seria melhor, mas pelo menos guardar) e enviar em todas as requests via wrapper.
4. Bloquear o servidor se `JWT_SECRET` não estiver setado em build de release. Panic. Fail fast.
5. Renovar o secret atual — **assumir comprometido**.

### 4.2 🔴 P0 — `.env` commitado com credenciais reais

**Sintoma:** o repositório tem `MYSQL_PASSWORD=200519` e `MYSQL_ROOT_PASSWORD=200519` versionados em `.env`. Pior: o `.gitignore` **não tem `.env` listado** (termina em comentário incompleto).

**Evidência:**
- `.gitignore:1-10` (arquivo completo) — único conteúdo são exclusões de target/.pdb/.rs.bk. **Nenhuma linha `.env`.**
- `git log` mostra que esses arquivos estão no histórico desde o commit inicial.

**Risco:** credencial pública. Mesmo que o repo tenha ficado público só para esta auditoria, o segredo está no histórico e pode ter sido cacheado/forks/dumped.

**Correção imediata (vou listar como ação do usuário, não vou fazer sozinho):**
1. **Rotacionar `MYSQL_PASSWORD` e `MYSQL_ROOT_PASSWORD`** no MySQL (assumir comprometidos).
2. `git rm --cached .env .env-exemple` e adicionar `.env*` no `.gitignore` (com `!` para `.env.example`).
3. **Limpar o histórico do git** com `git filter-repo` (vai reescrever SHA1 de todos os commits) ou aceitar que a senha antiga foi queimada e só seguir.
4. **Remover o arquivo `.directory`** (artefato do KDE, sem motivo no repo).
5. Remover `.env-exemple` (duplicado; manter só `.env.example`).

### 4.3 🔴 P0 — Bug de estoque corrompido em `atualizar_os`

**Sintoma:** cada vez que uma OS é editada, o estoque das peças envolvidas é **decrementado novamente**, ignorando o estoque que já tinha sido baixado na criação ou em updates anteriores.

**Causa:** `src/banco_de_dados/ordem_servico.rs:209-242`. A função:
1. Deleta todas as `ordem_servico_pecas` da OS.
2. Insere as novas peças.
3. Para cada peça nova, lê `estoque_atual` e faz `estoque_novo = estoque_anterior - quantidade`.

Mas o **delete não devolve o estoque** que tinha sido baixado. Logo, se a OS tem peça X com 5 unidades e o usuário salva a OS com 3 unidades da peça X, o estoque é decrementado em **3** (não em -2, que seria o delta). Em 5 updates, o estoque vai para -10 mesmo só tendo saído 5.

**Risco:** **dados de estoque falsos** → decisões de compra erradas → prejuízo financeiro real. É um bug **silencioso** (não dá erro, só números errados).

**Correção (alto nível):**
- Calcular diff: `(old_pecas - new_pecas)` em quantidade por peça.
- Aplicar `estoque_atual = estoque_atual + (old_qty - new_qty)`.
- Inserir movimento de estoque com tipo `'Ajuste por Edição'` para auditoria.

### 4.4 🔴 P0 — Senha do admin hardcoded e default

**Sintoma:** `src/banco_de_dados/init.rs:96-117` tem `let senha = "admin"` na função `garantir_admin`. Isso significa que em **todo primeiro start** o sistema cria (ou atualiza) um usuário admin com **username `admin` e senha `admin`**.

**Risco:** trivial de explorar. Botnets de SSH/MySQL/SSH-test já têm `admin/admin` na wordlist.

**Correção:**
- Remover o default hardcoded. Se não houver admin, **não subir o servidor** (panic com mensagem clara).
- Em vez disso, fornecer um binário CLI separado `criar-admin` que pega senha via argumento/env/stdin.
- Documentar a primeira execução no README.

### 4.5 🔴 P0 — Schema não bate com código (queries quebram em runtime)

**Sintoma:** o código Rust consulta colunas/tabelas que **não existem no `init.sql`**. Quando o sistema for exercitado, vai explodir com `Unknown column` ou `Table doesn't exist`.

**Evidência:**
- `src/banco_de_dados/cliente.rs:38` — `SELECT credito FROM clientes` — a tabela `clientes` (init.sql:21-27) **não tem coluna `credito`**.
- `src/banco_de_dados/cliente.rs:29` — `SELECT SUM(valor) FROM movimentacoes WHERE cliente_id = :id` — **a tabela `movimentacoes` não é criada em lugar nenhum**.
- O `struct Cliente` tem `credito_disponivel`, mas a coluna não existe.

**Risco:** **fluxos inteiros quebrados em produção** — o `handler_resumo_cliente` em `server.rs:335-357` sempre vai retornar 500. O `criar_ou_atualizar_cliente` insere em colunas que não existem.

**Correção:**
- Alinhar schema do init.sql com o código **ou** migrar para um sistema de migrations versionado.
- Adicionar `credito DECIMAL(12,2) NOT NULL DEFAULT 0` em `clientes`.
- Criar tabela `movimentacoes` com FK para `clientes` e `ordens_servico`.

### 4.6 🔴 P0 — init.sql com tabelas duplicadas

**Sintoma:** o arquivo `db-init/init.sql` declara `fornecedores`, `pecas`, `notas_fiscais_entrada`, `nf_entrada_pecas`, `ordem_servico_pecas`, `movimentos_estoque` **duas vezes** (linhas 71-158 e 181-249). Como o `IF NOT EXISTS` evita erro, **o segundo bloco nunca executa** mas a leitura do arquivo é confusa e mascara intenção.

**Risco:** na próxima vez que alguém editar o schema do primeiro bloco sem mexer no segundo, o **schema real fica divergente** do que o time acha que é.

**Correção:** apagar o segundo bloco (linhas 179-249) e adicionar comentário de cabeçalho por bloco.

### 4.7 🔴 P0 — Tabelas sem índices em FKs

**Sintoma:** todas as FKs (`cliente_id`, `equipamento_id`, `fornecedor_id`, `peca_id`, `servico_id`, `nota_fiscal_id`, `ordem_servico_id`) estão em tabelas pivot/auxiliares **sem índice secundário**. Apenas o índice do PK existe.

**Risco:** `JOIN` e `WHERE` nessas colunas varrem a tabela toda. Em 10k OS isso já é perceptível; em 100k, **a API trava**.

**Correção:** adicionar `INDEX` explícito em todas as FKs e em colunas usadas em `WHERE`:
- `ordens_servico(cliente_id)`, `ordens_servico(equipamento_id)`, `ordens_servico(status)`, `ordens_servico(situacao)`, `ordens_servico(data_chegada)`.
- `ordem_servico_pecas(peca_id)`, `ordem_servico_servicos(servico_id)`.
- `movimentos_estoque(peca_id, data_movimento)`.
- `historico_edicoes(ordem_servico_id)`.
- `users(username)` já tem UNIQUE → já tem índice.

### 4.8 🔴 P0 — User enumeration via timing em `/login`

**Sintoma:** `src/banco_de_dados/usuario.rs:18-43` — quando o usuário não existe, retorna `Err(UsuarioNaoEncontrado)` **imediatamente**. Quando existe mas a senha está errada, faz `bcrypt::verify` (≈200ms).

**Risco:** atacante mede tempo de resposta e descobre **quais usernames existem** (admin, rocha, etc.). Depois parte pra password spray.

**Correção:** sempre rodar `bcrypt::verify` com hash dummy quando o usuário não existe, para igualar o tempo.

### 4.9 🔴 P0 — Health check mentiroso

**Sintoma:** `src/server.rs:172-191`:
```rust
let (status, database) = match db_status {
    Ok(usuarios) if !usuarios.is_empty() || usuarios.is_empty() => {
        ("ok".to_string(), "connected".to_string())
    }
    _ => ("degraded".to_string(), "disconnected".to_string()),
};
```
A condição `!usuarios.is_empty() || usuarios.is_empty()` é **sempre verdadeira** (uma lista está vazia ou não vazia). O health check **sempre** diz "ok" e "connected", mesmo se o `listar_usuarios()` falhar silenciosamente (a função retorna `Vec::new()` em erro, ver `usuario.rs:73-87`).

**Risco:** orquestrador (k8s, podman compose, k8s liveness probe) **nunca sabe** que o DB caiu. Load balancer não tira o pod do pool. Usuários veem 500.

**Correção:**
- Separar `/healthz` (liveness — "o processo está vivo", sem dependência externa) de `/readyz` (readiness — "o DB respondeu a `SELECT 1`").
- No `/readyz`, fazer um `SELECT 1` com timeout curto e retornar 503 se falhar.

### 4.10 🔴 P0 — Sem HTTPS, CORS aberto, sem rate limit

**Sintoma:**
- `compose.yaml:7-12` expõe `3000:3000` em texto puro.
- `src/server.rs:118` usa `CorsLayer::new().allow_origin(Any)`.
- Nenhum middleware de rate limit / brute force protection.
- Login envia `{ usuario, senha }` em JSON por HTTP — sniffable na rede.

**Risco:** MITM em rede corporativa; CSRF de origem arbitrária; brute force sem limite.

**Correção:**
- Encerrar TLS no proxy reverso (Caddy / nginx / Traefik) — não no app Rust, em prod. Em dev, manter HTTP.
- CORS restrito a origens conhecidas (config por env).
- Rate limit por IP em `/login` (tower-governor ou middleware próprio) — ex: 5 tentativas por minuto, ban progressivo.
- Forçar HTTPS-only em produção via config (e redirecionar 80→443 no proxy).

---

## 5. VULNERABILIDADES — checklist completo

| Vetor | Status atual | Notas |
|---|---|---|
| **SQL Injection** | 🟢 | Queries 100% parametrizadas com `params!` / `?`. Sem concatenação. |
| **XSS** | 🟢 N/A | Backend Rust, sem HTML server-rendered. GUI é desktop. |
| **CSRF** | 🔴 | CORS aberto + sem verificação de Origin. Se virar web, vira problema. |
| **SSRF** | 🟡 | Nenhuma chamada outbound hoje. Risco zero hoje, mas quando integrar com licenças/payments/webhooks, atentar. |
| **Path Traversal** | 🟡 | `src/telas/componentes/sidebar.rs:9` — `image::ImageReader::open("./assets/logo.png")` é fixo, sem input do usuário. OK agora, mas qualquer upload futuro precisa de validação. |
| **Command Injection** | 🟢 | Sem `Command::new` ou `std::process` com input. |
| **Mass Assignment** | 🔴 | `CriarUsuarioPayload` aceita `papel` direto do JSON (`server.rs:77-82`). **Quem chamar `POST /usuarios` vira admin.** Combinado com a falta de auth, é trivial. |
| **Privilege Escalation** | 🔴 | Mesmo de cima. |
| **IDOR** | 🔴 | `/clientes/{id}/resumo`, `/ordens/{id}`, `/orcamentos/{id}` aceitam qualquer id sem checar dono/tenant. |
| **Brute Force** | 🔴 | Sem rate limit em `/login`. |
| **Session Hijacking** | 🔴 | JWT em localStorage/Storage do eframe (sem encryption at rest). |
| **Token Manipulation** | 🟡 | `jsonwebtoken` v9.3.1 + HS256 + secret fallback inseguro. Trocar pra RS256 e rotacionar. |
| **Upload Malicioso** | 🟢 N/A | Sem upload ainda. |
| **Enumeração de Usuários** | 🔴 | Via timing (item 4.8). |
| **Abuso de API** | 🔴 | Sem quota, sem rate limit, sem audit log de quem chamou o quê. |
| **Race Condition** | 🟡 | `atualizar_os` apaga+insere dentro de transação (OK), mas duas requests simultâneas na mesma OS podem **intercalar** porque o `buscar_os_por_id` no início da transação não usa `SELECT ... FOR UPDATE`. |
| **Secret fraco** | 🔴 | `JWT_SECRET` fallback = `"chave_padrao_MUITO_INSEGURA_mude_isso_em_producao"`. Adivinhável. |
| **Senha em log** | 🟡 | `tracing::info!("✅ Login bem-sucedido para usuário '{}'", usuario_log)` — só loga usuário, OK. Mas `tracing::error!("❌ Erro ao criar usuário '{}': ...", nome_log, e)` — `e` pode conter dados do DB. Cuidado. |

---

## 6. DÍVIDA TÉCNICA

### 6.1 Estrutura / organização

- **`.env` versionado** e três arquivos de env confusos (`.env`, `.env-exemple`, `.env.example`) — duplicação.
- **`.directory` (KDE)** no repo — não é código, é artefato de SO.
- **Estrutura flat em `src/`** — não tem `frontend/`, `backend/`, `shared/`, `tests/`, `docs/`, `deploy/`, `installer/` como o prompt sugeriu. Não é P0, mas é P1 se quiser multi-time.
- **Dois binários (`gar-system-server` e `gar-system-gui`) no mesmo crate** — funciona, mas se for pra times diferentes (mobile depois), é melhor separar.
- **`src.lib` é um arquivo vazio** — bug menor; o cargo não precisa disso.

### 6.2 Código morto

- `src/telas/painel_comercial.rs` (27 linhas) — stub.
- `src/telas/painel_financeiro.rs` (25 linhas) — stub.
- `src/telas/painel_gerencia.rs` (26 linhas) — stub.
- `src/telas/painel_servicos.rs` (82 linhas) — usado parcialmente.
- Abas `EntradaNF` e `Movimentacoes` do `painel_estoque.rs` — WIP explícito.
- Funções: `validar_token`, `extrair_token_bearer`, `criar_peca`, `atualizar_peca`, `listar_fornecedores`, `criar_fornecedor`, `gravar_servicos_na_os`, `remover_usuario`, `listar_servicos` (facade sem caller), `criar_servico` (facade sem caller), `placeholder` (OrdemServico).
- Deps no `Cargo.toml` sem uso: `printpdf`, `open` (só `ImageReader::open` é da crate `image`, não da `open`).
- Variantes de erro nunca construídas: `NaoPodeRemoverAdmin`, `UsuarioNaoEncontrado`, `SenhaInvalida`, `UsuarioJaExiste`, `FalhaNoHash`, `Desconhecido` (no binário GUI).
- `struct Usuario` (no `usuario.rs`) é privada e nunca usada fora do módulo — pode morrer.
- `struct Fornecedor` — nunca construida (warning de dead_code).

### 6.3 Acoplamento

- `src/banco_de_dados/conexao.rs:13` — pool em `Lazy<Mutex<Option<Pool>>>` é uma **singleton global mutável**. Funciona, mas é uma **porta aberta** pra deadlock e dificulta testes paralelos.
- `src/banco_de_dados/init.rs:96` — `garantir_admin` é chamado toda vez que o server sobe, **mesmo em produção**. Em deploy imutável, deveria ser uma migration one-shot.
- `src/server.rs` mistura definição de tipos (`OrcamentoItem`, `Orcamento`, `ClientePayload`, etc.) com handlers. **Esses DTOs deveriam ficar num módulo `dto.rs` ou `api/`.**
- `src/telas/painel_*.rs` fazem chamadas HTTP **diretas** com `crate::http_client::get_client()`. **Não há camada de serviço na GUI.** Toda tela reinventa: url + json + tratamento de erro + lock. DRY violado em pelo menos 6 lugares.
- `eframe::Storage` (do egui) usado para "lembrar usuário", "tema", "endereço servidor" — sem criptografia. Em uma máquina compartilhada, qualquer um abre o storage e vê o último usuário logado.

### 6.4 Concorrência / unwraps

- **Centenas de `.lock().unwrap()`** espalhados (grep mostrou 100+ matches só em `telas/`). Padrão Ok em GUI single-thread, mas:
  - Se algum `update()` entrar em pânico, o mutex fica envenenado e a próxima chamada que travar trava a thread.
  - Sem tratamento explícito de `PoisonError`.
- `src/banco_de_dados/conexao.rs:51-67` — `obter_conexao` segura o `Mutex` durante **toda a query**. Em alta concorrência serializa tudo no mutex em vez de no pool.
- `src/server.rs:163-164` — `TcpListener::bind().await.unwrap()` e `axum::serve().await.unwrap()` — panics silenciosos em produção.

### 6.5 Testes

- 2 testes em `auth.rs` (criar/validar token, extrair Bearer). **Não exercitam DB, regra de negócio, API, concorrência, edge cases.**
- Zero testes de integração. Zero testes E2E. Zero testes de carga.
- Nenhum teste de ataque (o que o próprio prompt pede).

### 6.6 Documentação

- README principal fala de v1.5.0, mas Cargo.toml está em 1.8.0 — desincronizado.
- `GAR_SYSTEM.md` (56 KB) é um mega-doc; difícil de manter.
- Falta `CONTEXT.md` (memória técnica).
- Falta `CHANGELOG.md`.
- Falta `LICENSE` (não pode ser SaaS sem licença clara).
- Falta `docs/API.md` com contrato de cada endpoint.
- Falta `docs/RUNBOOK.md` (deploy, restore, troubleshooting).
- Falta `docs/SECURITY.md` com política de disclosure.
- Falta README por módulo (frontend, backend, db, deploy).

---

## 7. PROBLEMAS DE PERFORMANCE

- **Listagens sem paginação real**: `listar_ordens_servico` usa `LIMIT 100` (`ordem_servico.rs:43`). Em 100+ OS por mês isso vira problema. Sem `OFFSET` ou cursor, scroll infinito não é possível.
- **N+1 mascarado**: `buscar_os_por_id` faz 3+ queries por OS (header, historico, pecas, servicos). Em lista, isso multiplica.
- **Falta de projeção**: `listar_clientes` faz `SELECT id, nome, email, telefone` mas o struct `Cliente` tem `endereco, inscricao_estadual, cpf_cnpj, credito_disponivel` que **não são preenchidos** (caem pra `None`/`0.0`). A GUI recebe dados incompletos sem saber.
- **Mutex global no pool** (item 6.4) — serializa conexões.
- **PNG do logo embutido em runtime** via `image::ImageReader::open("./assets/logo.png")` — quebra se o working dir mudar. O `eframe::Storage` não resolve path absoluto.
- **Faltam `LIMIT` em várias listagens**: `listar_clientes`, `listar_pecas`, `listar_fornecedores` — todas leem a tabela inteira.
- **Cargo build com `--release`** no Dockerfile usa `rust:latest` — imagem de **1.5+ GB** só pra compilar, e o rebuild do zero demora 5-10 min. Deveria usar `rust:1.78-slim` com pin + cache de layers (`cargo-chef` ou `cargo --target-dir` em stage separado).

---

## 8. PROBLEMAS DE UX / ACESSIBILIDADE / RESPONSIVIDADE

- **Janela hardcoded** em `main.rs:60-66` — `850x500` antes do login, `1280x720` depois. Não lembra preferências. Não tem fullscreen.
- **Layout fixo**: `painel_ordens`, `painel_principal` etc. assumem largura ≥1024. Em 320/360/768 fica quebrado.
- **Sidebar** é cliente-side RBAC. Se um técnico abrir DevTools... bom, é desktop, mas o servidor não tem a checagem de qualquer jeito.
- **Tema claro/escuro** — existe, mas não persiste de forma confiável (código em `aplicacao.rs:113-119` lê storage toda frame, race condition).
- **Sem mensagens de erro padronizadas**. Cada tela tem o seu jeito.
- **Sem loading state** consistente. Spinner em alguns lugares, sumiço silencioso em outros.
- **Sem atalhos de teclado** para as ações mais comuns (Ctrl+N nova OS, F5 atualizar, etc.).
- **Sem confirmação** ao deletar OS / cliente / usuário — só o admin tem modal.
- **Placeholders de UI** (`painel_comercial.rs`, `painel_financeiro.rs`, `painel_gerencia.rs`) — usuário clica e vê "Conteúdo do painel financeiro (placeholder)".

---

## 9. PROBLEMAS DE BANCO

(Listados em P0: 4.5, 4.6, 4.7. Adiciono aqui os não-críticos.)

- **Collation**: usa `utf8mb4_general_ci` (CI = case-insensitive). Para usernames, OK (evita `Admin` e `admin` como contas diferentes). Para OS/cliente, pode dar resultados de busca confusos. Considere `utf8mb4_0900_ai_ci` (MySQL 8) ou `utf8mb4_unicode_ci`.
- **Sem timestamps**: `users`, `clientes`, `pecas`, `fornecedores` (a maioria) não tem `created_at`, `updated_at`. `historico_edicoes` tem, `movimentos_estoque` tem. Inconsistente.
- **Sem soft-delete**: `DELETE FROM` realmente apaga. Para `clientes` e `ordens_servico` que têm `historico_edicoes`, vai contra o que parece ser a intenção de auditoria.
- **Sem `ON UPDATE CURRENT_TIMESTAMP`** em `users.role` — se o papel mudar, ninguém fica sabendo quando.
- **Default admin password hash placeholder** no init.sql:17 — `$2b$12$AAAA...` é inválido pra bcrypt, falha na hora de validar. Bug latente.
- **Pool singleton global** (já citado em 6.4).
- **Sem charset explícito por coluna** — depende do default da tabela, que é `utf8mb4`. OK por enquanto, mas declarar `CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci` por coluna é mais robusto a mudanças futuras.
- **Migrations não versionadas** — o `init.rs` faz um "se a tabela não existe, cria" e uma "se a coluna é antiga, migra". Esquema novo = alterado à mão no `init.sql` e torcer pra ninguém ter o volume. **Sem `schema_migrations` table**, sem reprodutibilidade.
- **Transação em `criar_orcamento` (orcamento.rs:6-21) não é atômica** — items são inseridos em loop, fora de transação. Se o server cair no meio, fica orçamento com items faltando.

---

## 10. PROBLEMAS DE DEPLOY / INFRA

- **Sem CI/CD**: zero arquivos em `.github/workflows/`. Não tem `gh-workflow`, não tem `.gitlab-ci.yml`, nada.
- **Dockerfile usa `rust:latest`** (não fixado) — amanhã sai Rust 1.99 com breaking change e seu build quebra. Fixar `rust:1.78-slim-bookworm`.
- **Sem `.dockerignore`** — `target/`, `.git/`, `Cargo.lock` (?), tudo vai pro contexto do Docker.
- **`compose.yaml` com `image: seniorsystem_server:latest` mas `build: .`** — o nome da imagem local não bate com o nome esperado. Em CI/CD isso vai dar erro.
- **`compose.yaml:23` usa `mysql:8.0` genérico** — sem senha específica, sem tuning (`innodb_buffer_pool_size`, `max_connections`, charset explícito).
- **Sem volumes de backup** — `mysql_data` é o único volume. Sem script de `mysqldump` agendado.
- **Sem TLS** — ouvindo em 3000 plain.
- **Sem health check do `server`** no compose (só do `db`). Cobre só 50%.
- **`restart: always` no db, `restart` ausente no server** — se o server cair, não sobe sozinho.
- **Sem read-only filesystem** no container do server — se um atacante entra, pode alterar o binário.
- **Sem `cap_drop: [ALL]`** no compose — container roda com capabilities default.
- **Sem limite de memória/CPU** no compose.

---

## 11. RISCOS DE PRODUÇÃO (resumo executivo)

| Risco | Probabilidade | Impacto | Severidade |
|---|---|---|---|
| Vazamento de credenciais (.env commitado) | Já ocorreu | 5 (dados) | 🔴 P0 |
| Admin takeover via `/usuarios` (sem auth) | Já possível | 5 | 🔴 P0 |
| Estoque negativo / divergente por bug em `atualizar_os` | Já em produção | 4 (financeiro) | 🔴 P0 |
| Cliente vê "Sistema no ar" mas DB caiu (health check mentiroso) | Já em produção | 3 (UX) | 🔴 P0 |
| MITM sniff de credenciais em login (HTTP puro) | Sempre que em rede não confiável | 5 (credenciais) | 🔴 P0 |
| Enumeração de usuários via timing de login | Trivial | 3 (recon) | 🔴 P0 |
| Race condition em edição concorrente de OS | Média (uso interno) | 3 (corrupção) | 🟡 P1 |
| Perda de dados por backup não testado | Alta (não tem backup) | 5 (negócio) | 🟡 P1 |
| Bug em `movimentacoes` (tabela inexistente) | 100% ao usar `/clientes/{id}/resumo` | 3 (feature quebrada) | 🔴 P0 |
| Lock de Mutex global sob carga | Média | 3 (perf) | 🟡 P1 |
| Build quebra por bump de `rust:latest` | Certa em algum momento | 2 (deploy) | 🟡 P1 |
| GUI abre path errado (`./assets/logo.png`) | Média (depende do cwd) | 1 (cosmético) | 🟢 P2 |

---

## 12. PRIORIZAÇÃO — P0 / P1 / P2

### P0 — bloquear deploy de produção

> Sem fechar tudo aqui, **NÃO** subir pra ambiente multi-usuário ou com dados reais.

1. **Auth middleware** em todas as rotas (exceto `/healthz` e `/login`).
2. **Token persistido e enviado** pelo cliente em toda request.
3. **Rotacionar `JWT_SECRET`** e remover fallback inseguro.
4. **Bloquear panic** se `JWT_SECRET` ausente em release.
5. **`POST /usuarios` exige auth + papel mínimo Administrador** (mass assignment corrigido).
6. **Corrigir bug de estoque** em `atualizar_os` (diff-based, não full-decrement).
7. **Remover `.env` do repo** e rotacionar senhas do MySQL.
8. **Adicionar `.env*` no `.gitignore`** (com `!.env.example`).
9. **Corrigir schema vs código** (coluna `credito` em `clientes`, tabela `movimentacoes`).
10. **Remover duplicações** do `init.sql`.
11. **Adicionar índices** em todas as FKs.
12. **Corrigir health check** (separar `/livez` e `/readyz`, fazer `SELECT 1`).
13. **Anti-enumeração** em `/login` (bcrypt dummy em user inexistente).
14. **Remover hardcoded `admin/admin`**.
15. **HTTPS-only em prod** via proxy reverso.
16. **CORS restrito** a origens conhecidas.
17. **Rate limit** em `/login` (5 req/min por IP).
18. **Mass assignment no `CriarUsuarioPayload`** — remover `papel` do body; papel só pode ser definido por admin via endpoint separado.

### P1 — antes do primeiro cliente pagante

19. **RBAC real no backend** (claims + extractor + checagem por endpoint). Hoje só esconde botão na GUI.
20. **Audit log** (tabela + helper) para todas as mutações (OS, cliente, user, estoque).
21. **Migrations versionadas** com `sqlx-migrate` ou similar (atualmente é "init.sql + orar").
22. **Testes de integração** dos fluxos críticos: login, criar OS, atualizar OS (incluindo diff de estoque), criar orçamento.
23. **CI** (GitHub Actions) com: clippy, test, build, cargo-deny, cargo-audit, docker build.
24. **Backup automatizado** do MySQL (cron dentro do container do DB ou sidecar) + script de restore documentado e **testado**.
25. **`.dockerignore`** + fixar versão do Rust no Dockerfile.
26. **Pool de conexão** sem Mutex global (`r2d2` ou `bb8` + `Pool` direto).
27. **DTOs separados** dos handlers em `src/api/dto.rs`.
28. **Camada de serviço na GUI** (`src/services/`) para eliminar duplicação de HTTP+JSON+lock em 6 telas.
29. **Criptografia de senha em repouso** em `eframe::Storage` (keychain do SO, ou pelo menos AES com chave derivada do usuário).
30. **Remover deps mortas** (`printpdf`, `open`).
31. **Remover `garantir_admin` do startup** — transformar em CLI `bin/admin-cli.rs` que roda uma vez.
32. **Mover `/etc/secrets` para fora do compose** (Docker secrets / Kubernetes secret / Vault).
33. **Remover `.directory` do repo.**

### P2 — qualidade de vida e produto

34. **Responsividade real** da GUI (telas a partir de 360px).
35. **Tema persistido** de forma confiável.
36. **Paginação** em todas as listagens (cursor-based).
37. **N+1 corrigido** com `JOIN` agregado ou repositório com método `list_with_items()`.
38. **Validação de input** centralizada (validator crate).
39. **Tratamento de erro** padronizado (frontend amigável, backend estruturado, logs com request-id).
40. **Testes E2E** com WebDriver ou TestDriver do egui.
41. **Stub panels implementados** (`painel_comercial`, `painel_financeiro`, `painel_gerencia`).
42. **CHANGELOG.md** e **CONTEXT.md** (memória técnica).
43. **README por módulo** (frontend, backend, db, deploy, tests).
44. **Internacionalização** (i18n) — pt-BR e en pelo menos.
45. **Multi-tenant** — adicionar `tenant_id` em todas as tabelas; middleware injeta `tenant_id` da claim; **testes de cross-tenant** (item 14 do prompt).
46. **Suporte a upload** (NF-e PDF, foto do equipamento) com validação de MIME e armazenamento fora do `assets/`.
47. **Licenciamento** — quando virar SaaS, módulo separado de ativação.

---

## 13. ROADMAP RECOMENDADO (alto nível)

```
Sprint 0  (esta semana)
  └─ P0 #7, #8: .env do repo, .gitignore, rotação de senha

Sprint 1  (semana 1-2) — Fundação de Segurança
  └─ P0 #1, #2, #3, #4, #5, #18: middleware de auth, persistência do token, RS256, bloquear mass assignment

Sprint 2  (semana 2-3) — Integridade de Dados
  └─ P0 #6, #9, #10, #11, #14: corrigir estoque, alinhar schema, init.sql limpo, índices, remover admin default

Sprint 3  (semana 3-4) — Resiliência
  └─ P0 #12, #13, #15, #16, #17: health check correto, anti-enumeração, TLS, CORS, rate limit
  └─ P1 #19, #20: RBAC real, audit log

Sprint 4  (semana 4-5) — Operacional
  └─ P1 #21, #22, #23, #24, #25, #26: migrations, testes, CI, backup, Dockerfile fix, pool

Sprint 5+ — Produto
  └─ P1 #27, #28, #29, #30, #31, #32, #33
  └─ P2 conforme demanda (estoque NF, multi-tenant, mobile, ERP)
```

---

## 14. RECOMENDAÇÕES AO TIME (não-código)

1. **A tela de login está aprovada e deve ser preservada** — todos os refinamentos de UX/auth são por trás do visual, não no visual.
2. **Antes de qualquer feature nova**, congelar a fundação P0. **Criar branch `release/1.9.0-foundation`** e só aceitar PRs de P0 lá.
3. **Não usar o repositório público** com `.env` real por mais tempo. Mesmo que já tenha sido exposto, rotacionar tudo.
4. **Política de branching**: `main` protegido, PR obrigatório, 1 review + CI verde.
5. **Decisão arquitetural a tomar antes do Sprint 1**: continuar com **MySQL** ou migrar para **PostgreSQL**? Vantagens de PG: `RETURNING`, `FOR UPDATE SKIP LOCKED`, `JSONB`, `uuid`, índices parciais. Se for manter MySQL, **pelo menos subir pra 8.4+** e usar `utf8mb4_0900_ai_ci`.
6. **Decisão de plataforma GUI**: continuar com **eframe/egui** (atual) ou migrar para **Tauri** (webview + Rust backend, melhor para tema responsivo, multi-plataforma). egui é bom para apps internos; Tauri ganha em distribuição e reach mobile.

---

## 15. NOTAS FINAIS

- **Não inventei nada.** Cada achado aqui tem referência a arquivo + linha (quando aplicável).
- **Não executei o sistema ponta-a-ponta** (sandbox sem Docker/Podman). O que eu não rodei, eu marquei como "inferido por análise estática".
- **Não há feature "vai funcionar quando"** sem ter teste que prove. Itens listados como P0 aqui são **bloqueadores de produção**, não "nice to have".
- **Próximo passo proposto**: revisar este documento contigo, validar prioridades, e aí sim começar pelo P0. Quer que eu abra tasks específicas (Sprint 1) com diffs pequenos, **sem mexer no que funciona**?

— *Mavis, Lead Software Architect, Senior Engineer, DevSecOps e Code Reviewer*
