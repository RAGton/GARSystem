# CHANGELOG — GAR System

Todas as mudanças notáveis neste projeto são documentadas aqui.
O formato segue [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/),
e este projeto segue [Versionamento Semântico](https://semver.org/lang/pt-BR/).

## [1.10.0] — 2026-10-09 — Rebranding GAR System

### ✨ Changed
- **Rebranding completo:** `senior_system` → `gar_system` em todo o projeto.
  - Crate Rust: `gar_system` (era `senior_system`)
  - Binários: `gar-system-gui`, `gar-system-server`, `gar-system-admin`, `audit-rbac-achado1`
  - Env vars: `gar_app` (era `senior_app`), `gar_system_server` (era `senior_system_server`),
    `GAR_BOOTSTRAP_PASSWORD` (era `SENIOR_BOOTSTRAP_PASSWORD`),
    `GAR_PERMISSION_CACHE_TTL_SECS` (era `SENIOR_PERMISSION_CACHE_TTL_SECS`),
    `GAR_SYSTEM_DISCOVERY_REQUEST` (era `SENIOR_SYSTEM_DISCOVERY_REQUEST`)
  - Compose service / imagem: `gar_system_server` (era `seniorsystem_server`)
  - DB default: `gar_system` (era `senior_system`)
- Documento legado `SENIORSYSTEM.md` renomeado para `GAR_SYSTEM.md` (preserva histórico via `git mv`).
- **Nova identidade visual:**
  - Paleta: cyan `#00E5FF` (destaque), azul `#0096FF` (primária), `#0047AB` (secundária),
    `#001A4D` (profundidade), prata `#C0C0C0` (metálico), ink `#0A0A0F` (background).
  - `assets/logo.png` substituída pela nova logo horizontal (RGBA, fundo transparente).
  - `assets/logo-mark.png` adicionado — só a garra (favicon/avatar/marca isolada).

### 🛡️ Preservado (deliberadamente)
- `Senior Engineer` (cargo humano em `AUDITORIA.md` e `P0-REPORT.md`).
- `/workspace/SeniorSystem` e `https://github.com/RAGton/SeniorSystem` (referências históricas
  de rollback — apontam para o repo antigo, antes do rebranding).

### 🔬 Verified
- `cargo check --all-targets` → ✅ exit 0 (5m28s, deps inalteradas, 1 warning de dep transitiva).
- `grep -ril senior --exclude-dir=target --exclude-dir=.git` → ✅ 0 hits (exceto preservados).
- Hash do `assets/logo.png` mudou de `452f052f…` para `9c9313eb…`.

### ⚠️ Not Verified
- `cargo clippy` (binário `clippy` ausente no toolchain Nix desta sessão).
- `cargo test --lib` → ✅ 92 passed; 0 failed; 0 ignored; finished in 0.02s.

## [Não lançado] — 1.9.0 — Fundação P0

### ⚠️ Breaking Changes

- **Senha padrão `admin/admin` removida.** O servidor não cria mais
  automaticamente. Use `gar-system-admin create-admin` (CLI) para
  criar o primeiro administrador.
- **`POST /usuarios` ignora `papel` no body.** Novos usuários sempre
  nascem com `Comercial`. Para promover, usar `PUT /usuarios/{username}/papel`
  (ainda em construção; precisa de um admin logado).
- **Autenticação obrigatória em todas as rotas** (exceto `/`, `/login`,
  `/livez`, `/readyz`). Sem `Authorization: Bearer <token>`, 401.
- **JWT_SECRET obrigatório em build release.** Servidor PANICA se a
  variável não estiver setada.
- **`db-init/init.sql` simplificado.** Tabelas duplicadas removidas.
  Schema fonte em `db-init/migrations/0001_*.sql`.

### Adicionado

- 🔐 **Auth middleware** via `Claims` extractor (axum 0.8). Tokens agora
  carregam `uid`, `papel`, `tenant`, `exp`, `iat`.
- 🔐 **Anti-enumeração no login** — bcrypt dummy roda quando usuário não
  existe. Tempo de resposta indistinguível.
- 🛡️ **Rate limit em /login** (5 req/min por IP, configurável via
  `LOGIN_RATE_LIMIT_PER_MINUTE`).
- 🛡️ **CORS por ambiente** — `*` apenas em dev; em prod, lista explícita
  via `CORS_ALLOWED_ORIGINS`.
- 🛡️ **Erros padronizados** com `request_id`, `code`, `message` —
  sem stack trace, sem SQL, sem segredos em logs.
- 🛡️ **Health checks separados** — `/livez` (processo vivo) e `/readyz`
  (banco responde `SELECT 1`).
- 🗄️ **Migrations versionadas** em `db-init/migrations/` com runner Rust
  em `src/banco_de_dados/migrations.rs`. Tabela `schema_migrations`.
- 🗄️ **Bug crítico de estoque corrigido** em `atualizar_os`: diff entre
  estado antigo e novo + `SELECT ... FOR UPDATE` para concorrência.
- 🗄️ **Lock pessimista** em `criar_os` e `atualizar_os` para serializar
  edições concorrentes.
- 🗄️ **Tabela `movimentacoes`** (consultada mas não existia — agora sim).
- 🗄️ **Coluna `credito` em `clientes`** (consultada mas não existia).
- 🗄️ **Coluna `tenant_id` em `users`, `clientes`, `ordens_servico`**
  (preparação multi-tenant).
- 🗄️ **Índices em todas as FKs** e em colunas de `WHERE` mais frequentes.
- 📦 **CLI `gar-system-admin`** para bootstrap seguro de admin.
- 📦 **`.env.example` reescrito** com placeholders + comentários.
- 📦 **`.gitignore` completo** — `.env*` ignorado, exceto `.env.example`.
- 🧪 **16 testes** (6 auth + 3 rate limit + 7 estoque diff).
- 📚 **Documentação nova:** `CONTEXT.md` (memória técnica), `CHANGELOG.md`
  (este), `P0-REPORT.md` (relatório desta sprint).

### Corrigido

- 🐛 Estoque corrompia a cada edição de OS (diff vs decrement total).
- 🐛 Health check mentiroso (sempre "ok" mesmo com DB caído).
- 🐛 Race condition em edição concorrente de OS.
- 🐛 `cliente.rs` lia coluna `credito` que não existia.
- 🐛 `cliente.rs` consultava tabela `movimentacoes` que não existia.
- 🐛 `init.sql` declarava 6 tabelas 2x (fornecedores, pecas, etc).
- 🐛 `init.rs` criava admin/admin hardcoded.
- 🐛 `auth.rs` fallback inseguro `chave_padrao_MUITO_INSEGURA`.
- 🐛 GUI descartava o token JWT retornado no login.
- 🐛 GUI escondia botões por papel mas servidor não validava.
- 🐛 `.env` versionado com credenciais reais (`200519`).
- 🐛 `.gitignore` incompleto (`.env` não estava listado).
- 🐛 Tabelas sem `created_at`/`updated_at`.

### Removido

- 🗑️ `.env-exemple` (duplicado do `.env.example`).
- 🗑️ `.directory` (artefato KDE, sem motivo no repo).
- 🗑️ `src.lib` (arquivo vazio).
- 🗑️ `garantir_admin()` — substituído pelo CLI `gar-system-admin`.
- 🗑️ Senha hardcoded `admin` no `init.rs`.
- 🗑️ Fallback inseguro de `JWT_SECRET`.

### Segurança (resumo)

- **Antes:** Backend sem auth, qualquer um fazia qualquer coisa via HTTP.
  Login era decorativo, GUI ignorava token.
- **Depois:** Todas as rotas (exceto login/health) exigem JWT válido.
  Token tem 24h de vida, é persistido no eframe::Storage. Mass
  assignment fechado. Rate limit em /login. Anti-enumeração.

## [1.8.0] — 2026-08-15

### Adicionado
- Tabela `servicos` e `ordem_servico_servicos` (mão de obra).
- Modal "📄 Ver/Imprimir" em ordens, com export TXT.

## [1.7.0] — 2026-08-10

### Adicionado
- Tabela `movimentos_estoque` para auditoria.
- Tabela `pecas` com estoque_atual/estoque_minimo.

## [1.6.0] — 2026-08-01

### Adicionado
- Tema claro/escuro.
- Tela de configuração de servidor.
- Login com "Lembrar de mim".

## [1.5.0] — 2026-07-20

### Adicionado
- Refatoração cliente-servidor (Axum + eframe/egui).
- Pool de conexões MySQL com inicialização preguiçosa.
- Tela de login com bcrypt.

## [1.0.0] — 2026-05-01

- Versão inicial: GUI desktop em Rust com acesso direto ao MySQL.
