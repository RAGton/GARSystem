// src/cotacao_orcamento/models.rs
//
// Structs de domínio do módulo Cotação/Orçamento.

use serde::{Deserialize, Serialize};

/// Status possíveis (cotação E orçamento compartilham o mesmo conjunto,
/// mas o ENUM no DB separa para clareza de auditoria).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StatusCotacao {
    Rascunho,
    AguardandoCotacao,
    Cotado,
    AguardandoAprovacao,
    Aprovado,
    Rejeitado,
    Finalizado,
}

impl StatusCotacao {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Rascunho => "RASCUNHO",
            Self::AguardandoCotacao => "AGUARDANDO_COTACAO",
            Self::Cotado => "COTADO",
            Self::AguardandoAprovacao => "AGUARDANDO_APROVACAO",
            Self::Aprovado => "APROVADO",
            Self::Rejeitado => "REJEITADO",
            Self::Finalizado => "FINALIZADO",
        }
    }

    /// Parse a partir da string do DB.
    pub fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "RASCUNHO" => Some(Self::Rascunho),
            "AGUARDANDO_COTACAO" => Some(Self::AguardandoCotacao),
            "COTADO" => Some(Self::Cotado),
            "AGUARDANDO_APROVACAO" => Some(Self::AguardandoAprovacao),
            "APROVADO" => Some(Self::Aprovado),
            "REJEITADO" => Some(Self::Rejeitado),
            "FINALIZADO" => Some(Self::Finalizado),
            _ => None,
        }
    }

    /// Status permitidos como próximo estado (transição de workflow).
    pub fn pode_transicionar_para(self, novo: StatusCotacao) -> bool {
        use StatusCotacao::*;
        match (self, novo) {
            (Rascunho, AguardandoCotacao) => true,
            (AguardandoCotacao, Cotado) => true,
            (Cotado, AguardandoAprovacao) => true,
            (Cotado, Rascunho) => true, // volta para editar
            (AguardandoAprovacao, Aprovado) => true,
            (AguardandoAprovacao, Rejeitado) => true,
            (Aprovado, Finalizado) => true,
            (Rejeitado, Rascunho) => true, // refaz
            _ => false,
        }
    }
}

/// Status possíveis para Orçamento (subset — sem COTADO).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StatusOrcamento {
    Rascunho,
    AguardandoAprovacao,
    Aprovado,
    Rejeitado,
    Finalizado,
}

impl StatusOrcamento {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Rascunho => "RASCUNHO",
            Self::AguardandoAprovacao => "AGUARDANDO_APROVACAO",
            Self::Aprovado => "APROVADO",
            Self::Rejeitado => "REJEITADO",
            Self::Finalizado => "FINALIZADO",
        }
    }
    pub fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "RASCUNHO" => Some(Self::Rascunho),
            "AGUARDANDO_APROVACAO" => Some(Self::AguardandoAprovacao),
            "APROVADO" => Some(Self::Aprovado),
            "REJEITADO" => Some(Self::Rejeitado),
            "FINALIZADO" => Some(Self::Finalizado),
            _ => None,
        }
    }
}

/// Decisão de aprovação.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Decisao {
    Aprovado,
    Rejeitado,
}

impl Decisao {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Aprovado => "APROVADO",
            Self::Rejeitado => "REJEITADO",
        }
    }
}

/// Tipo de anexo.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TipoAnexo {
    Imagem,
    Pdf,
    Audio,
    Video,
    Outro,
}

impl TipoAnexo {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Imagem => "IMAGEM",
            Self::Pdf => "PDF",
            Self::Audio => "AUDIO",
            Self::Video => "VIDEO",
            Self::Outro => "OUTRO",
        }
    }
    pub fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "IMAGEM" => Some(Self::Imagem),
            "PDF" => Some(Self::Pdf),
            "AUDIO" => Some(Self::Audio),
            "VIDEO" => Some(Self::Video),
            _ => Some(Self::Outro),
        }
    }
}

/// Categoria de item de orçamento.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CategoriaItem {
    Peca,
    Servico,
    Outro,
}

impl CategoriaItem {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            Self::Peca => "PECA",
            Self::Servico => "SERVICO",
            Self::Outro => "OUTRO",
        }
    }
}

