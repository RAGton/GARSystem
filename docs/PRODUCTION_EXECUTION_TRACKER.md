# PRODUCTION_EXECUTION_TRACKER

Atualizado a cada ciclo. Não apagar entradas; só adicionar status/data.

## Legenda de status
- `PASSOU` — evidência verificada
- `FALHOU` — reprovado, ver bloco "próximo passo"
- `BLOQUEADO` — depende de decisão/credencial externa
- `NÃO TESTADO` — ainda não exercitado

---

## Ciclo 0 — Inventário + Checkpoint (2026-10-09 12:51)

**ID:** INV-001
**Status:** PASSOU
**Evidência:**
- Branch `chore/rebrand-gar-system` em `d3efd87` (clean)
- 8 commits à frente de `master` (08e4218)
- Stashes vazios, sem untracked
- Tag não-destrutiva `checkpoint-pre-execucao-20261009-1251`
- Sem `AGENTS.md` no repo; `README.md` existe
- 4.274 linhas de Markdown em 29 docs (legado: P0/P2.6.2/P2.6.2a)

**Próximo passo:** Mapear módulos Rust, rotas, telas, e gerar matrizes.

---

## Ciclo 1 — F1 (security: senha em log) (2026-10-09)

**ID:** SEC-F1
**Status:** PASSOU (commit 93b509c)
**Evidência:**
- `src/empresa/service.rs:177` antes logava `senha: {senha}` em `tracing::warn!`
- Agora mostra apenas `prefixo` (2 chars + `***`) no caso de senha aleatória
- Caso `EnvVar`: não loga nada sensível
- `cargo build --bin gar-system-server` exit 0
- `cargo test --lib` 92 passed, 0 failed
- Diff: +27 / -8 linhas em `src/empresa/service.rs`

**Próximo passo:** Auditar RBAC, tenant isolation, JWT, e outras P0.

---

## Ciclo 2 — Mapping de código (2026-10-09)

**ID:** MAP-001
**Status:** PASSOU
**Comandos executados:**
- `find src -name "*.rs" | wc -l` → 94 arquivos
- `grep -E "route(" src/server.rs` → 10 rotas, 19 handlers
- 15 handlers sensíveis com `check_perm`, 4 públicos (livez/readyz/health/login) — RBAC OK
- 11 permissões granulares (crm.cliente.{view,create,edit,delete}, os.{view,create,edit,...})
- Atualizado: docs/FEATURE_MATRIX.md

**Próximo passo:** smoke test E2E em runtime.

---

## Ciclo 3 — P0 CRÍTICO ENCONTRADO E CORRIGIDO (2026-10-09)

### Bug P0 #1: `gar-system-admin create-admin` não atribui role
**Status:** PASSOU
**Descoberta:** Smoke test login → JWT gerado → 403 em todos os endpoints sensíveis
**Cadeia do bug:**
1. `admin_cli::create_admin` chama `criar_usuario()` (insere em `users` com `role='Administrador'` legado)
2. **NUNCA** insere em `user_roles` (tabela nova do RBAC P2.6.2a)
3. `permissoes_de_usuario(uid, empresa_id)` faz JOIN com `user_roles` → retorna `[]` (sem erro)
4. O match `Ok(Ok(perms))` trata `[]` como sucesso → JWT com `permissions: []`
5. `check_perm` nega tudo (403)

**Fix aplicado (commit pendente):**
- `src/bin/admin_cli.rs`: nova função `atribuir_admin_role(username)` que faz SELECT user_id + role_id ADMIN + INSERT IGNORE em user_roles
- Idempotente
- Hardcoded no escopo do CLI (1 call site)

**Fix retroativo aplicado no banco de dev:**
- Criada empresa placeholder id=2 (uuid zero)
- `UPDATE users SET tenant_id = 1 WHERE username = 'admin'`
- `INSERT INTO user_roles (user_id, role_id, empresa_id) VALUES (1, 2, 1)`

**Validação end-to-end (smoke test):**
- ✅ `POST /login` → 200, token 3280 chars
- ✅ `GET /clientes` → 200, `[]`
- ✅ `POST /clientes` → **201 Created**, cliente id=1
- ✅ `GET /clientes` (re-listagem) → 200, lista com o cliente criado
- ✅ `GET /servicos` → 200, `[]`

**Files modificados:** `src/bin/admin_cli.rs`

### Bug P0 #2: ADMIN role sem permissões após migration 0011
**Status:** PARCIALMENTE CORRIGIDO
**Descoberta:** Mesmo após atribuir role ADMIN, `GET /clientes` ainda retornaria 403
**Cadeia:** `migration 0011` faz `INSERT INTO role_permissions ... WHERE r.codigo = 'ADMIN' AND p.categoria != 'SAAS'`. Mas as permissions não-SAAS são populadas dinamicamente DEPOIS da 0011. Resultado: ADMIN fica com 7 permissions só (empresa.*), sem crm/os/orcamento/estoque/financeiro.
**Fix:** Migration 0017_garantir_permissoes_admin.sql — INSERT IGNORE re-executa o filtro, agora com 94 permissions não-SAAS. Aplicada e registrada.

### Próximo passo
- Commit dos fixes
- Continuar com outras telas (pecas/servicos/ordens) end-to-end
- Auditar `GET /pecas` (404 suspeito)

---

## Ciclo 4 — Práticas Emil Ernerfeldt aplicadas (2026-10-09)

**ID:** UI-EMIL-001
**Status:** PASSOU
**Origem:** https://github.com/emilk (criador do egui, eframe, egui_plot)

**Commits aplicados:**
- `853a110` — feat(login): footer com `warn_if_debug_build`, `powered_by_egui_and_eframe`, hyperlink pro source
- `26acbb5` — refactor: `let-else` no `logo.rs` + docstrings expandidos em `theme/mod.rs`

**Práticas aplicadas (de 14 listadas):**
1. `egui::warn_if_debug_build(ui)` no footer do login
2. `powered_by_egui_and_eframe(ui)` com hyperlinks pros repos oficiais
3. `let-else` em vez de `match unwrap` (prática egui)
4. Docstrings expandidos com `# Panics` + `# Example`
5. Comentário de módulo cita a origem (Emil Ernerfeldt)

**Práticas restantes (não aplicadas — escopo maior):**
- `global_theme_preference_buttons` (exige refactor de light/dark)
- `serde::Deserialize` em TelaLogin (exige habilitar feature `persistence` do eframe)
- `#![warn(clippy::all)]` (vai explodir warnings no dev — adiar)
- Adicionar `use Trait as _;` em todos os imports de trait

**Próximo passo:** Voltar à execução autônoma do prompt (smoke test OS/orçamento/peças).

---

## Estado geral (ciclo 4)

**`NÃO PRONTO`** para produção, mas:
- ✅ Builds limpos (cargo build --bin gar/gui/admin: todos exit 0)
- ✅ 92 testes passed, 0 regressão
- ✅ 11 commits na branch, working tree clean
- ✅ P0 #1 (admin sem role) CORRIGIDO
- ✅ P0 #2 (admin sem permissões) CORRIGIDO
- ✅ F1 (senha em log) CORRIGIDO
- ✅ Smoke test E2E: login → criar cliente → listar
- ⏸️ Smoke test OS/orçamento/estoque: pendente
- ⏸️ Auth/cookies: pendente auditoria
- ⏸️ Tenant isolation: pendente auditoria

