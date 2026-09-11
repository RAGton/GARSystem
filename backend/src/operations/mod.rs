// src/operations/mod.rs
//
// Módulo Operacional — Sprint P2.4.
//
// Plataforma de gestão operacional:
//   - Workflow engine (state machine centralizada)
//   - Kanban
//   - Agenda técnica
//   - SLA
//   - Alertas
//   - Dashboard executivo
//
// Toda alteração de status de OS **deve** passar por `mover_estado`.
// Nenhum módulo pode alterar `ordens_servico.status` diretamente.

pub mod models;
pub mod repository;
pub mod service;

pub use models::{
    Agenda, Alerta, AlertaOcorrencia, ClienteStats, DashboardExecutivo, EventoAgenda, KanbanCard,
    KanbanColuna, Severidade, SlaCalculo, SlaConfig, StatusEvento, TecnicoStats, TipoEvento,
    TipoSlaEvento, WorkflowDefinicao, WorkflowEstado, WorkflowMovimentacao, WorkflowStatus,
    WorkflowTransicao,
};
pub use service::{
    atualizar_status_evento, calcular_sla_os, criar_evento_agenda, dashboard_executivo,
    detectar_conflitos, kanban, kanban_para_tecnico, listar_alertas_ativos,
    listar_alertas_pendentes, listar_alertas_por_entidade, listar_estados, listar_eventos_agenda,
    listar_transicoes, marcar_alerta_resolvido, marcar_alerta_visualizado, mover_estado,
    obter_evento_agenda, obter_sla_default, obter_status_workflow, remover_evento,
    verificar_alertas, Contexto, ErroOperacao,
};
