// src/transcription/traits.rs
//
// Trait abstrato de transcrição — Sprint P2.2.1.
//
// Implementações futuras:
//   - Whisper local (whisper.cpp / whisper-rs)
//   - OpenAI (Whisper-1, GPT-4o-transcribe)
//   - Deepgram (Nova-2)
//   - MiniMax (placeholder para integração futura)
//
// Nenhuma implementação concreta é fornecida nesta sprint —
// apenas a interface + mock, para que o resto do sistema
// (upload, persistência, timeline) já funcione.

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum EngineTranscricao {
    WhisperLocal,
    Openai,
    Deepgram,
    MiniMax,
    Mock,
}

impl EngineTranscricao {
    pub fn as_db_str(self) -> &'static str {
        match self {
            EngineTranscricao::WhisperLocal => "whisper-local",
            EngineTranscricao::Openai => "openai",
            EngineTranscricao::Deepgram => "deepgram",
            EngineTranscricao::MiniMax => "minimax",
            EngineTranscricao::Mock => "mock",
        }
    }
}

impl fmt::Display for EngineTranscricao {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_db_str())
    }
}

/// Parâmetros de uma requisição de transcrição.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequisicaoTranscricao {
    pub arquivo_id: u32,
    pub mime_type: String,
    pub idioma_origem: Option<String>, // ISO 639-1: "pt", "en" — None = auto-detect
    pub tamanho_bytes: u64,
    pub modelo: Option<String>, // ex: "whisper-1", "large-v3"
}

/// Resultado de uma transcrição.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultadoTranscricao {
    pub texto: String,
    pub idioma: Option<String>,
    pub confianca: Option<f32>,
    pub duracao_audio_segundos: Option<u32>,
    pub tempo_processamento_ms: u32,
    pub modelo_usado: String,
}

/// Erros de transcrição. Mapeáveis para `StatusTranscricao::Falha`.
#[derive(Debug, thiserror::Error)]
pub enum ErroTranscricao {
    #[error("engine não configurada")]
    NaoConfigurada,

    #[error("formato não suportado pela engine: {0}")]
    FormatoNaoSuportado(String),

    #[error("arquivo vazio ou inválido")]
    ArquivoInvalido,

    #[error("timeout após {0}ms")]
    Timeout(u32),

    #[error("rate limit da engine atingido")]
    RateLimit,

    #[error("erro da engine: {0}")]
    Engine(String),

    #[error("erro desconhecido: {0}")]
    Desconhecido(String),
}

/// Trait principal. Toda engine implementa isso.
pub trait Transcriber: Send + Sync {
    fn engine(&self) -> EngineTranscricao;
    fn transcrever(
        &self,
        req: &RequisicaoTranscricao,
        audio: &[u8],
    ) -> Result<ResultadoTranscricao, ErroTranscricao>;
    /// Lista de modelos suportados.
    fn modelos_suportados(&self) -> Vec<String>;
    /// Mimes que esta engine aceita.
    fn mimes_aceitos(&self) -> Vec<String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_as_db_str() {
        assert_eq!(EngineTranscricao::WhisperLocal.as_db_str(), "whisper-local");
        assert_eq!(EngineTranscricao::Openai.as_db_str(), "openai");
        assert_eq!(EngineTranscricao::Mock.as_db_str(), "mock");
    }
}
