// src/os_mobile/mod.rs
//
// Módulo OS Mobile + Captura de Campo — Sprint P2.3.
//
// Backend completo para o técnico em campo:
//   - Checklist templates (reutilizáveis)
//   - Checklist aplicado a uma OS
//   - Evoluções de OS
//   - Assinaturas do cliente (placeholder — captura real em P2.3.x)
//   - Dashboard do técnico
//   - Auditoria offline-first (dedupe por device+evento)
//
// UI mobile real virá em P2.3.x (depende de PLATFORM-DECISION.md).
// Arquitetura documentada em docs/OS-MOBILE-MODULE.md.

pub mod models;
pub mod repository;
pub mod service;

pub use models::{
    Assinatura, AuditoriaCampo, Checklist, ChecklistCompleto, ChecklistItem, ChecklistTemplate,
    ChecklistTemplateCompleto, ChecklistTemplateItem, DashboardTecnico, Evolucao, OsResumo,
    TipoAcaoAuditoria, TipoEvolucao,
};
pub use service::{
    adicionar_item_checklist, adicionar_item_template, criar_checklist, criar_template,
    dashboard_tecnico, listar_auditoria_os, listar_checklists_os, listar_evolucoes,
    listar_os_resumo, listar_templates, marcar_item, obter_assinatura, obter_checklist,
    obter_template, registrar_assinatura, registrar_evolucao, remover_checklist, Contexto,
};
