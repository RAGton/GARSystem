// src/services/orcamentos.service.ts
// Orçamentos.
//
// Endpoints:
//   GET  /orcamentos           (pendente — BACKEND_MAPPING §4)
//   POST /orcamentos           (✓ exposto)
//   GET  /orcamentos/{id}      (✓ exposto)

import { fetchApi } from './api'

export interface OrcamentoItem {
  descricao: string
  quantidade: number
  preco_unitario: number
  preco_total: number
}

export interface Orcamento {
  id: number
  cliente_id: number
  items: OrcamentoItem[]
  total: number
}

export interface OrcamentoPayload {
  cliente_id: number
  items: OrcamentoItem[]
}

export const orcamentosService = {
  /** GET /orcamentos (pendente — fallback []). */
  async listar(params: { page?: number; limit?: number } = {}): Promise<Orcamento[]> {
    try {
      const data = await fetchApi<Orcamento[] | { items: Orcamento[] }>(
        '/orcamentos',
        { query: { page: params.page, limit: params.limit } },
      )
      if (Array.isArray(data)) return data
      if (data && Array.isArray(data.items)) return data.items
      return []
    } catch {
      return []
    }
  },

  /** GET /orcamentos/{id}. */
  async obter(id: number): Promise<Orcamento | null> {
    try {
      return await fetchApi<Orcamento>(`/orcamentos/${id}`)
    } catch {
      return null
    }
  },

  /** POST /orcamentos. */
  async criar(payload: OrcamentoPayload): Promise<Orcamento | null> {
    try {
      return await fetchApi<Orcamento>('/orcamentos', {
        method: 'POST',
        body: payload,
      })
    } catch {
      return null
    }
  },
}
