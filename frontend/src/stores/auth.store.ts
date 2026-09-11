// src/stores/auth.store.ts
// Store de autenticação do GAR System (Zustand + persist).
//
// REGRAS DE SEGURANÇA:
// 1. NUNCA persistir `jwt` / token raw / cookie value.
// 2. Persistir SOMENTE claims (user, papel, tenant, permissions, roles, expira_em).
// 3. O token HttpOnly é gerenciado pelo backend; o front nunca toca.
// 4. Migração automática do shape antigo {papel, expira_em} para o novo shape completo.

import { create } from 'zustand'
import { persist } from 'zustand/middleware'

/**
 * Claim JWT (sub=uid) — só os campos que o front precisa para autorização de UI.
 * A verdade canônica está no cookie HttpOnly `token` no backend.
 */
export interface User {
  /** uid numérico (claim `uid`) */
  id?: number
  /** username (claim `sub`) */
  username?: string
  /** Papel único (claim `papel`) */
  papel: string
  /** Tenant id numérico (claim `tenant`) */
  tenant?: number
  /** Permissions granulares (claim `permissions`) */
  permissions: string[]
  /** Roles (claim `roles`) */
  roles: string[]
  /** Epoch (segundos) em que o token expira */
  expira_em?: number
}

/** Estado persistido em localStorage. NUNCA inclui o token. */
interface PersistedAuth {
  user: User | null
  isAuthenticated: boolean
  isFreshLogin: boolean
}

interface AuthState extends PersistedAuth {
  /** Substitui o user inteiro (vinda de /login + /me). */
  setUser: (user: User | null) => void
  /** Marca flag de fresh-login (loader welcome). */
  setFreshLogin: (status: boolean) => void
  /** Limpa tudo (logout). */
  logout: () => void

  /** Helpers de autorização (RBAC). */
  hasPermission: (permission: string) => boolean
  hasAnyPermission: (permissions: string[]) => boolean
}

/**
 * Compara uma permission contra as permissions do user,
 * suportando wildcard de módulo (ex: `os.*` cobre `os.create`).
 */
function matchPermission(userPerm: string, required: string): boolean {
  if (userPerm === required) return true
  if (userPerm === '*') return true
  // userPerm é "modulo.*" → cobre qualquer "modulo.algo"
  if (userPerm.endsWith('.*')) {
    const prefix = userPerm.slice(0, -2) // "modulo"
    return required === prefix || required.startsWith(prefix + '.')
  }
  return false
}

/**
 * Migração: shape antigo `{user:{papel, expira_em}}` → novo `{user:{..., permissions:[], roles:[]}}`
 * Executada pelo `persist` ao ler do localStorage.
 */
function migrate(persisted: unknown, _version: number): PersistedAuth {
  if (!persisted || typeof persisted !== 'object') {
    return { user: null, isAuthenticated: false, isFreshLogin: false }
  }
  const p = persisted as Record<string, unknown>
  const legacyUser = p.user as Record<string, unknown> | null | undefined

  if (!legacyUser) {
    return {
      user: null,
      isAuthenticated: false,
      isFreshLogin: Boolean(p.isFreshLogin),
    }
  }

  // Já está no shape novo?
  const hasNewShape = Array.isArray(legacyUser.permissions) || Array.isArray(legacyUser.roles)

  if (hasNewShape) {
    return {
      user: {
        id: typeof legacyUser.id === 'number' ? legacyUser.id : undefined,
        username: typeof legacyUser.username === 'string' ? legacyUser.username : undefined,
        papel: String(legacyUser.papel ?? ''),
        tenant: typeof legacyUser.tenant === 'number' ? legacyUser.tenant : undefined,
        permissions: Array.isArray(legacyUser.permissions)
          ? (legacyUser.permissions as string[])
          : [],
        roles: Array.isArray(legacyUser.roles) ? (legacyUser.roles as string[]) : [],
        expira_em:
          typeof legacyUser.expira_em === 'number' ? legacyUser.expira_em : undefined,
      },
      isAuthenticated: Boolean(p.isAuthenticated),
      isFreshLogin: Boolean(p.isFreshLogin),
    }
  }

  // Migração do shape antigo {papel, expira_em} → novo shape com defaults seguros.
  return {
    user: {
      id: typeof legacyUser.id === 'number' ? (legacyUser.id as number) : undefined,
      papel: String(legacyUser.papel ?? ''),
      permissions: [],
      roles: [],
      expira_em:
        typeof legacyUser.expira_em === 'number'
          ? (legacyUser.expira_em as number)
          : undefined,
    },
    isAuthenticated: Boolean(p.isAuthenticated),
    isFreshLogin: Boolean(p.isFreshLogin),
  }
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      user: null,
      isAuthenticated: false,
      isFreshLogin: false,

      setUser: (user) => {
        if (!user) {
          set({ user: null, isAuthenticated: false })
          return
        }
        // Garante defaults de permissions/roles sem duplicar chaves no spread.
        const { permissions, roles, ...rest } = user
        const normalized = {
          ...rest,
          permissions: permissions ?? [],
          roles: roles ?? [],
        }
        set({
          user: normalized,
          isAuthenticated: true,
        })
      },

      setFreshLogin: (status) => set({ isFreshLogin: status }),

      logout: () =>
        set({ user: null, isAuthenticated: false, isFreshLogin: false }),

      hasPermission: (permission) => {
        const u = get().user
        if (!u) return false
        if (!Array.isArray(u.permissions) || u.permissions.length === 0) return false
        return u.permissions.some((p) => matchPermission(p, permission))
      },

      hasAnyPermission: (permissions) => {
        if (!permissions || permissions.length === 0) return true
        return permissions.some((p) => get().hasPermission(p))
      },
    }),
    {
      name: 'gar.auth',
      version: 2,
      migrate,
      // Persistência explícita: garante que NUNCA vaze `jwt`/`token` no JSON.
      partialize: (state) => ({
        user: state.user,
        isAuthenticated: state.isAuthenticated,
        isFreshLogin: state.isFreshLogin,
      }),
    },
  ),
)
