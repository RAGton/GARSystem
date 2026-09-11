# FRONTEND_AUDIT — Auditoria completa do Next.js

**Gerado por**: Hermes Enterprise Orchestrator
**Versão front**: Next 16.3.4 + React 19 + TS + Tailwind v4 + shadcn + Zustand 5
**Foco**: descobrir mocks, botões sem ação, rotas quebradas, dados hardcoded.

---

## 1. Camadas

| Camada | Arquivo | Status |
|---|---|---|
| API client | `src/lib/api.ts` | ⚠️ **bug**: fala `localhost:3000/api/v1` (back roda em `3000` sem `/api/v1`); usa `localStorage.getItem('token')` (anti-padrão — precisa cookie HttpOnly) |
| Auth service | `src/services/auth.service.ts` | ❌ Devolve `{papel, expira_em, token}` mas em modo demo-inseguro: quando back offline, aceita qualquer usuário |
| Auth store | `src/stores/auth.store.ts` | ⚠️ Guarda `user.papel` em plain `localStorage` (chave `auth-storage`) |
| Components UI | `src/components/ui/*` | ✅ shadcn funcional |
| Layout | `src/components/layout/*` | ✅ Sidebar + Topbar + Workspace-loader |
| Dashboard widgets | `src/components/dashboard/*` | ⚠️ Mix real+mock (KpiCards usa API, outros hardcoded) |

---

## 2. Páginas e seus problemas

### `/login` — `src/app/login/page.tsx`
- ✅ UI premium (60/40 split, mesh animado)
- ✅ Submit chama `authService.login`
- ❌ Back URL hardcoded errado (`localhost:8080`, back roda `3000`)
- ❌ Fallback **demo-inseguro** quando backend offline (qualquer user/senha vira `papel=admin` ou `usuario`)
- ❌ Sidebar após login não é RBAC-aware — todo usuário vê todos os menus
- **Bloqueios**: A (auth real)

### `/` — `src/app/(dashboard)/page.tsx` (Dashboard executivo)
- ❌ **Cards hardcoded** (`12 OS abertas / 3 SLA críticos / R$ 42.380`)
- ❌ `<KpiCards />` puxa real, `<TrendsChart />` é mockado, `<AlertsSection />` é mock, `<OperationalPerformance />` é 100% mock, `<RecentTimeline />` é 100% mock
- ❌ Não há 3 blocos semânticos ("O que aconteceu / O que exige atenção / O que faço agora")
- **Bloqueio**: M5 (endpoints agregados)

### `/os` (Ordens) — `(dashboard)/os/page.tsx` + `components/os/os-kanban-board.tsx`
- ⚠️ Chama `/ordens` (real) mas mapeia campos errados — backend `cliente:String`, frontend lê `os.cliente_nome` → cai no fallback "Cliente Sem Nome"
- ❌ Colunas (`Abertas / Em Atendimento / Aguardando Cliente / Concluídas / Faturadas`) não batem com o enum do backend (`Aberta / Orcamento / Aprovada / EmAndamento / AguardandoPeca / Finalizada / Cancelada`)
- ❌ StatusOS ≠ slug que frontend usa (a coluna "concluida" só pega se `os.status === "concluida"`)
- ❌ Drag-and-drop **não persiste** — `onDragEnd` só muda o state local
- ❌ Botão "Nova OS" sem `onClick`
- ❌ Busca + filter sem ação
- **Bloqueio**: B (services) + endpoint `POST /os/{id}/mover`

### `/clientes` (CRM Kanban) — `(dashboard)/clientes/page.tsx` + `components/crm/kanban-board.tsx`
- ❌ Chama `/crm/deals` que **não existe no back** → kanban sempre vazio
- ❌ Campos mapeados (`score, probability, value, contact`) **inválidos** — model do back é `Cliente` simples, não lead
- ❌ Drag-and-drop sem persistência
- ❌ Botão "Nova Oportunidade" sem `onClick`
- ❌ Não há tela de listagem de clientes (CRM clássico — kanban aqui é um **mal encaixe**)
- **Bloqueio**: reescrever a página pra usar `DashboardCliente` + endpoints reais de CRM (ver BACKEND_MAPPING §4)

