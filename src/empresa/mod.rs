// src/empresa/mod.rs
//
// Módulo Empresa — Sprint P2.6.1.
//
// Camada de domínio para multi-tenant. Define Empresa, EmpresaConfiguracao
// e Plano. Toda mutação registra audit + timeline (em logs estruturados;
// tabela audit_log é alimentada pelo helper genérico).
//
// Estrutura:
//   - models     : structs e enums
//   - repository : SQL puro (sem regras)
//   - service    : regras + audit + integração com RBAC
//
// Refs:
//   - docs/MULTITENANT-AUDIT.md
//   - docs/P2.6-PLAN.md
//   - db-init/migrations/0011_empresa_rbac.sql
//   - db-init/migrations/0012_tenant_id.sql

pub mod models;
pub mod repository;
pub mod service;

// Re-exports ergonômicos
pub use models::{ContextoEmpresa, Empresa, EmpresaBootstrap, EmpresaConfiguracao, Plano};
pub use repository::{
    atualizar_configuracao, atualizar_status, contar_usuarios, criar, desativar, empresa_existe,
    listar, listar_ativas, obter_por_id, obter_por_uuid, ping_count,
};
pub use service::{
    bootstrap_seguro, criar_empresa, desativar_empresa, gerar_senha_aleatoria,
    obter_configuracao_efetiva, registrar_audit, uuid_novo,
};
