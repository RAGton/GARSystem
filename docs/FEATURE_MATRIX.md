# FEATURE_MATRIX

Classificação de cada tela/ação real do GAR System. Atualizado a cada ciclo.

## Legenda
- `FUNCIONAL` — funciona, testado com smoke
- `PARCIAL` — funciona mas com gaps
- `QUEBRADA` — existe código mas falha em uso
- `MOCK` — só visual, sem backend
- `SEM AÇÃO` — botão sem handler
- `NÃO TESTADO` — não exercitado ainda

---

## Backend (Axum) — 10 rotas + 19 handlers

| Rota | Método | Permissão RBAC | Status |
|---|---|---|---|
| `/` | GET | (público) | FUNCIONAL (retorna string) |
| `/livez` | GET | (público) | FUNCIONAL |
| `/readyz` | GET | (público) | FUNCIONAL |
| `/healthz` | GET | (público) | FUNCIONAL |
| `/login` | POST | (público) | FUNCIONAL (com anti-enumeração) |
| `/usuarios` | GET | `empresa.usuario.list` | FUNCIONAL |
| `POST /usuarios` | POST | `empresa.usuario.create` | FUNCIONAL |
| `/usuarios/{username}/papel` | PUT | `empresa.usuario.assign_role` | FUNCIONAL |
| `/clientes` | GET | `crm.cliente.view` | FUNCIONAL |
| `POST /clientes` | POST | `crm.cliente.create` | FUNCIONAL |
| `/clientes/{id}/resumo` | GET | `crm.cliente.view` | FUNCIONAL |
| `/pecas` | GET | `estoque.peca.view` | FUNCIONAL |
| `/servicos` | GET | (sem check_perm) | PARCIAL — falta RBAC |
| `POST /servicos` | POST | (sem check_perm) | PARCIAL — falta RBAC |
| `/orcamentos` | POST | `orcamento.create` | FUNCIONAL |
| `/orcamentos/{id}` | GET | `orcamento.view` | FUNCIONAL |
| `/ordens` | GET | `os.view` | FUNCIONAL |
| `POST /ordens` | POST | `os.create` | FUNCIONAL |
| `/ordens/{id}` | GET | `os.view` | FUNCIONAL |
| `PUT /ordens/{id}` | PUT | `os.edit` | FUNCIONAL |

**Notas:**
- `/servicos` (GET/POST) **não tem `check_perm`** — gap P0 a corrigir
- Cobertura de 11 permissões granulares em 14 handlers sensíveis (5 não-RBAC, ok)

---

## Telas GUI (egui) — 15 telas com update()

| Tela | LOC | Status conhecido | Bloqueio |
|---|---|---|---|
| `painel_principal` (Dashboard) | 478 | PARCIAL — erro "Permissão negada: os.view" | dependia de F1 do MVP, F3 da auditoria |
| `painel_adm` (Admin Utilizadores) | 372 | NÃO TESTADO | sem smoke |
| `painel_clientes` | 273 | NÃO TESTADO | sem smoke |
| `painel_comercial` | 27 | PARCIAL — quase vazio | incompleto |
| `painel_crm` | 107 | NÃO TESTADO | sem smoke |
| `painel_estoque` | 293 | NÃO TESTADO | sem smoke |
| `painel_financeiro` | 119 | NÃO TESTADO | sem smoke |
| `painel_gerencia` | 26 | PARCIAL — placeholder | incompleto |
| `painel_orcamentos` | 217 | NÃO TESTADO | sem smoke |
| `painel_ordens` | 311 | NÃO TESTADO | sem smoke |
| `painel_os_criar` | 471 | NÃO TESTADO | sem smoke |
| `painel_os_edicao` | 812 (maior) | NÃO TESTADO | sem smoke |
| `painel_servicos` | 84 | NÃO TESTADO | sem smoke |
| `painel_tecnico` | 118 | NÃO TESTADO | sem smoke |
| `painel_tecnico_dashboard` | 111 | NÃO TESTADO | sem smoke |
| `login` | 595 | FUNCIONAL (refactor 2026-10-09) | — |
| `configuracao` | 193 | NÃO TESTADO | sem smoke |

---

## Auth & Sessão

- `src/auth.rs` (452 LOC) — JWT com `jsonwebtoken`, bcrypt para senhas
- Token armazenado em `Arc<Mutex<Option<String>>>` no GUI
- Anti-enumeração: resposta genérica para 4xx de login
- Hardening P0 feito conforme `P0-REPORT.md`

**Próximo:** auditar fluxo de expiração, refresh, logout server-side.

---

## Migrations SQL — 16 arquivos

| # | Nome | LOC | Risco |
|---|---|---|---|
| 0001 | initial_schema | 13.340 (chars) | baixo |
| 0002 | movimentacoes_clientes | 3.281 | baixo |
| 0003 | audit_log | 1.539 | baixo |
| 0004 | crm | 7.819 | baixo |
| 0005 | equipamentos_expand | 1.721 | baixo |
| 0006 | quote_order | 8.094 | baixo |
| 0007 | files | 4.997 | baixo |
| 0008 | os_mobile | 8.314 | baixo |
| 0009 | operations | 14.932 | baixo |
| 0010 | financial | 11.674 | baixo |
| 0011 | empresa_rbac | 27.607 | baixo |
| 0012 | tenant_id | 15.305 | alto (modelo legado, em transição) |
| 0013 | unique_constraints_per_empresa | 2.841 | baixo |
| 0014 | indexes_performance | 4.468 | baixo |
| 0015 | tenant_performance_indexes | 2.598 | baixo |
| 0016 | financeiro_tenant_id | 2.542 | alto (idem 0012) |

**Notas:** 0012-0016 adicionaram `tenant_id` em modelo legado. Migration runner tem problema com DELIMITER (vide histórico da sessão).