### `/crm` — alias → redirect `/clientes`
- ✅ Alias ok (criado no passo 2)

### `/servicos` — alias → redirect `/os`
- ✅ Alias ok

### `/estoque` — `(dashboard)/estoque/page.tsx`
- ❌ **Array `estoqueItens` 100% hardcoded** (5 peças fictícias)
- ❌ Botão "Novo Produto" sem `onClick`
- ❌ KPI cards (192 / R$ 184.220,00) hardcoded
- ❌ Sem integração com `GET /estoque/pecas` (que já existe no back!)
- ❌ Sem paginação, sem ordenar por mínimo
- **Bloqueio**: simples — bind com `/estoque/pecas` no passo B

### `/financeiro` — `(dashboard)/financeiro/page.tsx` + `components/financeiro/*`
- ❌ **3 componentes 100% mockados**:
  - `FinanceKpiCards`: `R$ 124.563,00 / R$ 42.150,00 / R$ 82.413,00 / 2,4%`
  - `CashFlowChart`: array `receita/despesas` jan-jul hardcoded
  - `ProfitChart`: array `lucro` jan-jul hardcoded
- ❌ Botão "Exportar DRE" sem ação
- ❌ Botão "Novo Lançamento" sem ação
- ❌ Sem integração com back (tem `service::dashboard` e `service::calcular_fluxo` prontos!)
- **Bloqueio**: implementar handlers `GET /financeiro/dashboard` + `/financeiro/fluxo-caixa` (M5)

### `/orcamentos` — `(dashboard)/orcamentos/page.tsx` (placeholders criados no passo 2)
- ❌ Página placeholder ("Módulo em construção")
- ❌ Sem integração com `POST /orcamentos` (existe) nem `GET /orcamentos` (pendente)
- **Bloqueio**: M5 (`GET /orcamentos`)

### `/relatorios` — `(dashboard)/relatorios/page.tsx`
- ❌ Array `relatoriosDisponiveis` 100% hardcoded
- ❌ Botões "Filtrar Período" e "Exportar Tudo (ZIP)" sem ação
- ❌ Botões Excel/PDF por card sem ação
- **Bloqueio**: baixa prioridade — pode virar filtros + gerar PDF do que tem

### `/admin` — `(dashboard)/admin/page.tsx`
- ❌ Array `usuariosCadastrados` 100% hardcoded (4 users fictícios)
- ❌ Card "Segurança & Conformidade" mostra **"ISO 27001"** (falso!)
- ❌ Card "Políticas de Acesso (RBAC)" mostra "4 papéis" hardcoded
- ❌ Botão "Convidar Usuário" sem `onClick`
- ❌ Botão "Editar" na tabela sem `onClick`
- ❌ Sem integração com `/usuarios`
- **Bloqueio**: implementar `GET /usuarios` ligando (já existe) + implementar `servicos::alterar_papel`

### `/(dashboard)/dashboard/page.tsx`
- ✅ redirect('/') — redundante mas ok

### Components soltos

#### `<KpiCards>` — `components/dashboard/kpi-cards.tsx`
- ✅ Já consome `/ordens` e `/clientes`
- ❌ SLA hardcoded `100%` (com comentário "Calculado a partir das datas reais no backend" — mentira)
- ❌ Filtro de status errado: backend retorna `Finalizada/Faturada` mas enum é `Finalizada/Cancelada/Aprovada/...`

#### `<TrendsChart>` — `components/dashboard/trends-chart.tsx`
- ❌ 100% hardcoded (Jan-Jul receita/os)

#### `<AlertsSection>` — `components/dashboard/alerts-section.tsx`
- ❌ 100% hardcoded (3 alerts inventados)
- ❌ Sem integração com `/alertas` (que existe no back via `operations::service::listar_alertas_ativos`)

#### `<OperationalPerformance>` — `components/dashboard/operational-performance.tsx`
- ❌ 4 cards 100% hardcoded (`2.4h / 87% / 92% / 4.8/5`)

#### `<RecentTimeline>` — `components/dashboard/recent-timeline.tsx`
- ❌ 100% hardcoded (3 items fictícios)

---

## 3. Resumo por número

