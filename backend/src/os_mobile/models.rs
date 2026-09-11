// src/os_mobile/models.rs
//
// Sprint P2.3 — OS Mobile + Captura de Campo.
//
// Domínio:
//   - Checklist templates (reutilizáveis)
//   - Checklist aplicado a uma OS
//   - Evoluções de OS (entradas de histórico)
//   - Assinaturas do cliente (placeholder — captura real em P2.3.x)
//   - Dashboard do técnico (agregações)

use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TipoEvolucao {
    Status,
    Observacao,
    Problema,
    Solucao,
    Outro,
}

impl TipoEvolucao {
    pub fn as_db_str(self) -> &'static str {
        match self {
            TipoEvolucao::Status => "STATUS",
            TipoEvolucao::Observacao => "OBSERVACAO",
            TipoEvolucao::Problema => "PROBLEMA",
            TipoEvolucao::Solucao => "SOLUCAO",
            TipoEvolucao::Outro => "OUTRO",
        }
    }
    pub fn from_db_str(s: &str) -> Self {
        match s {
            "STATUS" => TipoEvolucao::Status,
            "PROBLEMA" => TipoEvolucao::Problema,
            "SOLUCAO" => TipoEvolucao::Solucao,
            "OUTRO" => TipoEvolucao::Outro,
            _ => TipoEvolucao::Observacao,
        }
    }
}

impl fmt::Display for TipoEvolucao {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_db_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TipoAcaoAuditoria {
    OsCriada,
    OsAtualizada,
    StatusAlterado,
    ChecklistItemConcluido,
    ChecklistConcluido,
    AnexoEnviado,
    EvolucaoRegistrada,
    AssinaturaRegistrada,
}

impl TipoAcaoAuditoria {
    pub fn as_db_str(self) -> &'static str {
        match self {
            TipoAcaoAuditoria::OsCriada => "OS_CRIADA",
            TipoAcaoAuditoria::OsAtualizada => "OS_ATUALIZADA",
            TipoAcaoAuditoria::StatusAlterado => "STATUS_ALTERADO",
            TipoAcaoAuditoria::ChecklistItemConcluido => "CHECKLIST_ITEM_CONCLUIDO",
            TipoAcaoAuditoria::ChecklistConcluido => "CHECKLIST_CONCLUIDO",
            TipoAcaoAuditoria::AnexoEnviado => "ANEXO_ENVIADO",
            TipoAcaoAuditoria::EvolucaoRegistrada => "EVOLUCAO_REGISTRADA",
            TipoAcaoAuditoria::AssinaturaRegistrada => "ASSINATURA_REGISTRADA",
        }
    }
}

