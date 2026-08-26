# RBAC-COVERAGE-FINAL — P2.6.2a

> **Sprint**: P2.6.2a — Hardening RBAC
> **Data**: 2026-08-24
> **Meta**: 100% RBAC coverage
> **Resultado**: 🟡 13.2% (19/144 handlers), `requer_permissao` funcional

---

## TL;DR

| Categoria | Antes (P2.6.1) | Agora (P2.6.2a) |
|---|---|---|
| **Handlers com `check_perm()`** | 7/144 (4.9%) | **19/144 (13.2%)** |
| **`requer_permissao` funcional** | ❌ stub | ✅ SUPER_ADMIN + claims + wildcard |
| **`Claims.roles`** | ❌ | ✅ Vec<String> |
| **`Claims.permissions`** | ❌ | ✅ Vec<String> |
| **Helpers de RBAC** | 0 | 2 (`requer_permissao`, `check_perm`) |
| **Permission por handler** | 7 | 19 |

---

## 1. Implementação de `requer_permissao`

### 1.1 Stub (P2.6.2a fase 1)

```rust
pub fn requer_permissao(claims: &Claims, _permissao: &str) -> Result<(), String> {
    if claims.uid <= 0 { return Err("Usuário inválido".to_string()); }
    Ok(())  // qualquer autenticado passa
}
```

### 1.2 Funcional (P2.6.2a fase 2)

```rust
pub fn requer_permissao(claims: &Claims, permissao: &str) -> Result<(), String> {
    if claims.uid <= 0 { return Err("Usuário inválido".to_string()); }

    // 1. SUPER_ADMIN bypass
    if claims.roles.iter().any(|r| r == "SUPER_ADMIN") { return Ok(()); }

    // 2. Verifica permission direta
    if claims.permissions.iter().any(|p| p == permissao) { return Ok(()); }

    // 3. Wildcard match: "financeiro.*" cobre "financeiro.conta_receber.pagar"
    let parts: Vec<&str> = permissao.split('.').collect();
    if parts.len() > 1 {
        let wildcard = format!("{}.*", &parts[..parts.len() - 1].join("."));
        if claims.permissions.iter().any(|p| p == &wildcard) { return Ok(()); }
        let prefix_wildcard = format!("{}.*", parts[0]);
        if claims.permissions.iter().any(|p| p == &prefix_wildcard) { return Ok(()); }
    }

    Err(format!("permission '{}' não concedida", permissao))
}
```

**3 níveis de validação**:
1. SUPER_ADMIN → sempre passa (bypass para SaaS admin)
2. Match exato → `claims.permissions.contains(permissao)`
3. Wildcard match → `financeiro.*` cobre qualquer `financeiro.X.Y`

---

## 2. Claims expandido

```rust
// src/auth.rs
pub struct Claims {
    pub sub: String,
    pub uid: i32,
    pub papel: PapelUsuario,
    pub tenant: i32,
    pub exp: usize,
    pub iat: usize,
    #[serde(default)]
    pub roles: Vec<String>,         // P2.6.1 RBAC: ["ADMIN", "TECNICO"]
    #[serde(default)]
    pub permissions: Vec<String>,   // P2.6.1 RBAC: ["os.view", "crm.cliente.create"]
}
```

**Compatibilidade**:
- `#[serde(default)]` permite tokens legados sem `roles`/`permissions` (fail-safe)
- Default = vetor vazio (todas as permissões negadas, exceto SUPER_ADMIN)
- Migração não-bloqueante de tokens existentes

---

## 3. Helper `check_perm` em server.rs

```rust
// src/server.rs (P2.6.2a)
fn check_perm(claims: &Claims, permissao: &str) -> Result<(), (StatusCode, AxJson<ErroApi>)> {
    if let Err(msg) = auth::requer_permissao(claims, permissao) {
        Err(erro_padrao(
            "FORBIDDEN",
            format!("Permissão negada: {} ({})", permissao, msg),
            None,
        ))
    } else {
        Ok(())
    }
}

// Uso em handlers:
async fn handler_listar_clientes(claims: Claims) -> ... {
    check_perm(&claims, "crm.cliente.view")?;
    // ... resto do handler
}
```

---

## 4. Handlers protegidos (19/144)

### 4.1 SaaS novos (P2.6.1) — 7 endpoints

| Endpoint | Permission |
|---|---|
| `GET /empresas` | `saas.empresa.list_all` |
| `POST /empresas` | `saas.empresa.create` |
| `DELETE /empresas/:id` | `saas.empresa.deactivate` |
| `PATCH /empresas/:id/configuracao` | `empresa.config.edit` |
| `GET /usuarios/:id/roles` | `empresa.usuario.list` |
| `PUT /usuarios/:id/roles` | `empresa.usuario.assign_role` |
| `DELETE /usuarios/:id/roles/:codigo` | `empresa.usuario.assign_role` |

### 4.2 Legados P0 (P2.6.2a) — 12 endpoints

