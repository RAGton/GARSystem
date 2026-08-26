// src/transcription/service.rs
//
// Service de transcrição — Sprint P2.2.1.
//
// Singleton configurável. Default = MockTranscriber.
// Para trocar: `TranscriptionService::configurar(Box::new(WhisperLocal::novo()))`.

use super::mock::MockTranscriber;
use super::traits::Transcriber;
use std::sync::OnceLock;

static ENGINE: OnceLock<Box<dyn Transcriber>> = OnceLock::new();

/// Configura a engine (chamado uma vez no boot).
pub fn configurar(e: Box<dyn Transcriber>) -> Result<(), String> {
    ENGINE
        .set(e)
        .map_err(|_| "engine já configurada".to_string())
}

/// Lê a engine ativa (panics se não foi configurada).
fn engine() -> &'static dyn Transcriber {
    ENGINE
        .get()
        .expect("transcription::service::engine chamado sem configurar() prévio")
        .as_ref()
}

/// Auto-configura com mock (default).
pub fn auto_configurar() {
    if ENGINE.get().is_some() {
        return;
    }
    let _ = configurar(Box::new(MockTranscriber::novo()));
}

/// Wrapper de conveniência.
pub fn transcrever(
    req: &super::traits::RequisicaoTranscricao,
    audio: &[u8],
) -> Result<super::traits::ResultadoTranscricao, super::traits::ErroTranscricao> {
    engine().transcrever(req, audio)
}

pub fn engine_id() -> &'static str {
    engine().engine().as_db_str()
}
