# 🗺️ Roadmap de Produto — GAR System

> Visão de longo prazo, fases, features e direção estratégica.
> Última atualização: 2026-10-10

---

## 🎯 Visão

Transformar o **GAR System** em uma plataforma **ERP + CRM + OaaS (Outcome-as-a-Service)**
premium para o nicho de **assistência técnica, lojas de informática e pequenos comércios**.

Diferencial: **IA local + automação + UX Linear/Stripe-grade** num sistema que cabe em
máquinas modestas (8 GB RAM mínimo, recomendado 16 GB).

---

## 📍 Onde estamos hoje (2026-10-10)

| Status | Detalhe |
|---|---|
| **Versão atual** | v1.11.3 (tag pushed em master) |
| **Stack** | Rust 1.98 + Axum 0.7 + eframe 0.33 (GUI) + MySQL 8.0 |
| **Módulos funcionais** | 11 (auth, rbac, empresa, crm, cotação/orçamento, OS, estoque, financeiro, operações, storage, transcription) |
| **Telas** | 19 + componentes (sidebar, logo, ui_kit) |
| **Endpoints REST** | 19+ handlers, 11 permissões granulares |
| **Smoke test** | 9/9 endpoints OK (login + CRUD cliente + CRUD OS) |
| **Bugs P0** | Resolvidos (500 do mysql_null, GUI glutin Xwayland, POST /ordens DTO) |
| **Bugs P1** | Pendentes (POST /orcamentos DTO, audit auth/cookies, tenant isolation) |

---

## 🚀 Roadmap por Fase

### FASE 0 — Fundação (✅ CONCLUÍDA — v1.10.0 → v1.11.3)
- [x] Rebranding `senior_system` → `gar_system`
- [x] Tema dark GAR (cyan + blue + silver + ink)
- [x] Login redesign (gradient + mosaico + card ativação)
- [x] Inter typography embedada
- [x] Práticas Emil Ernerfeldt (warn_if_debug_build, let-else, docstrings)
- [x] Fix P0: admin role + permissions (migration 0017)
- [x] Fix P0: 500 no GET /ordens (mysql_common Value::Null)
- [x] Fix GUI: feature `glow` do eframe (Xwayland rootless)
- [x] DTO leve POST /ordens (CriarOrdemPayload)

### FASE 1 — UI Premium (🔜 PRÓXIMA, 4-8 semanas)
**Inspiração:** Linear, Stripe, Vercel, Salesforce Lightning, Hubspot
- [ ] Refactor de **todas as 19 telas** com tema unificado
- [ ] Componentes profissionais:
  - [ ] **Modais** (única escolha marcada pelo usuário)
  - [ ] **Toast + inline** (mix inteligente de feedback)
  - [ ] **Tabelas modernas** (sortable, filterable, paginated, dark)
  - [ ] **Cards hierárquicos** (header, body, footer consistentes)
  - [ ] **Formulários** (validação inline, estados disabled, focus rings)
  - [ ] **Sidebars retráteis** com sub-menus
  - [ ] **Tooltips + keyboard shortcuts**
- [ ] Tema dark GAR v2: contraste WCAG AA, animações suaves
- [ ] Light theme opcional (switch no header + persistência)
- [ ] **Densidade híbrida**: compacto em listas, espaçoso em forms
- [ ] **Estados vazios** profissionais ("Nenhum cliente cadastrado" + CTA)
- [ ] **Loading states** com skeletons (não spinners genéricos)
- [ ] **Confetti** em ações de sucesso (criar OS, fechar venda)

### FASE 2 — Módulos Reais (4-6 semanas)
- [ ] **CRM Kanban**: pipeline visual (Lead → Qualificação → Proposta → Fechado)
- [ ] **OS Kanban**: Abertas → Atendimento → Aguardando → Concluídas → Faturadas
- [ ] **Dashboard executivo** com KPIs REAIS do banco:
  - [ ] OS abertas, em atraso, SLA em risco
  - [ ] Clientes ativos, técnicos online
  - [ ] Receita do período, inadimplência
  - [ ] Boletos vencidos, próximos do vencimento
- [ ] **Timeline** na OS (criada → atribuída → atendimento → finalizada → faturada)
- [ ] **Histórico de edições** com undo/redo
- [ ] **Busca global** (Cmd+K) por clientes, OS, produtos

### FASE 3 — OaaS (Outcome-as-a-Service) (8-12 semanas)
**Mudança de modelo de negócio:** vender RESULTADO, não ferramenta.
- [ ] **Agentes autônomos** em background:
  - [ ] Conciliação fiscal automática
  - [ ] Auditoria dupla de estoque (IA + humano)
  - [ ] Detecção de inconsistências em NF-e (auto-correção)
  - [ ] Sugestão de peças baseado em OS similar
- [ ] **Métricas de outcome** visíveis pro cliente:
  - [ ] "99.2% das OSs foram concluídas no prazo"
  - [ ] "0.3% de divergência de estoque"
  - [ ] "100% das NFes foram conciliadas sem erro humano"
- [ ] **Pricing por outcome** (não por usuário):
  - [ ] Cobrança por OS concluída no prazo
  - [ ] Bonus por SLA 100%
  - [ ] Penalty por divergência > 0.5%

### FASE 4 — Mobile (12-16 semanas)
**App nativo Flutter ou React Native** (ou PWA como MVP)
- [ ] **Cotação móvel do técnico** (ideia original preservada):
  - [ ] Abrir OS pelo celular
  - [ ] Tirar foto do componente defeituoso
  - [ ] Gravar áudio da necessidade
  - [ ] Transcrição automática (Whisper local ou API)
  - [ ] Comparar opções de fornecedor
  - [ ] Enviar cotação pro cliente aprovar