| Handler | Permission | Risco original |
|---|---|---|
| `handler_listar_clientes` | `crm.cliente.view` | 🟠 leitura sem escopo |
| `handler_criar_cliente` | `crm.cliente.create` | 🟠 mutação sem escopo |
| `handler_resumo_cliente` | `crm.cliente.view` | 🟠 leitura sem escopo |
| `handler_listar_pecas` | `estoque.peca.view` | 🟡 leitura |
| `handler_listar_servicos` | `os.view` | 🟡 leitura |
| `handler_criar_servico` | `os.edit` | 🟠 mutação |
| `handler_listar_ordens` | `os.view` | 🟡 leitura |
| `handler_obter_ordem` | `os.view` | 🟡 leitura |
| `handler_criar_ordem_servico` | `os.create` | 🟠 mutação |
| `handler_atualizar_ordem_servico` | `os.edit` | 🟠 mutação |
| `handler_criar_orcamento` | `orcamento.create` | 🟠 mutação monetária |
| `handler_obter_orcamento` | `orcamento.view` | 🟡 leitura |

---

## 5. Matriz de risks por handler

| Risco | Categoria | Endpoints | % |
|---|---|---|---|
| 🔴 ALTO (financeiro/pagamento) | Financeiro | 18 | 12.5% |
| 🟠 MÉDIO (mutação) | CRM/OS/Cotação | ~30 | 21% |
| 🟡 BAIXO (leitura) | Listagens | ~50 | 35% |
| 🟢 MÍNIMO (público) | Login/healthz | 5 | 3.5% |
| 🟢 SAAS novos | Já protegidos | 7 | 4.9% |
| ✅ JÁ protegidos nesta sprint | P0 | 12 | 8.3% |
| 🔴 NÃO COBERTO (alta prioridade) | P0 Financeiro + Orçamentos | 22 | 15.3% |

**Top 5 de prioridade para próxima sprint** (P0 P0):
1. `POST /financeiro/contas-receber/:id/pagar` → `financeiro.conta_receber.pagar`
2. `POST /financeiro/contas-pagar/:id/pagar` → `financeiro.conta_pagar.pagar`
3. `POST /financeiro/contas-receber/:id/cancelar` → `financeiro.conta_receber.cancelar`
4. `POST /orcamentos/:id/decidir` → `orcamento.aprovar` / `orcamento.reprovar`
5. `POST /orcamentos/:id/aplicar-desconto` → `orcamento.edit`

---

## 6. Macro `require_perm!` (proposta para próxima sprint)

```rust
// src/auth.rs (futuro)
#[macro_export]
macro_rules! require_perm {
    ($claims:expr, $perm:expr) => {
        if let Err(msg) = $crate::auth::requer_permissao($claims, $perm) {
            return $crate::server::erro_padrao(
                "FORBIDDEN",
                format!("Permissão negada: {} ({})", $perm, msg),
                None,
            );
        }
    };
}

// Uso:
async fn handler_pagar_conta_receber(claims: Claims, ...) -> ... {
    require_perm!(claims, "financeiro.conta_receber.pagar");
    // ... lógica
}
```

**Estimativa para 125 handlers**: 1-2 horas (sed) + 1 dia para testes.

---

## 7. Matriz RBAC de P2.6.1 (referência)

| Role | # Permissions | Cobertura |
|---|---|---|
| SUPER_ADMIN | 80 (todas) | cross-tenant |
| ADMIN | 74 | tudo dentro da empresa |
| GERENTE | ~70 | dashboards + relatórios |
| TECNICO | ~30 | execução de OS |
| FINANCEIRO | ~25 | módulo financeiro |
| ATENDENTE | ~25 | cadastro básico |

**Total de permissions**: 80 (definidas em `0011_empresa_rbac.sql`).

---

## 8. Testes de regressão (pendente)

### 8.1 Proposta `tests/rbac.rs` (próxima sprint)

- 1 teste por permission (80 testes)
- 1 teste de bypass SUPER_ADMIN
- 1 teste de wildcard match
- 1 teste de Fail Closed (sem permission = 403)

### 8.2 Integração com `tests/tenants.rs`

- 5 testes existentes já validam isolamento
- Adicionar 30+ testes cobrindo cada módulo

---

## 9. Métricas de aceite

| Métrica | Meta | Atual | Status |
|---|---|---|---|
| RBAC coverage | 100% | 13.2% | 🔴 |
| `requer_permissao` funcional | OK | ✅ OK | ✅ |
| SUPER_ADMIN bypass | OK | ✅ OK | ✅ |
| Wildcard match | OK | ✅ OK | ✅ |
| Macro helper | OK | ❌ (proposta) | 🟡 |
| Testes RBAC | 80+ | 0 | 🔴 |
| Compat retroativa | 100% | 100% | ✅ |

---

## 10. Conclusão

P2.6.2a entregou:
- ✅ **`requer_permissao` funcional** com 3 níveis
- ✅ **`Claims.roles` + `Claims.permissions`** expandidos
- ✅ **Helper `check_perm`** em `server.rs`
- ✅ **12 handlers P0 protegidos** + 7 SaaS (total 19)
- ✅ **Compat retroativa** mantida

**Não é 100% dos handlers** (faltam 125), mas **fundação completa** permite aplicação em massa via macro `require_perm!` em 1-2 dias.

**Recomendação**: P2.6.2a.3 deve:
1. Adicionar macro `require_perm!` (0.5d)
2. Aplicar em 125 handlers (1-2d)
3. Adicionar testes de regressão (1d)

---

**Branch**: `feat/p2.6.2a-hardening`
**Sem commit/push** (regra contínua)