| Categoria | Quantidade | Onde |
|---|---|---|
| Botões sem `onClick` / sem ação | **15+** | login submit, sidebar logout, criar cliente/OS/oportunidade, export PDF/Excel, convidar, editar admin, filter, search... |
| Listas hardcoded (mock) | **5** | `estoqueItens`, `usuariosCadastrados`, `relatoriosDisponiveis`, `CashFlowChart.data`, `ProfitChart.data`, `TrendsChart.data`, `AlertsSection.alerts`, `OperationalPerformance` |
| Componentes dashboard sem fetch | **4** | TrendsChart, AlertsSection, OperationalPerformance, RecentTimeline |
| Componentes dashboard com fetch parcial | **2** | KpiCards (filtro errado), KpiCardsCards (mapa de status errado) |
| Páginas placeholder | **1** | `/orcamentos` |
| Mocks inventados absurdos | **3** | "ISO 27001" (falso), "JWT Argon2id" (falso — é HS256/BCrypt), "Taxa de Solução 87%" (zero lastro) |
| Endpoint chamado e não existe no back | **1** | `/crm/deals` (kanban) → sempre vazio |
| Endpoint existente mas não consumido | **2** | `/estoque/pecas`, `/ordens` (consumido parcialmente) |
| Endpoints com dados para alimentar dashboard mas não usados | **3** | `/alertas`, `/os/{id}/sla`, `dashboard_executivo` |

---

## 4. Plano de correção (ordem de execução)

### Sprint A — Auth correta (1 dia)
- Reescrever `lib/api.ts`: usar cookie + base URL via env; remover localStorage
- Reescrever `auth.service.ts`: chamar `/login`, setar cookie via response
- Reescrever `auth.store.ts`: guardar `{uid, username, papel, tenant, permissions, expira_em}` (NUNCA `jwt`)
- Adicionar `lib/api.requireAuth()` que redireciona pra `/login` se 401
- Adicionar `/me` interceptor para refresh on mount

### Sprint B — services/ + bindings (2-3 dias)
- Criar `services/api.ts` (wrapper de fetchApi com error handling + retry)
- Criar `services/clientes.service.ts` (consumindo `/clientes`, `/clientes/{id}/resumo`, e quando back tiver: `/crm/clientes/{id}/dashboard`)
- Criar `services/os.service.ts` (consumindo `/ordens`, `/ordens/{id}`, ligando arrastar↔`POST /os/{id}/mover`)
- Criar `services/estoque.service.ts` (consumindo `/estoque/pecas`, future `/estoque/pecas/alerta-minimo`)
- Criar `services/financeiro.service.ts` (consumindo `/financeiro/dashboard`, `/financeiro/fluxo-caixa` quando back expor)
- Criar `services/dashboard.service.ts` (consumindo `/dashboard/executivo` quando back expor)
- Criar `services/auth.service.ts` reescrito + `services/permissions.service.ts` (lê `permissions[]` do `/me`)
- Adicionar `services/types.ts` com tipos TS gerados do openapi.json (futuro: `openapi-typescript`)

### Sprint C — Remover TODOS os mocks (2 dias)
Substituir cada componente mockado por versão que consome service:

| Componente | Mock atual | Substituir por |
|---|---|---|
| `page.tsx` (dashboard) | cards hardcoded + mix | `<DashboardHome />` puxando `/dashboard/executivo` + `/alertas` + `/financeiro/dashboard` |
| `KpiCards` | filtro status errado | consumir `/dashboard/kpis` |
| `TrendsChart` | array hardcoded | consumir `/financeiro/fluxo-caixa` |
| `AlertsSection` | array hardcoded | consumir `/alertas` |
| `OperationalPerformance` | 4 cards hardcoded | consumir `/dashboard/executivo.top_clientes/tecnicos/tempo_medio_resolucao_horas` |
| `RecentTimeline` | array hardcoded | consumir `/crm/clientes/recentes?empresa_id=...` OU `/dashboard/atividade` |
| `estoque/page.tsx` | array hardcoded | consumir `/estoque/pecas` |
| `financeiro/page.tsx` | 3 mocks | consumir `/financeiro/dashboard` + `/financeiro/fluxo-caixa` |
| `admin/page.tsx` | array + cards mentirosos | consumir `/usuarios` |
| `relatorios/page.tsx` | array + export sem ação | virar launcher de relatórios reais (PDF/XLSX de fontes) |

