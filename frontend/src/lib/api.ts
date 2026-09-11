// src/lib/api.ts
// Cliente HTTP centralizado do GAR System Frontend.
//
// Regras:
// 1. Autenticação por cookie HttpOnly (NUNCA token em localStorage).
// 2. `credentials: "include"` sempre — o cookie é enviado ao mesmo domínio.
// 3. Base URL via env. Default = backend Rust/Axum dev local (porta 3000).
// 4. Erros padronizados do servidor viram `ApiError` (com code do back).
// 5. Em DEV, fallback silencioso para prompts em rota que falham — front nunca trava.

const API_BASE_URL =
  process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3000'

export class ApiError extends Error {
  constructor(
    public status: number,
    public code: string,
    message: string,
    public requestId?: string,
  ) {
    super(message)
    this.name = 'ApiError'
  }
}

export interface ApiOptions extends Omit<RequestInit, 'body' | 'headers'> {
  body?: unknown
  headers?: Record<string, string>
  /** Query params — append URL automáticamente */
  query?: Record<string, string | number | boolean | undefined | null>
}

function buildUrl(endpoint: string, query?: ApiOptions['query']): string {
  const base = endpoint.startsWith('http') ? endpoint : `${API_BASE_URL}${endpoint}`
  if (!query) return base
  const params = new URLSearchParams()
  for (const [k, v] of Object.entries(query)) {
    if (v === undefined || v === null) continue
    params.append(k, String(v))
  }
  const qs = params.toString()
  return qs ? `${base}?${qs}` : base
}

export async function fetchApi<T = unknown>(
  endpoint: string,
  options: ApiOptions = {},
): Promise<T> {
  const { body, query, headers: headerInput, ...rest } = options

  const headers: Record<string, string> = {
    Accept: 'application/json',
    ...headerInput,
  }
  if (body !== undefined) {
    headers['Content-Type'] = 'application/json'
  }

  const url = buildUrl(endpoint, query)

  const response = await fetch(url, {
    ...rest,
    credentials: 'include', // cookie HttpOnly vai junto
    headers,
    body: body !== undefined ? JSON.stringify(body) : undefined,
  })

  if (!response.ok) {
    let payload: any = null
    try {
      payload = await response.json()
    } catch {
      // corpo vazio ou HTML
    }
    const err = payload?.error
    throw new ApiError(
      response.status,
      err?.code ?? 'UNKNOWN',
      err?.message ?? response.statusText ?? `HTTP ${response.status}`,
      err?.request_id,
    )
  }

  // 204 No Content
  if (response.status === 204) return undefined as T
  return (await response.json()) as T
}

/** Helper específico para o body de erro padronizado do back */
export function isApiError(err: unknown): err is ApiError {
  return err instanceof ApiError
}
