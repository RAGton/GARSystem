# BACKEND_MAPPING — Auditoria completa (Rust/Axum + MySQL)

**Gerado por**: Hermes Enterprise Orchestrator
**Data**: sprint GAR System Enterprise
**Fonte**: leitura direta de `backend/src/**/*.rs`, `db-init/migrations/*.sql`, `docs/openapi.json`

---

## 1. Tabelas MySQL (state of DB)

### Núcleo (Sprint P2.0-P2.6.x)

| Tabela | Migration | Colunas-chave | Tenant? |
|---|---|---|---|
| `users` | 0001 | id, nome_usuario, senha_hash, papel, ativo, created_at, empresa_id, tenant_id | sim |
| `clientes` | 0001/0012/0016 | id, nome, email, telefone, endereco, cpf_cnpj, credito_disponivel, tenant_id | sim |
| `pecas` | 0001/0012 | id, codigo_interno, part_number, descricao, fabricante, localizacao, estoque_atual, estoque_minimo, preco_custo, preco_venda, tenant_id | sim |
| `fornecedores` | 0001 | id, nome, cnpj, contato, telefone, email, tenant_id | sim |
| `ordens_servico` | 0001/0009/0012 | id, cliente_id, equipamento_id, defeito_relatado, status, situacao, tecnico_responsavel, atendente, data_chegada, prazo_entrega, ..., tenant_id | sim |
| `orcamentos` | 0001/0006/0012 | id, cliente_id, total, tenant_id | sim |
| `itens_orcamento` | 0006 | id, orcamento_id, descricao, quantidade, preco_unitario, preco_total | sim |
| `equipamentos` | 0001/0005 | id, cliente_id, descricao, marca, modelo, numero_serie, patrimonio, observacao, tipo, tenant_id | sim |
| `historico_edicoes_os` | 0001 | id, os_id, usuario, data_hora, campo_alterado, valor_antigo, valor_novo | sim |
| `audit_log` | 0003/0017 | id, usuario_id, username, acao, entidade, entity_id, antes(depois_json), ip, user_agent, request_id, occurred_at | sim |

### CRM (P2.1 — migration 0004)

| Tabela | Migration | Função |
|---|---|---|
| `cliente_timeline_eventos` | 0004 | Eventos da timeline (criação, OS, orçamentos, observações…) |
| `cliente_observacoes` | 0004/0012 | Observações editáveis (com contador de edições) |
| `cliente_tags` | 0004/0012 | Tags globais por tenant |
| `cliente_tag_atribuicoes` | 0004 | NxN cliente↔tag |
| `cliente_contatos` | 0004/0012 | Telefone/WhatsApp/Email/Site por cliente |
| `cliente_movimentacoes` | 0002 | Movimentações de crédito |

### Operations (P2.4 — migration 0009)

| Tabela | Função |
|---|---|
| `workflow_definicao` | Workflow (state machine) por entidade |
| `workflow_estado` | Estados do workflow (slug, cor, ordem, é_inicial/final) |
| `workflow_transicao` | Transições válidas (origem→destino, requer_papel, exige_motivo) |
| `workflow_movimentacao` | Histórico de movimentações de OS |
| `agenda` | Agendas por tenant |
| `agenda_evento` | Eventos da agenda (visita/coleta/entrega…), conflita por intervalo |
| `sla_config` | Configuração de SLA por tenant |
| `sla_evento` | Eventos de SLA (RECEBIDO, DIAGNOSTICO_INICIO…) |
| `sla_calculo` | Cálculo agregado de SLA por OS |
| `alerta` | Definição de alerta (condição) |
| `alerta_ocorrencia` | Ocorrência concreta (severidade, mensagem, contexto_json) |

### Financeiro (P2.5 — migration 0010/0016)

| Tabela | Função |
|---|---|
| `financeiro_configuracao` | Singleton config por tenant |
| `plano_contas` | Plano de contas (ATIVO/PASSIVO/RECEITA/DESPESA/CUSTO/PATRIMONIO) |
| `centro_custo` | Centro de custo |
| `conta_receber` | Contas a receber (vinculável a orçamento) |
| `conta_pagar` | Contas a pagar |
| `lancamento` | Lançamentos append-only, imutáveis |
| `financeiro_alerta` | Alerta financeiro |
| `financeiro_alerta_ocorrencia` | Ocorrência concreta |

