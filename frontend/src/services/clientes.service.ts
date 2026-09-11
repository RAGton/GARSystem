// src/services/clientes.service.ts
// Tipos e chamadas para o módulo de Clientes.
//
// Endpoints (de docs/openapi.json + BACKEND_MAPPING §3/§4):
//   GET    /clientes
//   POST   /clientes
//   GET    /clientes/{id}/resumo
//   GET    /crm/clientes/{id}/dashboard     (pendente — BACKEND_MAPPING §4)
//   GET    /crm/clientes/{id}/timeline      (pendente — BACKEND_MAPPING §4)

import { fetchApi } from './api'

// === Tipos (alinhados com openapi.json) ===

export interface Cliente {
  id: number
  nome: string
  email: string
  telefone: string
  endereco?: string | null
  inscricao_estadual?: string | null
  cpf_cnpj?: string | null
  credito_disponivel: number
}

export interface ClientePayload {
  nome: string
  email: string
  telefone: string
  endereco?: string | null
  cpf_cnpj?: string | null
}

export interface ResumoCliente {
  gastos_totais: number
  credito_disponivel: number
}

export interface PaginaCliente {
  items: Cliente[]
  page: number
  limit: number
  total: number
  total_paginas: number
}

// === Timeline (CRM módulo, endpoint futuro) ===
export interface TimelineEvento {
  id: number | string
  tipo: 'os' | 'orcamento' | 'observacao' | 'contato' | 'movimentacao'
  data: string
  titulo: string
  descricao?: string
  usuario?: string
}

// === Service ===

export interface ListarParams {
  page?: number
  limit?: number
  q?: string
}

export const clientesService = {
  /** GET /clientes (paginado, quando o back expor; senão retorna lista plana). */
  async listar(params: ListarParams = {}): Promise<Cliente[]> {
    try {
      const data = await fetchApi<Cliente[] | PaginaCliente>('/clientes', {
        query: { page: params.page, limit: params.limit, q: params.q },
      })
      // aceita tanto array plano quanto PaginaCliente
      if (Array.isArray(data)) return data
      if (data && Array.isArray(data.items)) return data.items
      return []
    } catch {
      return []
    }
  },

  /** GET /clientes/{id} — ainda não exposto; placeholder defensivo. */
  async obter(id: number): Promise<Cliente | null> {
    try {
      const data = await fetchApi<Cliente>(`/clientes/${id}`)
      return data
    } catch {
      return null
    }
  },

  /** GET /clientes/{id}/resumo — credito + gastos. */
  async resumo(id: number): Promise<ResumoCliente | null> {
    try {
      return await fetchApi<ResumoCliente>(`/clientes/${id}/resumo`)
    } catch {
      return null
    }
  },

  /** POST /clientes — cria novo cliente (requer crm.cliente.create). */
  async criar(payload: ClientePayload): Promise<Cliente | null> {
    try {
      return await fetchApi<Cliente>('/clientes', {
        method: 'POST',
        body: payload,
      })
    } catch {
      return null
    }
  },

  /**
   * GET /crm/clientes/{id}/timeline (pendente no back).
   * Best-effort: retorna [] se 404.
   */
  async timeline(id: number): Promise<TimelineEvento[]> {
    try {
      const data = await fetchApi<TimelineEvento[] | { items: TimelineEvento[] }>(
        `/crm/clientes/${id}/timeline`,
        { query: { limite: 50 } },
      )
      if (Array.isArray(data)) return data
      if (data && Array.isArray(data.items)) return data.items
      return []
    } catch {
      return []
    }
  },
}
