// src/storage/service.rs
//
// Service de storage — ponto único de acesso para o resto do app.
// Sprint P2.2.1.
//
// Padrão: `OnceLock<Box<dyn StorageBackend>>` — configurado uma
// única vez no boot do server, lido de qualquer lugar.
//
// Trocar de backend (local → S3, etc) é uma linha:
//   `STORAGE_BACKEND=s3` → factory instancia S3Storage em vez de LocalStorage.

use super::local::LocalStorage;
use super::traits::{Bytes, Localizacao, StorageBackend, StorageError, StorageResult};
use std::sync::OnceLock;

static BACKEND: OnceLock<Box<dyn StorageBackend>> = OnceLock::new();

/// Configura o backend (chamado uma vez no boot).
pub fn configurar(b: Box<dyn StorageBackend>) -> StorageResult<()> {
    BACKEND
        .set(b)
        .map_err(|_| StorageError::Indisponivel("backend já configurado".into()))
}

/// Lê o backend configurado (panics se não foi configurado).
fn backend() -> &'static dyn StorageBackend {
    BACKEND
        .get()
        .expect("storage::service::backend chamado sem configurar() prévio")
        .as_ref()
}

/// Auto-configura com o backend default (local filesystem).
/// Idempotente. Útil para testes e para o GUI.
pub fn auto_configurar() -> StorageResult<()> {
    if BACKEND.get().is_some() {
        return Ok(());
    }
    let b: Box<dyn StorageBackend> = Box::new(LocalStorage::padrao()?);
    configurar(b)
}

/// Escreve bytes.
pub fn escrever(chave: &str, bytes: &[u8]) -> StorageResult<Localizacao> {
    backend().escrever(chave, bytes)
}

/// Lê bytes.
pub fn ler(local: &Localizacao) -> StorageResult<Bytes> {
    backend().ler(local)
}

/// Remove.
pub fn remover(local: &Localizacao) -> StorageResult<()> {
    backend().remover(local)
}

/// Existe?
pub fn existe(local: &Localizacao) -> StorageResult<bool> {
    backend().existe(local)
}

/// ID do backend ativo (para logs/diagnóstico).
pub fn backend_id() -> &'static str {
    backend().id()
}
