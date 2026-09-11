// src/services/estoque.service.ts
// Estoque — peças e fornecedores.
//
// Endpoints:
//   GET  /estoque/pecas                (✓ exposto)
//   POST /estoque/pecas                (✓ exposto — BACKEND_MAPPING §4)
//   PUT  /estoque/pecas/{id}           (✓ exposto — BACKEND_MAPPING §4)
//   GET  /estoque/fornecedores         (✓ exposto — BACKEND_MAPPING §4)
//   GET  /estoque/pecas/alerta-minimo  (✓ service pronto — filtro WHERE atual<=minimo)

import { fetchApi } from './api'

export interface Peca {
  id: number
  nome: string
  codigo_interno: string
  part_number: string
  descricao: string
  fabricante: string
  localizacao: string
  estoque_atual: number
  estoque_minimo: number
  preco_custo: number
  preco_venda: number
}

export interface Fornecedor {
  id: number
  nome: string
  cnpj: string
  contato: string
  telefone: string
  email: string
}

export type PecaPayload = Omit<Peca, 'id'>

export const estoqueService = {
  /** GET /estoque/pecas. */
  async listarPecas(): Promise<Peca[]> {
    try {
      const data = await fetchApi<Peca[]>('/estoque/pecas')
      return Array.isArray(data) ? data : []
    } catch {
      return []
    }
  },

  /** GET /estoque/pecas/alerta-minimo (filtro client-side como fallback). */
  async alertaMinimo(): Promise<Peca[]> {
    try {
      const data = await fetchApi<Peca[]>('/estoque/pecas/alerta-minimo')
      if (Array.isArray(data) && data.length > 0) return data
    } catch {
      /* endpoint pode ainda não existir — cai pro fallback */
    }
    // Fallback defensivo: filtra client-side.
    const todas = await this.listarPecas()
    return todas.filter((p) => p.estoque_atual <= p.estoque_minimo)
  },

  /** GET /estoque/fornecedores. */
  async listarFornecedores(): Promise<Fornecedor[]> {
    try {
      const data = await fetchApi<Fornecedor[]>('/estoque/fornecedores')
      return Array.isArray(data) ? data : []
    } catch {
      return []
    }
  },

  /** POST /estoque/pecas. */
  async criarPeca(peca: PecaPayload): Promise<Peca | null> {
    try {
      return await fetchApi<Peca>('/estoque/pecas', {
        method: 'POST',
        body: peca,
      })
    } catch {
      return null
    }
  },

  /** PUT /estoque/pecas/{id}. */
  async atualizarPeca(id: number, peca: Partial<PecaPayload>): Promise<Peca | null> {
    try {
      return await fetchApi<Peca>(`/estoque/pecas/${id}`, {
        method: 'PUT',
        body: peca,
      })
    } catch {
      return null
    }
  },
}
