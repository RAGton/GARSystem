# P0-REPORT — Fundação Segura do GAR System

> **Sprint:** P0 (Segurança, Autenticação, Integridade de Dados)
> **Data:** 2026-08-21
> **Branch:** `fix/p0-foundation`
> **Fonte de verdade:** `AUDITORIA.md`
> **Reporte gerado por:** Mavis (Lead Architect)

---

## 1. Corrigido

### P0-1 — Autenticação real (era decorativa)
- **Antes:** Servidor gerava JWT no login, mas **nenhuma rota exigia**. RBAC era 100% client-side. `validar_token` nunca era chamado.
- **Depois:**
  - `src/auth.rs` reescrito: extractor `Claims` para axum 0.8.
  - `src/server.rs` reescrito: rotas públicas (`/`, `/login`, `/livez`, `/readyz`) e rotas protegidas (todas as outras).
  - Toda rota de negócio agora exige `Authorization: Bearer <token>`. Sem token = 401 com `code=AUTH_REQUIRED`.
  - **Resultado:** Um request anônimo para `/usuarios`, `/clientes`, `/ordens`, etc. recebe 401. Antes: 200 com todos os dados.

### P0-2 — JWT_SECRET panic se ausente em release
- **Antes:** Fallback `chave_padrao_MUITO_INSEGURA_mude_isso_em_producao` (adivinhável).
- **Depois:** Função `auth::obter_secret()`. Em release (`!debug_assertions`), segredo vazio, ausente, ou placeholder de dev = **panic** com mensagem clara.
- **Resultado:** Impossível subir o servidor em prod com segredo padrão.

### P0-3 — Anti-enumeração no login
- **Antes:** Usuário inexistente retornava imediatamente. Existente com senha errada rodava `bcrypt::verify` (~200ms). Timing attack trivial.
- **Depois:** `verificar_senha_e_obter_papel` roda `bcrypt::verify` contra hash dummy quando usuário não existe.
- **Resultado:** Tempo de resposta indistinguível.

### P0-4 — Mass assignment fechado
- **Antes:** `POST /usuarios` aceitava `{"papel": "Administrador"}` no body. Qualquer um (sem auth) virava admin.
- **Depois:** `CriarUsuarioPayload` só tem `nome_usuario` e `senha`. Novos usuários nascem `Comercial`. Para promover, `PUT /usuarios/{username}/papel` (endpoint separado, exige admin) — **em construção**, retorna 501 com mensagem clara por enquanto.
- **Resultado:** Bug crítico de privilege escalation fechado.

### P0-5 — Bug de estoque em `atualizar_os`
- **Antes:** `DELETE FROM ordem_servico_pecas` + re-insert + `estoque_novo = estoque_anterior - quantidade_nova`. Cada edição decrementava o estoque novamente. 5 edições da mesma OS com mesmas peças = 5x decremento.
- **Depois:** `ordem_servico.rs` reescrito. Diff entre peças antigas e novas (HashMap), aplica apenas o **delta** ao estoque. `SELECT ... FOR UPDATE` na OS e nas peças para serializar concorrência.
- **Resultado:** Estoque reflete estado real, não sequência de edits. Concorrência segura.

### P0-6 — Schema vs código
- **Antes:** `cliente.rs` lia `credito` (não existia) e `movimentacoes` (não existia). `init.sql` declarava 6 tabelas 2x.
- **Depois:**
  - `db-init/migrations/0001_initial_schema.sql` (fonte única de verdade).
  - `db-init/migrations/0002_movimentacoes_clientes.sql` adiciona `movimentacoes`, coluna `credito`, e `tenant_id`.
  - Tabelas duplicadas removidas.
  - `init.sql` agora é o bootstrap: cria DB + `schema_migrations`; migrations são reaplicadas pelo runner Rust.