### Multi-tenant & RBAC (P2.6.1/2.6.2 — migrations 0011-0015)

| Tabela | Função |
|---|---|
| `empresa` | Empresa/tenant (multi-tenant SaaS) |
| `empresa_configuracao` | Config por tenant |
| `permissions` | Catálogo de permissions (string, ex: `crm.cliente.view`) |
| `roles` | Roles (ADMIN, GERENTE, TECNICO, FINANCEIRO, ATENDENTE) |
| `role_permissions` | NxN role↔permission |
| `user_roles` | NxN user↔role, **escopado por tenant_id** |

### SaaS / UX (0017/0018)

| Tabela | Função |
|---|---|
| `user_preferences` | Preferências UI por usuário |
| `login_audit` | Auditoria de logins (com IP, user-agent, sucesso/erro) |
| `tenant_branding` | Branding/tema por tenant |
| `dashboard_layouts` | Layout do dashboard customizável por usuário |
| `workspace_preferences` | Preferências de workspace |

### OS Mobile / Files (0007/0008)

| Tabela | Função |
|---|---|
| `arquivos` | Metadados de arquivos anexados |
| `os_mobile_*` | Tabelas auxiliares do app mobile de OS |

---

## 2. Módulos Rust (camada lib + bin)

| Módulo | Path | Status | Comentário |
|---|---|---|---|
| `auth` | `backend/src/auth.rs` | ✅ | JWT fail-fast, RBAC granular, cookie HttpOnly |
| `servicos` | `backend/src/servicos.rs` | ✅ | DTOs legados + funções de negócio |
| `banco_de_dados` | `backend/src/banco_de_dados/` | ✅ | Pool MySQL + repos |
| `crm` | `backend/src/crm/` | ✅ service+repo, ❌ handler | Falta API HTTP |
| `cotacao_orcamento` | `backend/src/cotacao_orcamento/` | ✅ | Existe — falta ver |
| `empresa` | `backend/src/empresa/` | ✅ | Existe — falta ver |
| `operations` | `backend/src/operations/` | ✅ service+repo, ❌ handler | Falta API HTTP |
| `os_mobile` | `backend/src/os_mobile/` | ✅ | Existe — falta ver |
| `rbac` | `backend/src/rbac/` | ✅ service+repo+cache, ❌ handler | Falta API HTTP |
| `storage` | `backend/src/storage/` | ✅ | Abstração local |
| `transcription` | `backend/src/transcription/` | ✅ | Mock + service |
| `arquivos` | `backend/src/arquivos/` | ✅ | CRUD com storage |
| `financial` | `backend/src/financial/` | ✅ service+repo, ❌ handler | Falta API HTTP |
| `rate_limit` | `backend/src/rate_limit.rs` | ✅ | Limiter 60s janela |
| `executor` | `backend/src/executor.rs` | ✅ | Utilitário |
| `http_client` | `backend/src/http_client.rs` | ✅ | client p/ GUI legacy |
| `gui_services` | `backend/src/gui_services/` | ✅ | Bridge p/ GUI egui |

### Binários (Cargo.toml)

| Binário | Path | Função | Status |
|---|---|---|---|
| `senior-system-server` | `src/server.rs` | API HTTP | ✅ compila |
| `senior-system-gui` | `src/main.rs` | Desktop egui | ✅ compila (legacy) |
| `senior-system-admin` | `src/bin/admin_cli.rs` | CLI admin | ✅ compila |
| `audit-rbac-achado1` | `src/bin/audit_rbac_achado1.rs` | Auditoria RBAC | ✅ compila |

---

## 3. Endpoints HTTP EXPOSTOS (server.rs:main)

Lista REAL de rotas registradas em `server.rs:290-324`:

