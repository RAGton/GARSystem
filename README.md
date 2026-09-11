# Senior System

Sistema de gestão empresarial (ERP) para oficinas de assistência técnica.
**Stack:** Backend Rust/Axum + Frontend Next.js, executado em containers Podman.

---

## Arquitetura

```
┌─────────────────────────────────────────────────────────────┐
│  Podman Compose (compose.yaml)                              │
│                                                              │
│  ┌──────────────────┐      ┌──────────────────────────┐    │
│  │  frontend (Next)  │─────▶│  server (Rust/Axum:8080) │    │
│  │  React 19 / Next  │ HTTP │  API REST + JWT          │    │
│  │  16 / Tailwind    │      │  RBAC + tenant isolation │    │
│  └──────────────────┘      └──────────┬───────────────┘    │
│                                        │                     │
│                                  ┌─────▼──────┐             │
│                                  │  MySQL 8.0 │             │
│                                  │  + 18 SQL  │             │
│                                  │  migrations│             │
│                                  └────────────┘             │
└─────────────────────────────────────────────────────────────┘
```

| Camada | Tecnologia | Porta | Diretório |
|---|---|---|---|
| Frontend | Next.js 16 + React 19 + Tailwind v4 | 3000 | `frontend/` |
| Backend  | Rust/Axum 0.8 + Tokio + sqlx-mysql | 8080 | `backend/` |
| Banco    | MySQL 8.0 | 3306 | (volume) |
| Orquestração | Podman Compose | — | `compose.yaml` |

---

## Migração: GUI egui → Web Next.js (P2.7, 2026-09-07)

**Antes (P2.6.x):** monolito Rust com GUI egui (`eframe 0.33`) + servidor Axum no mesmo crate.
- 19 telas nativas em `src/telas/*` consumindo `gui_services::*` via HTTP loopback
- Binário `senior-system-gui` abria janela nativa
- Total: ~22 arquivos `.rs` com `use eframe::*` / `use egui::*`

**Depois (P2.7):** cliente Next.js separado consumindo apenas HTTP REST.
- GUI desktop (`eframe`, `egui_plot`, `egui_extras`, bin `senior-system-gui`) **removida**
- `image` crate mantida (uso server-side em `arquivos::service` para compressão JPEG)
- Frontend em `frontend/` com App Router, Server Components, shadcn/ui + Tailwind v4
- Mapeamento Rust ↔ Next.js documentado em [`docs/BACKEND_MAPPING.md`](docs/BACKEND_MAPPING.md)

**O que ficou idêntico:** schema MySQL, endpoints HTTP, contratos de API, RBAC, tenant isolation, JWT auth.

---

## Subir o sistema (Podman)

```bash
# 1. Variáveis de ambiente
cp .env.example .env  # se existir; senão criar MYSQL_USER/PASSWORD/ROOT_PASSWORD/DATABASE

# 2. Build das imagens
podman compose build

# 3. Sobe backend + banco (com migrations automáticas via db-init/)
podman compose up -d server db

# 4. Sobe frontend (após server estar healthy)
podman compose up -d frontend

# 5. Criar admin inicial
podman compose exec server ./senior-system-admin create-admin
```

**Health checks:**
- Backend: `curl http://localhost:8080/health`
- Frontend: abrir `http://localhost:3000`
- Banco: `podman compose exec db mysqladmin ping`

---

## Estrutura do repositório

| Diretório | Propósito |
|---|---|
| `backend/` | API Rust/Axum. Ver [`backend/README.md`](backend/README.md). |
| `frontend/` | Cliente Next.js. Ver [`frontend/README.md`](frontend/README.md). |
| `db-init/` | Scripts SQL de bootstrap + 18 migrations. |
| `docs/` | Relatórios P2.6.x, auditoria RBAC, mapping frontend↔backend, OpenAPI. |
| `scripts/` | Utilitários Python (audit_tenant_id, etc). |
| `assets/` | Fontes (JetBrainsMono, NotoColorEmoji) e logo. |
| `compose.yaml` | Orquestração Podman: server + db (+ frontend opcional). |
| `Dockerfile` | Imagem única do backend Rust (multi-stage, distroless). |

---

## Releases & versões

- **P2.6.2c** — clippy cleanup, 3 dashboards, RBAC coverage (commit `39c8eed`)
- **P2.7** — migração egui → Next.js, backend API-only, este README (2026-09-07)

Veja [`CHANGELOG.md`](CHANGELOG.md) para histórico completo.