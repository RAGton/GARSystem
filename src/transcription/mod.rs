// src/transcription/mod.rs
//
// Camada de transcrição — Sprint P2.2.1.
//
// Publica:
//   - traits  → contrato agnóstico de engine
//   - mock    → implementação placeholder (default)
//   - service → singleton configurável em runtime
//
// Stubs futuros (a implementar): whisper_local, openai, deepgram, minimax.

pub mod mock;
pub mod service;
pub mod traits;

pub use service::{auto_configurar, configurar, engine_id, transcrever};
pub use traits::{
    EngineTranscricao, ErroTranscricao, RequisicaoTranscricao, ResultadoTranscricao, Transcriber,
};
