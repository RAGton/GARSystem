# 📚 Documentação — GAR System

> Toda documentação do projeto, organizada por pasta e tópico.
> Última reorganização: 2026-10-10 (de 23 arquivos soltos → 8 pastas temáticas)

---

## 🗂️ Estrutura de Pastas

| Pasta | Conteúdo | Quando consultar |
|---|---|---|
| **[`roadmap/`](./roadmap/)** | Visão de produto, fases, features, ideias validadas | Planejando sprint / decidindo prioridade |
| **[`architecture/`](./architecture/)** | Decisões técnicas, RBAC matrix, feature matrix, performance | Modificando código, integrando módulos |
| **[`security/`](./security/)** | Auditorias, threat model, signoffs de segurança | Antes de deploy, mudanças de auth/RBAC |
| **[`operations/`](./operations/)** | Relatórios de execução, deploy, runbooks | Operando o sistema em produção |
| **[`ui-spec/`](./ui-spec/)** | Especificação da UI Premium (Fase 1) | Mudando design/tema/componentes |
| **[`process/`](./process/)** | Workflow de dev, Git, releases, superpowers | Dúvidas sobre como trabalhar |
| **[`reference/`](./reference/)** | Links externos, glossário, cheatsheets | Procurando referência rápida |
| **[`archive/`](./archive/)** | Docs legados preservados (Auditoria, Context) | Contexto histórico, NUNCA deletar |
| **[`superpowers/specs/`](./superpowers/specs/)** | Specs de features (formato Superpowers) | Planejando nova feature |

---

## 🚀 Quickstart — Leitura por papel

### Se tu é **dono do projeto / estratégia de produto**
1. [`roadmap/README.md`](./roadmap/README.md) — visão, fases, ideias
2. [`roadmap/PRODUCTION_EXECUTION_TRACKER.md`](./roadmap/PRODUCTION_EXECUTION_TRACKER.md) — status atual
3. `archive/RELATORIO-EXECUTIVO.md` — overview executivo histórico

### Se tu é **desenvolvedor backend (Rust)**
1. [`architecture/FEATURE_MATRIX.md`](./architecture/FEATURE_MATRIX.md) — mapa de features
2. [`architecture/RBAC-ENDPOINT-MATRIX.md`](./architecture/RBAC-ENDPOINT-MATRIX.md) — quem pode fazer o quê
3. `archive/AUDITORIA.md` — auditoria técnica original
4. `db-init/migrations/` — schema do banco
5. `docs/process/` — workflow de dev

### Se tu é **desenvolvedor frontend (egui/Rust)**
1. [`ui-spec/`](./ui-spec/) — como a UI deve ser
2. `src/telas/theme/` — tokens GAR
3. `src/telas/componentes/ui_kit.rs` — componentes reutilizáveis
4. `archive/DASHBOARD_INTEGRACAO_BANCO.md` — exemplos de dashboards

### Se tu é **auditor de segurança**
1. [`security/`](./security/) — todas as auditorias
2. [`architecture/`](./architecture/) — pra entender o escopo
3. `archive/P0-REPORT.md` — issues P0 históricos

### Se tu é **novo no projeto**
1. [`roadmap/README.md`](./roadmap/README.md) — onde estamos e pra onde vamos
2. [`architecture/FEATURE_MATRIX.md`](./architecture/FEATURE_MATRIX.md) — o que existe
3. `archive/CONTEXT.md` — contexto histórico
4. `README.md` (raiz do projeto) — setup e instalação

---

## 📋 Convenções de documentação

- **Markdown** (.md) pra tudo (legível no GitHub, em IDE, em qualquer lugar)
- **READMEs** em cada pasta servem de índice
- **Datas** sempre em ISO (YYYY-MM-DD) pra ordenação automática
- **Status** em CAPS no início do parágrafo: PASSOU, FALHOU, BLOQUEADO, NÃO TESTADO
- **Links** relativos entre pastas (`../security/`)
- **Specs Superpowers** seguem o formato da skill `superpowers:writing-plans`
- **Nada é deletado** — se ficar obsoleto, vai pra `archive/`

---

## 🔄 Workflow de atualização

| Quando | O que atualizar | Quem |
|---|---|---|
| A cada ciclo de execução | `roadmap/PRODUCTION_EXECUTION_TRACKER.md` | Agente executor |
| Decisão arquitetural nova | `architecture/SPECS-*.md` (criar) | AURA-PLANNER |
| Bug P0/P1 corrigido | `security/AUDIT-*.md` (adicionar entrada) | KORA-ARCHITECT |
| Release nova | `CHANGELOG.md` (raiz) | Release manager |
| Mudança de UI | `ui-spec/SPECS-*.md` | Designer / AURA-PLANNER |
| Lição aprendida | `process/LESSONS-*.md` | Qualquer um |

---

## 🗺️ Mapa visual

```
docs/
├── README.md (este arquivo)        ← VOCÊ ESTÁ AQUI
│
├── roadmap/                        ← Visão de produto
│   ├── README.md
│   ├── PRODUCTION_EXECUTION_TRACKER.md
│   └── DASHBOARDS-BENCHMARK.md
│
├── architecture/                    ← Decisões técnicas
│   ├── FEATURE_MATRIX.md
│   ├── RBAC-ENDPOINT-MATRIX.md
│   └── PERFORMANCE-BEFORE-AFTER.md
│
├── security/                        ← Auditorias
│   ├── AUDIT-2026-10-09.md
│   ├── AUDIT-REAL-P2.6.2a.md
│   ├── SECURITY-REVIEW-*.md
│   ├── HARDENING-REPORT.md
│   ├── RBAC-COVERAGE-*.md
│   └── TENANT-COVERAGE-*.md
│
├── operations/                      ← Deploy, runbooks
│   ├── P2.6.2c-REPORT.md
│   └── RELATORIO-EXECUTIVO.md
│
├── ui-spec/                         ← Especificação UI Premium
│   └── (vazio — será populado na Fase 1)
│
├── process/                         ← Workflow de dev
│   └── (vazio — será populado)
│
├── reference/                       ← Referências externas
│   └── (vazio)
│
├── archive/                         ← Legado (NÃO DELETAR)
│   ├── AUDITORIA.md
│   ├── CONTEXT.md
│   ├── GAR_SYSTEM.md
│   ├── DASHBOARD_INTEGRACAO_BANCO.md
│   └── (outros)
│
└── superpowers/
    └── specs/                       ← Specs de features
        ├── 2026-10-09-elevacao-ui-gar-system.md
        └── 2026-10-09-mvp-login-tema.md
```
