// src/storage/local.rs
//
// Backend de storage local (filesystem) — Sprint P2.2.1.
//
// Configurável via env `STORAGE_LOCAL_ROOT` (default: `./var/storage`).
// Anti path-traversal garantido em `traits::juntar_seguro`.
//
// Não usa `unsafe`. Não usa threads. Erros de I/O são
// traduzidos para `StorageError`.

use super::traits::{
    juntar_seguro, Bytes, Localizacao, StorageBackend, StorageError, StorageRead, StorageResult,
};
use std::fs;
use std::io::{Cursor, Read};
use std::path::PathBuf;

pub struct LocalStorage {
    raiz: PathBuf,
}

impl LocalStorage {
    pub fn novo(raiz: impl Into<PathBuf>) -> StorageResult<Self> {
        let raiz = raiz.into();
        fs::create_dir_all(&raiz)?;
        Ok(Self { raiz })
    }

    /// Padrão: ./var/storage (relativo ao CWD).
    pub fn padrao() -> StorageResult<Self> {
        let raiz = std::env::var("STORAGE_LOCAL_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./var/storage"));
        Self::novo(raiz)
    }

    pub fn raiz(&self) -> &std::path::Path {
        &self.raiz
    }
}

impl StorageBackend for LocalStorage {
    fn id(&self) -> &'static str {
        "local"
    }

    fn escrever(&self, chave: &str, bytes: &[u8]) -> StorageResult<Localizacao> {
        let p = juntar_seguro(&self.raiz, chave)?;
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&p, bytes)?;
        Ok(Localizacao::nova("local", chave.to_string()))
    }

    fn ler(&self, local: &Localizacao) -> StorageResult<Bytes> {
        let p = juntar_seguro(&self.raiz, &local.chave)?;
        let mut f = fs::File::open(&p)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        Ok(buf)
    }

    fn ler_stream(&self, local: &Localizacao) -> StorageResult<Box<dyn StorageRead>> {
        let p = juntar_seguro(&self.raiz, &local.chave)?;
        let f = fs::File::open(&p)?;
        let metadata = f.metadata().ok();
        let tamanho = metadata.map(|m| m.len());
        // embrulha em Cursor — Read genérico + tamanho
        let cursor = CursorFile {
            inner: Some(f),
            tamanho,
        };
        Ok(Box::new(cursor))
    }

    fn remover(&self, local: &Localizacao) -> StorageResult<()> {
        let p = juntar_seguro(&self.raiz, &local.chave)?;
        if !p.exists() {
            return Err(StorageError::NaoEncontrado(local.chave.clone()));
        }
        fs::remove_file(&p)?;
        Ok(())
    }

    fn existe(&self, local: &Localizacao) -> StorageResult<bool> {
        let p = juntar_seguro(&self.raiz, &local.chave)?;
        Ok(p.exists())
    }

    fn bytes_usados(&self) -> u64 {
        walkdir_size(&self.raiz).unwrap_or(0)
    }
}

/// Wrapper de Read com tamanho conhecido.
struct CursorFile {
    inner: Option<fs::File>,
    tamanho: Option<u64>,
}

impl StorageRead for CursorFile {
    fn ler_tudo(&mut self) -> StorageResult<Bytes> {
        if let Some(mut f) = self.inner.take() {
            let mut buf = Vec::new();
            f.read_to_end(&mut buf)?;
            Ok(buf)
        } else {
            Ok(Vec::new())
        }
    }

    fn tamanho(&self) -> Option<u64> {
        self.tamanho
    }
}

fn walkdir_size(p: &std::path::Path) -> std::io::Result<u64> {
    let mut total = 0u64;
    if p.is_file() {
        return Ok(p.metadata()?.len());
    }
    if p.is_dir() {
        for entry in fs::read_dir(p)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                total += walkdir_size(&path)?;
            } else {
                total += path.metadata()?.len();
            }
        }
    }
    Ok(total)
}