/// Item de cotação (estimativa, antes do orçamento final).
#[derive(Debug, Clone, Serialize, Deserialize, mysql::prelude::FromRow)]
pub struct CotacaoItem {
    pub id: u32,
    pub cotacao_id: u32,
    pub nome: String,
    pub quantidade: f64,
    pub valor_estimado: f64,
    pub observacao: Option<String>,
    pub ordem: u32,
}

/// Anexo de cotação.
#[derive(Debug, Clone, Serialize, Deserialize, mysql::prelude::FromRow)]
pub struct CotacaoAnexo {
    pub id: u32,
    pub cotacao_id: u32,
    pub tipo: String,
    pub nome: String,
    pub arquivo_path: String,
    pub tamanho_bytes: Option<i64>,
    pub mime_type: Option<String>,
    pub hash_sha256: Option<String>,
    pub usuario_id: Option<u32>,
    pub username: Option<String>,
    pub data_upload: String,
}

/// Cotação completa (com itens + anexos).
#[derive(Debug, Clone, Serialize, Deserialize, mysql::prelude::FromRow)]
pub struct Cotacao {
    pub id: u32,
    pub os_id: Option<u32>,
    pub cliente_id: u32,
    pub descricao: String,
    pub observacoes: Option<String>,
    pub status: String,
    pub criado_por: Option<u32>,
    pub criado_por_username: Option<String>,
    pub decidido_por: Option<u32>,
    pub decidido_por_username: Option<String>,
    pub decidido_em: Option<String>,
    pub decisao_observacao: Option<String>,
    pub orcamento_id: Option<u32>,
    pub created_at: String,
    pub updated_at: String,
}

/// Cotação com seus itens + anexos embutidos.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CotacaoCompleta {
    #[serde(flatten)]
    pub cotacao: Cotacao,
    pub itens: Vec<CotacaoItem>,
    pub anexos: Vec<CotacaoAnexo>,
}

/// Item de orçamento (versão final, com categoria).
#[derive(Debug, Clone, Serialize, Deserialize, mysql::prelude::FromRow)]
pub struct OrcamentoItem {
    pub id: u32,
    pub orcamento_id: u32,
    pub descricao: String,
    pub quantidade: u32,
    pub preco_unitario: f64,
    pub preco_total: f64,
    pub categoria: String,
    pub peca_id: Option<u32>,
    pub servico_id: Option<u32>,
    pub observacao: Option<String>,
}

/// Orçamento completo.
#[derive(Debug, Clone, Serialize, Deserialize, mysql::prelude::FromRow)]
pub struct Orcamento {
    pub id: u32,
    pub cliente_id: u32,
    pub total: f64,
    pub status: String,
    pub desconto: f64,
    pub subtotal: f64,
    pub impostos_estimado: f64,
    pub observacoes: Option<String>,
    pub cotacao_origem_id: Option<u32>,
    pub criado_por: Option<u32>,
    pub criado_por_username: Option<String>,
    pub decidido_por: Option<u32>,
    pub decidido_por_username: Option<String>,
    pub decidido_em: Option<String>,
    pub decisao_observacao: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Orçamento com itens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrcamentoCompleto {
    #[serde(flatten)]
    pub orcamento: Orcamento,
    pub itens: Vec<OrcamentoItem>,
}

/// Decisão de aprovação registrada.
#[derive(Debug, Clone, Serialize, Deserialize, mysql::prelude::FromRow)]
pub struct Aprovacao {
    pub id: u64,
    pub orcamento_id: u32,
    pub decisao: String,
    pub observacao: Option<String>,
    pub decidido_por: Option<u32>,
    pub decidido_por_username: Option<String>,
    pub data_hora: String,
}

/// Entrada de histórico (transição de status).
#[derive(Debug, Clone, Serialize, Deserialize, mysql::prelude::FromRow)]
pub struct HistoricoEntrada {
    pub id: u64,
    pub orcamento_id: u32,
    pub status_anterior: Option<String>,
    pub status_novo: String,
    pub observacao: Option<String>,
    pub usuario_id: Option<u32>,
    pub username: Option<String>,
    pub data_hora: String,
}

/// Contadores para o dashboard.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DashboardCotaoes {
    pub cotacoes_abertas: u64,
    pub aguardando_aprovacao: u64,
    pub aprovadas: u64,
    pub rejeitadas: u64,
    pub finalizadas: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_as_db_str_mapeia_corretamente() {
        assert_eq!(StatusCotacao::Rascunho.as_db_str(), "RASCUNHO");
        assert_eq!(StatusCotacao::Finalizado.as_db_str(), "FINALIZADO");
        assert_eq!(
            StatusCotacao::AguardandoCotacao.as_db_str(),
            "AGUARDANDO_COTACAO"
        );
    }