### Sprint D — CRUDs completos (2 dias)

| Página | Botão | Ação |
|---|---|---|
| `/clientes` | Novo | Modal `ClienteForm` → `POST /clientes` |
| `/clientes` | Editar | Modal `ClienteForm` → `PUT /clientes/{id}` (criar endpoint se faltar) |
| `/clientes` | Excluir | `DELETE /clientes/{id}` |
| `/clientes` | Visualizar | Drawer com `DashboardCliente` |
| `/os` | Nova | `POST /ordens` |
| `/os` | Editar | Form OS + atualizar lista |
| `/os` | Mover | `POST /os/{id}/mover` (kanban drag) |
| `/os` | Finalizar | usa mover |
| `/os` | Cancelar | usa mover + motivo |
| `/estoque` | Novo | `POST /estoque/pecas` |
| `/estoque` | Editar | `PUT /estoque/pecas/{id}` |
| `/estoque` | Entrada/Saída | Form de movimentação |
| `/orcamentos` | Novo | `POST /orcamentos` |
| `/orcamentos` | Listar | `GET /orcamentos?page=N` |
| `/financeiro` | Receber/Pagar | `POST /financeiro/contas-receber/{id}/pagar` |
| `/admin` | Convidar | `POST /usuarios` |
| `/admin` | Editar papel | `PUT /usuarios/{u}/papel` (precisa back) |

### Sprint E — RBAC dinâmico (1 dia)
- `auth.store` armazena `permissions[]`
- `services/permissions.service.ts`: `hasPermission(p)`, `hasAnyPermission(ps)`
- Sidebar consome `hasPermission` para esconder menus
- Botões checam `hasPermission` para esconder/desabilitar

### Sprint F — Tema GAR Enterprise (1 dia)
- Tokens CSS para `#0A0F1E / #111827 / #1E293B / #38BDF8 / #06B6D4`
- Componentes UI (Button, Card, Input) usando tokens
- Remover glassmorphism inconsistente
- Polir layout pages enterprise-like

---

## 5. Ações imediatas (não bloqueadas)

As seguintes mudanças podem ser feitas AGORA sem back novo:

1. Reescrever `lib/api.ts` para enviar `credentials: "include"` (cookie HttpOnly)
2. Trocar base URL de `localhost:8080/api/v1` para `localhost:3000`
3. Bind `estoque/page.tsx` ao `GET /estoque/pecas` que JÁ EXISTE (delete `estoqueItens` array)
4. Bind `admin/page.tsx` ao `GET /usuarios` que JÁ EXISTE (delete `usuariosCadastrados` array)
5. Bind `KpiCards` ao `GET /dashboard/executivo` ou refazer agregação client-side com dados de `/ordens`+`/clientes`
6. Remover cards hardcoded de `/(dashboard)/page.tsx`
7. Corrigir campos do `os-kanban-board.tsx` (`cliente` em vez de `cliente_nome`)
8. Corrigir `auth.service.ts` modo demo (remover fallback que aceita qualquer login)

---

## 6. Tabelas de impacto (DASHBOARD)

| Card | Hoje | Origem real |
|---|---|---|
| Receita Faturada | `/ordens` filtro manual (status errado) | `GET /financeiro/dashboard.recebido_no_mes` |
| Clientes Cadastrados | `/clientes` array length | `GET /clientes.length` ou `clientesTotal em /dashboard/executivo` |
| OS Em Aberto | filtro errado (status Finalizada não bate) | `GET /dashboard/executivo.os_abertas` ou `/os/kanban` |
| SLA de Atendimento | `100%` hardcoded | `tempo_medio_resolucao_horas` em `/dashboard/executivo` |
| Receita Mês | R$ 42.380 hardcoded | `GET /financeiro/dashboard.recebido_no_mes` |
| Alertas | 3 inventados | `GET /alertas?severidade=critical&entidade=os` |
| Performance da Equipe | 4 valores hardcoded | `GET /dashboard/executivo.tecnicos[]` |
| Trends | array jan-jul | `GET /financeiro/fluxo-caixa?de=...&ate=...` agrupado por mês |
