// src/arquivos/models.rs
//
// Entidade universal `Arquivo` — Sprint P2.2.1.
//
// Pensada para ser usada por TODO módulo que precise de anexo:
// OS, Cliente, Cotação, Orçamento, Financeiro, Fiscal, etc.
//
// 1 linha na tabela `arquivos` = 1 arquivo físico no storage.
// A mesma linha pode ser "vinculada" a N entidades via
// `arquivo_vinculos` (sem duplicar o arquivo).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TipoArquivo {
    Imagem,
    Pdf,
    Audio,
    Video,
    Outro,
}

impl TipoArquivo {
    pub fn as_db_str(self) -> &'static str {
        match self {
            TipoArquivo::Imagem => "IMAGEM",
            TipoArquivo::Pdf => "PDF",
            TipoArquivo::Audio => "AUDIO",
            TipoArquivo::Video => "VIDEO",
            TipoArquivo::Outro => "OUTRO",
        }
    }

    pub fn from_db_str(s: &str) -> Self {
        match s {
            "IMAGEM" => TipoArquivo::Imagem,
            "PDF" => TipoArquivo::Pdf,
            "AUDIO" => TipoArquivo::Audio,
            "VIDEO" => TipoArquivo::Video,
            _ => TipoArquivo::Outro,
        }
    }

    /// Infere pelo mime type.
    pub fn from_mime(mime: &str) -> Self {
        let m = mime.to_lowercase();
        if m.starts_with("image/") {
            TipoArquivo::Imagem
        } else if m == "application/pdf" {
            TipoArquivo::Pdf
        } else if m.starts_with("audio/") {
            TipoArquivo::Audio
        } else if m.starts_with("video/") {
            TipoArquivo::Video
        } else {
            TipoArquivo::Outro
        }
    }
}

impl fmt::Display for TipoArquivo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_db_str())
    }
}

/// Entidades que podem receber arquivos anexos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TipoEntidade {
    Os,
    Cliente,
    Cotacao,
    Orcamento,
    Equipamento,
    Observacao,
    Contato,
    TimelineEvento,
    Financeiro,
    Fiscal,
    Outro,
}

impl TipoEntidade {
    pub fn as_db_str(self) -> &'static str {
        match self {
            TipoEntidade::Os => "OS",
            TipoEntidade::Cliente => "CLIENTE",
            TipoEntidade::Cotacao => "COTACAO",
            TipoEntidade::Orcamento => "ORCAMENTO",
            TipoEntidade::Equipamento => "EQUIPAMENTO",
            TipoEntidade::Observacao => "OBSERVACAO",
            TipoEntidade::Contato => "CONTATO",
            TipoEntidade::TimelineEvento => "TIMELINE_EVENTO",
            TipoEntidade::Financeiro => "FINANCEIRO",
            TipoEntidade::Fiscal => "FISCAL",
            TipoEntidade::Outro => "OUTRO",
        }
    }

    pub fn from_db_str(s: &str) -> Self {
        match s {
            "OS" => TipoEntidade::Os,
            "CLIENTE" => TipoEntidade::Cliente,
            "COTACAO" => TipoEntidade::Cotacao,
            "ORCAMENTO" => TipoEntidade::Orcamento,
            "EQUIPAMENTO" => TipoEntidade::Equipamento,
            "OBSERVACAO" => TipoEntidade::Observacao,
            "CONTATO" => TipoEntidade::Contato,
            "TIMELINE_EVENTO" => TipoEntidade::TimelineEvento,
            "FINANCEIRO" => TipoEntidade::Financeiro,
            "FISCAL" => TipoEntidade::Fiscal,
            _ => TipoEntidade::Outro,
        }
    }
}

/// Arquivo físico (1 linha por arquivo).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arquivo {
    pub id: u32,
    pub nome_original: String,
    pub nome_armazenado: String,
    pub extensao: String,
    pub mime_type: String,
    pub tamanho: u64,
    pub hash_sha256: String,
    pub tipo: TipoArquivo,
    pub storage_backend: String,
    pub storage_chave: String,
    pub duracao_segundos: Option<u32>,
    pub codec: Option<String>,
    pub taxa_amostral_hz: Option<u32>,
    pub largura: Option<u32>,
    pub altura: Option<u32>,
    pub duracao_video_segundos: Option<u32>,
    pub usuario_upload_id: u32,
    pub data_upload: DateTime<Utc>,
    pub removido_em: Option<DateTime<Utc>>,
}

impl Arquivo {
    pub fn is_imagem(&self) -> bool {
        matches!(self.tipo, TipoArquivo::Imagem)
    }
    pub fn is_audio(&self) -> bool {
        matches!(self.tipo, TipoArquivo::Audio)
    }
    pub fn is_video(&self) -> bool {
        matches!(self.tipo, TipoArquivo::Video)
    }
    pub fn is_pdf(&self) -> bool {
        matches!(self.tipo, TipoArquivo::Pdf)
    }
}

/// Vínculo N:M entre arquivo e entidade.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArquivoVinculo {
    pub id: u32,
    pub arquivo_id: u32,
    pub entidade_tipo: TipoEntidade,
    pub entidade_id: u32,
    pub usuario_id: u32,
    pub data_vinculo: DateTime<Utc>,
    pub papel: Option<String>,
    pub observacao: Option<String>,
}

/// Tamanho de thumbnail.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TamanhoThumb {
    P128,
    P256,
    P512,
}

