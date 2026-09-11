# Backend — Senior System (Rust/Axum)

API REST multi-tenant com JWT, RBAC e isolamento por `tenant_id`.
**Sem interface gráfica** desde a migração P2.7 (egui removido). Apenas servidor HTTP.

---

## Estrutura

```
backend/
├── Cargo.toml          # Deps (axum 0.8, mysql, jsonwebtoken, image)
├── Cargo.lock
├── Dockerfile          # Multi-stage → distroless
├── src/
│   ├── lib.rs          # Declara módulos públicos da lib `senior_system`
│   ├── main.rs         # ❌ REMOVIDO (era entrypoint GUI egui)
│   ├── aplicacao.rs    # ❌ REMOVIDO (era orquestrador de telas egui)
│   ├── server.rs       # Entry binário `senior-system-server` (Axum app)
│   ├── bootstrap.rs    # Inicialização (DB pool, JWT secret, migrations)
│   ├── auth.rs         # JWT, bcrypt, anti-enumeração
│   ├── rate_limit.rs   # Rate limit por IP/tenant
│   ├── http_client.rs  # Cliente HTTP interno (usado pelo admin_cli)
│   ├── dto.rs          # Data Transfer Objects (request/response)
│   ├── executor.rs     # Executor de queries assíncronas
│   ├── servicos.rs     # Camada de serviço de alto nível
│   ├── arquivos/       # Upload/processamento de anexos (compressão JPEG)
│   ├── banco_de_dados/ # Conexão MySQL + repositórios por entidade
│   ├── bin/
│   │   ├── admin_cli.rs        # `senior-system-admin` — bootstrap seguro
│   │   └── audit_rbac_achado1.rs # `audit-rbac-achado1` — auditoria offline
│   ├── cotacao_orcamento/ # Cotações e orçamentos
│   ├── crm/             # CRM (deals, leads, stages, kanban)
│   ├── empresa/         # Multi-tenant: empresa, workspace
│   ├── financial/       # Títulos a pagar/receber, fluxo de caixa
│   ├── operations/      # Operações internas (auditoria, jobs)
│   ├── os_mobile/       # OS otimizadas para mobile
│   ├── rbac/            # Role-Based Access Control (papéis, permissões)
│   ├── storage/         # Storage abstraction (S3-compatible, local)
│   └── transcription/   # Transcrição de áudio (anexos OS)
└── tests/
    ├── estoque_diff.rs        # Teste de diff de estoque
    ├── regra11_audit.rs       # Regra 11: auditoria
    ├── tenants.rs             # Isolamento multi-tenant
    └── integration/           # Testes de integração (#[ignore] por padrão)
```

---

## Build & execução

```bash
# Build (release recomendado em produção)
cargo build --release

# Executar servidor
./target/release/senior-system-server

# Validar compilação sem executar
cargo check

# Admin CLI
./target/release/senior-system-admin create-admin

# Auditoria RBAC offline
./target/release/audit-rbac-achado1

# Testes unitários
cargo test

# Testes de integração (requer MySQL rodando)
cargo test --features integration-tests -- --ignored
```

---

## Endpoints

| Método | Path | Auth | Descrição |
|---|---|---|---|
| GET | `/health` | público | Health check |
| POST | `/api/v1/auth/login` | público | Login JWT |
| POST | `/api/v1/auth/refresh` | refresh | Renovar access token |
| GET | `/api/v1/dashboard/resumo` | JWT | KPIs executivo |
| GET | `/api/v1/clientes` | JWT | Lista paginada |
| GET | `/api/v1/ordens` | JWT | Ordens de serviço |
| GET | `/api/v1/crm/deals` | JWT | Pipeline CRM |
| GET | `/api/v1/financial/titulos` | JWT | Títulos financeiros |
| GET | `/api/v1/estoque/pecas` | JWT | Peças em estoque |

Catálogo completo em [`/docs/openapi.json`](../docs/openapi.json) e [`docs/BACKEND_MAPPING.md`](../docs/BACKEND_MAPPING.md).

---

## Variáveis de ambiente

| Var | Obrigatória | Exemplo | Descrição |
|---|---|---|---|
| `DATABASE_URL` | sim | `mysql://user:pass@db:3306/seniorsystem` | Connection string MySQL |
| `JWT_SECRET` | sim | base64 32+ bytes | Segredo HMAC-SHA256 |
| `RUST_LOG` | não | `info` | Nível tracing |
| `BIND_ADDR` | não | `0.0.0.0:8080` | Endereço de bind |
| `TENANT_HEADER` | não | `x-tenant-id` | Header multi-tenant |

---

## Multi-tenant & RBAC

- Todo request autenticado carrega `tenant_id` (do JWT).
- Repositórios filtram por `tenant_id` automaticamente — **nunca confiar em input do cliente**.
- RBAC: papéis por empresa → permissões por recurso (RBAC-ENDPOINT-MATRIX em [`docs/`](../docs/RBAC-ENDPOINT-MATRIX.md)).

---

## Remoções recentes (P2.7)

Removido em 2026-09-07 durante migração egui → Next.js:
- `src/main.rs` (entrypoint GUI)
- `src/aplicacao.rs` (orquestrador de telas)
- `src/gui_services/` (cliente HTTP da GUI → API)
- `src/telas/` (19 telas nativas: login, painel_clientes, painel_ordens, ...)
- Deps: `eframe 0.33`, `egui_plot 0.34`, `egui_extras 0.33`
- Binário `senior-system-gui`

Mantido: `image 0.25` (uso server-side em `arquivos::service::comprimir_jpeg`).

Backup do código removido em `/tmp/seniorsystem-migration-20260907-191547/` (caso precise reverter).