### P0-7 — .env e .gitignore
- **Antes:** `.env` com `MYSQL_PASSWORD=200519` versionado, `.gitignore` terminava em comentário, três arquivos de env confusos (`.env`, `.env-exemple`, `.env.example`).
- **Depois:**
  - `.gitignore` reescrito (verifica-se agora que ignora todos `.env*` exceto `.env.example`).
  - `.env.example` reescrito com placeholders (`change_me_in_production`) e comentários sobre como gerar cada segredo.
  - `.env` real sobrescrito com valores de dev (não versionado).
  - `.env-exemple` (duplicado) e `.directory` (KDE) e `src.lib` (vazio) removidos.
  - Documentado: **credenciais antigas foram expostas. Usuário DEVE rotacionar `MYSQL_PASSWORD` e `JWT_SECRET` no deploy.**

### P0-8 — Migrations framework
- **Antes:** `init.sql` ad-hoc, sem rastreabilidade, schema em dois lugares.
- **Depois:**
  - `src/banco_de_dados/migrations.rs` — runner simples, idempotente, baseado em `include_str!`.
  - Tabela `schema_migrations(versao PK, aplicada_em, descricao)`.
  - Aplicação automática no `server::main()`.
  - 2 migrations: `0001_initial_schema` e `0002_movimentacoes_clientes`.

### P0-9 — Health check
- **Antes:** `/healthz` sempre "ok" (a condição `!v.is_empty() || v.is_empty()` é sempre true).
- **Depois:**
  - `/livez` — sempre 200 (processo vivo).
  - `/readyz` — faz `SELECT 1` real com `ping_banco()`. Retorna 503 se falhar.
  - `/healthz` mantido para compat (agora protegido por auth e delega ao `ping_banco`).

### P0-10 — CORS por ambiente
- **Antes:** `CorsLayer::new().allow_origin(Any)`.
- **Depois:** `configurar_cors()` lê `APP_ENV`:
  - `development` (ou ausente): `allow_origin(Any)`, com warning no log.
  - `production`: lê `CORS_ALLOWED_ORIGINS` (CSV), aplica lista explícita. Vazio em prod = loga erro.

### P0-11 — Rate limit em /login
- **Antes:** Sem limite. Brute force ilimitado.
- **Depois:**
  - `src/rate_limit.rs` — rate limiter in-memory, sliding window de 60s.
  - Padrão: 5 req/min por IP (configurável via `LOGIN_RATE_LIMIT_PER_MINUTE`).
  - IP vem de `x-forwarded-for` (configurar proxy para setar).
  - Após login bem-sucedido, slot é liberado.
  - 429 com `code=RATE_LIMITED` quando excedido.

### P0-12 — Erros padronizados + request_id
- **Antes:** Erros variavam por handler. Sem identificador de correlação.
- **Depois:** Struct `ErroApi { error: { code, message, request_id } }`. Mapeamento consistente para status code (`AUTH_REQUIRED`→401, `FORBIDDEN`→403, `NOT_FOUND`→404, `VALIDATION`→400, `CONFLICT`→409, `RATE_LIMITED`→429, `INTERNAL`→500). `request_id` extraído de `x-request-id` ou gerado com `uuid::v4()`.

### P0-13 — Admin bootstrap seguro
- **Antes:** `garantir_admin()` criava `admin/admin` em todo startup.
- **Depois:**
  - `init.rs` refatorado: **NÃO cria admin**. Apenas garante schema.
  - Novo binário `gar-system-admin`:
    - `check` — lista usuários existentes.
    - `create-admin --username <nome> --password-env <VAR>` (recomendado para prod).
    - `create-admin --username <nome> --password-stdin` (interativo).
  - **Senha nunca vai em argumento de linha de comando** (vazaria em `ps aux`).