### Públicas (sem auth)
| Método | Path | Handler | Permissão | Tenant | DTO Request | DTO Response | Service | Repository | Tabela MySQL |
|---|---|---|---|---|---|---|---|---|---|
| GET | `/` | anônimo | — | — | — | String | — | — | — |
| GET | `/livez` | `handler_livez` | — | — | — | `{status: "alive"}` | — | — | — |
| GET | `/readyz` | `handler_readyz` | — | — | — | `{status: "ready", database: "connected"}` | — | `conexao::ping_banco` | SELECT 1 |
| POST | `/login` | `handler_login` | — | — | `{usuario, senha}` | `{token, papel, expira_em}` | `servicos::verificar_login` + `rbac::repository::permissoes_de_usuario` | (legado `usuario_repo`) | `users`, `roles`, `role_permissions`, `user_roles` |

### Protegidas (exigem JWT)
| Método | Path | Handler | Permission | Tenant | DTO Request | DTO Response | Service | Repository | Tabela |
|---|---|---|---|---|---|---|---|---|---|
| GET | `/healthz` | `handler_health_check` | (Claims) | sim | — | `{status, database}` | — | `conexao::ping_banco` | SELECT 1 |
| GET | `/usuarios` | `handler_listar_usuarios` | `empresa.usuario.list` | sim | — | `[InfoUsuario]` | `servicos::listar_usuarios` | `usuario::listar` | `users` |
| POST | `/usuarios` | `handler_criar_usuario` | `empresa.usuario.create` | sim | `{nome_usuario, senha}` | 201 | `servicos::criar_usuario` | `usuario::criar` | `users` |
| PUT | `/usuarios/{username}/papel` | `handler_alterar_papel` ⚠️ STUB 501 | `empresa.usuario.assign_role` + papel=Admin | sim | `{novo_papel}` | 501 | `servicos::alterar_papel` ❌ não existe | — | `users.papel` (UPDATE) |
| GET | `/clientes` | `handler_listar_clientes` | `crm.cliente.view` | sim | — | `[Cliente]` | `servicos::listar_clientes` | `cliente::listar` | `clientes` |
| POST | `/clientes` | `handler_criar_cliente` | `crm.cliente.create` | sim | `{nome,email,telefone,endereco?,cpf_cnpj?}` | 201 | `servicos::criar_ou_atualizar_cliente` | `cliente::upsert` | `clientes` |
| GET | `/clientes/{id}/resumo` | `handler_resumo_cliente` | `crm.cliente.view` | sim | — | `{gastos_totais, credito_disponivel}` | `servicos::obter_gastos_e_credito` | `cliente::gastos_e_credito` | `clientes`, `ordens_servico`, `orcamentos` |
| GET | `/estoque/pecas` | `handler_listar_pecas` | `estoque.peca.view` | sim | — | `[Peca]` | `servicos::listar_pecas` | `estoque::listar_pecas` | `pecas` |
| GET | `/servicos` | `handler_listar_servicos` | `os.view` | sim | — | `[Servico]` | `servicos::listar_servicos_db` | `servico::listar` | `servicos` |
| POST | `/servicos` | `handler_criar_servico` | `os.edit` | sim | `Servico` | `Servico` | `servicos::criar_servico_db` | `servico::criar` | `servicos` |
| GET | `/ordens` | `handler_listar_ordens` | `os.view` | sim | — | `[OrdemServico]` | `servicos::listar_ordens_servico` | `ordem_servico::listar` | `ordens_servico` |
| POST | `/ordens` | `handler_criar_ordem_servico` | `os.create` | sim | `OrdemServico` | `OrdemServico` | `servicos::criar_ordem_servico` | `ordem_servico::criar` | `ordens_servico` |
| GET | `/ordens/{id}` | `handler_obter_ordem` | `os.view` | sim | — | `OrdemServico` | `servicos::buscar_os_por_id` | `ordem_servico::obter` | `ordens_servico` |
| PUT | `/ordens/{id}` | `handler_atualizar_ordem_servico` | `os.edit` | sim | `{os, usuario}` | 200 | `servicos::atualizar_os` | `ordem_servico::atualizar` | `ordens_servico`, `historico_edicoes_os` |
| POST | `/orcamentos` | `handler_criar_orcamento` | `orcamento.create` | sim | `Orcamento` | `Orcamento` | `servicos::criar_orcamento` | `orcamento::criar` | `orcamentos`, `itens_orcamento` |
| GET | `/orcamentos/{id}` | `handler_obter_orcamento` | `orcamento.view` | sim | — | `Orcamento` | `servicos::obter_orcamento` | `orcamento::obter` | `orcamentos`, `itens_orcamento` |

