// src/crm/models.rs
//
// Structs de domínio do CRM. Apenas dados + serialização, sem lógica.

use serde::{Deserialize, Serialize};

/// Tipo de evento da timeline.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TipoEventoTimeline {
    ClienteCriado,
    ClienteAtualizado,
    ClienteRemovido,
    ObservacaoAdicionada,
    ObservacaoEditada,
    TagAdicionada,
    TagRemovida,
    ContatoAdicionado,
    ContatoRemovido,
    EquipamentoAdicionado,
    EquipamentoRemovido,
    OrcamentoCriado,
    OrcamentoAtualizado,
    OsCriada,
    OsAtualizada,
    OsFinalizada,
    AnexoAdicionado,
    Outro,
}

impl TipoEventoTimeline {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::ClienteCriado => "CLIENTE_CRIADO",
            Self::ClienteAtualizado => "CLIENTE_ATUALIZADO",
            Self::ClienteRemovido => "CLIENTE_REMOVIDO",
            Self::ObservacaoAdicionada => "OBSERVACAO_ADICIONADA",
            Self::ObservacaoEditada => "OBSERVACAO_EDITADA",
            Self::TagAdicionada => "TAG_ADICIONADA",
            Self::TagRemovida => "TAG_REMOVIDA",
            Self::ContatoAdicionado => "CONTATO_ADICIONADO",
            Self::ContatoRemovido => "CONTATO_REMOVIDO",
            Self::EquipamentoAdicionado => "EQUIPAMENTO_ADICIONADO",
            Self::EquipamentoRemovido => "EQUIPAMENTO_REMOVIDO",
            Self::OrcamentoCriado => "ORCAMENTO_CRIADO",
            Self::OrcamentoAtualizado => "ORCAMENTO_ATUALIZADO",
            Self::OsCriada => "OS_CRIADA",
            Self::OsAtualizada => "OS_ATUALIZADA",
            Self::OsFinalizada => "OS_FINALIZADA",
            Self::AnexoAdicionado => "ANEXO_ADICIONADO",
            Self::Outro => "OUTRO",
        }
    }
}

/// Evento individual na timeline do cliente.
#[derive(Debug, Clone, Serialize, Deserialize, mysql::prelude::FromRow)]
pub struct TimelineEvento {
    pub id: u64,
    pub cliente_id: u32,
    /// Tipo do evento (string do ENUM no DB).
    pub tipo: String,
    pub descricao: String,
    pub payload: Option<serde_json::Value>,
    pub usuario_id: Option<u32>,
    pub username: Option<String>,
    pub data_hora: String,
}

/// Observação interna sobre um cliente.
#[derive(Debug, Clone, Serialize, Deserialize, mysql::prelude::FromRow)]
pub struct Observacao {
    pub id: u32,
    pub cliente_id: u32,
    pub conteudo: String,
    pub autor_id: Option<u32>,
    pub autor_username: Option<String>,
    pub editada: bool,
    pub vezes_editada: u32,
    pub created_at: String,
    pub updated_at: String,
}

/// Tag do sistema (extensível).
#[derive(Debug, Clone, Serialize, Deserialize, mysql::prelude::FromRow)]
pub struct Tag {
    pub id: u32,
    pub nome: String,
    pub cor: String,
    pub descricao: Option<String>,
    pub publica: bool,
    pub created_at: String,
}

