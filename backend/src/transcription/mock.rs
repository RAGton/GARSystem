// src/transcription/mock.rs
//
// Mock Transcriber — Sprint P2.2.1.
//
// Implementação placeholder que devolve texto determinístico
// baseado no hash do áudio. Serve para:
//   - testes unit
//   - fluxo end-to-end sem dependência externa
//
// Não chama rede. Não processa áudio. Em produção, substituir
// por `WhisperLocal` ou `OpenAITranscriber` via `service::configurar`.

use super::traits::{
    EngineTranscricao, ErroTranscricao, RequisicaoTranscricao, ResultadoTranscricao, Transcriber,
};
use sha2::{Digest, Sha256};

pub struct MockTranscriber;

impl MockTranscriber {
    pub fn novo() -> Self {
        Self
    }
}

impl Transcriber for MockTranscriber {
    fn engine(&self) -> EngineTranscricao {
        EngineTranscricao::Mock
    }

    fn transcrever(
        &self,
        req: &RequisicaoTranscricao,
        _audio: &[u8],
    ) -> Result<ResultadoTranscricao, ErroTranscricao> {
        // Hash do arquivo_id + mime para gerar "texto" determinístico
        let mut hasher = Sha256::new();
        hasher.update(req.arquivo_id.to_le_bytes());
        hasher.update(req.mime_type.as_bytes());
        let digest = hasher.finalize();
        let hex = digest
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        let texto = format!(
            "[transcrição mock] arquivo_id={} mime={} hash={}",
            req.arquivo_id,
            req.mime_type,
            &hex[..16]
        );
        Ok(ResultadoTranscricao {
            texto,
            idioma: req.idioma_origem.clone().or_else(|| Some("pt".to_string())),
            confianca: Some(0.0),
            duracao_audio_segundos: None,
            tempo_processamento_ms: 1,
            modelo_usado: "mock-v0".to_string(),
        })
    }

    fn modelos_suportados(&self) -> Vec<String> {
        vec!["mock-v0".to_string()]
    }

    fn mimes_aceitos(&self) -> Vec<String> {
        vec!["audio/*".to_string()]
    }
}
