# CONTEXT — Senior System

> **Memória técnica** do projeto. Documenta decisões, arquitetura, regras de
> negócio, dependências, e tudo que um novo dev (ou agente de IA) precisa
> saber para ser produtivo no projeto **sem ter que re-ler todo o código**.
>
> Última atualização: **2026-08-21** (Sprint P0 — fundação).

## 1. Visão

O **Senior System** é um sistema de gestão cliente-servidor para uso interno
em empresas de assistência técnica / manutenção. Cobre Ordens de Serviço,
clientes, orçamento, peças, serviços, e dashboard.

A ambição é evoluir para **ERP + CRM + mobile**, mas o foco atual é
**consolidar a fundação** antes de adicionar novas features de produto.

## 2. Stack

| Camada | Tecnologia | Versão |
|---|---|---|
| Backend (servidor) | Rust + Axum + Tokio | rustc 1.78+ |
| Frontend (cliente) | Rust + eframe/egui | 0.33.0 |
| Banco de dados | MySQL 8 | 8.0+ |
| Driver MySQL | `mysql` crate | 26.0.1 |
| Auth | `jsonwebtoken` (HS256) + `bcrypt` | 9 / 0.17 |
| HTTP cliente (GUI) | `reqwest` blocking | 0.12 |
| Serialização | `serde` + `serde_json` | 1.0 |
| Logs | `tracing` + `tracing-subscriber` | 0.1 / 0.3 |
| Migrations | `include_str!` + tabela `schema_migrations` | próprio |
| Container | Podman / Docker Compose | — |
| Build release | `cargo build --release` | — |

## 3. Arquitetura

```
┌──────────────────────────────────────────┐
│ Cliente (Rust + eframe/egui, desktop)    │
│  - Telas por papel do usuário            │
│  - Estado: eframe::Storage + Mutex       │
│  - HTTP: reqwest blocking + bearer auth  │
└──────────────────┬───────────────────────┘
                   │ http://host:3000
                   │ Authorization: Bearer <jwt>
                   ▼
┌──────────────────────────────────────────┐
│ Servidor (Rust + Axum + Tokio)            │
│  - Router: rotas públicas + protegidas   │
│  - Middleware: extração de Claims (JWT)   │
│  - Handlers: tokio::task::spawn_blocking │
│  - Services: fachada pura em `servicos.rs`│
│  - DB: pool mysql + transactions         │
└──────────────────┬───────────────────────┘
                   │ mysql
                   ▼
┌──────────────────────────────────────────┐
│ MySQL 8 (Podman / Docker)                │
│  - Tabelas de negócio (users, os, ...)   │
│  - Movimentações de estoque (auditoria)   │
│  - Schema migrations: schema_migrations  │
└──────────────────────────────────────────┘
```

## 4. Decisões arquiteturais (ADRs)

### ADR-001: Cliente-servidor com HTTP entre eles
- **Status:** Aceito (v1.5.0).
- **Contexto:** Originalmente a GUI acessava o DB diretamente.
- **Decisão:** GUI fala HTTP com o servidor. Centraliza regra de negócio e
  simplifica multi-tenant no futuro.
- **Consequência:** Toda request precisa de token. Latência de rede em dev.

### ADR-002: Autenticação via JWT (HS256) com segredo obrigatório
- **Status:** Aceito (v1.9.0).
- **Contexto:** Antes o JWT era gerado e descartado; backend não exigia.
- **Decisão:** Middleware de auth valida `Authorization: Bearer ...`. Em
  build release, ausência de `JWT_SECRET` causa **panic** na inicialização.
- **Consequência:** Mais seguro. Erro claro se env faltar.

### ADR-003: Mass assignment corrigido — papel só por endpoint admin
- **Status:** Aceito (v1.9.0).
- **Contexto:** `POST /usuarios` aceitava `papel` no body — qualquer um
  podia se tornar admin.
- **Decisão:** `POST /usuarios` cria usuários sempre com `Comercial`. Para
  promover, usar `PUT /usuarios/{username}/papel` (em construção).
- **Consequência:** Bug crítico fechado. Endpoint de admin ainda pendente.

### ADR-004: Estoque — diff + lock pessimista
- **Status:** Aceito (v1.9.0).
- **Contexto:** `atualizar_os` apaga+inseria peças da OS e recalculava
  estoque como `anterior - qtd_nova`, corrompendo saldo a cada edição.
- **Decisão:** Diff entre peças antigas e novas; aplica **delta**; usa
  `SELECT ... FOR UPDATE` para serializar concorrência.
