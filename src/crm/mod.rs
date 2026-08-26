// src/crm/mod.rs
//
// Módulo CRM (Customer Relationship Management) — Sprint P2.1.
//
// Primeira feature de produto do ERP. Transforma o cadastro de clientes
// em um histórico completo: timeline, observações, tags, contatos,
// equipamentos.
//
// Estrutura interna:
//   - `models`   → structs de domínio (TimelineEvento, Observacao, Tag, etc)
//   - `repository` → acesso puro ao DB (CRUD sem lógica)
//   - `service`  → regras de negócio (orquestra repository + audit + timeline)
//
// Toda mutação passa por `service::*` que automaticamente:
//   1. Grava no repositório
//   2. Adiciona evento na timeline
//   3. Registra no audit log (best-effort)
//
// Sem camada de handler aqui — handlers ficam em `src/server.rs` ou
// `src/handlers/crm.rs` (separação segue o padrão atual do projeto).

pub mod models;
pub mod repository;
pub mod service;
