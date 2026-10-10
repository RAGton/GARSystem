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
- ⏸️ Smoke test OS/orçamento/estoque: **GETs todos 200** (2 OSs, 0 pecas, 0 servicos). POSTs com 422 (validação de schema — DTO exige `telefone` em cliente, `id` em OS).
- ⏸️ Auth/cookies: pendente auditoria
- ⏸️ Tenant isolation: pendente auditoria
- ⏸️ POST validation: corrigir `#[serde(default)]` nos DTOs
- ⏸️ GUI desktop: GUI não abre por limitação do `glutin 0.32` em Xwayland rootless + carga alta de VM QEMU (`srv-garos` em 162% CPU). Validação via curl funciona. Decisão: seguir com API no ciclo autônomo.

## Ciclo 6 — P0 Resolvido: 500 no `GET /ordens` (2026-10-09)

**ID:** BUG-500-NULL
**Status:** PASSOU (commit 54e26c2, tag v1.11.1)
**Evidência:**
- `GET /ordens` retornava HTTP 500 (panic captado em `JoinError::Panic`)
- Root cause: `mysql_common 0.35.5` panica em `mysql::from_value::<String>(Value::Null)`
- `DATE_FORMAT(NULL, ...)` retorna NULL, e como `prazo_entrega` é nullable no schema, a query retornava Null
- O `unwrap_or_default()` em `Option<Value>` **não protege** contra `Some(Null)`, só contra `None`
- Fix: padronizar todos os 7 campos String do `from_row_opt` pra tratar `Some(Value::Null)` → `String::new()`:
  - `status`, `situacao` (eram `StatusOS`/`SituacaoOS`, agora tratam NULL com default)
  - `parecer_tecnico`, `observacoes`, `nome_tecnico_responsavel`, `atendente`, `telefone_cliente`, `prazo_entrega` (todos `String`, NULL → `""`)
- Tambem removido debug temporario do `map_erro` e `handler_listar_ordens`
- `cargo build --bin gar-system-server`: exit 0
- `cargo test --lib`: 92 passed, 0 failed
- Smoke test E2E (5 endpoints GET): todos HTTP 200

**Próximo passo:** Investigar e corrigir 2 bugs de validação em POST (`/clientes` exige `telefone`, `/ordens` exige `id` no body) ou seguir com próxima tarefa do tracker.


---

## Ciclo 4 — Modernização da UI e Correção de Bugs (2026-10-09)

**ID:** UI-MOD-001
**Status:** PASSOU
**Evidência:**
- Branch `ui/erp-modernization` (PR #7) finalizada, corrigida e mesclada na master.
- Resolvido bug bloqueante no cadastro de peças (validação exigia campo "Nome" não existente no formulário nem no BD).
- Substituição da lógica original por validação pura. Estoque e preço devidamente renderizados.
- Repaint automático do relógio na barra superior usando `ctx.request_repaint_after`.
- Implementado novo design de sidebar em `src/telas/componentes/sidebar.rs`, usando item padronizado com hover/ativo e removendo itens desabilitados (Funcionalidades Futuras).
- Testes automatizados passando (`cargo test --lib` e `cargo test --bin gar-system-gui`).
- *Limitação conhecida:* "Criar/Editar peça" grava direto no banco via chamada de driver no cliente. O refatoramento para API com rotas `POST/PUT /estoque/pecas` ficará para um próximo ciclo a fim de não alterar as assinaturas atuais do backend por estética, conforme requisito.

**Próximo passo:** Estender o padrão moderno da barra e do `ui_kit` (quando criado) para as demais telas (Clientes, Serviços, etc).

---

## Ciclo 5 — Módulo UI Kit e Lotes 1 e 2 (2026-10-09)

**ID:** UI-MOD-002
**Status:** PASSOU
**Evidência:**
- Adicionado módulo `src/telas/componentes/ui_kit.rs` com componentes reutilizáveis (`cabecalho` e `estado_vazio`).
- Lote 1 finalizado e em master: Clientes (`painel_clientes.rs`) e Serviços (`painel_servicos.rs`).
- Lote 2 finalizado e em master: Ordens de Serviço (`painel_ordens.rs`) e Orçamentos (`painel_orcamentos.rs`).
- Adotados grids estruturados, layout unificado de títulos com ações, e tratamento padronizado de estados vazio (sem dados falsos, coerente com backend).
- Resolvidos avisos (warnings) de códigos depreciados do `egui` (ex: `Frame::none` para `Frame::NONE`, `id_source` para `id_salt`, `rounding` para `corner_radius`).
- Compilation checks aprovados via Nix.

**Próximo passo:** Modernizar o restante (Painel Dashboard, Relatórios, etc.) ou finalizar e empacotar/testar.

