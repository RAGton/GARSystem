// src/services/financeiro.service.ts
// Módulo Financeiro — todos os endpoints estão PENDENTES no backend
// (BACKEND_MAPPING §4), mas os services já existem (financial/service.rs).
//
// A camada service aqui é typed defensivamente: cada chamada tenta o
// endpoint real e cai pra fallback vazio se o back ainda não expôs.

import { fetchApi } from './api'

// === Tipos baseados nos services Rust (financial/service.rs) ===

export interface DashboardFinanceiro {
  recebido_no_mes: number
  a_receber: number
  despesa_no_mes: number
  a_pagar: number
  saldo: number
  inadimplencia: number
  variacao_receita_mes_anterior: number
  variacao_despesa_mes_anterior: number
}

export interface FluxoCaixaPonto {
  data: string // YYYY-MM-DD
  receitas: number
  despesas: number
  saldo_dia: number
}

export interface FluxoCaixaResultado {
  pontos: FluxoCaixaPonto[]
  total_receitas: number
  total_despesas: number
  saldo_periodo: number
}

export interface ContaReceber {
  id: number
  descricao: string
  cliente_id: number
  cliente_nome?: string
  valor: number
  data_vencimento: string
  data_pagamento?: string | null
  status: 'aberta' | 'paga' | 'cancelada' | 'vencida'
  orcamento_id?: number | null
}

export interface ContaPagar {
  id: number
  descricao: string
  fornecedor_id?: number
  fornecedor_nome?: string
  valor: number
  data_vencimento: string
  data_pagamento?: string | null
  status: 'aberta' | 'paga' | 'cancelada' | 'vencida'
}

export interface PlanoConta {
  id: number
  nome: string
  tipo: 'ATIVO' | 'PASSIVO' | 'RECEITA' | 'DESPESA' | 'CUSTO' | 'PATRIMONIO'
  parent_id?: number | null
  ativo: boolean
}

export interface CentroCusto {
  id: number
  nome: string
  codigo: string
  ativo: boolean
}

export interface AlertaFinanceiro {
  id: number
  severidade: 'info' | 'warning' | 'critical'
  mensagem: string
  contexto?: Record<string, unknown>
  created_at: string
  resolvido: boolean
}

export interface Pagina<T> {
  items: T[]
  page: number
  limit: number
  total: number
  total_paginas: number
}

// === Service ===

export interface ListarContasParams {
  status?: 'aberta' | 'paga' | 'cancelada' | 'vencida'
  page?: number
  limit?: number
}

export const financeiroService = {
  /** GET /financeiro/dashboard (pendente). */
  async dashboard(): Promise<DashboardFinanceiro | null> {
    try {
      return await fetchApi<DashboardFinanceiro>('/financeiro/dashboard')
    } catch {
      return null
    }
  },

  /** GET /financeiro/fluxo-caixa?de=...&ate=... (pendente). */
  async fluxoCaixa(de: string, ate: string): Promise<FluxoCaixaResultado | null> {
    try {
      const data = await fetchApi<FluxoCaixaResultado>('/financeiro/fluxo-caixa', {
        query: { de, ate },
      })
      return data
    } catch {
      return null
    }
  },

  /** GET /financeiro/contas-receber (pendente). */
  async listarContasReceber(params: ListarContasParams = {}): Promise<ContaReceber[]> {
    try {
      const data = await fetchApi<ContaReceber[] | Pagina<ContaReceber>>(
        '/financeiro/contas-receber',
        { query: { status: params.status, page: params.page, limit: params.limit } },
      )
      if (Array.isArray(data)) return data
      if (data && Array.isArray(data.items)) return data.items
      return []
    } catch {
      return []
    }
  },

  /** GET /financeiro/contas-pagar (pendente). */
  async listarContasPagar(params: ListarContasParams = {}): Promise<ContaPagar[]> {
    try {
      const data = await fetchApi<ContaPagar[] | Pagina<ContaPagar>>(
        '/financeiro/contas-pagar',
        { query: { status: params.status, page: params.page, limit: params.limit } },
      )
      if (Array.isArray(data)) return data
      if (data && Array.isArray(data.items)) return data.items
      return []
    } catch {
      return []
    }
  },

  /** POST /financeiro/contas-receber/{id}/pagar (pendente). */
  async pagarContaReceber(id: number, data: string, valor: number): Promise<boolean> {
    try {
      await fetchApi(`/financeiro/contas-receber/${id}/pagar`, {
        method: 'POST',
        body: { data, valor },
      })
      return true
    } catch {
      return false
    }
  },

  /** POST /financeiro/contas-pagar/{id}/pagar (pendente). */
  async pagarContaPagar(id: number, data?: string, valor?: number): Promise<boolean> {
    try {
      await fetchApi(`/financeiro/contas-pagar/${id}/pagar`, {
        method: 'POST',
        body: { data, valor },
      })
      return true
    } catch {
      return false
    }
  },

  /** GET /financeiro/plano-contas (pendente). */
  async planoContas(): Promise<PlanoConta[]> {
    try {
      const data = await fetchApi<PlanoConta[] | { items: PlanoConta[] }>(
        '/financeiro/plano-contas',
      )
      if (Array.isArray(data)) return data
      if (data && Array.isArray(data.items)) return data.items
      return []
    } catch {
      return []
    }
  },

  /** GET /financeiro/centros-custo (pendente). */
  async centrosCusto(): Promise<CentroCusto[]> {
    try {
      const data = await fetchApi<CentroCusto[] | { items: CentroCusto[] }>(
        '/financeiro/centros-custo',
      )
      if (Array.isArray(data)) return data
      if (data && Array.isArray(data.items)) return data.items
      return []
    } catch {
      return []
    }
  },

  /** GET /financeiro/alertas (pendente). */
  async alertas(params: { severidade?: AlertaFinanceiro['severidade'] } = {}): Promise<AlertaFinanceiro[]> {
    try {
      const data = await fetchApi<AlertaFinanceiro[] | { items: AlertaFinanceiro[] }>(
        '/financeiro/alertas',
        { query: { severidade: params.severidade } },
      )
      if (Array.isArray(data)) return data
      if (data && Array.isArray(data.items)) return data.items
      return []
    } catch {
      return []
    }
  },
}
