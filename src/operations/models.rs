// src/operations/models.rs
//
// Sprint P2.4 — Plataforma de Gestão Operacional.
//
// Domínio:
//   - Workflow engine (state machine centralizada)
//   - Kanban (pipeline de OS)
//   - Agenda técnica
//   - SLA engine
//   - Alertas operacionais
//   - Dashboard executivo

use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

// =============================================================================
// Workflow Engine
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinicao {
    pub id: u32,
    pub nome: String,
    pub descricao: Option<String>,
    pub entidade_tipo: String,
    pub ativo: bool,
    pub data_criacao: DateTime<Utc>,
    pub usuario_criacao_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEstado {
    pub id: u32,
    pub workflow_id: u32,
    pub nome: String,
    pub slug: String,
    pub descricao: Option<String>,
    pub cor: Option<String>,
    pub ordem: u32,
    pub eh_inicial: bool,
    pub eh_final: bool,
    pub requer_responsavel: bool,
    pub bloqueia_sla: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTransicao {
    pub id: u32,
    pub workflow_id: u32,
    pub estado_origem_id: u32,
    pub estado_destino_id: u32,
    pub requer_papel: Option<String>,
    pub exige_motivo: bool,
    pub exige_arquivo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMovimentacao {
    pub id: u32,
    pub workflow_id: u32,
    pub entidade_id: u32,
    pub estado_origem_id: Option<u32>,
    pub estado_destino_id: u32,
    pub usuario_id: Option<u32>,
    pub username: Option<String>,
    pub motivo: Option<String>,
    pub data_movimentacao: DateTime<Utc>,
    pub duracao_no_estado_anterior_segundos: Option<u32>,
    pub ip_origem: Option<String>,
}

/// Estado para o cliente: workflow + estado atual + histórico.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStatus {
    pub workflow: WorkflowDefinicao,
    pub estado_atual: WorkflowEstado,
    pub estados: Vec<WorkflowEstado>,
    pub transicoes: Vec<WorkflowTransicao>,
    pub transicoes_validas: Vec<u32>, // IDs de transições permitidas
    pub historico: Vec<WorkflowMovimentacao>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KanbanColuna {
    pub estado: WorkflowEstado,
    pub transicoes_validas: Vec<u32>, // IDs de WorkflowTransicao
    pub cards: Vec<KanbanCard>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KanbanCard {
    pub os_id: u32,
    pub cliente_nome: String,
    pub descricao: Option<String>,
    pub responsavel_id: Option<u32>,
    pub responsavel_nome: Option<String>,
    pub data_entrada: DateTime<Utc>,
    pub prioridade: Option<String>,
    pub bloqueia_sla: bool,
}

// =============================================================================
// Agenda
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TipoEvento {
    Visita,
    Coleta,
    Entrega,
    Manutencao,
    Retorno,
    Outro,
}

impl TipoEvento {
    pub fn as_db_str(self) -> &'static str {
        match self {
            TipoEvento::Visita => "VISITA",
            TipoEvento::Coleta => "COLETA",
            TipoEvento::Entrega => "ENTREGA",
            TipoEvento::Manutencao => "MANUTENCAO",
            TipoEvento::Retorno => "RETORNO",
            TipoEvento::Outro => "OUTRO",
        }
    }
    pub fn from_db_str(s: &str) -> Self {
        match s {
            "VISITA" => TipoEvento::Visita,
            "COLETA" => TipoEvento::Coleta,
            "ENTREGA" => TipoEvento::Entrega,
            "MANUTENCAO" => TipoEvento::Manutencao,
            "RETORNO" => TipoEvento::Retorno,
            _ => TipoEvento::Outro,
        }
    }
}

impl fmt::Display for TipoEvento {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_db_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StatusEvento {
    Agendado,
    Confirmado,
    EmAndamento,
    Concluido,
    Cancelado,
    Faltou,
}

impl StatusEvento {
    pub fn as_db_str(self) -> &'static str {
        match self {
            StatusEvento::Agendado => "AGENDADO",
            StatusEvento::Confirmado => "CONFIRMADO",
            StatusEvento::EmAndamento => "EM_ANDAMENTO",
            StatusEvento::Concluido => "CONCLUIDO",
            StatusEvento::Cancelado => "CANCELADO",
            StatusEvento::Faltou => "FALTOU",
        }
    }
    pub fn from_db_str(s: &str) -> Self {
        match s {
            "CONFIRMADO" => StatusEvento::Confirmado,
            "EM_ANDAMENTO" => StatusEvento::EmAndamento,
            "CONCLUIDO" => StatusEvento::Concluido,
            "CANCELADO" => StatusEvento::Cancelado,
            "FALTOU" => StatusEvento::Faltou,
            _ => StatusEvento::Agendado,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agenda {
    pub id: u32,
    pub nome: String,
    pub tecnico_id: Option<u32>,
    pub cor: Option<String>,
    pub ativo: bool,
    pub data_criacao: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventoAgenda {
    pub id: u32,
    pub agenda_id: u32,
    pub tecnico_id: u32,
    pub os_id: Option<u32>,
    pub cliente_id: Option<u32>,
    pub titulo: String,
    pub descricao: Option<String>,
    pub tipo: TipoEvento,
    pub status: StatusEvento,
    pub inicio: DateTime<Utc>,
    pub fim: DateTime<Utc>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub endereco: Option<String>,
    pub usuario_criacao_id: u32,
    pub data_criacao: DateTime<Utc>,
    pub data_atualizacao: DateTime<Utc>,
}

impl EventoAgenda {
    pub fn conflita_com(&self, outro: &EventoAgenda) -> bool {
        if self.tecnico_id != outro.tecnico_id || self.id == outro.id {
            return false;
        }
        // Considera conflito mesmo se status é CANCELADO? Não.
        if matches!(self.status, StatusEvento::Cancelado)
            || matches!(outro.status, StatusEvento::Cancelado)
        {
            return false;
        }
        // Interseção de intervalos
        self.inicio < outro.fim && outro.inicio < self.fim
    }
}

// =============================================================================
// SLA
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaConfig {
    pub id: u32,
    pub nome: String,
    pub descricao: Option<String>,
    pub ativo: bool,
    pub max_horas_diagnostico: Option<u32>,
    pub max_horas_execucao: Option<u32>,
    pub max_horas_total: Option<u32>,
    pub max_horas_espera_peca: Option<u32>,
    pub max_horas_espera_aprovacao: Option<u32>,
    pub data_criacao: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TipoSlaEvento {
    Recebido,
    DiagnosticoInicio,
    DiagnosticoFim,
    PecaSolicitada,
    PecaRecebida,
    AprovacaoSolicitada,
    AprovacaoRecebida,
    ExecucaoInicio,
    ExecucaoFim,
    TesteInicio,
    TesteFim,
    Finalizado,
    Entregue,
    Cancelado,
}

impl TipoSlaEvento {
    pub fn as_db_str(self) -> &'static str {
        match self {
            TipoSlaEvento::Recebido => "RECEBIDO",
            TipoSlaEvento::DiagnosticoInicio => "DIAGNOSTICO_INICIO",
            TipoSlaEvento::DiagnosticoFim => "DIAGNOSTICO_FIM",
            TipoSlaEvento::PecaSolicitada => "PECA_SOLICITADA",
            TipoSlaEvento::PecaRecebida => "PECA_RECEBIDA",
            TipoSlaEvento::AprovacaoSolicitada => "APROVACAO_SOLICITADA",
            TipoSlaEvento::AprovacaoRecebida => "APROVACAO_RECEBIDA",
            TipoSlaEvento::ExecucaoInicio => "EXECUCAO_INICIO",
            TipoSlaEvento::ExecucaoFim => "EXECUCAO_FIM",
            TipoSlaEvento::TesteInicio => "TESTE_INICIO",
            TipoSlaEvento::TesteFim => "TESTE_FIM",
            TipoSlaEvento::Finalizado => "FINALIZADO",
            TipoSlaEvento::Entregue => "ENTREGUE",
            TipoSlaEvento::Cancelado => "CANCELADO",
        }
    }
    pub fn from_db_str(s: &str) -> Self {
        match s {
            "RECEBIDO" => TipoSlaEvento::Recebido,
            "DIAGNOSTICO_INICIO" => TipoSlaEvento::DiagnosticoInicio,
            "DIAGNOSTICO_FIM" => TipoSlaEvento::DiagnosticoFim,
            "PECA_SOLICITADA" => TipoSlaEvento::PecaSolicitada,
            "PECA_RECEBIDA" => TipoSlaEvento::PecaRecebida,
            "APROVACAO_SOLICITADA" => TipoSlaEvento::AprovacaoSolicitada,
            "APROVACAO_RECEBIDA" => TipoSlaEvento::AprovacaoRecebida,
            "EXECUCAO_INICIO" => TipoSlaEvento::ExecucaoInicio,
            "EXECUCAO_FIM" => TipoSlaEvento::ExecucaoFim,
            "TESTE_INICIO" => TipoSlaEvento::TesteInicio,
            "TESTE_FIM" => TipoSlaEvento::TesteFim,
            "ENTREGUE" => TipoSlaEvento::Entregue,
            "CANCELADO" => TipoSlaEvento::Cancelado,
            _ => TipoSlaEvento::Recebido,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlaCalculo {
    pub id: u32,
    pub os_id: u32,
    pub sla_config_id: Option<u32>,
    pub tempo_diagnostico_segundos: Option<u32>,
    pub tempo_espera_peca_segundos: Option<u32>,
    pub tempo_espera_aprovacao_segundos: Option<u32>,
    pub tempo_execucao_segundos: Option<u32>,
    pub tempo_total_segundos: Option<u32>,
    pub sla_violado: bool,
    pub motivo_violacao: Option<String>,
    pub data_calculo: DateTime<Utc>,
}

// =============================================================================
// Alertas
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Severidade {
    Info,
    Warning,
    Critical,
}

impl Severidade {
    pub fn as_db_str(self) -> &'static str {
        match self {
            Severidade::Info => "INFO",
            Severidade::Warning => "WARNING",
            Severidade::Critical => "CRITICAL",
        }
    }
    pub fn from_db_str(s: &str) -> Self {
        match s {
            "INFO" => Severidade::Info,
            "CRITICAL" => Severidade::Critical,
            _ => Severidade::Warning,
        }
    }
}

impl fmt::Display for Severidade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_db_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alerta {
    pub id: u32,
    pub codigo: String,
    pub titulo: String,
    pub descricao: Option<String>,
    pub severidade: Severidade,
    pub ativo: bool,
    pub condicao_tipo: String,
    pub condicao_threshold_horas: Option<u32>,
    pub condicao_estado_slug: Option<String>,
    pub entidade_tipo: String,
    pub data_criacao: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertaOcorrencia {
    pub id: u32,
    pub alerta_id: u32,
    pub entidade_id: u32,
    pub data_ocorrencia: DateTime<Utc>,
    pub severidade: Severidade,
    pub mensagem: String,
    pub contexto_json: Option<serde_json::Value>,
    pub visualizado: bool,
    pub data_visualizacao: Option<DateTime<Utc>>,
    pub usuario_visualizacao_id: Option<u32>,
    pub resolvido: bool,
    pub data_resolucao: Option<DateTime<Utc>>,
}

// =============================================================================
// Dashboard Executivo
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DashboardExecutivo {
    pub os_abertas: u32,
    pub os_atrasadas: u32,
    pub os_aguardando_peca: u32,
    pub os_aguardando_aprovacao: u32,
    pub os_em_execucao: u32,
    pub os_concluidas: u32,
    pub os_entregues: u32,
    pub os_total: u32,
    pub alertas_pendentes: u32,
    pub alertas_criticos: u32,
    pub tempo_medio_resolucao_horas: Option<f32>,
    pub tempo_medio_por_tecnico_horas: Option<f32>,
    pub tecnicos: Vec<TecnicoStats>,
    pub top_clientes: Vec<ClienteStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TecnicoStats {
    pub tecnico_id: u32,
    pub tecnico_nome: String,
    pub os_total: u32,
    pub os_concluidas: u32,
    pub tempo_medio_horas: Option<f32>,
    pub taxa_aprovacao: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClienteStats {
    pub cliente_id: u32,
    pub cliente_nome: String,
    pub os_total: u32,
    pub os_concluidas: u32,
    pub tempo_medio_horas: Option<f32>,
}

// =============================================================================
// Helpers
// =============================================================================

pub fn parse_dt(s: &str) -> DateTime<Utc> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return dt.with_timezone(&Utc);
    }
    if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc);
    }
    chrono::Utc::now()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tipo_evento_roundtrip() {
        for t in [
            TipoEvento::Visita,
            TipoEvento::Coleta,
            TipoEvento::Entrega,
            TipoEvento::Manutencao,
            TipoEvento::Retorno,
            TipoEvento::Outro,
        ] {
            assert_eq!(TipoEvento::from_db_str(t.as_db_str()), t);
        }
    }

    #[test]
    fn status_evento_roundtrip() {
        for s in [
            StatusEvento::Agendado,
            StatusEvento::Confirmado,
            StatusEvento::EmAndamento,
            StatusEvento::Concluido,
            StatusEvento::Cancelado,
            StatusEvento::Faltou,
        ] {
            assert_eq!(StatusEvento::from_db_str(s.as_db_str()), s);
        }
    }

    #[test]
    fn severidade_roundtrip() {
        for s in [Severidade::Info, Severidade::Warning, Severidade::Critical] {
            assert_eq!(Severidade::from_db_str(s.as_db_str()), s);
        }
    }

    #[test]
    fn tipo_sla_evento_roundtrip() {
        let t = TipoSlaEvento::DiagnosticoFim;
        assert_eq!(TipoSlaEvento::from_db_str(t.as_db_str()), t);
    }

    #[test]
    fn conflito_eventos_intersectam() {
        use chrono::TimeZone;
        let a = EventoAgenda {
            id: 1,
            agenda_id: 1,
            tecnico_id: 1,
            os_id: None,
            cliente_id: None,
            titulo: "A".into(),
            descricao: None,
            tipo: TipoEvento::Visita,
            status: StatusEvento::Agendado,
            inicio: Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap(),
            fim: Utc.with_ymd_and_hms(2026, 1, 15, 11, 0, 0).unwrap(),
            latitude: None,
            longitude: None,
            endereco: None,
            usuario_criacao_id: 1,
            data_criacao: Utc::now(),
            data_atualizacao: Utc::now(),
        };
        let b = EventoAgenda {
            id: 2,
            inicio: Utc.with_ymd_and_hms(2026, 1, 15, 10, 30, 0).unwrap(),
            fim: Utc.with_ymd_and_hms(2026, 1, 15, 11, 30, 0).unwrap(),
            ..a.clone()
        };
        assert!(a.conflita_com(&b));

        // Sem intersecção
        let c = EventoAgenda {
            inicio: Utc.with_ymd_and_hms(2026, 1, 15, 12, 0, 0).unwrap(),
            fim: Utc.with_ymd_and_hms(2026, 1, 15, 13, 0, 0).unwrap(),
            ..a.clone()
        };
        assert!(!a.conflita_com(&c));

        // Outro técnico não conflita
        let d = EventoAgenda {
            tecnico_id: 2,
            ..b.clone()
        };
        assert!(!a.conflita_com(&d));
    }
}
