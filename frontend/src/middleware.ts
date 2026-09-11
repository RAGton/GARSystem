import { NextResponse, type NextRequest } from 'next/server'

// Guarda de rota: rotas `(dashboard)/*` exigem autenticação.
// P0 — antes do passo A definitivo, checa localStorage via cookie
// persistido (zustand persist grava em nome `auth-storage`).
// O ideal é checar o cookie HttpOnly `token` do backend — mas ele
// só é setado via /login real. Por ora:
// 1. Bloqueia TODA rota `(dashboard)/*` sem cookie `auth-storage`
//    com `isAuthenticated=true` (que zustand grava automaticamente
//    após setUser).
// 2. Permite `/login` e assets.
//
// NOTA: Como não temos JWT cookie ainda (passo A), essa checagem é
// fraca contra tampering manual — mas elimina o caso "F5 abre
// dashboard" quando o usuário nunca logou.

const COOKIE_AUTH = 'gar.auth'

export function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl

  // Rotas que exigem login
  const isDashboard = pathname === '/' || pathname.startsWith('/dashboard')
    || pathname.startsWith('/clientes') || pathname.startsWith('/crm')
    || pathname.startsWith('/os') || pathname.startsWith('/servicos')
    || pathname.startsWith('/orcamentos') || pathname.startsWith('/estoque')
    || pathname.startsWith('/financeiro') || pathname.startsWith('/relatorios')
    || pathname.startsWith('/admin')

  if (!isDashboard) return NextResponse.next()

  const cookie = request.cookies.get(COOKIE_AUTH)
  let isAuthed = false
  if (cookie?.value) {
    try {
      // zustand persist grava JSON escaped: {"state":{"isAuthenticated":true,...},"version":0}
      const decoded = JSON.parse(decodeURIComponent(cookie.value))
      isAuthed = decoded?.state?.isAuthenticated === true
    } catch {
      isAuthed = false
    }
  }

  if (!isAuthed) {
    const url = request.nextUrl.clone()
    url.pathname = '/login'
    url.searchParams.set('next', pathname)
    return NextResponse.redirect(url)
  }

  return NextResponse.next()
}

export const config = {
  matcher: [
    // Tudo exceto /login, /api, /_next, estáticos
    '/((?!login|api|_next/static|_next/image|favicon\\.png|icon\\.png|logo\\.png|assets|.*\\..*).*)',
  ],
}