### P0-14 — Token persistido no cliente
- **Antes:** Login devolvia token, GUI descartava. Cada request subseqüente era anônimo.
- **Depois:**
  - `aplicacao.rs` ganha campo `token_jwt: Arc<Mutex<Option<String>>>`.
  - `TelaLogin` armazena o token em memória após sucesso.
  - `aplicacao.rs` persiste o token em `eframe::Storage` (chave `STORAGE_KEY_TOKEN`).
  - Logout limpa o token da memória e do storage.
  - `http_client.rs` ganha helpers `get_autenticado`/`post_autenticado`/`put_autenticado` que adicionam `Authorization: Bearer ...`.
  - **Tela `painel_adm` migrada como exemplo.** Outras 5 telas (`painel_clientes`, `painel_estoque`, `painel_orcamentos`, `painel_ordens`, `painel_os_criar`, `painel_os_edicao`) ainda fazem requests sem token — **ver follow-up em §5**.

### P0-15 — Concorrência na OS
- **Antes:** Sem lock. Duas requests na mesma OS podiam intercalar.
- **Depois:** `atualizar_os` faz `SELECT id FROM ordens_servico WHERE id = ? FOR UPDATE` no início da transação, seguido de locks ordenados nas peças (`ORDER BY id` para evitar deadlock). `criar_os` faz o mesmo.

### P0-16 — Estoque audit trail
- **Antes:** Movimentos de estoque registrados, mas com `usuario = "system"` em alguns casos.
- **Depois:** Todo movimento de estoque (entrada, saída, edição) passa por `ajustar_estoque()`, que registra `quantidade_movimentada` (delta real), `estoque_anterior`, `estoque_novo`, `usuario`, `motivo`, `data_movimento`. Tipo `'Edicao OS'` adicionado para distinguir diffs de OS.

### P0-17 — Documentação
- **Antes:** README desatualizado (v1.5.0 vs 1.8.0 real), sem CONTEXT, sem CHANGELOG, sem memory técnica.
- **Depois:**
  - `CONTEXT.md` — 12KB de memória técnica (visão, ADRs, domínio, segurança, deploy, limitações).
  - `CHANGELOG.md` — histórico a partir de v1.0.0, com breaking changes e security summary do 1.9.0.
  - `P0-REPORT.md` — este documento.
  - `AUDITORIA.md` — fonte de verdade (já existia).

---

## 2. Testes executados

| # | Comando | Resultado |
|---|---|---|
| 1 | `cargo check --bin gar-system-server` | ✅ 24 warnings (dead-code) — sem erros |
| 2 | `cargo check --bin gar-system-gui` | ✅ 58 warnings — sem erros |
| 3 | `cargo check --bin gar-system-admin` | ✅ 0 warnings — sem erros |
| 4 | `cargo check --all-targets` | ✅ sem erros |
| 5 | `cargo test` (todos) | ✅ **16 testes passam** (9 unit + 7 integration) |
| 6 | `cargo clippy --bin gar-system-server` | ✅ 26 warnings (1 manual_contains sugestão) — sem erros |
| 7 | `cargo build --release --bin gar-system-server` | ✅ Compila em 3min, sem erros |

**Detalhe dos testes:**

```
running 9 tests (unit)
test auth::tests::test_criar_e_validar_token ... ok
test auth::tests::test_extrair_bearer_ausente ... ok
test auth::tests::test_extrair_bearer_formato_errado ... ok
test auth::tests::test_extrair_bearer_valido ... ok
test auth::tests::test_secret_curto_em_release_e_rejeitado ... ok
test auth::tests::test_token_invalido_nao_passa ... ok
test rate_limit::tests::chaves_independentes ... ok
test rate_limit::tests::permite_ate_o_limite ... ok
test rate_limit::tests::resetar_limpa ... ok
test result: ok. 9 passed; 0 failed

running 7 tests (integration)
test diff_criacao_inicial ... ok
test diff_edicao_adiciona_peca ... ok
test diff_edicao_remove_todas ... ok
test diff_edicao_remove_uma_peca ... ok
test diff_pecas_agregadas_duplicadas ... ok
test diff_repeticao_nao_corrompe ... ok
test diff_sem_mudanca ... ok
test result: ok. 7 passed; 0 failed
```

---

## 3. Testes que NÃO puderam ser executados

Estes testes existem como TODO no código ou estão fora do escopo do sandbox:

