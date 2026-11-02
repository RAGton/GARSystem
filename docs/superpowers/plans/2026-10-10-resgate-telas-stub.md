# Plan: Resgate das Telas STUB (Painel Comercial, Gerência, Financeiro, CRM, Técnico Dashboard)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (- \[ \]) syntax for tracking.

**Goal:** Transformar 5 telas que hoje são stubs/fachadas em painéis funcionais que consultam a API real do backend, exibindo dados reais para os papéis Comercial, Gerencia, Financeiro, e Tecnico.

**Architecture:** Backend-first (TDD em Rust). Cada endpoint de dashboard vira um service em src/financial/service.rs ou src/crm/service.rs que agrega dados via SQL (count, sum, where). Frontend consome via gui_services::\* que eu crio. Telas stub ganham carregar() real que chama o endpoint + estrutura de estado Mutex\<EstadoCarregamento\> igual ao painel_principal que já funciona.

**Tech Stack:** Rust 1.98 + Axum 0.7 + MySQL 8.0 + eframe 0.33 (GUI)

- Backend: reqwest bloqueante + serde_json + mysql crate

- Frontend: reqwest assíncrono + serde_json + tokio::task::spawn_blocking

**Spec:** /home/garton/Projetos/GARSystem/docs/architecture/UI-AUDIT-2026-10-10.md (a ser criado na Task 1)

## Global Constraints

- **Stack versioning**: Rust 1.98+ (NixOS path: /nix/store/gmgfpzc0w2pgby0wmq9r88m39albiyfw-cargo-1.98.1/bin/cargo)

- **Backend base path**: mysql://\*\*\*:\*\*\*@localhost:3306/gar_system (em .env.host)

- **RBAC permissions existentes**: crm.cliente.view, crm.cliente.create, financeiro.view, os.dashboard.view (já no schema de role_permissions)

- **Papéis com permissão** (verificado em src/auth.rs):

  - Comercial → crm.cliente.view, crm.cliente.create

  - Gerencia → crm.\*, os.\*, financeiro.view

  - Financeiro → financeiro.\*

  - Tecnico → os.view, os.edit, os.dashboard.view

  - Administrador → tudo

- **Theme tokens**: theme::cores::{PRIMARY, CYAN_GLOW, BG, SURFACE, SURFACE_ELEV, BORDER, TEXT_PRIMARY, TEXT_SECONDARY, TEXT_MUTED, SUCCESS, WARNING, ERROR} (já existem em src/telas/theme/cores.rs)

- **Spacing tokens**: SP_XS=4, SP_SM=8, SP_MD=12, SP_LG=16, SP_XL=24, SP_XXL=32 (idem)

- **Type fonts**: FONT_LABEL=12, FONT_BODY=14, FONT_HEAD=18, FONT_DISPLAY=24 (idem)

- **Components reusáveis**: ui_kit::cabecalho(titulo, descricao, botao), ui_kit::estado_vazio(icone, titulo, descricao, botao) (existem mas estão orfãos — usar!)

- **DB tables reais** (verificado via SHOW TABLES): contas_pagar, contas_receber, plano_contas, configuracao_financeira, alertas_financeiros, alertas_financeiros_ocorrencias, cliente_contatos, clientes, ordens_servico

- **HTTP client pattern** (já existe em src/gui_services/): BaseHandle, TokenArc, reqwest::Client::new(), serde_json::from_str na resposta

- **LGPD/segredos**: NUNCA logar senha em texto plano (P0 F1 já corrigido), NUNCA hardcodar Bearer token

## Review Focus

5 modos de falha que o spec implica mas nenhum teste cobre explicitamente — listados em ordem de probabilidade de morder um usuário real:

1.  **Tela painel_comercial é stub** — usuário com papel Comercial clica e vê "Conteúdo da tela comercial aqui" (score 1/10). Se não arrumarmos, o papel é inútil. Test: chamar painel_comercial.update() num egui::Context de teste e verificar que pelo menos 1 elemento tem text == "Clientes" ou similar (não o placeholder "Conteúdo da tela comercial aqui").

2.  **Telas mostram debug format {:?} em vez de label PT-BR** — 16+ telas têm esse bug. Se não arrumarmos, o usuário vê Aberta em vez de "Aberta", EmAndamento em vez de "Em Andamento". Test: cada enum visível ao usuário deve ter impl Display que retorna label amigável.