impl TamanhoThumb {
    pub fn as_db_str(self) -> &'static str {
        match self {
            TamanhoThumb::P128 => "THUMB_128",
            TamanhoThumb::P256 => "THUMB_256",
            TamanhoThumb::P512 => "THUMB_512",
        }
    }

    pub fn from_db_str(s: &str) -> Self {
        match s {
            "THUMB_128" => TamanhoThumb::P128,
            "THUMB_512" => TamanhoThumb::P512,
            _ => TamanhoThumb::P256,
        }
    }

    pub fn pixels(self) -> u32 {
        match self {
            TamanhoThumb::P128 => 128,
            TamanhoThumb::P256 => 256,
            TamanhoThumb::P512 => 512,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thumbnail {
    pub id: u32,
    pub arquivo_id: u32,
    pub tamanho: TamanhoThumb,
    pub largura: u32,
    pub altura: u32,
    pub tamanho_bytes: u64,
    pub storage_chave: String,
    pub data_geracao: DateTime<Utc>,
}

/// Status de transcrição.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum StatusTranscricao {
    Pendente,
    EmAndamento,
    Concluida,
    Falha,
    Cancelada,
}

impl StatusTranscricao {
    pub fn as_db_str(self) -> &'static str {
        match self {
            StatusTranscricao::Pendente => "PENDENTE",
            StatusTranscricao::EmAndamento => "EM_ANDAMENTO",
            StatusTranscricao::Concluida => "CONCLUIDA",
            StatusTranscricao::Falha => "FALHA",
            StatusTranscricao::Cancelada => "CANCELADA",
        }
    }
    pub fn from_db_str(s: &str) -> Self {
        match s {
            "EM_ANDAMENTO" => StatusTranscricao::EmAndamento,
            "CONCLUIDA" => StatusTranscricao::Concluida,
            "FALHA" => StatusTranscricao::Falha,
            "CANCELADA" => StatusTranscricao::Cancelada,
            _ => StatusTranscricao::Pendente,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transcricao {
    pub id: u32,
    pub arquivo_id: u32,
    pub engine: String,
    pub engine_modelo: Option<String>,
    pub texto: Option<String>,
    pub idioma: Option<String>,
    pub confianca: Option<f32>,
    pub tempo_processamento_ms: Option<u32>,
    pub data_inicio: Option<DateTime<Utc>>,
    pub data_fim: Option<DateTime<Utc>>,
    pub status: StatusTranscricao,
    pub erro_mensagem: Option<String>,
}

// =============================================================================
// Validações de segurança (Fase 9)
// =============================================================================

/// Extensões bloqueadas (RCE / scripts / executáveis).
pub const EXTENSOES_BLOQUEADAS: &[&str] = &[
    "exe", "dll", "bat", "scr", "js", "com", "cmd", "vbs", "vbe", "jse", "wsf", "wsh", "ps1",
    "psm1", "msi", "jar", "hta", "cpl", "lnk", "inf", "reg", "scf",
];

/// Tamanho máximo default: 50 MB.
pub const TAMANHO_MAX_DEFAULT: u64 = 50 * 1024 * 1024;

/// Mime types permitidos (whitelist conservadora).
/// Aceita `image/*`, `application/pdf`, `audio/*`, `video/*`.
/// O resto cai em OUTRO se quiser — mas o validador permite só estes.
pub fn mime_permitido(mime: &str) -> bool {
    let m = mime.to_lowercase();
    m.starts_with("image/")
        || m == "application/pdf"
        || m.starts_with("audio/")
        || m.starts_with("video/")
}

pub fn extensao_bloqueada(ext: &str) -> bool {
    EXTENSOES_BLOQUEADAS
        .iter()
        .any(|b| ext.eq_ignore_ascii_case(b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tipo_arquivo_as_db_str() {
        assert_eq!(TipoArquivo::Imagem.as_db_str(), "IMAGEM");
        assert_eq!(TipoArquivo::Pdf.as_db_str(), "PDF");
        assert_eq!(TipoArquivo::Audio.as_db_str(), "AUDIO");
    }

    #[test]
    fn tipo_arquivo_from_mime() {
        assert_eq!(TipoArquivo::from_mime("image/jpeg"), TipoArquivo::Imagem);
        assert_eq!(TipoArquivo::from_mime("application/pdf"), TipoArquivo::Pdf);
        assert_eq!(TipoArquivo::from_mime("audio/ogg"), TipoArquivo::Audio);
        assert_eq!(TipoArquivo::from_mime("video/mp4"), TipoArquivo::Video);
        assert_eq!(
            TipoArquivo::from_mime("application/zip"),
            TipoArquivo::Outro
        );
    }

    #[test]
    fn extensao_bloqueada_bloqueia_exe() {
        assert!(extensao_bloqueada("exe"));
        assert!(extensao_bloqueada("EXE"));
        assert!(extensao_bloqueada("bat"));
        assert!(extensao_bloqueada("scr"));
        assert!(!extensao_bloqueada("jpg"));
        assert!(!extensao_bloqueada("pdf"));
    }

    #[test]
    fn mime_permitido_apenas_seguros() {
        assert!(mime_permitido("image/jpeg"));
        assert!(mime_permitido("image/png"));
        assert!(mime_permitido("image/webp"));
        assert!(mime_permitido("application/pdf"));
        assert!(mime_permitido("audio/mpeg"));
        assert!(mime_permitido("audio/wav"));
        assert!(mime_permitido("video/mp4"));
        assert!(!mime_permitido("application/zip"));
        assert!(!mime_permitido("application/x-msdownload"));
        assert!(!mime_permitido("text/html"));
    }

    #[test]
    fn tamanho_thumb_pixels() {
        assert_eq!(TamanhoThumb::P128.pixels(), 128);
        assert_eq!(TamanhoThumb::P256.pixels(), 256);
        assert_eq!(TamanhoThumb::P512.pixels(), 512);
    }

    #[test]
    fn status_transcricao_roundtrip() {
        let s = StatusTranscricao::Concluida;
        assert_eq!(StatusTranscricao::from_db_str(s.as_db_str()), s);
    }
}
