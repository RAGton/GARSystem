// src/storage/traits.rs
//
// Trait abstrato de storage — Sprint P2.2.1.
//
// Define o contrato que qualquer backend de armazenamento
// (disco local, MinIO, S3, Azure Blob, GCS) deve implementar.
//
// Implementações:
//   - `local.rs`     → filesystem local (default, funcional)
//   - `s3.rs`        → AWS S3 (futuro, stub)
//   - `azure.rs`     → Azure Blob (futuro, stub)
//   - `gcs.rs`       → Google Cloud Storage (futuro, stub)
//
// Toda a lógica de negócio opera contra `dyn StorageBackend`,
// nunca contra o filesystem direto. Trocar de backend é uma
// questão de mudar o `Box::new(...)` no `service.rs`.

use std::io::Read;
use std::path::PathBuf;

/// Bytes lidos do storage.
pub type Bytes = Vec<u8>;

/// Resultado padrão do storage.
pub type StorageResult<T> = Result<T, StorageError>;

/// Erros de storage. Backend-agnóstico; cada implementação
/// converte seu erro nativo (ex: `rusoto::s3::GetObjectError`)
/// em uma dessas variantes.
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("arquivo não encontrado: {0}")]
    NaoEncontrado(String),

    #[error("permissão negada: {0}")]
    PermissaoNegada(String),

    #[error("backend indisponível: {0}")]
    Indisponivel(String),

    #[error("operação não suportada pelo backend: {0}")]
    NaoSuportado(&'static str),

    #[error("erro de I/O: {0}")]
    Io(#[from] std::io::Error),

    #[error("erro desconhecido: {0}")]
    Desconhecido(String),
}

impl From<StorageError> for String {
    fn from(e: StorageError) -> String {
        format!("{:?}", e)
    }
}

/// Localização lógica de um arquivo dentro do backend.
///
/// Não é um path de filesystem — é um identificador opaco
/// que o backend traduz para seu próprio namespace.
///   - Local  → `PathBuf` (relativo à raiz configurada)
///   - S3     → `s3://bucket/key`
///   - Azure  → `https://account.blob.core.windows.net/container/blob`
pub struct Localizacao {
    pub backend_id: String, // ex: "local", "s3", "azure"
    pub chave: String,      // ex: "2025/01/abc123.pdf"
}

impl Localizacao {
    pub fn nova(backend_id: impl Into<String>, chave: impl Into<String>) -> Self {
        Self {
            backend_id: backend_id.into(),
            chave: chave.into(),
        }
    }
}

/// Stream de leitura — implementações podem ler de file, S3 GetObject, etc.
pub trait StorageRead: Send {
    fn ler_tudo(&mut self) -> StorageResult<Bytes>;
    fn tamanho(&self) -> Option<u64>;
}

impl StorageRead for Bytes {
    fn ler_tudo(&mut self) -> StorageResult<Bytes> {
        Ok(std::mem::take(self))
    }
    fn tamanho(&self) -> Option<u64> {
        Some(self.len() as u64)
    }
}

/// Trait principal do backend de storage.
pub trait StorageBackend: Send + Sync {
    /// ID legível para logs (`"local"`, `"s3"`, etc).
    fn id(&self) -> &'static str;

    /// Escreve bytes. Retorna a `Localizacao` resultante.
    fn escrever(&self, chave: &str, bytes: &[u8]) -> StorageResult<Localizacao>;

    /// Lê o arquivo inteiro em memória.
    fn ler(&self, local: &Localizacao) -> StorageResult<Bytes>;

    /// Lê em chunks (para arquivos grandes).
    fn ler_stream(&self, local: &Localizacao) -> StorageResult<Box<dyn StorageRead>>;

    /// Remove o arquivo.
    fn remover(&self, local: &Localizacao) -> StorageResult<()>;

    /// Verifica se existe.
    fn existe(&self, local: &Localizacao) -> StorageResult<bool>;

    /// URL pública (presigned/expirada) — opcional.
    /// Retorna `NaoSuportado` se o backend não oferece.
    fn url_publica(&self, _local: &Localizacao, _ttl_segundos: u32) -> StorageResult<String> {
        Err(StorageError::NaoSuportado("url_publica"))
    }

    /// Bytes totais consumidos (para métricas de billing).
    /// Default: 0 (backend pode não saber).
    fn bytes_usados(&self) -> u64 {
        0
    }
}

/// Helper de leitura a partir de qualquer `Read`.
pub fn ler_reader<R: Read>(mut r: R) -> StorageResult<Bytes> {
    let mut buf = Vec::new();
    r.read_to_end(&mut buf)?;
    Ok(buf)
}

/// Constrói um `PathBuf` seguro (anti path-traversal).
/// `base` é a raiz do storage; `chave` é o caminho relativo.
pub fn juntar_seguro(base: &std::path::Path, chave: &str) -> StorageResult<PathBuf> {
    if chave.contains("..") || chave.starts_with('/') {
        return Err(StorageError::PermissaoNegada(format!(
            "chave inválida: {}",
            chave
        )));
    }
    let p = base.join(chave);
    // canonicaliza o base e garantir que o resultado está dentro
    let base_canon = base.canonicalize().unwrap_or_else(|_| base.to_path_buf());
    let p_canon = p.canonicalize().unwrap_or_else(|_| p.clone());
    if !p_canon.starts_with(&base_canon) {
        return Err(StorageError::PermissaoNegada(format!(
            "path traversal detectado: {}",
            chave
        )));
    }
    Ok(p)
}