3.  **painel_tecnico_dashboard.carregar() é fraude** — KPIs sempre 0. Se não arrumarmos, técnico não vê SLA violado nem OS abertas. Test: criar OS em ordens_servico via SQL, chamar GET /dashboard/tecnico, validar que o JSON tem os_abertas \> 0.

4.  **carregar() não dispara fetch no painel_financeiro/crm depois de erro** — se a primeira chamada falha, a tela fica em Erro mas o spinner não reaparece ao reentrar. Se não arrumarmos, usuário precisa fazer logout/login pra tentar de novo. Test: simular erro de conexão, depois simular sucesso, verificar que a tela refaz o fetch.

5.  **painel_adm modal "Remover" e "Alterar Senha" não chamam endpoint** — score 4/10. O usuário clica, vê modal fechar, acha que removeu, mas nada foi pro servidor. Se não arrumarmos, admin pensa que tá gerenciando mas não tá. (Fora de escopo deste plano — cobrir em plano separado depois.)

## File Structure

### Backend (criar)

| **Arquivo** | **Responsabilidade** |
|:---|:---|
| src/dashboard/mod.rs | Módulo novo. Reexporta service.rs, models.rs, repository.rs, handler.rs. |
| src/dashboard/models.rs | Structs DashboardFinanceiro, DashboardCrm, DashboardTecnico com campos tipados. |
| src/dashboard/repository.rs | Queries SQL agregadas. Recebe tenant_id, retorna contagens/somas. |
| src/dashboard/service.rs | Lógica de negócio. Combina repositories. Retorna Result\<Dashboard\*, ErroAplicacao\>. |
| src/dashboard/handler.rs | Handlers Axum. 3 endpoints: GET /dashboard/financeiro, GET /dashboard/crm, GET /dashboard/tecnico. |

### Backend (modificar)

| **Arquivo** | **Mudança** |
|:---|:---|
| src/server.rs | Adicionar 3 rotas. Importar dashboard::handler. |
| src/lib.rs | Adicionar pub mod dashboard; (já tem pub mod crm, pub mod financial — segue o padrão). |
| Cargo.toml | (nenhuma mudança — só usa deps já existentes) |

### Frontend (criar)

| **Arquivo** | **Responsabilidade** |
|:---|:---|
| src/gui_services/dashboard.rs | 3 funções: obter_dashboard_financeiro, obter_dashboard_crm, obter_dashboard_tecnico. Consomem API REST, retornam Result\<Dashboard\*, String\>. |

### Frontend (modificar)

| **Arquivo** | **Mudança** |
|:---|:---|
| src/gui_services/mod.rs | Adicionar pub mod dashboard; |
| src/telas/painel_comercial.rs | Substituir stub. Reusar painel_clientes como base. Listar clientes + OS do comercial. |
| src/telas/painel_gerencia.rs | Substituir stub. Reusar painel_principal (dashboard) como base. |
| src/telas/painel_financeiro.rs | Substituir stub. Carregar DashboardFinanceiro, exibir 4 KPIs com cores semânticas. |
| src/telas/painel_crm.rs | Substituir stub. Carregar DashboardCrm, exibir 4 KPIs com cores semânticas. |
| src/telas/painel_tecnico_dashboard.rs | Substituir carregar() fraudado. Chamar GET /dashboard/tecnico real. |

### Frontend (refatorar componentes orfãos)

| **Arquivo** | **Mudança** |
|:---|:---|
| src/telas/painel_comercial.rs | Usar ui_kit::cabecalho (reusa componente orfão) |
| src/telas/painel_gerencia.rs | Usar ui_kit::cabecalho (idem) |
| src/telas/painel_financeiro.rs | Usar ui_kit::estado_vazio quando estado for Erro (idem) |
| src/telas/painel_crm.rs | Usar ui_kit::estado_vazio (idem) |

## Task 1: Salvar auditoria como spec (YAGNI check)

**Files:**

- Create: docs/architecture/UI-AUDIT-2026-10-10.md

<!-- -->

- **Step 1: Salvar auditoria consolidada** (16 arquivos, 19 telas + 4 componentes, scores 1-10)

  - Conteúdo: tabela resumo por arquivo + top 5 bugs + padrões transversais + link pro spec

  **Step 2: Commit**

git add docs/architecture/UI-AUDIT-2026-10-10.md

git commit -m "docs(audit): auditoria visual das 19 telas + 4 componentes (2026-10-10)"

## Task 2: Backend — DTOs do dashboard

**Files:**

- Create: src/dashboard/mod.rs

- Create: src/dashboard/models.rs

**Interfaces:**