// =============================================================================
// Templates de checklist
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistTemplate {
    pub id: u32,
    pub nome: String,
    pub descricao: Option<String>,
    pub ativo: bool,
    pub data_criacao: DateTime<Utc>,
    pub usuario_criacao_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistTemplateItem {
    pub id: u32,
    pub template_id: u32,
    pub ordem: u32,
    pub texto: String,
    pub obrigatorio: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistTemplateCompleto {
    #[serde(flatten)]
    pub template: ChecklistTemplate,
    pub itens: Vec<ChecklistTemplateItem>,
}

// =============================================================================
// Checklist aplicado a uma OS
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checklist {
    pub id: u32,
    pub os_id: u32,
    pub template_id: Option<u32>,
    pub titulo: String,
    pub data_inicio: Option<DateTime<Utc>>,
    pub data_conclusao: Option<DateTime<Utc>>,
    pub total_itens: u32,
    pub concluidos: u32,
}

impl Checklist {
    pub fn progresso_percentual(&self) -> f32 {
        if self.total_itens == 0 {
            0.0
        } else {
            (self.concluidos as f32 / self.total_itens as f32) * 100.0
        }
    }

    pub fn is_completo(&self) -> bool {
        self.total_itens > 0 && self.concluidos >= self.total_itens
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub id: u32,
    pub checklist_id: u32,
    pub template_item_id: Option<u32>,
    pub ordem: u32,
    pub texto: String,
    pub obrigatorio: bool,
    pub concluido: bool,
    pub data_conclusao: Option<DateTime<Utc>>,
    pub observacao: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistCompleto {
    #[serde(flatten)]
    pub checklist: Checklist,
    pub itens: Vec<ChecklistItem>,
}

// =============================================================================
// Evoluções de OS
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evolucao {
    pub id: u32,
    pub os_id: u32,
    pub usuario_id: u32,
    pub texto: String,
    pub tipo: TipoEvolucao,
    pub data_evolucao: DateTime<Utc>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

// =============================================================================
// Assinaturas (placeholder)
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assinatura {
    pub id: u32,
    pub os_id: u32,
    pub arquivo_id: u32,
    pub nome_assinante: String,
    pub documento_assinante: Option<String>,
    pub data_assinatura: DateTime<Utc>,
    pub ip_assinatura: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub observacao: Option<String>,
}

// =============================================================================
// Auditoria offline
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditoriaCampo {
    pub id: u32,
    pub cliente_device_id: Option<String>,
    pub usuario_id: u32,
    pub tipo_acao: TipoAcaoAuditoria,
    pub os_id: Option<u32>,
    pub checklist_id: Option<u32>,
    pub checklist_item_id: Option<u32>,
    pub evolucao_id: Option<u32>,
    pub arquivo_id: Option<u32>,
    pub assinatura_id: Option<u32>,
    pub data_evento_device: DateTime<Utc>,
    pub data_sincronizacao: DateTime<Utc>,
    pub processado: bool,
    pub payload_json: Option<serde_json::Value>,
}

// =============================================================================
// Dashboard do técnico
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DashboardTecnico {
    pub os_abertas: u32,
    pub os_em_andamento: u32,
    pub os_aguardando_peca: u32,
    pub os_concluidas: u32,
    pub checklists_pendentes: u32,
    pub os_sem_checklist: u32,
    pub proximas_os: Vec<OsResumo>,
    pub checklists_recentes: Vec<ChecklistCompleto>,
}

/// Resumo de OS para listagens mobile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsResumo {
    pub id: u32,
    pub cliente_id: u32,
    pub cliente_nome: String,
    pub equipamento_id: Option<u32>,
    pub equipamento_descricao: Option<String>,
    pub status: String,
    pub descricao: Option<String>,
    pub data_criacao: DateTime<Utc>,
    pub data_inicio: Option<DateTime<Utc>>,
    pub data_conclusao: Option<DateTime<Utc>>,
    pub total_anexos: u32,
    pub total_checklists: u32,
    pub total_evolucoes: u32,
}

// =============================================================================
// Helpers
// =============================================================================

pub fn parse_dt(s: &str) -> DateTime<Utc> {
    // Aceita ISO8601 com ou sem 'Z'
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return dt.with_timezone(&Utc);
    }
    if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc);
    }
    chrono::Utc::now()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tipo_evolucao_roundtrip() {
        for t in [
            TipoEvolucao::Status,
            TipoEvolucao::Observacao,
            TipoEvolucao::Problema,
            TipoEvolucao::Solucao,
            TipoEvolucao::Outro,
        ] {
            assert_eq!(TipoEvolucao::from_db_str(t.as_db_str()), t);
        }
    }

    #[test]
    fn checklist_progresso_vazio() {
        let c = Checklist {
            id: 1,
            os_id: 1,
            template_id: None,
            titulo: "x".into(),
            data_inicio: None,
            data_conclusao: None,
            total_itens: 0,
            concluidos: 0,
        };
        assert_eq!(c.progresso_percentual(), 0.0);
        assert!(!c.is_completo());
    }

    #[test]
    fn checklist_progresso_parcial() {
        let c = Checklist {
            id: 1,
            os_id: 1,
            template_id: None,
            titulo: "x".into(),
            data_inicio: None,
            data_conclusao: None,
            total_itens: 10,
            concluidos: 5,
        };
        assert_eq!(c.progresso_percentual(), 50.0);
        assert!(!c.is_completo());
    }

    #[test]
    fn checklist_progresso_completo() {
        let c = Checklist {
            id: 1,
            os_id: 1,
            template_id: None,
            titulo: "x".into(),
            data_inicio: None,
            data_conclusao: None,
            total_itens: 8,
            concluidos: 8,
        };
        assert_eq!(c.progresso_percentual(), 100.0);
        assert!(c.is_completo());
    }

    #[test]
    fn parse_dt_iso8601() {
        let dt = parse_dt("2026-01-15T10:30:00Z");
        assert_eq!(dt.to_rfc3339(), "2026-01-15T10:30:00+00:00");
    }

    #[test]
    fn parse_dt_formato_banco() {
        let dt = parse_dt("2026-01-15 10:30:00");
        assert_eq!(dt.to_rfc3339(), "2026-01-15T10:30:00+00:00");
    }
}