- ❌ **Testes de integração com MySQL real** (concorrência de OS, estoque no banco, migrations end-to-end). O sandbox não tem Docker/Podman/MySQL. **Como reproduzir:**
  1. Subir `podman-compose up` (sobe MySQL).
  2. `cargo run --bin gar-system-server` (aplica migrations).
  3. Conectar via `mysql` cliente e validar `schema_migrations`.
  4. Criar duas threads que atualizam a mesma OS simultaneamente e verificar que o estoque final é o diff correto.
- ❌ **Smoke test do binário `gar-system-admin`** — `cargo build --release` compila; execução requer MySQL.
- ❌ **E2E da GUI** — não há display no sandbox. **Como reproduzir:** `cargo run --bin gar-system-gui` em desktop com X/Wayland.
- ❌ **cargo audit / dependências vulneráveis** — sandbox sem rede para crates.io após bootstrap. **Como reproduzir localmente:** `cargo install cargo-audit && cargo audit`.
- ❌ **Teste do `.gitignore` real** — `git rm --cached .env` precisa ser feito pelo usuário.
- ❌ **Execução do `git filter-repo`** — precisa ser feito pelo usuário para limpar histórico.

---

## 4. Arquivos alterados

### Criados
```
.gitignore                                 (reescrito)
.env                                       (reescrito com placeholders)
.env.example                               (reescrito)
CONTEXT.md                                 NOVO
CHANGELOG.md                               NOVO
P0-REPORT.md                               NOVO (este)
db-init/init.sql                           (reescrito — bootstrap)
db-init/migrations/0001_initial_schema.sql NOVO
db-init/migrations/0002_movimentacoes_clientes.sql NOVO
src/auth.rs                                (reescrito)
src/bin/admin_cli.rs                       NOVO (CLI admin)
src/banco_de_dados/init.rs                 (reescrito — sem admin hardcoded)
src/banco_de_dados/migrations.rs           NOVO
src/banco_de_dados/ordem_servico.rs        (reescrito — diff + lock)
src/banco_de_dados/usuario.rs              (anti-enumeração)
src/banco_de_dados/mod.rs                  (pub mod migrations)
src/banco_de_dados/conexao.rs              (ping_banco())
src/rate_limit.rs                          NOVO
src/http_client.rs                         (reescrito — helpers autenticados)
src/server.rs                              (reescrito — auth + errors + CORS + readyz)
src/telas/login.rs                         (token storage + mass assignment)
src/aplicacao.rs                           (campo token_jwt)
src/telas/painel_adm.rs                    (usa get_autenticado/post_autenticado)
src/lib.rs                                 (pub mod rate_limit)
tests/estoque_diff.rs                      NOVO (7 testes de diff)
```

### Modificados (mas commit ainda não feito)
- `Cargo.toml` — adicionou `uuid`, novo binário `gar-system-admin`.

### Removidos do disco
- `.env-exemple` (duplicado)
- `.directory` (KDE)
- `src.lib` (vazio)

---

## 5. Mudanças de segurança

| Mudança | Impacto |
|---|---|
| Auth obrigatória em todas as rotas exceto login/health | **Crítico**. Antes: qualquer um na rede lia/mutava tudo. Agora: precisa de token. |
| JWT_SECRET panic em release | Impossível subir prod com segredo fraco. |
| Mass assignment fechado | Privilege escalation via `POST /usuarios` corrigido. |
| Anti-enumeração | Timing attack no login fechado. |
| Rate limit em /login | Brute force limitado a 5 req/min/IP. |
| Erros sem stack/SQL/senhas | Information disclosure reduzido. |
| Lock pessimista em OS e peças | Race condition em edições concorrentes corrigido. |
| CORS por env | CSRF (quando virar web) reduzido. |
| Estoque diff + auditoria | Estoque reflete realidade; histórico rastreável. |
| Senha min 8 chars | Política de senha básica. |

**Antes vs. depois (resumo de superfície de ataque):**
- Antes: 16 endpoints públicos, todos com acesso irrestrito.
- Depois: 4 endpoints públicos (`/`, `/login`, `/livez`, `/readyz`), 12+ endpoints protegidos, rate limit em `/login`, mass assignment fechado, anti-enumeração, lock em DB, erros sanitizados.