**Total exposto: 12 endpoints úteis**. Backend tem ~7 módulos com service/repository completos **SEM handler HTTP** (ver §4).

---

## 4. Endpoints PENDENTES (existem service+repository, faltam handlers)

### CRM (`backend/src/crm/`)
- `GET /crm/tags?tenant=...` → `service::listar_tags`
- `POST /crm/tags` → `service::criar_tag`
- `GET /crm/clientes/{id}/dashboard` → `service::dashboard_cliente` (DashboardCliente)
- `GET /crm/clientes/{id}/timeline?limite=50` → `service::listar_timeline`
- `POST /crm/clientes/{id}/observacoes` (body: `{conteudo}`) → `service::adicionar_observacao`
- `GET /crm/clientes/{id}/observacoes?limite=20` → `service::listar_observacoes`
- `PUT /crm/observacoes/{id}` → `service::editar_observacao`
- `POST /crm/clientes/{id}/contatos` (body: `{tipo,valor,rotulo?,principal,observacao?}`) → `service::adicionar_contato`
- `GET /crm/clientes/{id}/contatos` → `service::listar_contatos`
- `DELETE /crm/contatos/{id}?cliente_id=...` → `service::remover_contato`
- `POST /crm/clientes/{id}/equipamentos` (body: `{descricao,marca?,modelo?,numero_serie?,patrimonio?,observacao?,tipo}`) → `service::adicionar_equipamento`
- `GET /crm/clientes/{id}/equipamentos` → `service::listar_equipamentos`
- `DELETE /crm/equipamentos/{id}?cliente_id=...` → `service::remover_equipamento`
- `POST /crm/clientes/{id}/tags/{tagId}` → `service::atribuir_tag`
- `DELETE /crm/clientes/{id}/tags/{tagId}` → `service::remover_tag`
- `GET /crm/buscar?q=...&limite=20` → `service::buscar_clientes` (busca global)

### Operations (`backend/src/operations/`)
- `GET /workflow/definicoes` → `service::listar_estados`/`listar_transicoes`
- `GET /os/{id}/workflow` → `service::obter_status_workflow`
- `POST /os/{id}/mover` (body: `{transicao_id, motivo?}`) → `service::mover_estado` ⚠️ state machine central
- `GET /os/kanban` → `service::kanban` (KanbanColuna[])
- `GET /agenda?de=YYYY-MM-DD&ate=YYYY-MM-DD` → `service::listar_eventos_agenda`
- `POST /agenda` → `service::criar_evento_agenda`
- `GET /agenda/{id}` → `service::obter_evento_agenda`
- `PUT /agenda/{id}` → `service::atualizar_status_evento`
- `DELETE /agenda/{id}` → `service::remover_evento`
- `GET /agenda/conflitos?tecnico_id=...&inicio=...&fim=...` → `service::detectar_conflitos`
- `GET /os/{id}/sla` → `service::calcular_sla_os`
- `GET /alertas?severidade=critical&entidade=os` → `service::listar_alertas_ativos`
- `POST /alertas/{id}/resolver` → `service::marcar_alerta_resolvido`
- `POST /alertas/{id}/visualizar` → `service::marcar_alerta_visualizado`
- **`GET /dashboard/executivo`** → `service::dashboard_executivo` (KPIs OS + tempo médio + top clientes/técnicos) — ESSENCIAL P/ MISSÃO 5

