// src/services/auth.service.ts
// Autenticação do GAR System Frontend.
//
// Regra: zero token em JS. O cookie HttpOnly `token` é gerenciado pelo back.
// O front só decodifica as claims (via /me) ou usa o payload do login (mínimo)
// para autorizar UI antes do /me chegar.

import { fetchApi, ApiError } from '@/lib/api'
import { useAuthStore, type User } from '@/stores/auth.store'

/** Payload do POST /login — campos do openapi LoginResponse. */
export interface LoginResult {
  papel: string
  expira_em: number
  /** Sempre vazio no body — JWT vem no cookie HttpOnly. */
  token?: string
}

/** Payload do GET /me — claims completas do JWT. */
export interface MeResult {
  uid: number
  username: string
  papel: string
  tenant: number
  roles: string[]
  permissions: string[]
  expira_em: number
}

/** Helper interno: mensagens amigáveis a partir do `code` do backend. */
const ERROR_MAP: Record<string, string> = {
  AUTH_INVALID: 'Usuário ou senha incorretos.',
  RATE_LIMITED:
    'Muitas tentativas. Aguarde alguns segundos antes de tentar novamente.',
  VALIDATION: 'Preencha usuário e senha.',
  AUTH_REQUIRED: 'Sessão expirada. Faça login novamente.',
  FORBIDDEN: 'Sem permissão para essa operação.',
}

function humanize(err: unknown): Error {
  if (err instanceof ApiError) {
    const msg = ERROR_MAP[err.code] ?? err.message
    return new Error(msg)
  }
  return new Error(
    'Não foi possível contatar o servidor. Verifique se o backend está em execução.',
  )
}

/**
 * Tenta login. Em sucesso, atualiza a store com claims mínimas
 * (papel + expira_em). O /me completa o resto logo depois.
 */
async function login(usuario: string, senha: string): Promise<LoginResult> {
  try {
    const res = await fetchApi<LoginResult>('/login', {
      method: 'POST',
      body: { usuario, senha },
    })
    // Back setou o cookie HttpOnly `token`. Front NUNCA toca no token.
    // Hidrata store com o mínimo até o /me chegar.
    const partialUser: User = {
      papel: res.papel,
      expira_em: res.expira_em,
      permissions: [],
      roles: [],
    }
    useAuthStore.getState().setUser(partialUser)
    useAuthStore.getState().setFreshLogin(true)
    return res
  } catch (err) {
    throw humanize(err)
  }
}

/**
 * GET /me — devolve as claims completas do JWT (uid, username, papel,
 * tenant, roles, permissions, expira_em).
 *
 * Quando o endpoint existir, hidrate a store com tudo. Se não existir
 * (404), mantém o estado mínimo do login (best-effort).
 */
async function me(): Promise<MeResult | null> {
  try {
    const res = await fetchApi<MeResult>('/me')
    useAuthStore.getState().setUser({
      id: res.uid,
      username: res.username,
      papel: res.papel,
      tenant: res.tenant,
      permissions: res.permissions ?? [],
      roles: res.roles ?? [],
      expira_em: res.expira_em,
    })
    return res
  } catch (err) {
    // Em dev, /me pode ainda não existir — log e segue.
    if (err instanceof ApiError && (err.status === 404 || err.status === 401)) {
      return null
    }
    console.warn('[auth.me] falhou:', err)
    return null
  }
}

/**
 * POST /logout — invalida JWT (blacklist in-memory no backend) + limpa a store.
 * Best-effort: se o back ainda não expõe /logout, limpa local mesmo assim.
 */
async function logout(): Promise<void> {
  try {
    await fetchApi<void>('/logout', { method: 'POST' })
  } catch {
    // ignora — back pode estar offline ou não ter /logout ainda
  }
  useAuthStore.getState().logout()
}

export const authService = {
  login,
  logout,
  me,
}

export type { User }
