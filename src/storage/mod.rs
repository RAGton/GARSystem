// src/storage/mod.rs
//
// Camada de abstração de storage — Sprint P2.2.1.
//
// Publica:
//   - traits    → contrato agnóstico de backend
//   - local     → implementação de filesystem
//   - service   → singleton configurável em runtime
//
// Stubs futuros (a implementar): s3, azure, gcs.
//
// O resto do app usa APENAS `service::*` — nunca `local::*`
// nem `traits::*` diretamente.

pub mod local;
pub mod service;
pub mod traits;

pub use service::{auto_configurar, backend_id, configurar, escrever, existe, ler, remover};
pub use traits::{Bytes, Localizacao, StorageBackend, StorageError, StorageRead, StorageResult};