### Financial (`backend/src/financial/`)
- `GET /financeiro/dashboard` → `service::dashboard` (DashboardFinanceiro)
- `GET /financeiro/configuracao` → `service::obter_configuracao`
- `PUT /financeiro/configuracao` → `service::atualizar_configuracao`
- `GET /financeiro/plano-contas` → `service::listar_plano_contas`
- `POST /financeiro/plano-contas` → `service::criar_plano_conta` (?)
- `GET /financeiro/centros-custo` → `service::listar_centros_custo`
- `GET /financeiro/contas-receber?status=aberta|paga|cancelada&page=N&limit=M` (PAGINADO)
- `POST /financeiro/contas-receber` (body: ...) 
- `POST /financeiro/contas-receber/{id}/pagar`
- `POST /financeiro/contas-receber/{id}/cancelar`
- `GET /financeiro/contas-pagar?status=...` (PAGINADO)
- `POST /financeiro/contas-pagar` 
- `POST /financeiro/contas-pagar/{id}/pagar`
- `GET /financeiro/fluxo-caixa?de=YYYY-MM-DD&ate=YYYY-MM-DD` → `service::calcular_fluxo`
- `GET /financeiro/alertas?severidade=...` → `service::listar_alertas_ativos`
- `POST /financeiro/alertas/{id}/resolver`

### RBAC (`backend/src/rbac/`)
- `GET /rbac/roles` → `service::listar_roles`
- `GET /rbac/permissions` → `service::listar_permissions`
- `POST /rbac/users/{user_id}/roles/{role_id}`
- `DELETE /rbac/users/{user_id}/roles/{role_id}`
- `GET /rbac/users/{user_id}/roles` → `service::listar_roles_usuario`
- `GET /rbac/users/{user_id}/permissions` → `service::permissoes_de_usuario` (já usada em login)

### Estoque (`backend/src/banco_de_dados/estoque.rs`)
- `POST /estoque/pecas` → `estoque::criar_peca` (já existe)
- `PUT /estoque/pecas/{id}` → `estoque::atualizar_peca` (já existe)
- `GET /estoque/fornecedores` → `estoque::listar_fornecedores` (já existe)
- `GET /estoque/pecas/alerta-minimo` (filtro `WHERE estoque_atual <= estoque_minimo`)

### Usuários (gaps)
- `GET /usuarios/{username}` (detalhes)
- `DELETE /usuarios/{username}` (soft delete)
- `PUT /auth/senha` (trocar a própria senha)
- `POST /auth/senha/reset` (admin reseta de outro)

### Global (auth)
- **`POST /logout`** — invalida JWT (blacklist in-memory)
- **`GET /me`** — devolve claims do cookie HttpOnly (essencial p/ F5)

---

## 5. Endpoints agregados pedidos pela MISSÃO 5 (ainda não existem)

| Endpoint | Models/DTOs envolvidos | Service já existe? | Repository |
|---|---|---|---|
| `GET /dashboard/resumo` | mix | ❌ não | ❌ |
| `GET /dashboard/kpis` | mix | ❌ | ❌ |
| `GET /dashboard/alertas` | `AlertaOcorrencia` | ✅ `operations::listar_alertas_ativos` | ✅ |
| `GET /dashboard/atividade` | `TimelineEvento` + custom | ✅ `crm::listar_timeline` (por cliente) | ✅ |
| `GET /crm/pipeline` | `KanbanColuna[]` | ✅ `operations::kanban` | ✅ |
| **`GET /financeiro/resumo`** | `DashboardFinanceiro` | ✅ `financial::dashboard` | ✅ |
| **`GET /financeiro/fluxo-caixa`** | `FluxoCaixaResultado` | ✅ `financial::calcular_fluxo` | ✅ |
| **`GET /estoque/resumo`** | `(estoque_total, valor_estoque, alerta_minimo, qtd_total)` | ❌ precisa montar | parcial |

**Ação**: adicionar handlers que invoquem esses services. Apenas **`GET /financeiro/resumo` e `/estoque/resumo`** precisam de agregação nova. Demais já têm service.

---

## 6. RBAC — Permissions catalog

Vistas em uso no backend (servem de base p/ RBAC dinâmico da MISSÃO 6):

