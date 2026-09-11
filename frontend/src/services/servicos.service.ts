// src/services/servicos.service.ts
// Catálogo de Serviços (vinculáveis a OS).
//
// Endpoints:
//   GET  /servicos
//   POST /servicos

import { fetchApi } from './api'

export interface Servico {
  id: number
  nome: string
  descricao: string
  preco: number
}

export const servicosService = {
  /** GET /servicos. */
  async listar(): Promise<Servico[]> {
    try {
      const data = await fetchApi<Servico[] | { items: Servico[] }>('/servicos')
      if (Array.isArray(data)) return data
      if (data && Array.isArray(data.items)) return data.items
      return []
    } catch {
      return []
    }
  },

  /** POST /servicos. */
  async criar(servico: Omit<Servico, 'id'>): Promise<Servico | null> {
    try {
      return await fetchApi<Servico>('/servicos', {
        method: 'POST',
        body: servico,
      })
    } catch {
      return null
    }
  },
}
