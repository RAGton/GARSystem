// src/services/os.service.ts
// Ordens de Serviço.
//
// Endpoints expostos (openapi):
//   GET  /ordens
//   POST /ordens
//   GET  /ordens/{id}
//   PUT  /ordens/{id}
//
// Endpoints PENDENTES (BACKEND_MAPPING §4 — operations):
//   POST /os/{id}/mover  (state machine)
//   GET  /os/{id}/workflow
//   GET  /os/{id}/sla
//   GET  /os/kanban
//
// O tipo `status` do backend é o enum Rust:
//   Aberta | Orcamento | Aprovada | EmAndamento | AguardandoPeca | Finalizada | Cancelada
//
// `mover()` está tipado defensivamente: aceita id+transicao_id e falha gracioso
// enquanto o endpoint não existir (alertando no console).

import { fetchApi, ApiError } from './api'

// === Tipos ===

export type StatusOS =
  | 'Aberta'
  | 'Orcamento'
  | 'Aprovada'
  | 'EmAndamento'
  | 'AguardandoPeca'
  | 'Finalizada'
  | 'Cancelada'

export type SituacaoOS =
  | 'Orcamento'
  | 'Aprovado'
  | 'EmAndamento'
  | 'AutorizadoAguardandoPeca'
  | 'ServicoConcluido'
  | 'AguardandoAutorizacao'
  | 'AguardandoRetirada'
  | 'Reprovado'
  | 'AguardandoFaturar'
  | 'Faturado'

export interface PecaOS {
  id_peca: number
  codigo_interno: string
  descricao: string
  quantidade: number
  preco_venda_unitario: number
  preco_total: number
}

export interface ServicoOS {
  id_servico: number
  nome: string
  descricao: string
  quantidade: number
  preco_unitario: number
  preco_total: number
}

export interface HistoricoEdicao {
  usuario: string
  data_hora: string
  campo_alterado: string
  valor_antigo: string
  valor_novo: string
}

export interface OrdemServico {
  id: number
  cliente: string
  equipamento: string
  defeito_relatado: string
  status: StatusOS
  parecer_tecnico: string
  situacao: SituacaoOS
  numero_serie_equipamento: string
  observacoes: string
  nome_tecnico_responsavel: string
  atendente: string
  horario_abertura: string
  telefone_cliente: string
  data_chegada: string
  prazo_entrega: string
  historico_edicoes: HistoricoEdicao[]
  pecas: PecaOS[]
  total_pecas: number
  servicos: ServicoOS[]
  total_servicos: number
}

export interface PaginaOrdemServico {
  items: OrdemServico[]
  page: number
  limit: number
  total: number
  total_paginas: number
}

export interface MoverPayload {
  transicao_id: number
  motivo?: string
}

// === Service ===

export interface ListarParams {
  page?: number
  limit?: number
  status?: StatusOS
}

export const osService = {
  /** GET /ordens (paginado quando back expor; senão array plano). */
  async listar(params: ListarParams = {}): Promise<OrdemServico[]> {
    try {
      const data = await fetchApi<OrdemServico[] | PaginaOrdemServico>('/ordens', {
        query: { page: params.page, limit: params.limit, status: params.status },
      })
      if (Array.isArray(data)) return data
      if (data && Array.isArray(data.items)) return data.items
      return []
    } catch {
      return []
    }
  },

  /** GET /ordens/{id}. */
  async obter(id: number): Promise<OrdemServico | null> {
    try {
      return await fetchApi<OrdemServico>(`/ordens/${id}`)
    } catch {
      return null
    }
  },

  /** POST /ordens. */
  async criar(os: Partial<OrdemServico>): Promise<OrdemServico | null> {
    try {
      return await fetchApi<OrdemServico>('/ordens', {
        method: 'POST',
        body: os,
      })
    } catch {
      return null
    }
  },

  /** PUT /ordens/{id} — payload exige { os, usuario }. */
  async atualizar(
    id: number,
    os: Partial<OrdemServico>,
    usuario: string,
  ): Promise<OrdemServico | null> {
    try {
      return await fetchApi<OrdemServico>(`/ordens/${id}`, {
        method: 'PUT',
        body: { os, usuario },
      })
    } catch {
      return null
    }
  },

  /**
   * POST /os/{id}/mover — state machine (PENDENTE no back).
   * Best-effort: se o endpoint não existir (404), retorna false e loga warning.
   */
  async mover(id: number, payload: MoverPayload): Promise<boolean> {
    try {
      await fetchApi<void>(`/os/${id}/mover`, {
        method: 'POST',
        body: payload,
      })
      return true
    } catch (err) {
      if (err instanceof ApiError && (err.status === 404 || err.status === 501)) {
        console.warn(
          `[os.mover] endpoint /os/{id}/mover ainda não exposto pelo backend. ` +
            `Movimentação local apenas (id=${id}, transicao=${payload.transicao_id}).`,
        )
        return false
      }
      console.error('[os.mover] erro:', err)
      return false
    }
  },

  /** GET /os/{id}/sla — pendente. */
  async sla(id: number): Promise<unknown> {
    try {
      return await fetchApi(`/os/${id}/sla`)
    } catch {
      return null
    }
  },

  /** GET /os/kanban — pendente (state machine agrupada por coluna). */
  async kanban(): Promise<unknown> {
    try {
      return await fetchApi('/os/kanban')
    } catch {
      return null
    }
  },
}