    #[test]
    fn status_from_db_str() {
        assert_eq!(
            StatusCotacao::from_db_str("RASCUNHO"),
            Some(StatusCotacao::Rascunho)
        );
        assert_eq!(StatusCotacao::from_db_str("INVALIDO"), None);
    }

    #[test]
    fn transicoes_validas() {
        use StatusCotacao::*;
        // Caminho feliz
        assert!(Rascunho.pode_transicionar_para(AguardandoCotacao));
        assert!(AguardandoCotacao.pode_transicionar_para(Cotado));
        assert!(Cotado.pode_transicionar_para(AguardandoAprovacao));
        assert!(AguardandoAprovacao.pode_transicionar_para(Aprovado));
        assert!(Aprovado.pode_transicionar_para(Finalizado));
        // Voltas permitidas
        assert!(Cotado.pode_transicionar_para(Rascunho));
        assert!(Rejeitado.pode_transicionar_para(Rascunho));
        // Transições inválidas
        assert!(!Rascunho.pode_transicionar_para(Aprovado));
        assert!(!Finalizado.pode_transicionar_para(Rascunho));
        assert!(!Aprovado.pode_transicionar_para(Rejeitado));
    }

    #[test]
    fn tipo_anexo_as_db_str() {
        assert_eq!(TipoAnexo::Imagem.as_db_str(), "IMAGEM");
        assert_eq!(TipoAnexo::Pdf.as_db_str(), "PDF");
        assert_eq!(TipoAnexo::Audio.as_db_str(), "AUDIO");
        assert_eq!(TipoAnexo::Video.as_db_str(), "VIDEO");
    }

    #[test]
    fn decisao_as_db_str() {
        assert_eq!(Decisao::Aprovado.as_db_str(), "APROVADO");
        assert_eq!(Decisao::Rejeitado.as_db_str(), "REJEITADO");
    }

    #[test]
    fn cotacao_serializa_basico() {
        let c = Cotacao {
            id: 1,
            os_id: None,
            cliente_id: 42,
            descricao: "Notebook não liga".to_string(),
            observacoes: Some("cliente trouxe sem carregador".to_string()),
            status: "COTADO".to_string(),
            criado_por: Some(1),
            criado_por_username: Some("admin".to_string()),
            decidido_por: None,
            decidido_por_username: None,
            decidido_em: None,
            decisao_observacao: None,
            orcamento_id: None,
            created_at: "2026-08-22T00:00:00.000Z".to_string(),
            updated_at: "2026-08-22T00:00:00.000Z".to_string(),
        };
        let s = serde_json::to_string(&c).unwrap();
        assert!(s.contains("\"status\":\"COTADO\""));
        assert!(s.contains("\"cliente_id\":42"));
    }

    #[test]
    fn orcamento_serializa_com_valores() {
        let o = Orcamento {
            id: 1,
            cliente_id: 1,
            total: 1500.0,
            status: "AGUARDANDO_APROVACAO".to_string(),
            desconto: 50.0,
            subtotal: 1550.0,
            impostos_estimado: 0.0,
            observacoes: None,
            cotacao_origem_id: Some(7),
            criado_por: Some(1),
            criado_por_username: Some("tecnico1".to_string()),
            decidido_por: None,
            decidido_por_username: None,
            decidido_em: None,
            decisao_observacao: None,
            created_at: "2026-08-22T00:00:00.000Z".to_string(),
            updated_at: "2026-08-22T00:00:00.000Z".to_string(),
        };
        let s = serde_json::to_string(&o).unwrap();
        assert!(s.contains("\"total\":1500"));
        assert!(s.contains("\"desconto\":50"));
        assert!(s.contains("\"cotacao_origem_id\":7"));
    }
}