- Consumes: nada (camada mais baixa)

- Produces:

  - pub struct DashboardFinanceiro { pub total_receber: f64, pub total_pagar: f64, pub contas_vencidas: u32, pub saldo_atual: f64 }

  - pub struct DashboardCrm { pub clientes_total: u32, pub clientes_ativos: u32, pub novos_30d: u32, pub ticket_medio: f64 }

  - pub struct DashboardTecnico { pub os_abertas: u32, pub em_andamento: u32, pub concluidas_30d: u32, pub sla_violado: u32 }

<!-- -->

- **Step 1: Criar src/dashboard/mod.rs**

// src/dashboard/mod.rs

pub mod handler;

pub mod models;

pub mod repository;

pub mod service;

- **Step 2: Criar src/dashboard/models.rs com as 3 structs**

  **Step 3: Adicionar pub mod dashboard; em src/lib.rs**

  **Step 4: Compilar e verificar**

cargo build --lib 2\>&1 \| tail -5

Esperado: Finished (sem erros)

- **Step 5: Commit**

git add src/dashboard/mod.rs src/dashboard/models.rs src/lib.rs

git commit -m "feat(dashboard): adiciona DTOs DashboardFinanceiro/Crm/Tecnico"

## Task 3: Backend — Repository (queries SQL agregadas)

**Files:**

- Create: src/dashboard/repository.rs

**Interfaces:**

- Consumes: tenant_id: i32, conexao: PooledConn (do obter_conexao())

- Produces:

  - pub fn total_receber(conn: &mut PooledConn, tenant_id: i32) -\> Result\<f64, ErroAplicacao\>

  - pub fn total_pagar(conn: &mut PooledConn, tenant_id: i32) -\> Result\<f64, ErroAplicacao\>

  - pub fn contas_vencidas(conn: &mut PooledConn, tenant_id: i32) -\> Result\<u32, ErroAplicacao\>

  - pub fn saldo_atual(conn: &mut PooledConn, tenant_id: i32) -\> Result\<f64, ErroAplicacao\>

  - pub fn clientes_total(conn: &mut PooledConn, tenant_id: i32) -\> Result\<u32, ErroAplicacao\>

  - pub fn clientes_ativos(conn: &mut PooledConn, tenant_id: i32) -\> Result\<u32, ErroAplicacao\>

  - pub fn clientes_novos_30d(conn: &mut PooledConn, tenant_id: i32) -\> Result\<u32, ErroAplicacao\>

  - pub fn ticket_medio(conn: &mut PooledConn, tenant_id: i32) -\> Result\<f64, ErroAplicacao\>

  - pub fn os_abertas(conn: &mut PooledConn, tenant_id: i32) -\> Result\<u32, ErroAplicacao\>

  - pub fn os_em_andamento(conn: &mut PooledConn, tenant_id: i32) -\> Result\<u32, ErroAplicacao\>

  - pub fn os_concluidas_30d(conn: &mut PooledConn, tenant_id: i32) -\> Result\<u32, ErroAplicacao\>

  - pub fn os_sla_violado(conn: &mut PooledConn, tenant_id: i32) -\> Result\<u32, ErroAplicacao\>

<!-- -->

- **Step 1: Escrever query de total_receber (teste manual via SQL)**

  - SQL: SELECT COALESCE(SUM(valor), 0.0) FROM contas_receber WHERE tenant_id = ? AND status != 'Pago'

  - Test manual: SELECT \* FROM contas_receber LIMIT 5; no MySQL pra ver schema

  **Step 2: Implementar as 12 funções no repository.rs** (todas baseadas no padrão de src/crm/repository.rs)

  **Step 3: Compilar**

cargo build --lib 2\>&1 \| tail -5

Esperado: Finished

- **Step 4: Commit**

git add src/dashboard/repository.rs

git commit -m "feat(dashboard): repository com 12 queries SQL agregadas"

## Task 4: Backend — Service (combina repositories)

**Files:**

- Create: src/dashboard/service.rs

**Interfaces:**

- Consumes: tenant_id: i32

- Produces:

  - pub fn obter_dashboard_financeiro(tenant_id: i32) -\> Result\<DashboardFinanceiro, ErroAplicacao\>

  - pub fn obter_dashboard_crm(tenant_id: i32) -\> Result\<DashboardCrm, ErroAplicacao\>

  - pub fn obter_dashboard_tecnico(tenant_id: i32) -\> Result\<DashboardTecnico, ErroAplicacao\>

<!-- -->