- **Consequência:** Estoque reflete estado real, não a sequência de edits.

### ADR-005: Schema único via migrations versionadas
- **Status:** Aceito (v1.9.0).
- **Contexto:** `init.sql` estava com tabelas duplicadas, sem `credito` em
  `clientes`, sem `movimentacoes` (consultada mas não criada).
- **Decisão:** Runner simples em `src/banco_de_dados/migrations.rs`.
  Cada migration é um `.sql` com `include_str!`. Tabela `schema_migrations`
  rastreia o que foi aplicado.
- **Consequência:** Reproduzível. Em produção, deployments determinísticos.
- **Próximo passo (Sprint 1.5):** considerar `sqlx-migrate` ou `refinery`
  para migrations com `down`.

### ADR-006: Login com anti-enumeração por timing
- **Status:** Aceito (v1.9.0).
- **Contexto:** Usuário inexistente retornava imediatamente; existente
  fazia `bcrypt::verify` (~200ms). Atacante enumerava usernames.
- **Decisão:** Quando usuário não existe, rodar `bcrypt::verify` contra um
  hash dummy. Tempo indistinguível.
- **Consequência:** Timing attack prevenido. Custo: ~200ms mesmo para
  usuários inexistentes.

### ADR-007: Health check dividido em /livez e /readyz
- **Status:** Aceito (v1.9.0).
- **Contexto:** `/healthz` antigo sempre retornava "ok" mesmo se DB caísse.
- **Decisão:** `/livez` (sempre 200, sem dep externa) e `/readyz` (faz
  `SELECT 1` real, retorna 503 se falhar).
- **Consequência:** Load balancers distinguem "vivo" de "pronto".

### ADR-008: Bootstrap admin via CLI separado
- **Status:** Aceito (v1.9.0).
- **Contexto:** Servidor auto-criava admin/admin no startup (P0 crítico).
- **Decisão:** Binário `senior-system-admin` para criar o primeiro admin.
  Senha via env (`--password-env`) ou stdin (sem eco).
- **Consequência:** Sem senha default. Operador precisa provisionar.

### ADR-009: Migrations rodam em ambos (Docker init.sql + Rust runner)
- **Status:** Aceito (v1.9.0).
- **Contexto:** Em dev, container MySQL roda `init.sql` na primeira vez.
  Em prod, queremos migrations reproduzíveis sem depender do volume.
- **Decisão:** `init.sql` é o **bootstrap** (cria DB + `schema_migrations`).
  Migrations são reaplicadas pelo runner Rust se volume persistir.
- **Consequência:** Idempotente. Em dev (volume novo) e prod (volume
  existente), o sistema chega ao mesmo estado.

## 5. Domínio — regras de negócio

### Usuários
- Papéis: `Administrador`, `Gerencia`, `Tecnico`, `Financeiro`, `Comercial`, `Estoquista`.
- Senha: bcrypt com `DEFAULT_COST`.
- `tenant_id`: coluna preparada para multi-tenant (default 0 = single-tenant).
- Username `UNIQUE` na tabela.
- Username e senha em branco rejeitados (validação server-side).
- Senha mínimo 8 caracteres (validação server-side).
- `last_login_at` rastreável (coluna existe, ainda não escrita).

### Clientes
- `nome`, `email`, `telefone`, `endereco`, `cpf_cnpj`, `credito` (saldo).
- `cpf_cnpj` indexado.
- `tenant_id` preparado.
- `created_at`/`updated_at` em timestamps.

### Ordens de Serviço
- Status: `Aberta`, `EmAndamento`, `AguardandoPeca`, `Finalizada`, `Cancelada`.
- Situação: 10 estados (do `Orcamento` ao `Faturado`).
- Histórico de edições: por campo alterado, usuário, data/hora.
- Peças: pivô `ordem_servico_pecas`.
- Serviços: pivô `ordem_servico_servicos`.
- Lock pessimista na edição (`SELECT ... FOR UPDATE`).
- Concorrência: duas requests simultâneas na mesma OS são serializadas.

### Estoque
- Peças com `estoque_atual` e `estoque_minimo`.
- Toda mudança passa por `movimentos_estoque` (auditoria).
- Tipos de movimento: `Entrada NF`, `Saída OS`, `Ajuste Manual`, `Edicao OS`.
- Concorrência: lock pessimista na peça.

### Orçamentos
- Cliente + lista de items com quantidade, preço unitário, preço total.
- Total no cabeçalho (denormalizado para query rápida).
- Transação atômica (itens + cabeçalho).

## 6. Segurança

