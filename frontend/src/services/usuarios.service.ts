// src/services/usuarios.service.ts
// Administração de usuários e papéis (RBAC).
//
// Endpoints:
//   GET  /usuarios                          (✓ exposto)
//   POST /usuarios                          (✓ exposto)
//   PUT  /usuarios/{username}/papel         (✓ exposto — ⚠️ handler STUB 501)

import { fetchApi, ApiError } from './api'

export interface InfoUsuario {
  id: number
  nome_usuario: string
}

export interface CriarUsuarioPayload {
  nome_usuario: string
  senha: string
}

export interface AlterarPapelPayload {
  novo_papel:
    | 'Administrador'
    | 'Gerencia'
    | 'Tecnico'
    | 'Financeiro'
    | 'Comercial'
    | 'Estoquista'
}

export class AlterarPapelNaoImplementadoError extends Error {
  code = 'STUB_NOT_IMPLEMENTED'
  constructor(message = 'Backend ainda não implementou a alteração de papel (501 STUB).') {
    super(message)
  }
}

export const usuariosService = {
  /** GET /usuarios. */
  async listar(): Promise<InfoUsuario[]> {
    try {
      const data = await fetchApi<InfoUsuario[]>('/usuarios')
      return Array.isArray(data) ? data : []
    } catch {
      return []
    }
  },

  /** POST /usuarios. */
  async criar(payload: CriarUsuarioPayload): Promise<InfoUsuario | null> {
    try {
      const created = await fetchApi<InfoUsuario | null>('/usuarios', {
        method: 'POST',
        body: payload,
      })
      return created
    } catch {
      return null
    }
  },

  /**
   * PUT /usuarios/{username}/papel — STUB 501 (BACKEND_MAPPING §3).
   * Lança `AlterarPapelNaoImplementadoError` quando back retorna 501.
   */
  async alterarPapel(username: string, novoPapel: AlterarPapelPayload['novo_papel']): Promise<boolean> {
    try {
      await fetchApi(`/usuarios/${encodeURIComponent(username)}/papel`, {
        method: 'PUT',
        body: { novo_papel: novoPapel },
      })
      return true
    } catch (err) {
      if (err instanceof ApiError && err.status === 501) {
        throw new AlterarPapelNaoImplementadoError()
      }
      throw err instanceof Error ? err : new Error(String(err))
    }
  },
}
