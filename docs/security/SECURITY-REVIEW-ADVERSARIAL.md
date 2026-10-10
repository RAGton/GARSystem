# SECURITY-REVIEW-ADVERSARIAL — P2.6.2a

> **Data**: 2026-08-24
> **Método**: análise adversarial — tentar QUEBRAR o sistema
> **Postura**: atacante, não defensor

---

## 🚨 ACHADOS CRÍTICOS

### 🔴 **CRÍTICO #1: RBAC completamente quebrado em produção**

**Métrica**: Tokens JWT emitidos pelo `handler_login` têm `roles: vec![]` e `permissions: vec![]`.

**Método de medição**: Análise estática de `src/auth.rs:155-186` (função `criar_token`) e `src/server.rs:488` (chamada em `handler_login`).

**Ferramenta usada**: `grep` + leitura direta do código.

**Arquivo auditado**:
- `src/auth.rs:155-186` — `pub fn criar_token`
- `src/server.rs:488` — `match auth::criar_token(&payload_usuario(&headers).unwrap_or_default(), 0, papel, 0)`

**Evidência**:
```rust
// src/auth.rs:167-176
let claims = Claims {
    sub: usuario.to_owned(),
    uid,                              // ← sempre 0
    papel,
    tenant,                           // ← sempre 0
    roles: vec![],                    // ← SEMPRE VAZIO
    permissions: vec![],              // ← SEMPRE VAZIO
    exp: expiracao as usize,
    iat: agora.timestamp() as usize,
};
```

```rust
// src/server.rs:488
auth::criar_token(&payload_usuario(&headers).unwrap_or_default(), 0, papel, 0)
//                                                                  ↑  ↑
//                                                          uid=0  tenant=0
```

**Resultado**: **BLOQUEADOR** — TODOS os 15 endpoints privados retornam 403 (FORBIDDEN) para usuários não-SUPER_ADMIN, porque `requer_permissao` exige `roles` ou `permissions` que estão sempre vazias.

**Como reproduzir**:
1. Login via `POST /login` (qualquer usuário válido)
2. Capturar token JWT retornado
3. Decodificar (base64): `claims.roles = []`, `claims.permissions = []`
4. Chamar `GET /clientes` com `Authorization: Bearer <token>` → **403 FORBIDDEN**

**Impacto**: Aplicação INUTILIZÁVEL exceto para SUPER_ADMIN. P2.6.2b Billing dependente deste RBAC — **BLOQUEADO**.

**Recomendação**:
1. Modificar `criar_token` para aceitar `Vec<String>` de roles e permissions
2. Modificar `handler_login` para buscar roles/permissions do banco (via `rbac::repository::listar_roles_usuario` e `permissoes_de_usuario`)
3. Adicionar testes que verifiquem roles/permissions NÃO vazios no token

---

### 🟠 **ALTO #2: BOLA — handler_alterar_papel é stub**

**Métrica**: 1 endpoint com Path param de objeto (`username`) ainda não implementado.

**Método de medição**: Análise estática de `src/server.rs:593-608`.

**Ferramenta usada**: `grep` + leitura direta.

**Arquivo auditado**: `src/server.rs:593-608`

**Evidência**:
```rust
async fn handler_alterar_papel(
    claims: Claims,
    Path(username): Path<String>,
    Json(payload): Json<AlterarPapelPayload>,
) -> Result<StatusCode, (StatusCode, AxJson<ErroApi>)> {
    check_perm(&claims, "empresa.usuario.assign_role")?;
    exigir_papel(&claims, &[PapelUsuario::Administrador])?;
    // TODO: implementar `servicos::alterar_papel(username, novo_papel)`.
    // Por enquanto, retornamos 501 para sinalizar o stub.
    Err(erro_padrao(
        "INTERNAL",
        "alterar_papel ainda não implementado",
        None,
    ))
}
```

**Resultado**: Quando implementado, DEVE:
1. Receber `tenant_id` (extraído de `claims`)
2. Validar que o `username` alvo PERTENCE ao mesmo `tenant_id` (defense in depth)
3. Auditar a mudança de papel em `audit_log`

**Impacto**: BOLA potencial quando implementado (admin do tenant A promover usuário do tenant B).

**Recomendação**: Implementar com `tenant_id` na assinatura de `servicos::alterar_papel` e validar ownership.

---

### 🟡 **MÉDIO #3: SQL Injection em `ping_count` (função morta)**

**Métrica**: 1 query construída via `format!` com `tabela` controlado por argumento.