### Implementado (P0)
- ✅ Autenticação JWT obrigatória em todas as rotas (exceto `/login`, `/livez`, `/readyz`, `/`).
- ✅ JWT_SECRET obrigatório em release (panic se ausente).
- ✅ Anti-enumeração no login.
- ✅ Mass assignment fechado em `POST /usuarios`.
- ✅ CORS configurável por env (sem `*` em produção).
- ✅ Rate limit em `/login` (5 req/min por IP, configurável).
- ✅ Senha mínimo 8 caracteres (validação server).
- ✅ Erros padronizados (sem stack trace, sem SQL, sem senha em logs).
- ✅ Tokens emitidos com `uid`, `papel`, `tenant`, `exp`, `iat`.

### Pendente (P1 / Sprint 1.5)
- ⏳ Refresh tokens (atualmente token só expira em 24h, sem refresh).
- ⏳ MFA (opcional).
- ⏳ Recuperação de senha (fluxo seguro com token por email).
- ⏳ Audit log completo (tabela + helper) para todas as mutações.
- ⏳ HTTPS no app Rust (hoje depende de proxy reverso).
- ⏳ Rotação automática de segredos.

## 7. Migrations

Aplicadas:
- `0001_initial_schema` — schema inicial limpo (v1.9.0).
- `0002_movimentacoes_clientes` — `movimentacoes` + coluna `credito` + `tenant_id`.

Como adicionar nova migration:
1. Criar `db-init/migrations/NNNN_descricao.sql`.
2. Adicionar entrada em `src/banco_de_dados/migrations.rs` (`MIGRATIONS`).
3. Garantir idempotência (`IF NOT EXISTS`).
4. Testar em banco existente.

## 8. Testes

| Categoria | Cobertura | Localização |
|---|---|---|
| Auth unit | 6 testes | `src/auth.rs` |
| Rate limit unit | 3 testes | `src/rate_limit.rs` |
| Estoque diff integration | 7 testes | `tests/estoque_diff.rs` |
| DB integration | 0 testes | (precisa MySQL — Sprint 1.5) |
| Concorrência | 0 testes | (precisa MySQL — Sprint 1.5) |
| E2E | 0 testes | (precisa MySQL + GUI — Sprint 2) |

**Total atual:** 16 testes, 0 falhas. Cobertura de regra de negócio ainda
muito baixa. Meta: ≥ 60% até o fim do Sprint 2.

## 9. Deploy

### Dev (Docker Compose)
```bash
cp .env.example .env
# Editar .env (especialmente JWT_SECRET — gerar com `openssl rand -base64 64`)
podman-compose up --build
# Em outro terminal:
cargo run --bin senior-system-gui
# Criar primeiro admin:
cargo run --bin senior-system-admin -- check
ADMIN_PASSWORD=$(openssl rand -base64 24) \
  cargo run --bin senior-system-admin -- create-admin --username admin --password-env ADMIN_PASSWORD
```

### Prod
- TLS no proxy reverso (Caddy / nginx / Traefik).
- `APP_ENV=production` no servidor.
- `CORS_ALLOWED_ORIGINS` com lista explícita.
- `JWT_SECRET` em vault / k8s secret (não em arquivo).
- `LOGIN_RATE_LIMIT_PER_MINUTE` ajustado por tráfego esperado.
- Backup automatizado do MySQL (cron + off-site).
- Monitoramento: `/readyz` em load balancer, logs centralizados.

## 10. Limitações conhecidas

- GUI ainda faz chamadas HTTP em **5 telas** sem enviar o token. Essas
  telas vão receber 401 até serem migradas. **P0 está resolvido no
  backend**; a migração do cliente é P1 (Sprint 1.5).
- Estoque não tem validação de `estoque_minimo` (alerta de peça baixa).
- Sem upload de NF-e / foto de equipamento.
- Sem suporte a NF-e impressa.
- Sem multi-tenant (coluna existe mas middleware não injeta).
- Sem licenciamento.

## 11. Roadmap

Veja `AUDITORIA.md` (seção 13) e o CHANGELOG para histórico.

- **Sprint 1.5 (P1):** refresh tokens, audit log, backup, CI, RBAC granular,
  migração das 5 telas restantes, testes de DB.
- **Sprint 2 (P2):** UX, paginação, i18n, multi-tenant, mobile, ERP+CRM.
- **Sprint 3+:** licenciamento, instalador, NF-e, fiscal.

## 12. Pessoas

Este projeto é mantido por:
- RAG (proprietário, decisão de produto)
- Mavis (Lead Architect / DevSecOps / Code Reviewer) — automação IA