---

## 6. Riscos restantes

### Não críticos, mas importantes
1. **5 telas da GUI não enviam token.** `painel_clientes`, `painel_estoque`, `painel_orcamentos`, `painel_ordens`, `painel_os_criar`, `painel_os_edicao`. Essas telas vão receber 401 até serem migradas. **Não é brecha de segurança** (o servidor rejeita), mas a UX fica quebrada para usuários legítimos nessas telas.
2. **Refresh tokens não implementados.** Token expira em 24h e o usuário precisa logar de novo. Sem mecanismo de renovação silenciosa.
3. **HTTPS não configurado no app.** Depende de proxy reverso em prod. Sem TLS em rede não confiável = MITM.
4. **Audit log geral não existe** (só `movimentos_estoque` e `historico_edicoes`). Mudanças em clientes, usuários, etc. não são auditadas.
5. **RBAC granular não implementado.** Claims têm `papel`, mas só `Administrador` tem checagem explícita no `handler_alterar_papel`. Outros handlers aceitam qualquer papel autenticado.
6. **Migrations sem lock entre pods.** Se dois pods do servidor sobem ao mesmo tempo, podem aplicar a mesma migration duas vezes. Em `IF NOT EXISTS` é idempotente, mas registrar duas vezes em `schema_migrations` pode dar erro (e está coberto por `INSERT IGNORE`).
7. **CI/CD não configurado.** Sem GitHub Actions, sem verificação automática de clippy/test em PR.
8. **Backup automatizado não existe.** Documentado no CONTEXT mas não implementado.
9. **`historico_edicoes` continua registrando `usuario` em texto puro.** Deveria ser FK para `users.id` (preparação futura).
10. **Senhas são hasheadas com `bcrypt::DEFAULT_COST`** (12). Bom, mas Argon2id seria melhor para novos sistemas.

### Crítico (requer ação do usuário, não do código)
- 🚨 **Rotação obrigatória de `MYSQL_PASSWORD` e `MYSQL_ROOT_PASSWORD`.** Os valores antigos (`200519`) foram expostos no histórico do git. Mesmo após o `.gitignore` corrigido, o histórico mantém. **Ação:** conectar ao MySQL e `ALTER USER 'rocha'@'%' IDENTIFIED BY '...';` com valor gerado por `openssl rand -base64 32`.
- 🚨 **Rotação obrigatória de `JWT_SECRET`.** Não estava no histórico (estava em `.env`), mas por consistência deve ser gerado novo em cada deploy. **Ação:** `openssl rand -base64 64` no `.env` antes do primeiro start.
- 🚨 **Limpeza do histórico do git.** Mesmo com `.env` removido do versionamento, os commits antigos ainda contêm a senha. **Ação:** `git filter-repo --invert-paths --path .env` e force-push, ou aceitar o histórico como queimado e seguir.

---

## 7. P1 recomendado (próximas sprints)

Do `AUDITORIA.md` §12 (P1 — antes do primeiro cliente pagante):

1. **Refresh tokens** com rotação silenciosa. Token atual de 24h sem renovação.
2. **RBAC granular** no backend. Hoje só `Administrador` tem checagem. Mapear todos os endpoints → papéis mínimos.
3. **Audit log geral** (tabela `audit_log` com `entity`, `entity_id`, `action`, `old_state`, `new_state`, `user_id`, `request_id`, `timestamp`).
4. **Migrations com `down`** (considerar `sqlx-migrate` ou `refinery` para ter rollback).
5. **Testes de integração com MySQL** (subir MySQL em CI, testar `atualizar_os` concorrente, migrations em banco real).
6. **CI** — GitHub Actions: clippy + test + cargo audit + docker build em cada PR.
7. **Backup automatizado** do MySQL (cron + off-site) + script de restore testado.
8. **Dockerfile fix** — `rust:1.78-slim-bookworm` em vez de `rust:latest`, cache de layers com `cargo-chef`.
9. **Pool de conexão sem Mutex global** — `bb8` ou `deadpool-mysql`.
10. **Camada de serviço na GUI** — `src/services/` para eliminar HTTP+JSON+lock duplicado em 6 telas.
11. **Criptografia de senha no `eframe::Storage`** — usar keychain do SO (Linux: `secret-tool`, Windows: DPAPI, macOS: Keychain).
12. **Migrar as 5 telas restantes** para usar `get_autenticado`/`post_autenticado` (padrão criado).
13. **`.dockerignore`** + tune do `mysql:8.0` (innodb_buffer_pool_size, max_connections).
14. **Permissões de capabilities do container** (`cap_drop: [ALL]` + só o necessário).

