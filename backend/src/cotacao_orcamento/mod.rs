// src/cotacao_orcamento/mod.rs
//
// Módulo Cotação e Orçamento — Sprint P2.2.
//
// Pensado para uso do técnico em campo (mobile-first).
//
// Estrutura:
//   - models       → structs de domínio
//   - repository   → acesso DB (sem regra de negócio)
//   - service      → workflow + regras + audit
//
// Workflow completo:
//
//   [RASCUNHO] ─────criar──────> [AGUARDANDO_COTACAO]
//                                  │
//                                  │ técnico preenche itens + anexos
//                                  ▼
//                              [COTADO]
//                                  │
//                                  │ solicita aprovação ao cliente
//                                  ▼
//                              [AGUARDANDO_APROVACAO]
//                                  │
//                  ┌───────────────┴───────────────┐
//                  ▼                               ▼
//              [APROVADO]                      [REJEITADO]
//                  │                               │
//                  ▼                               ▼
//              [FINALIZADO]                  [RASCUNHO] (volta, edita)
//
// Eventos de timeline (CRM): COTACAO_CRIADA, ORCAMENTO_CRIADO,
//   ORCAMENTO_APROVADO, ORCAMENTO_REJEITADO, etc.

pub mod models;
pub mod repository;
pub mod service;
