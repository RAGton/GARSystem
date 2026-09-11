# Frontend — Senior System (Next.js)

Cliente web para a API Axum em [`../backend`](../backend).
**Substitui** a GUI desktop egui (removida em P2.7, 2026-09-07).

---

## Stack

- **Next.js 16.3** (App Router, Server Components)
- **React 19.2**
- **TypeScript 5**
- **Tailwind CSS v4** (PostCSS)
- **shadcn/ui** + **Radix UI** (`@base-ui/react`)
- **Zustand 5** (state global)
- **Axios** (HTTP)
- **Recharts 3** (gráficos)
- **@hello-pangea/dnd** (drag-and-drop kanban)
- **lucide-react** (ícones)

> ⚠️ Next.js 16 tem breaking changes vs versões anteriores. Leia `node_modules/next/dist/docs/` antes de mexer em APIs.

---

## Estrutura

```
frontend/
├── package.json
├── tsconfig.json
├── next.config.ts
├── tailwind.config.ts
├── postcss.config.mjs
├── public/                  # Assets estáticos (favicon, logo)
└── src/
    ├── app/                 # App Router (rotas)
    │   ├── layout.tsx       # Root layout
    │   ├── globals.css      # Tailwind v4 entrypoint
    │   ├── icon.png
    │   ├── (dashboard)/     # Grupo de rotas autenticadas
    │   │   ├── layout.tsx   # Layout com sidebar/topbar
    │   │   └── page.tsx     # Dashboard principal
    │   ├── dashboard/
    │   │   └── page.tsx     # Dashboard executivo (KPIs)
    │   └── login/
    │       └── page.tsx     # Tela de login
    ├── components/
    │   ├── ui/              # shadcn primitives (button, card, input, label)
    │   ├── layout/          # sidebar, topbar, workspace-loader
    │   ├── dashboard/       # kpi-cards, alerts, trends-chart, ...
    │   ├── financeiro/      # cash-flow, profit-chart, finance-kpi-cards
    │   ├── os/              # os-kanban-board
    │   └── crm/             # kanban-board
    ├── lib/
    │   ├── api.ts           # Cliente HTTP (interceptors JWT, tenant header)
    │   └── utils.ts         # cn() class merger, helpers
    ├── services/
    │   └── auth.service.ts  # Login, refresh, logout
    └── stores/
        └── auth.store.ts    # Zustand: user, token, tenant
```

---

## Comandos

```bash
# Instalar deps
npm install

# Dev server (hot reload, porta 3000)
npm run dev

# Build
npm run build

# Servir build de produção
npm run start

# Lint
npm run lint
```

---

## Variáveis de ambiente

Criar `frontend/.env.local`:

```bash
NEXT_PUBLIC_API_URL=http://localhost:8080
NEXT_PUBLIC_TENANT_HEADER=x-tenant-id
```

---

## Auth & state

- Login: `POST /api/v1/auth/login` → access token (JWT) + refresh token.
- Access token armazenado em memória (Zustand). Refresh automático antes de expirar.
- Todas as requisições HTTP passam por `lib/api.ts`, que injeta `Authorization: Bearer <token>` e `x-tenant-id`.
- Rotas autenticadas ficam no grupo `(dashboard)` com layout protegido.

---

## Rotas implementadas (estado em 2026-09-07)

| Rota | Status | Notas |
|---|---|---|
| `/login` | ✅ funcional | Tela de login |
| `/` (dashboard principal) | ✅ funcional | Dashboard executivo |
| `/dashboard` | ✅ funcional | Alias do dashboard |
| `/clientes` | 🔶 parcial | Componente `CRMClienteGridCard` mapeado em BACKEND_MAPPING, página não criada |
| `/os` | 🔶 parcial | `os-kanban-board.tsx` existe, falta `app/os/page.tsx` |
| `/orcamentos` | ❌ falta | Mapeado, não implementado |
| `/servicos` | ❌ falta | Mapeado, não implementado |
| `/financeiro` | 🔶 parcial | Componentes (`cash-flow`, `profit-chart`) existem, falta página |
| `/crm` | 🔶 parcial | `kanban-board.tsx` existe, falta página |
| `/estoque` | ❌ falta | Mapeado, não implementado |
| `/admin` | ❌ falta | Mapeado, não implementado |
| `/relatorios` | ❌ falta | Mapeado, não implementado |

**Próximos passos:** portar cada tela egui → página Next.js seguindo [`docs/BACKEND_MAPPING.md`](../docs/BACKEND_MAPPING.md) como contrato.

---

## Mapeamento Rust/Axum ↔ Next.js

Veja [`docs/BACKEND_MAPPING.md`](../docs/BACKEND_MAPPING.md) para o mapeamento auditado entre:
- tabelas MySQL ↔ endpoints REST ↔ rotas Next.js ↔ componentes UI

Cada tela egui removida tem seu equivalente mapeado. Onde a rota Next.js ainda não existe (ver tabela acima), a migração da tela correspondente está pendente.