// src/services/dashboard.service.ts
// KPIs e agregados do Dashboard Executivo.
//
// Todos os endpoints são PENDENTES (BACKEND_MAPPING §4 — operations/service.rs).
// Best-effort: se o endpoint não existir (404), cada função retorna null/vazio.

import { fetchApi } from './api'

// === Tipos — alinhados com operations/service.rs::dashboard_executivo ===

export interface DashboardExecutivo {
  os_abertas: number
  os_em_andamento: number
  os_finalizadas_mes: number
  os_canceladas_mes: number
  clientes_total: number
  clientes_novos_mes: number
  receita_mes: number
  despesa_mes: number
  saldo: number
  sla_cumprimento_percent: number
  tempo_medio_resolucao_horas: number
  top_clientes: Array<{
    cliente_id: number
    cliente_nome: string
    total_gasto: number
    os_count: number
  }>
  tecnicos: Array<{
    tecnico_id: number
    nome: string
    os_concluidas: number
    tempo_medio_horas: number
    ocupacao_percent: number
  }>
  variacao_receita_mes_anterior_percent?: number
}

export interface DashboardAlerta {
  id: number
  severidade: 'info' | 'warning' | 'critical'
  mensagem: string
  contexto?: Record<string, unknown>
  created_at: string
  entidade: 'os' | 'cliente' | 'financeiro' | 'estoque' | 'sla' | string
}

export interface DashboardKanbanColuna {
  status: string
  titulo: string
  ordem: number
  ordens: Array<{
    id: number
    cliente: string
    titulo: string
    tecnico: string
    sla_status: 'on_track' | 'at_risk' | 'breached'
    prazo: string
  }>
}

export interface SlaOS {
  os_id: number
  percentual_cumprido: number
  eventos: Array<{
    tipo: 'RECEBIDO' | 'DIAGNOSTICO_INICIO' | 'DIAGNOSTICO_FIM' | 'SERVICO_INICIO' | 'SERVICO_FIM' | 'PRONTO' | 'ENTREGUE'
    timestamp: string
    dentro_sla: boolean
  }>
  status_atual: 'on_track' | 'at_risk' | 'breached'
}

// === Service ===

export interface AlertasParams {
  severidade?: DashboardAlerta['severidade']
  entidade?: DashboardAlerta['entidade']
}

export const dashboardService = {
  /** GET /dashboard/executivo — KPIs agregados. */
  async dashboardExecutivo(): Promise<DashboardExecutivo | null> {
    try {
      return await fetchApi<DashboardExecutivo>('/dashboard/executivo')
    } catch {
      return null
    }
  },

  /** GET /alertas?severidade=...&entidade=... (operations). */
  async alertas(params: AlertasParams = {}): Promise<DashboardAlerta[]> {
    try {
      const data = await fetchApi<DashboardAlerta[] | { items: DashboardAlerta[] }>(
        '/alertas',
        { query: { severidade: params.severidade, entidade: params.entidade } },
      )
      if (Array.isArray(data)) return data
      if (data && Array.isArray(data.items)) return data.items
      return []
    } catch {
      return []
    }
  },

  /** GET /os/kanban (state machine agrupada por coluna). */
  async kanban(): Promise<DashboardKanbanColuna[]> {
    try {
      const data = await fetchApi<DashboardKanbanColuna[]>('/os/kanban')
      return Array.isArray(data) ? data : []
    } catch {
      return []
    }
  },

  /** GET /os/{id}/sla. */
  async slaOS(id: number): Promise<SlaOS | null> {
    try {
      return await fetchApi<SlaOS>(`/os/${id}/sla`)
    } catch {
      return null
    }
  },

  /**
   * GET /financeiro/fluxo-caixa?de=...&ate=... (re-export para conveniência no dashboard).
   * Mantido aqui para evitar import circular entre financeiro e dashboard.
   */
  async fluxoCaixa(de: string, ate: string) {
    try {
      return await fetchApi('/financeiro/fluxo-caixa', { query: { de, ate } })
    } catch {
      return null
    }
  },
}
