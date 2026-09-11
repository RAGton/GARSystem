// src/arquivos/mod.rs
//
// Módulo Anexos & Mídia — Sprint P2.2.1.
//
// Entidade universal "Arquivo" + storage + transcrição + timeline.
// Reutilizável por OS, Cliente, Cotação, Orçamento, Financeiro, Fiscal.

pub mod models;
pub mod repository;
pub mod service;

pub use models::{
    Arquivo, ArquivoVinculo, StatusTranscricao, TamanhoThumb, Thumbnail, TipoArquivo, TipoEntidade,
    Transcricao, EXTENSOES_BLOQUEADAS, TAMANHO_MAX_DEFAULT,
};
pub use service::{
    calcular_sha256, desvincular, download, listar_arquivos, listar_arquivos_por_entidade,
    listar_vinculos, obter, obter_ou_gerar_thumbnail, obter_por_hash, obter_transcricao, remover,
    transcrever_existente, upload, vincular, ContextoUpload, ErroArquivo, ParametrosUpload,
    ResultadoUpload,
};