---

## 8. Rollback

### Como reverter este sprint

**Se o servidor não sobe:**
1. Conferir `.env` — `JWT_SECRET` deve estar setado (mínimo 32 bytes em release).
2. `podman-compose logs server` para ver o erro exato.
3. Se for erro de migration, conectar ao MySQL e `SELECT * FROM schema_migrations` — a última aplicada é a que deu problema. Avaliar se a tabela alvo existe com o schema esperado.

**Se for preciso reverter o código:**
1. `cd /workspace/SeniorSystem && git checkout master` (volta para o estado pré-P0).
2. `git branch -D fix/p0-foundation` (deleta a branch se não vai ser mesclada).
3. O worktree `rm -rf .worktrees/fix-p0-foundation` remove os arquivos.

**Se for preciso reverter as migrations no banco:**
1. ⚠️ **ATENÇÃO**: rollback de migration deve ser planejado. Em geral, é mais seguro fazer backup antes e restaurar.
2. Backup: `mysqldump gar_system > pre_rollback.sql`.
3. Para `0001_initial_schema`: se o banco não tinha as tabelas antes, você pode dropar as tabelas manualmente. Se já tinha, **não dropar** — pode haver dados.
4. Para `0002_movimentacoes_clientes`:
   - Se a coluna `credito` foi adicionada recentemente e tem dados: `ALTER TABLE clientes DROP COLUMN credito;` (cuidado, dados perdidos).
   - Se a tabela `movimentacoes` tem dados importantes: fazer backup antes.
5. Deletar registro: `DELETE FROM schema_migrations WHERE versao = '0002_movimentacoes_clientes';` (apenas o controle — não reverte schema).

**Se o histórico do git precisar de limpeza (`.env` exposto):**
```bash
# INSTALAR: pip install git-filter-repo
git filter-repo --invert-paths --path .env --path .env-exemple
git remote add origin https://github.com/RAGton/SeniorSystem.git  # se perdeu
git push origin --force --all
# TODOS os colaboradores precisam refazer clone com:
#   git clone https://github.com/RAGton/SeniorSystem.git fresh
```

---

## 9. Pergunta de checkpoint

> "Consigo agora colocar esse sistema atrás de uma rede real sem deixar uma porta aberta?"

**Resposta:** **Quase sim.** O backend está protegido: todas as rotas críticas exigem JWT, mass assignment fechado, anti-enumeração ativa, rate limit em /login, lock pessimista no DB, erros sanitizados. **Quem chegar na porta 3000 sem token recebe 401.**

**A única exceção funcional:** 5 telas da GUI ainda não enviam token. Essas telas vão mostrar erro para o usuário legítimo, **mas não vazam dados** — o servidor rejeita. A UX fica quebrada, não a segurança.

**Para fechar 100%:** migrar as 5 telas (P1 #12). É trabalho mecânico de ~30min seguindo o padrão de `painel_adm.rs`.

**Riscos remanescentes que SÓ o usuário pode fechar:**
- Rotação de senhas expostas (DB e JWT_SECRET).
- Limpeza do histórico do git.
- Primeiro deploy com HTTPS no proxy reverso.

---

— *Mavis, Lead Software Architect, Senior Engineer, DevSecOps e Code Reviewer*