```
empresa.usuario.list
empresa.usuario.create
empresa.usuario.assign_role
crm.cliente.view
crm.cliente.create
os.view
os.create
os.edit
estoque.peca.view
orcamento.create
orcamento.view
financeiro.conta_receber.pagar
financeiro.*
crm.*
os.*  (cobre os.create/os.edit/os.view)
estoque.*
empresa.*
... + wildcard '*' cobre tudo do módulo
```

SUPER_ADMIN bypassa (`auth.rs:81-83`).

---

## 7. Multi-tenant — modelo

**Padrão**: `tenant_id` em **toda tabela** (migration 0012/0016). Padrão de queries:
```rust
let conn = obter_conexao()?;
conn.exec("SELECT ... FROM X WHERE tenant_id = :tid", params! { "tid" => tenant })
```

**Origem**: `claims.tenant` (token JWT). Fallback `TENANT_LEGACY = 1` quando ausente (compat retroativa).

**Defesa em profundidade (P2.6.2a)**: a função `tenant_do_usuario(claims)` é chamada por **toda** rota protegida.

---

## 8. Auth — pipeline

```
POST /login
  ↓
  rate_limit (60s, login:{ip})
  ↓
  servicos::verificar_login(usuario, senha)
  ↓
  bcrypt::verify → true/false (mesmo tempo pra user inexistente / senha errada — anti-enumeração)
  ↓
  rbac::repository::permissoes_de_usuario(uid, tenant_id) → Vec<String>
  ↓ fallback por papel (login.rs:546-580)
  ↓
  auth::criar_token(uid, papel, tenant, roles, permissions) → JWT (HS256)
  ↓
  Set-Cookie: token=<jwt>; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=86400
  ↓
  {papel, expira_em}  ← body, token=""
```

**Fail-fast**: em release, sem `JWT_SECRET` ou < 32 bytes → panic na inicialização.

---

## 9. Resumo executivo — gaps críticos que travam MISSÕES 1-8

| # | Gap | Impacto | Bloqueia |
|---|---|---|---|
| 1 | `servicos::alterar_papel` é STUB (handler 501) | Painel admin não consegue promover papel | M3 |
| 2 | `POST /logout` não existe | Front não consegue invalidar JWT | A (front) |
| 3 | `GET /me` não existe | Front perde sessão em F5 | A (front) |
| 4 | Sem handlers CRM/operations/financial/RBAC | Páginas CRM/OS/Financeiro/Agenda não têm dados | M3/M5 |
| 5 | Sem `/dashboard/executivo`, `/financeiro/resumo`, `/estoque/resumo` | Dashboard é template | M5 |
| 6 | Sem paginação nos GETs de listagem | Performance ruim com qualquer volume real | M7 |
| 7 | Front fala `localhost:8080`, back roda `3000` | Auth nem chega no servidor | A (front) |
| 8 | Front usa localStorage; deveria usar cookie HttpOnly + `/me` | Risco de vazamento JWT | A (front) |

---

## 10. Onde mexer primeiro (ordem de execução recomendada)

**Sprint 1 — Destrava Auth + Dashboard**
1. `backend/src/servicos.rs` — implementar `alterar_papel(username, novo_papel)`
2. `backend/src/server.rs` — handlers: `POST /logout`, `GET /me`
3. `backend/src/server.rs` — registrar handlers de `GET /dashboard/executivo`, `GET /financeiro/dashboard`, `GET /financeiro/fluxo-caixa`, `GET /estoque/resumo`
4. `backend/src/server.rs` — handlers de CRM (timeline, observações, contatos, equipamentos)

**Sprint 2 — CRUDs completos**
5. `backend/src/server.rs` — handlers OS (POST /os/{id}/mover), CRUDs estoque, agenda
6. `backend/src/server.rs` — handlers financial completo

**Sprint 3 — RBAC dinâmico**
7. `backend/src/server.rs` — handlers RBAC (`/rbac/roles`, `/rbac/permissions`)

**Sprint 4 — Front**
8. Front (passos A → B → Dashboard)