**Método de medição**: `grep -rn 'format!.*SELECT' src/`.

**Ferramenta usada**: `grep`.

**Arquivo auditado**: `src/empresa/repository.rs:207-213`

**Evidência**:
```rust
pub fn ping_count(pool: &mysql::Pool, tabela: &str, tenant_id: i32) -> Result<u32, ErroAplicacao> {
    let query = format!("SELECT COUNT(*) FROM {} WHERE tenant_id = ?", tabela);
    //                                                  ↑
    //                                          Sem sanitização
    let row: Option<(i64,)> = conn.exec_first(query, (tenant_id,))?;
    Ok(row.map(|(c,)| c as u32).unwrap_or(0))
}
```

**Resultado**: Risco BAIXO porque `ping_count` não é chamado em nenhum lugar do código (`grep -rn "ping_count(" src/` retorna só a definição).

**Como reproduzir**: Função não tem callers, então não é explorável.

**Impacto**: Nenhum no estado atual. **Risco futuro** se alguém chamar com input do usuário.

**Recomendação**:
1. Validar `tabela` contra whitelist (`matches!(tabela, "clientes" | "ordens_servico" | ...)`)
2. OU deletar a função se for código morto

---

## ✅ VETORES SEGUROS

### Path Traversal

**Métrica**: Função `juntar_seguro` rejeita `..` e caminhos absolutos, valida canonicalização.

**Método de medição**: Leitura de `src/storage/traits.rs` (`juntar_seguro`).

**Ferramenta usada**: leitura direta.

**Evidência**:
```rust
// src/storage/traits.rs
pub fn juntar_seguro(base: &Path, chave: &str) -> StorageResult<PathBuf> {
    if chave.contains("..") || chave.starts_with('/') {
        return Err(StorageError::PermissaoNegada(...));
    }
    // ... validação de canonicalização
}
```

**Resultado**: ✅ **PROTEGIDO**

---

### SQL Injection em queries dinâmicas

**Métrica**: Queries com `push_str` usam apenas literais (`" AND status = ?"`, `" AND cliente_id = ?"`).

**Método de medição**: `grep -rn 'sql.push_str' src/` + leitura.

**Evidência**:
```rust
// src/cotacao_orcamento/repository.rs:85-95
let mut conds: Vec<String> = Vec::new();
if status.is_some() {
    conds.push("status = :status".into());  // String literal, não user input
}
if cliente_id.is_some() {
    conds.push("cliente_id = :cid".into());  // String literal
}
```

**Resultado**: ✅ **SEGURO** — conds são strings literais do código, não input.

---

### BOLA — endpoints com Path<u32>

**Métrica**: 5 endpoints com `Path<u32>` propagam `tenant_id` end-to-end.

**Método de medição**: Rastreamento handler → service → repository.

**Ferramenta usada**: `grep` + leitura de signatures.

**Evidência** (rastreamento completo):

| Handler | service::X() | tenant_id propagado? |
|---|---|---|
| `handler_obter_orcamento` | `obter_orcamento(tenant, id)` | ✅ |
| `handler_obter_ordem` | `buscar_os_por_id(tenant, id)` | ✅ |
| `handler_resumo_cliente` | `obter_gastos_e_credito(tenant, id)` | ✅ |
| `handler_atualizar_ordem_servico` | `atualizar_os(tenant, &os, &usuario)` | ✅ |
| `handler_alterar_papel` | `alterar_papel(username, novo_papel)` | ❌ **STUB** |

