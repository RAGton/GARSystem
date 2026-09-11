// src/services/api.ts
// Re-export do cliente HTTP para que services/* não dependam de @/lib/api direto.
// Permite trocar implementação no futuro (axios, ofetch, etc.) sem refactor em massa.

export { fetchApi, ApiError, isApiError } from '@/lib/api'
export type { ApiOptions } from '@/lib/api'
