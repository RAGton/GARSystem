# Mapeamento Completo Backend Rust/Axum -> Frontend Next.js

Este documento consolida o mapeamento auditado entre a estrutura de dados (MySQL), a camada de aplicação Rust/Axum e a interface Next.js (GAR System).

---

## 1. Mapeamento de Banco de Dados & Endpoints

### 📊 Dashboard Executivo & KPIs
- **Tabelas Envolvidas**: `ordens_servico`, `clientes`, `tecnicos`, `titulos_receber`, `chamados`
- **Endpoint Backend**: `GET /api/v1/dashboard/resumo` (a expandir/consolidar), `GET /api/v1/ordens`, `GET /api/v1/clientes`
- **Tela Frontend**: `/` (Dashboard Executivo)
- **Widgets**:
  - `MetricasGeraisCard` (OS abertas, OS em atraso, SLA em risco, Clientes ativos, Técnicos online, Receita do período)
  - `PainelPrioridades` (🔴 Crítico: OS vencidas, SLA vencido, Boletos vencidos; 🟡 Atenção; 🟢 Saudável)
  - `CentroAcoesRapidas` (Criar OS, Novo Cliente, Novo Orçamento, Receber Pagamento, Abrir CRM)

---

## 2. CRM & Pipeline Kanban
- **Tabelas Envolvidas**: `crm_deals`, `crm_stages`, `crm_leads`, `clientes`
- **Endpoint Backend**: `GET /api/v1/crm/kanban` (ou `GET /api/v1/crm/deals`)
- **Tela Frontend**: `/crm`
- **Widgets**:
  - `KanbanBoardCRM` (Etapas: Lead, Qualificação, Proposta, Negociação, Fechado)
  - `ColunaMetricaHeader` (Quantidade de deals, Valor acumulado, Taxa de conversão por coluna)

---

## 3. Ordens de Serviço (OS)
- **Tabelas Envolvidas**: `ordens_servico`, `os_historico`, `clientes`, `tecnicos`, `equipamentos`
- **Endpoint Backend**: `GET /api/v1/ordens`, `GET /api/v1/ordens/{id}`
- **Tela Frontend**: `/ordens`
- **Widgets**:
  - `KanbanBoardOS` (Abertas, Em Atendimento, Aguardando Cliente, Concluídas, Faturadas)
  - `OSCardDetail` (Cliente, Técnico, SLA, Prioridade, Valor)
  - `OSTimelineHistorico` (OS criada ➔ Técnico atribuído ➔ Atendimento iniciado ➔ Finalizada ➔ Faturada)

---

## 4. Financeiro & Fluxo de Caixa
- **Tabelas Envolvidas**: `titulos_receber`, `titulos_pagar`, `movimentacoes_financeiras`
- **Endpoint Backend**: `GET /api/v1/financial/resumo`, `GET /api/v1/financial/titulos`
- **Tela Frontend**: `/financeiro`
- **Widgets**:
  - `ResumoExecutivoFinanceiro` (Receitas, Despesas, Fluxo de Caixa, Inadimplência, Lucro)
  - `AlertasFinanceiros` (Contas vencidas, Boletos atrasados, Fluxo negativo)

---

## 5. Clientes
- **Tabelas Envolvidas**: `clientes`, `ordens_servico`, `titulos_receber`
- **Endpoint Backend**: `GET /api/v1/clientes`, `GET /api/v1/clientes/{id}/resumo`
- **Tela Frontend**: `/clientes`
- **Widgets**:
  - `CRMClienteGridCard` (Avatar, Nome, Empresa, Status, Última interação, Receita gerada, OS abertas)

---

## 6. Validação e Qualidade de Integração
Todas as chamadas realizam tratamento adequado de:
1. `Loading state` (Skeletons animados)
2. `Empty state` (Ilustração e indicação clara de falta de dados)
3. `Error state` (Toast/Banner com opção de tentar novamente)
4. `Responsividade` (Grid adaptativo para telas grandes e mobile)