/// Contato do cliente.
#[derive(Debug, Clone, Serialize, Deserialize, mysql::prelude::FromRow)]
pub struct Contato {
    pub id: u32,
    pub cliente_id: u32,
    pub tipo: String, // TELEFONE / WHATSAPP / EMAIL / SITE / OUTRO
    pub valor: String,
    pub rotulo: Option<String>,
    pub principal: bool,
    pub observacao: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Equipamento do cliente (estendido com tipo + patrimonio).
#[derive(Debug, Clone, Serialize, Deserialize, mysql::prelude::FromRow)]
pub struct Equipamento {
    pub id: u32,
    pub cliente_id: u32,
    pub descricao: String,
    pub marca: Option<String>,
    pub modelo: Option<String>,
    pub numero_serie: Option<String>,
    pub patrimonio: Option<String>,
    pub observacao: Option<String>,
    pub tipo: String, // NOTEBOOK / DESKTOP / ...
    pub created_at: String,
    pub updated_at: String,
}

/// Dashboard agregado de um cliente.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardCliente {
    pub cliente: serde_json::Value,
    pub tags: Vec<Tag>,
    pub contatos: Vec<Contato>,
    pub equipamentos: Vec<Equipamento>,
    pub observacoes_recentes: Vec<Observacao>,
    pub ultimas_os: Vec<serde_json::Value>,
    pub ultimos_orcamentos: Vec<serde_json::Value>,
    pub timeline: Vec<TimelineEvento>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tipo_evento_as_db_str_mapeia_corretamente() {
        assert_eq!(
            TipoEventoTimeline::ClienteCriado.as_db_str(),
            "CLIENTE_CRIADO"
        );
        assert_eq!(
            TipoEventoTimeline::OsFinalizada.as_db_str(),
            "OS_FINALIZADA"
        );
        assert_eq!(
            TipoEventoTimeline::TagAdicionada.as_db_str(),
            "TAG_ADICIONADA"
        );
        assert_eq!(
            TipoEventoTimeline::EquipamentoRemovido.as_db_str(),
            "EQUIPAMENTO_REMOVIDO"
        );
        assert_eq!(TipoEventoTimeline::Outro.as_db_str(), "OUTRO");
    }

    #[test]
    fn dashboard_serializa_vazio() {
        let d = DashboardCliente {
            cliente: serde_json::json!({"id": 1, "nome": "Test"}),
            tags: vec![],
            contatos: vec![],
            equipamentos: vec![],
            observacoes_recentes: vec![],
            ultimas_os: vec![],
            ultimos_orcamentos: vec![],
            timeline: vec![],
        };
        let s = serde_json::to_string(&d).unwrap();
        assert!(s.contains("\"tags\":[]"));
        assert!(s.contains("\"timeline\":[]"));
        assert!(s.contains("\"nome\":\"Test\""));
    }

    #[test]
    fn tag_serialize_inclui_cor() {
        let t = Tag {
            id: 1,
            nome: "VIP".to_string(),
            cor: "#FFD700".to_string(),
            descricao: Some("Cliente VIP".to_string()),
            publica: true,
            created_at: "2026-08-22T00:00:00.000Z".to_string(),
        };
        let s = serde_json::to_string(&t).unwrap();
        assert!(s.contains("\"cor\":\"#FFD700\""));
        assert!(s.contains("\"nome\":\"VIP\""));
    }

    #[test]
    fn contato_serializa_principal() {
        let c = Contato {
            id: 1,
            cliente_id: 42,
            tipo: "WHATSAPP".to_string(),
            valor: "+5511999999999".to_string(),
            rotulo: Some("comercial".to_string()),
            principal: true,
            observacao: None,
            created_at: "2026-08-22T00:00:00.000Z".to_string(),
            updated_at: "2026-08-22T00:00:00.000Z".to_string(),
        };
        let s = serde_json::to_string(&c).unwrap();
        assert!(s.contains("\"principal\":true"));
        assert!(s.contains("\"WHATSAPP\""));
    }

    #[test]
    fn equipamento_serializa_tipo() {
        let e = Equipamento {
            id: 1,
            cliente_id: 1,
            descricao: "Notebook Dell Inspiron".to_string(),
            marca: Some("Dell".to_string()),
            modelo: Some("Inspiron 15".to_string()),
            numero_serie: Some("SN123".to_string()),
            patrimonio: Some("PAT-001".to_string()),
            observacao: Some("carregador com defeito".to_string()),
            tipo: "NOTEBOOK".to_string(),
            created_at: "2026-08-22T00:00:00.000Z".to_string(),
            updated_at: "2026-08-22T00:00:00.000Z".to_string(),
        };
        let s = serde_json::to_string(&e).unwrap();
        assert!(s.contains("\"patrimonio\":\"PAT-001\""));
        assert!(s.contains("\"NOTEBOOK\""));
    }

    #[test]
    fn timeline_evento_serializa_com_payload() {
        let ev = TimelineEvento {
            id: 1,
            cliente_id: 1,
            tipo: "OS_CRIADA".to_string(),
            descricao: "OS #42 criada".to_string(),
            payload: Some(serde_json::json!({"os_id": 42, "valor": 100.0})),
            usuario_id: Some(1),
            username: Some("admin".to_string()),
            data_hora: "2026-08-22T00:00:00.000Z".to_string(),
        };
        let s = serde_json::to_string(&ev).unwrap();
        assert!(s.contains("\"OS_CRIADA\""));
        assert!(s.contains("\"os_id\":42"));
    }
}