**Resultado**: 4/5 OK, 1/5 stub (ver Achado #2).

---

### Cross-tenant por IDs previsíveis (INT PK)

**Métrica**: Apenas 6 tabelas com PK INT/BIGINT, todas filtram por `tenant_id` no WHERE.

**Método de medição**: Regex em migrations + grep em repositórios.

**Ferramenta usada**: `python` (regex) + `grep`.

**Evidência**:
```
Tabelas com PK INT: 6
- os_checklist_templates (id INT)         ✓ tem tenant_id
- workflow_definicoes (id INT)            ✓ tem tenant_id
- configuracao_financeira (id INT)        ✓ tem tenant_id
- plano_contas (id INT)                   ✓ tem tenant_id
- centros_custo (id INT)                  ✓ tem tenant_id
- contas_receber (id INT)                 ✓ tem tenant_id
```

**Resultado**: ✅ **PROTEGIDO** — 6/6 tabelas com INT PK têm `tenant_id` no WHERE.

---

### Wildcard permissions

**Métrica**: Teste de 19 cenários de wildcards.

**Método de medição**: Implementação Python equivalente ao `requer_permissao` real + 19 casos de teste.

**Ferramenta usada**: Python script.

**Evidência** (todos os 19 cenários testados passam):

```
['financeiro.*']          + financeiro.conta_receber.pagar = True  ✓
['financeiro.*']          + financeiro.outro_recurso       = True  ✓
['crm.*']                 + os.view                        = False ✓ (cross-module bloqueado)
['crm.cliente.view']      + crm.cliente.view               = True  ✓ (exato)
['crm.cliente.view']      + crm.cliente.outro              = False ✓ (sem escala)
['financeiro.*']          + admin.system                   = False ✓
['crm.*']                 + usuarios.delete                = False ✓
['crm.cliente.view']      + crm.*.view                     = False ✓
['crm.cliente']           + crm.cliente.view               = False ✓
['crm']                   + crm                            = True  ✓
['']                      + crm.cliente.view               = False ✓
['crm.*', 'os.view']      + os.edit                        = False ✓
```

**Resultado**: ✅ **SEGURO** — wildcards não vazam entre módulos.

---

### Bypass SUPER_ADMIN

**Métrica**: Bypass intencional para emergências operacionais.

**Método de medição**: Leitura de `src/auth.rs` (função `requer_permissao`).

**Evidência**:
```rust
if claims.roles.iter().any(|r| r == "SUPER_ADMIN") {
    return Ok(());
}
```

**Resultado**: ✅ **INTENCIONAL** — mas depende de `claims.roles` que está VAZIO (ver Achado #1).

**Nota**: Bypass não é explorável porque tokens não têm `roles` populadas.

---

### JWT adulterado

**Métrica**: Validação usa `jsonwebtoken` v9 com `Validation::default()`.

**Método de medição**: Leitura de `src/auth.rs:188-198`.

**Evidência**:
```rust
pub fn validar_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let secret = obter_secret().map_err(...)?;
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}
```

**Resultado**: ✅ **PROTEGIDO** — `jsonwebtoken` v9:
- Rejeita `alg: none` por padrão
- Valida assinatura HMAC/SHA256
- Valida `exp` (expiração)
- **NÃO valida** `iss` (issuer) e `aud` (audience), mas isso é OK se o JWT é só para esta aplicação

---

## Conclusão

| Vetor | Status | Bloqueia produção? |
|---|---|---|
| **RBAC quebrado (Achado #1)** | 🔴 CRÍTICO | **SIM** — aplicação inútil |
| **BOLA em alterar_papel (Achado #2)** | 🟠 ALTO | Não (stub) |
| **SQL Injection em ping_count (Achado #3)** | 🟡 MÉDIO | Não (função morta) |
| Path Traversal | ✅ SEGURO | Não |
| SQL Injection (queries dinâmicas) | ✅ SEGURO | Não |
| BOLA (outros 4 endpoints) | ✅ SEGURO | Não |
| Cross-tenant IDs previsíveis | ✅ SEGURO | Não |
| Wildcard permissions | ✅ SEGURO | Não |
| Bypass SUPER_ADMIN | ✅ INTENCIONAL | Depende de Achado #1 |
| JWT adulterado | ✅ PROTEGIDO | Não |

---

## 🚦 DECISÃO DE GO-LIVE

**STATUS**: 🔴 **NÃO LIBERAR PARA PRODUÇÃO**

**Bloqueador único**: Achado #1 (RBAC quebrado) impede 100% das funcionalidades em produção.

**Esforço para corrigir**:
1. Modificar `auth::criar_token` para aceitar `roles: Vec<String>`, `permissions: Vec<String>`, `uid: i32`, `tenant_id: i32`
2. Modificar `handler_login` para:
   - Buscar `uid` real (não 0)
   - Buscar `tenant_id` real (não 0)
   - Chamar `rbac::repository::listar_roles_usuario(uid, empresa_id)` e `permissoes_de_usuario(uid, empresa_id)`
3. Adicionar teste: `criar_token(...).unwrap()` → `validar_token(token).unwrap()` → `assert!(claims.roles.len() > 0)`
4. Verificar que TODOS os 15 endpoints privados retornam 200 para usuário com permissão correta

**Estimativa**: 1-2 horas + 1 sprint de QA.

---

**P2.6.2a: APROVADO PARCIALMENTE** — segurança de SQL/Path/BOLA OK, mas RBAC está quebrado.

**P2.6.2b Billing**: **AINDA BLOQUEADO** — corrigir Achado #1 antes de qualquer nova funcionalidade.