- [ ] **Conferência de estoque** pelo celular:
  - [ ] Tirar foto de prateleira
  - [ ] IA monta inventário corrigido
  - [ ] Compara com sistema, marca divergências
- [ ] **Aprovação de orçamento** pelo cliente (link WhatsApp)

### FASE 5 — Integrações (16-20 semanas)
- [ ] **WhatsApp Business API** (via Meta Cloud API ou Z-API):
  - [ ] Mensagens transacionais automáticas
    - [ ] Boleto vencendo em 1-3 dias
    - [ ] OS pronta pra retirada
    - [ ] Pesquisa de satisfação pós-venda
  - [ ] **Bot conversacional** que sugere produtos baseado em OS
  - [ ] Atrelar conversa ao cliente (via número de telefone)
  - [ ] Auto-criar OS/venda a partir do WhatsApp
- [ ] **NF-e (Nota Fiscal Eletrônica)**:
  - [ ] Emissão automática
  - [ ] Auto-correção de inconsistências
  - [ ] Retry inteligente em caso de falha SEFAZ
  - [ ] Sincronização de horário NTP (nunca deixar máquina com hora errada)
- [ ] **Impressoras plug-and-play**:
  - [ ] Detectar modelo conectado (USB/rede)
  - [ ] Escolher melhor formato PDF (etiqueta, cupom, OS, NF-e)
  - [ ] Cache de drivers conhecidos
- [ ] **Pagamentos** (Pix, cartão, boleto):
  - [ ] Integração com adquirentes (Stone, Cielo, Rede)
  - [ ] Pix automático (Open Finance)

### FASE 6 — IA Local + Cloud Hybrid (20-24 semanas)
**Cuidado:** IA local é cara em RAM (20 GB+ pra 50 usuários simultâneos).
- [ ] **Estratégia híbrida**:
  - [ ] LLM pequeno local (Phi-3, Llama-3 8B quantized) pra tarefas simples
  - [ ] Cloud API (OpenAI/Anthropic) pra tarefas pesadas (quando cliente permitir)
  - [ ] Fila de prioridade: tarefas offline → local → cloud
- [ ] **Casos de uso**:
  - [ ] Sugestão de peças em OS (lookup semântico no estoque)
  - [ ] Auto-categorização de produtos
  - [ ] Detecção de anomalias em padrões de venda
  - [ ] Geração de descrição de produto a partir de foto
- [ ] **Hardware requirement** (especificado pro cliente):
  - [ ] Mínimo: GPU dedicada 8GB VRAM (RTX 3060+)
  - [ ] Recomendado: GPU 12GB+ (RTX 4070+)
  - [ ] Storage: 100 GB só pra modelos
  - [ ] **Servidor dedicado** (não roda junto com ERP)

### FASE 7 — Venda como SaaS (24+ semanas)
- [ ] **Multi-tenant escalável** (já temos base, falta hardening)
- [ ] **Onboarding wizard** (10 passos pro cliente configurar)
- [ ] **Billing automático** (Stripe / Iugu / Asaas)
- [ ] **Suporte in-app** (chat + ticket system)
- [ ] **Marketing site** (landing + pricing + case studies)
- [ ] **Documentação pública** (docs.garsystem.com)

---

## 💡 Ideias Backlog (validadas mas sem fase definida)

### Curto prazo (qualquer fase)
- [ ] **Sistema de pontos/crédito** pro cliente (cliente que compra muito ganha desconto)
- [ ] **Programa de fidelidade** (cashback, indicação)
- [ ] **Mensagem integrada** ao invés de 2 sistemas de conversa
- [ ] **Cotação separada de orçamento** (3 processos distintos: fornecedor / cliente / compra)
- [ ] **Auto-approve** de compras pequenas (até R$ 50 sem aprovação humana)
- [ ] **Histórico de edições auditável** (quem mudou o quê, quando)

### Médio prazo
- [ ] **BI embutido** (dashboards analíticos, não só operacionais)
- [ ] **Comparador de fornecedores** em tempo real (integração com B2B)
- [ ] **Catálogo digital** (cliente consulta produtos pelo link público)
- [ ] **Marketplace** (vender pra clientes finais via WhatsApp)
- [ ] **Reutilização de OS** (transformar orçamento aprovado em OS com 1 click)

### Longo prazo
- [ ] **Rede neural de sugestão** (cliente que comprou X tende a precisar Y em 6 meses)
- [ ] **API pública** (clientes B2B integram via REST)
- [ ] **White-label** (revender pra outras assistências técnicas)
- [ ] **App marketplace** (plugins de terceiros)

---

## ❌ Fora de escopo (decidido NÃO fazer)

- **Mobile-first redesign** do ERP atual (manter desktop, fazer app separado na Fase 4)
- **Blockchain** (sem caso de uso real pro nicho)
- **Crypto** (Brasil não tem demanda)
- **Realidade aumentada** (custo de implementação > valor agregado)
- **Voice-only interface** (UX ainda é cedo pro mercado brasileiro)
- **Migrar pra web** (egui é estável, frontend web é fase 7+)

---

## 📚 Referências

- `docs/ui-spec/` — Especificação detalhada da UI Premium (Fase 1)
- `docs/architecture/` — Decisões técnicas, RBAC, multi-tenant
- `docs/security/` — Auditorias e threat model
- `docs/process/` — Workflow de desenvolvimento (Superpowers, Git, releases)
- `docs/archive/` — Decisões antigas preservadas (Auditoria.md, Context.md)
