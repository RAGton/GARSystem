# Prompt: Relatório Geral do Projeto (GARSystem)

> Use este prompt em QUALQUER IA nova (Claude Code, Gemini CLI, Codex, etc.) para gerar um relatório executivo completo do GARSystem. Cole integralmente abaixo da persona do agente.

---

## INSTRUÇÃO

Faça uma auditoria completa do **GARSystem** (plataforma SaaS multi-tenant em Rust).

Gere um relatório executivo contendo EXATAMENTE estas seções:

---

# 1. Visão Geral

- O que é o projeto
- Objetivo final
- Estágio atual (com número de sprint e % pronto)

# 2. Arquitetura Atual

- Módulos existentes (com LOC e número de arquivos cada)
- Dependências (do `Cargo.toml`)
- Stack utilizada (backend, frontend, DB, segurança)

# 3. Evolução desde o início

Liste, **com números reais extraídos do `git log` e do código**:

- O que existia originalmente
- O que foi implementado (sprint por sprint, com métricas)
- O que foi corrigido (achados forenses, com severidade)
- O que foi refatorado

**PROIBIDO usar estimativas. Apenas números auditáveis.**

# 4. Segurança

Avaliar (status + detalhe):
- Tenant Isolation (rodar `python3 scripts/audit_tenant_id.py`)
- RBAC (verificar `check_perm()` em cada handler de `server.rs`)
- JWT (verificar `Claims { roles, permissions, uid, tenant }`)
- SQL Injection (verificar uso de prepared statements)
- Path Traversal
- BOLA (Broken Object Level Auth)
- Auditoria

Classificar cada vetor como: **Crítico / Alto / Médio / Baixo / OK**.

# 5. Performance

Avaliar:
- Dashboards (quantos estão consolidados? quantos com N+1?)
- Queries (número de queries, índices usados)
- Índices (migrations de índice)
- Gargalos (listar com arquivo:linha)

# 6. Qualidade de Código

Avaliar:
- Complexidade ciclomática (estimada)
- Dívida técnica conhecida
- Acoplamento (lib vs bins)
- Cobertura de testes (lib tests / integration / ignored)

# 7. Status SaaS — Maturidade por Módulo

Avaliar maturidade (0-100) de cada:
- CRM
- ERP Financeiro / Estoque / Compras / Faturamento / Fiscal
- Service Desk (OS Mobile, SLA, Agenda)
- Cotação/Orçamento
- Arquivos & Mídia
- Dashboards
- RBAC
- Multi-Tenant
- Billing
- Self-service / Onboarding
- Admin CLI
- GUI Desktop / Web
- API Externa (OpenAPI)

Calcular **média ponderada final**.

# 8. O que ainda está faltando

Listar:
- Bugs conhecidos (severidade + onde)
- Limitações técnicas
- Pendências (com sprint de origem)
- Riscos de produção (probabilidade × impacto)

# 9. Roadmap

- **Curto prazo (30 dias)**: lista de tarefas concretas
- **Médio prazo (90 dias)**: features
- **Longo prazo (1 ano)**: visão

# 10. Conclusão (Resposta Direta)

Responder objetivamente, **sem opinião, só evidência**:

- Está pronto para produção? (Sim/Não/Parcial — com justificativa)
- Está pronto para SaaS?
- Está pronto para Billing?
- O que bloqueia o próximo passo? (cite a regra do usuário, se houver)

# 11. Anexos

- Como reproduzir esta auditoria (comandos literais)
- Documentos de referência
- Skills permanentes disponíveis

---

## REGRAS DO RELATÓRIO

1. **NUNCA inventar números.** Se não tem no código, marque "não auditado" ou rode o comando.
2. **Rodar ANTES de escrever**:
   ```bash
   cargo check --all-targets
   cargo test --all
   python3 scripts/audit_tenant_id.py
   cargo run --bin audit-rbac-achado1
   ```
3. **Cada afirmação** com % ou "100%" deve ter **evidência executável** (comando + saída).
4. **Comparar com regra do usuário** (se citada na sessão).
5. **Salvar** em `docs/RELATORIO-EXECUTIVO.md` (sobrescrever) + commit (se autorizado).
6. **Tamanho**: ~500 linhas markdown (~16 KB) é o sweet spot. Nem mais, nem menos.

---

## COMANDOS ÚTEIS

```bash
# Estrutura
cd /workspace/GARSystem/.worktrees/p2.6.2a-hardening/
ls src/

# LOC por módulo
for m in crm arquivos cotacao_orcamento os_mobile operations financial rbac empresa banco_de_dados telas; do
  echo "$m: $(find src/$m -name '*.rs' | xargs wc -l 2>/dev/null | tail -1 | awk '{print $1}') linhas"
done

# Migrations
ls db-init/migrations/ | wc -l

# Testes
CARGO_TARGET_DIR=/tmp/cargo-target-p262a cargo test --all

# Auditoria tenant
python3 scripts/audit_tenant_id.py

# Commits
git log --oneline | wc -l

# Documentos
ls docs/*.md | wc -l
```

---

**Versão**: 1.0 (2026-08-25)
**Mantido em**: `prompts/RELATORIO-EXECUTIVO-PROMPT.md`