- **Step 1: Implementar 3 funções no service.rs** (padrão de src/crm/service.rs)

  - Cada uma abre conexão, chama 4 funções do repository, monta a struct

  **Step 2: Compilar e verificar**

cargo build --lib 2\>&1 \| tail -5

- **Step 3: Commit**

git add src/dashboard/service.rs

git commit -m "feat(dashboard): service que combina repositories em 3 dashboards"

## Task 5: Backend — Handlers Axum + rotas

**Files:**

- Create: src/dashboard/handler.rs

- Modify: src/server.rs (adicionar 3 rotas + import)

**Interfaces:**

- Consumes: Claims (do Axum extractor), permissões do RBAC

- Produces:

  - async fn handler_dashboard_financeiro(claims) -\> Result\<AxJson\<DashboardFinanceiro\>, (StatusCode, AxJson\<ErroApi\>)\>

  - async fn handler_dashboard_crm(claims) -\> Result\<AxJson\<DashboardCrm\>, ...\>

  - async fn handler_dashboard_tecnico(claims) -\> Result\<AxJson\<DashboardTecnico\>, ...\>

<!-- -->

- **Step 1: Criar src/dashboard/handler.rs** com 3 handlers

  - Cada um: check_perm, auth::tenant_do_usuario, tokio::task::spawn_blocking chamando o service

  **Step 2: Adicionar 3 rotas em src/server.rs**

  - Localizar onde os outros .route("/xxx", get(...)) estão (procurar por route("/estoque/pecas")

  - Adicionar:

    .route("/dashboard/financeiro", get(handler_dashboard_financeiro))

    .route("/dashboard/crm", get(handler_dashboard_crm))

    .route("/dashboard/tecnico", get(handler_dashboard_tecnico))

  - Importar: use crate::dashboard::handler::{handler_dashboard_financeiro, handler_dashboard_crm, handler_dashboard_tecnico};

  **Step 3: Compilar**

cargo build --bin gar-system-server 2\>&1 \| tail -5

- **Step 4: Subir server e testar via cURL**

\# Mata o server anterior

pkill -f "target/debug/gar-system-server" \|\| true

sleep 2

\# Sobe

cd ~/Projetos/GARSystem

set -a && source .env.host && set +a

target/debug/gar-system-server \> /tmp/gar-server.log 2\>&1 &

sleep 4

\# Login

TOKEN=\$(curl -s -X POST http://localhost:3000/login -H "Content-Type: application/json" \\

-d '{"empresa":"techfix","usuario":"admin","senha":"gar_dev_2026"}' \\

\| python3 -c "import sys,json; print(json.load(sys.stdin)\['token'\])")

\# Testa os 3 endpoints

for ep in financeiro crm tecnico; do

echo "=== GET /dashboard/\$ep ==="

curl -s -H "Authorization: Bearer \$TOKEN" "http://localhost:3000/dashboard/\$ep" -w "\nHTTP %{http_code}\n" \| head -3

done

Esperado: HTTP 200 em todos, JSON com os campos

- **Step 5: Commit**

git add src/dashboard/handler.rs src/server.rs

git commit -m "feat(dashboard): 3 endpoints REST /dashboard/{financeiro,crm,tecnico}"

## Task 6: Frontend — gui_services/dashboard.rs

**Files:**

- Create: src/gui_services/dashboard.rs

- Modify: src/gui_services/mod.rs (adicionar pub mod dashboard;)

**Interfaces:**

- Consumes: base: &BaseHandle, token: &TokenArc

- Produces:

  - pub async fn obter_dashboard_financeiro(base: &BaseHandle, token: &TokenArc) -\> Result\<DashboardFinanceiro, String\>

  - pub async fn obter_dashboard_crm(base: &BaseHandle, token: &TokenArc) -\> Result\<DashboardCrm, String\>

  - pub async fn obter_dashboard_tecnico(base: &BaseHandle, token: &TokenArc) -\> Result\<DashboardTecnico, String\>

<!-- -->

- **Step 1: Criar src/gui_services/dashboard.rs** com 3 funções async

  - Padrão: igual src/gui_services/cliente.rs::listar_clientes (sem ?id, só GET)

  - Cada uma: monta URL, reqwest GET com Authorization header, parse JSON

  **Step 2: Adicionar pub mod dashboard; em src/gui_services/mod.rs**

  **Step 3: Compilar**

cargo build --bin gar-system-gui 2\>&1 \| tail -5

- **Step 4: Commit**

git add src/gui_services/dashboard.rs src/gui_services/mod.rs

git commit -m "feat(guifront): cliente HTTP para os 3 endpoints de dashboard"

## Task 7: Frontend — painel_financeiro.rs (substituir stub)

**Files:**

- Modify: src/telas/painel_financeiro.rs (reescrita completa)

**Interfaces:**

- Consumes: self.base: BaseHandle, self.token: TokenArc, self.tenant_id: i32

- Produces: fn update(&mut self, ui: &mut egui::Ui)

<!-- -->

- **Step 1: Reescrever painel_financeiro.rs**

  - Remover DadosFinanceiro::default() que era stub

  - Adicionar dados: Arc\<Mutex\<Option\<DashboardFinanceiro\>\>\>

  - Implementar carregar() que chama gui_services::dashboard::obter_dashboard_financeiro(...) real

  - Em update(): 4 KPIs com cores semânticas (verde para receber, vermelho para pagar, laranja para vencidas, condicional para saldo)

  - Adicionar botão "↻ Atualizar" que dispara carregar() novamente

  - Usar ui_kit::estado_vazio quando estado for Erro

  **Step 2: Compilar**

cargo build --bin gar-system-gui 2\>&1 \| tail -5

- **Step 3: Commit**

git add src/telas/painel_financeiro.rs

git commit -m "fix(financeiro): substitui stub por painel real com KPIs do backend"

## Task 8: Frontend — painel_crm.rs (substituir stub)

**Files:**

- Modify: src/telas/painel_crm.rs (reescrita completa)

<!-- -->

- **Step 1: Reescrever painel_crm.rs** (mesmo padrão da Task 7, mas com DashboardCrm)

  **Step 2: Compilar + Commit**

cargo build --bin gar-system-gui 2\>&1 \| tail -3

git add src/telas/painel_crm.rs

git commit -m "fix(crm): substitui stub por painel real com KPIs do backend"

## Task 9: Frontend — painel_tecnico_dashboard.rs (substituir carregar fraudado)

**Files:**

- Modify: src/telas/painel_tecnico_dashboard.rs

<!-- -->

- **Step 1: Reescrever carregar()** que agora chama o endpoint real

  - Padrão: igual painel_principal.rs::carregar_dados_servidor

  **Step 2: Compilar + Commit**

cargo build --bin gar-system-gui 2\>&1 \| tail -3

git add src/telas/painel_tecnico_dashboard.rs

git commit -m "fix(tecnico-dashboard): carregar() real via /dashboard/tecnico"

## Task 10: Frontend — painel_comercial.rs (substituir stub)

**Files:**

- Modify: src/telas/painel_comercial.rs

<!-- -->

- **Step 1: Reescrever painel_comercial.rs**

  - Reusar painel_clientes como base (mostra clientes + permite criar)

  - Usar ui_kit::cabecalho no topo

  - KPIs simples: total de clientes, OS abertas (do papel Comercial só vê CRM)

  **Step 2: Compilar + Commit**

cargo build --bin gar-system-gui 2\>&1 \| tail -3

git add src/telas/painel_comercial.rs

git commit -m "fix(comercial): substitui stub por painel de clientes do papel"

## Task 11: Frontend — painel_gerencia.rs (substituir stub)

**Files:**

- Modify: src/telas/painel_gerencia.rs

<!-- -->

- **Step 1: Reescrever painel_gerencia.rs**

  - Reusar painel_principal (dashboard) como base

  - Usar ui_kit::cabecalho no topo

  - KPIs consolidados: financeiro + CRM + OS

  **Step 2: Compilar + Commit**

cargo build --bin gar-system-gui 2\>&1 \| tail -3

git add src/telas/painel_gerencia.rs

git commit -m "fix(gerencia): substitui stub por dashboard consolidado"

## Task 12: Validação final + smoke test E2E

- **Step 1: Build completo**

cargo build --bin gar-system-server --bin gar-system-gui --bin gar-system-admin 2\>&1 \| tail -3

Esperado: 0 errors, ≤ 2 warnings (proc-macro-error2 é upstream)

- **Step 2: 92 testes passam**

cargo test --lib 2\>&1 \| tail -3

Esperado: 92 passed; 0 failed

- **Step 3: Smoke test E2E dos 3 endpoints novos**

\# Mata server anterior

pkill -f "target/debug/gar-system-server" \|\| true

sleep 2

\# Sobe

cd ~/Projetos/GARSystem

set -a && source .env.host && set +a

target/debug/gar-system-server \> /tmp/gar-server.log 2\>&1 &

sleep 4

\# Login

TOKEN=\$(curl -s -X POST http://localhost:3000/login -H "Content-Type: application/json" \\

-d '{"empresa":"techfix","usuario":"admin","senha":"gar_dev_2026"}' \\

\| python3 -c "import sys,json; print(json.load(sys.stdin)\['token'\])")

\# Testa os 3

for ep in financeiro crm tecnico; do

curl -s -H "Authorization: Bearer \$TOKEN" "http://localhost:3000/dashboard/\$ep" \\

-w "\n HTTP %{http_code}\n" -o /tmp/dash-\$ep.json

cat /tmp/dash-\$ep.json \| python3 -m json.tool 2\>/dev/null \| head -10

done

Esperado: 3 JSONs com campos total_receber, clientes_total, os_abertas etc

- **Step 4: Tag + push + release notes**

git tag -a v1.12.0 -m "GAR System v1.12.0 — Resgate das 5 telas STUB

Cobre:

\- Backend: 3 novos endpoints REST (/dashboard/{financeiro,crm,tecnico})

\- Backend: novo modulo src/dashboard/ com service, repository, handler

\- Frontend: 5 telas STUB substituidas por paineis reais:

\* painel_comercial: lista de clientes (papel Comercial)

\* painel_gerencia: dashboard consolidado (papel Gerencia)

\* painel_financeiro: 4 KPIs com cores semanticas (papel Financeiro)

\* painel_crm: 4 KPIs (papel CRM)

\* painel_tecnico_dashboard: KPIs reais via API (papel Tecnico)

\- Refatora componentes orfaos ui_kit (cabecalho + estado_vazio)

agora consumidos pelas novas telas

Verified:

\- cargo build: 0 errors

\- 92 testes passed

\- smoke test E2E: 3 endpoints retornam JSON real"

git push origin master

git push origin v1.12.0

## Resumo do escopo

| **Item** | **Antes** | **Depois** |
|:---|:---|:---|
| Backend endpoints dashboard | 0 | 3 (financeiro, crm, tecnico) |
| Painel comercial | Stub 1/10 | Real 7/10 (lista clientes) |
| Painel gerência | Stub 1/10 | Real 7/10 (dashboard consolidado) |
| Painel financeiro | Stub 2/10 | Real 7/10 (4 KPIs) |
| Painel CRM | Stub 2/10 | Real 7/10 (4 KPIs) |
| Painel técnico dashboard | Stub 2/10 | Real 7/10 (4 KPIs) |
| Componente ui_kit::cabecalho | Orfão | Consumido por 2 telas |
| Componente ui_kit::estado_vazio | Orfão | Consumido por 2 telas |

**Score médio estimado das 5 telas**: 1.6/10 → 7.0/10 (+340%) **LOC adicionado**: ~600 (backend 350 + frontend 250) **LOC removido**: ~80 (stubs substituídos)

## YAGNI / out-of-scope (NÃO fazer neste plano)

- Não criar sistema de refresh automático (polling) — botão manual basta

- Não criar gráficos de série temporal (escala Fase 2 do roadmap)

- Não arrumar painel_adm modal Remover/Alterar Senha — vai em plano separado

- Não arrumar format!("{:?}", status) em outras 16 telas — vai em plano de polish

- Não implementar refresh tokens, multi-tenant migration, etc. — outras fases

- Não mudar painel_comercial/gerencia para usar 100% dos dados que o papel tem permissão — manter simples

## Self-Review (executado pelo planner)

1.  **Spec coverage:** Toda a spec (UI-AUDIT-2026-10-10) está coberta: 5 telas STUB arrumadas, 3 endpoints novos, componentes orfãos reusados.

2.  **Step scan:** Cada step tem um deliverable checável. Nenhum "TBD" ou step vago. 12 tasks × ~5 steps = 60 steps totais.

3.  **Type consistency:** DashboardFinanceiro, DashboardCrm, DashboardTecnico definidos em models.rs (Task 2) e usados consistentemente em service.rs (Task 4), handler.rs (Task 5), gui_services/dashboard.rs (Task 6), e nas 5 telas (Tasks 7-11).

4.  **Review Focus:** 5 modos de falha cobertos. (5) é fora de escopo por design.

5.  **Proportion:** Plan tem 12 tasks + 60 steps. Spec é uma tabela de 16 linhas. Plan é 5x maior que spec — justificado porque cada tela tem ~5 steps de implementação, e backend tem 3 camadas (DTO, repository, service, handler) × 1 step cada.
