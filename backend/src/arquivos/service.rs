// src/arquivos/service.rs
//
// Camada de regras — Sprint P2.2.1.
//
// Orquestra:
//   1. Validação (mime, extensão, tamanho)
//   2. Hash SHA256
//   3. Storage (via `storage::service`)
//   4. Persistência (via `repository`)
//   5. Deduplicação (hash como chave)
//   6. Geração de thumbnails (imagens)
//   7. Transcrição (áudios)
//   8. Timeline (CRM)
//   9. Audit log

use super::models::{
    Arquivo, ArquivoVinculo, StatusTranscricao, TamanhoThumb, Thumbnail, TipoArquivo, TipoEntidade,
    Transcricao, EXTENSOES_BLOQUEADAS, TAMANHO_MAX_DEFAULT,
};
use super::repository;
use crate::crm::repository as crm_repo;
use crate::servicos::ErroAplicacao;
use crate::storage::{self, Localizacao};
use crate::transcription::{self, RequisicaoTranscricao, ResultadoTranscricao};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContextoUpload {
    pub usuario_id: u32,
    pub username: Option<String>,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultadoUpload {
    pub arquivo: Arquivo,
    pub novo_registro: bool,
    pub vinculo_id: Option<u32>,
    pub thumbnails: Vec<TamanhoThumb>,
    pub transcricao: Option<Transcricao>,
}

#[derive(Debug, thiserror::Error)]
pub enum ErroArquivo {
    #[error("arquivo vazio")]
    Vazio,
    #[error("arquivo muito grande: {0} bytes (max {1})")]
    MuitoGrande(u64, u64),
    #[error("mime type não permitido: {0}")]
    MimeNaoPermitido(String),
    #[error("extensão bloqueada: {0}")]
    ExtensaoBloqueada(String),
    #[error("extensão inválida")]
    ExtensaoInvalida,
    #[error("nome do arquivo inválido")]
    NomeInvalido,
    #[error("path traversal detectado: {0}")]
    PathTraversal(String),
    #[error("storage: {0}")]
    Storage(String),
    #[error("persistência: {0}")]
    Persistencia(String),
    #[error("imagem corrompida ou formato não suportado: {0}")]
    ImagemInvalida(String),
    #[error("transcrição falhou: {0}")]
    Transcricao(String),
    #[error("não encontrado")]
    NaoEncontrado,
    #[error("desconhecido: {0}")]
    Desconhecido(String),
}

impl From<storage::StorageError> for ErroArquivo {
    fn from(e: storage::StorageError) -> Self {
        match e {
            storage::StorageError::PermissaoNegada(msg) => ErroArquivo::PathTraversal(msg),
            storage::StorageError::NaoEncontrado(_) => ErroArquivo::NaoEncontrado,
            other => ErroArquivo::Storage(format!("{:?}", other)),
        }
    }
}

impl From<ErroArquivo> for ErroAplicacao {
    fn from(e: ErroArquivo) -> Self {
        ErroAplicacao::Desconhecido(format!("{:?}", e))
    }
}

impl From<ErroAplicacao> for ErroArquivo {
    fn from(e: ErroAplicacao) -> Self {
        ErroArquivo::Persistencia(format!("{:?}", e))
    }
}

// =============================================================================
// Hash
// =============================================================================

pub fn calcular_sha256(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    let digest = h.finalize();
    hex::encode(digest)
}

fn validar_nome(nome: &str) -> Result<String, ErroArquivo> {
    let nome = nome.trim();
    if nome.is_empty() {
        return Err(ErroArquivo::NomeInvalido);
    }
    if nome.contains("..") || nome.contains('/') || nome.contains('\\') {
        return Err(ErroArquivo::PathTraversal(nome.to_string()));
    }
    Ok(nome.to_string())
}

fn extrair_extensao(nome: &str) -> Result<String, ErroArquivo> {
    let ext = nome
        .rsplit('.')
        .next()
        .ok_or(ErroArquivo::ExtensaoInvalida)?
        .to_lowercase();
    if ext.is_empty() || ext == nome.to_lowercase() {
        return Err(ErroArquivo::ExtensaoInvalida);
    }
    if super::models::extensao_bloqueada(&ext) {
        return Err(ErroArquivo::ExtensaoBloqueada(ext));
    }
    if ext.len() > 16 {
        return Err(ErroArquivo::ExtensaoInvalida);
    }
    Ok(ext)
}

// =============================================================================
// Upload (com deduplicação)
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParametrosUpload {
    pub nome_original: String,
    pub mime_type: String,
    pub bytes: Vec<u8>,
    /// Contexto de vinculação (opcional). Se preenchido, cria vínculo.
    pub entidade_tipo: Option<TipoEntidade>,
    pub entidade_id: Option<u32>,
    pub papel: Option<String>,
    pub observacao: Option<String>,
    /// Força geração de thumbnail mesmo se não for imagem.
    pub gerar_thumbnails: bool,
    /// Força transcrição mesmo se não for áudio.
    pub transcrever: bool,
}

pub fn upload(
    tenant_id: i32,
    params: ParametrosUpload,
    ctx: &ContextoUpload,
) -> Result<ResultadoUpload, ErroArquivo> {
    // 1. Validações
    let nome = validar_nome(&params.nome_original)?;
    if params.bytes.is_empty() {
        return Err(ErroArquivo::Vazio);
    }
    if params.bytes.len() as u64 > TAMANHO_MAX_DEFAULT {
        return Err(ErroArquivo::MuitoGrande(
            params.bytes.len() as u64,
            TAMANHO_MAX_DEFAULT,
        ));
    }
    let ext = extrair_extensao(&nome)?;
    let mime = params.mime_type.to_lowercase();
    if !super::models::mime_permitido(&mime) {
        return Err(ErroArquivo::MimeNaoPermitido(mime));
    }
    if super::models::extensao_bloqueada(&ext) {
        return Err(ErroArquivo::ExtensaoBloqueada(ext));
    }

    // 2. Hash
    let hash = calcular_sha256(&params.bytes);

    // 3. Tipo
    let tipo = TipoArquivo::from_mime(&mime);

    // 4. Chave de storage (organiza por ano/mês)
    let agora = chrono::Utc::now();
    let ano: u32 = agora.format("%Y").to_string().parse().unwrap_or(2025);
    let mes: u32 = agora.format("%m").to_string().parse().unwrap_or(1);
    let chave = format!(
        "{:04}/{:02}/{}.{}",
        ano,
        mes,
        hash.chars().take(16).collect::<String>(),
        ext
    );

    // 5. Escreve no storage (sempre, mesmo se hash já existe — para
    //    cobrir caso de storage separado entre tenants no futuro).
    //    Aqui, para deduplicação, só escreve se ainda não existir.
    let local_existente = storage::Localizacao::nova(storage::backend_id(), chave.clone());
    let ja_existe_no_storage = storage::existe(&local_existente).unwrap_or(false);
    if !ja_existe_no_storage {
        storage::escrever(&chave, &params.bytes)?;
    }

    // 6. Persiste (com deduplicação via hash)
    let nome_armazenado = format!("{}.{}", hash.chars().take(16).collect::<String>(), ext);
    let (id, novo_registro) = repository::inserir_ou_obter_por_hash(
        tenant_id,
        &nome,
        &nome_armazenado,
        &ext,
        &mime,
        params.bytes.len() as u64,
        &hash,
        tipo,
        storage::backend_id(),
        &chave,
        ctx.usuario_id,
    )?;
    let arquivo = repository::obter_arquivo(tenant_id, id)?.ok_or(ErroArquivo::NaoEncontrado)?;

    // 7. Vínculo (opcional)
    let vinculo_id = if let (Some(et), Some(eid)) = (params.entidade_tipo, params.entidade_id) {
        Some(
            repository::criar_vinculo(
                tenant_id,
                id,
                et,
                eid,
                ctx.usuario_id,
                params.papel.as_deref(),
                params.observacao.as_deref(),
            )
            .map_err(ErroArquivo::from)?,
        )
    } else {
        None
    };

    // 8. Thumbnails (somente imagens)
    let mut thumbnails = Vec::new();
    if arquivo.is_imagem() || params.gerar_thumbnails {
        for t in [TamanhoThumb::P128, TamanhoThumb::P256, TamanhoThumb::P512] {
            if gerar_thumbnail(tenant_id, &arquivo, t).is_ok() {
                thumbnails.push(t);
            }
        }
    }

    // 9. Transcrição (somente áudios)
    let transcricao = if arquivo.is_audio() || params.transcrever {
        transcrever_arquivo(tenant_id, &arquivo, &params.bytes, ctx).ok()
    } else {
        None
    };

    // 10. Timeline (CRM)
    registrar_evento_upload(tenant_id, &arquivo, vinculo_id, novo_registro, ctx);

    Ok(ResultadoUpload {
        arquivo,
        novo_registro,
        vinculo_id,
        thumbnails,
        transcricao,
    })
}

// =============================================================================
// Thumbnails (Fase 4)
// =============================================================================

fn gerar_thumbnail(
    tenant_id: i32,
    arquivo: &Arquivo,
    tamanho: TamanhoThumb,
) -> Result<Localizacao, ErroArquivo> {
    if !arquivo.is_imagem() {
        return Err(ErroArquivo::ImagemInvalida("não é imagem".into()));
    }
    let bytes = storage::ler(&Localizacao::nova(
        &arquivo.storage_backend,
        &arquivo.storage_chave,
    ))?;
    let img =
        image::load_from_memory(&bytes).map_err(|e| ErroArquivo::ImagemInvalida(e.to_string()))?;
    let thumb = img.thumbnail(tamanho.pixels(), tamanho.pixels());
    let (w, h) = (thumb.width(), thumb.height());
    let mut out: Vec<u8> = Vec::new();
    // JPEG para uniformidade; webp/png ficam para versão futura
    let rgb = thumb.to_rgb8();
    let mut encoder = image::codecs::jpeg::JpegEncoder::new(&mut out);
    encoder
        .encode(rgb.as_raw(), w, h, image::ExtendedColorType::Rgb8)
        .map_err(|e| ErroArquivo::ImagemInvalida(e.to_string()))?;

    let chave_thumb = format!(
        "thumbs/{}/{:?}_{}.jpg",
        arquivo.hash_sha256.chars().take(8).collect::<String>(),
        tamanho,
        Uuid::new_v4()
    );
    storage::escrever(&chave_thumb, &out)?;
    let loc = Localizacao::nova(storage::backend_id(), chave_thumb.clone());
    repository::inserir_thumbnail(
        tenant_id,
        arquivo.id,
        tamanho,
        w,
        h,
        out.len() as u64,
        &chave_thumb,
    )?;
    Ok(loc)
}

pub fn obter_ou_gerar_thumbnail(
    tenant_id: i32,
    arquivo_id: u32,
    tamanho: TamanhoThumb,
) -> Result<Option<(Vec<u8>, Thumbnail)>, ErroArquivo> {
    if let Some(t) = repository::obter_thumbnail(tenant_id, arquivo_id, tamanho)? {
        let bytes = storage::ler(&Localizacao::nova(storage::backend_id(), &t.storage_chave))?;
        return Ok(Some((bytes, t)));
    }
    // Tenta gerar
    let arquivo =
        repository::obter_arquivo(tenant_id, arquivo_id)?.ok_or(ErroArquivo::NaoEncontrado)?;
    if !arquivo.is_imagem() {
        return Ok(None);
    }
    gerar_thumbnail(tenant_id, &arquivo, tamanho)?;
    if let Some(t) = repository::obter_thumbnail(tenant_id, arquivo_id, tamanho)? {
        let bytes = storage::ler(&Localizacao::nova(storage::backend_id(), &t.storage_chave))?;
        return Ok(Some((bytes, t)));
    }
    Ok(None)
}

// =============================================================================
// Transcrição (Fase 6)
// =============================================================================

fn transcrever_arquivo(
    tenant_id: i32,
    arquivo: &Arquivo,
    bytes: &[u8],
    _ctx: &ContextoUpload,
) -> Result<Transcricao, ErroArquivo> {
    if !arquivo.is_audio() {
        return Err(ErroArquivo::Transcricao("não é áudio".into()));
    }
    let req = RequisicaoTranscricao {
        arquivo_id: arquivo.id,
        mime_type: arquivo.mime_type.clone(),
        idioma_origem: Some("pt".into()),
        tamanho_bytes: arquivo.tamanho,
        modelo: None,
    };
    let transcricao_id =
        repository::criar_transcricao(tenant_id, arquivo.id, transcription::engine_id())
            .map_err(ErroArquivo::from)?;
    let _ = repository::atualizar_status_transcricao(
        tenant_id,
        transcricao_id,
        StatusTranscricao::EmAndamento,
        None,
    );
    let inicio = std::time::Instant::now();
    let resultado: ResultadoTranscricao = transcription::transcrever(&req, bytes).map_err(|e| {
        let msg = format!("{:?}", e);
        let _ = repository::atualizar_status_transcricao(
            tenant_id,
            transcricao_id,
            StatusTranscricao::Falha,
            Some(&msg),
        );
        ErroArquivo::Transcricao(msg)
    })?;
    let _ = repository::atualizar_resultado_transcricao(
        tenant_id,
        transcricao_id,
        &resultado.texto,
        resultado.idioma.as_deref(),
        resultado.confianca,
        resultado.tempo_processamento_ms,
    );
    let _ = inicio; // tempo já vem da engine
    repository::obter_transcricao(tenant_id, arquivo.id)?
        .ok_or(ErroArquivo::Transcricao("registro sumiu".into()))
}

pub fn transcrever_existente(tenant_id: i32, arquivo_id: u32) -> Result<Transcricao, ErroArquivo> {
    let arquivo =
        repository::obter_arquivo(tenant_id, arquivo_id)?.ok_or(ErroArquivo::NaoEncontrado)?;
    if !arquivo.is_audio() {
        return Err(ErroArquivo::Transcricao("não é áudio".into()));
    }
    let bytes = storage::ler(&Localizacao::nova(
        &arquivo.storage_backend,
        &arquivo.storage_chave,
    ))?;
    transcrever_arquivo(tenant_id, &arquivo, &bytes, &ContextoUpload::default())
}

pub fn obter_transcricao(
    tenant_id: i32,
    arquivo_id: u32,
) -> Result<Option<Transcricao>, ErroArquivo> {
    Ok(repository::obter_transcricao(tenant_id, arquivo_id)?)
}

// =============================================================================
// Listagem, vínculos, remoção
// =============================================================================

pub fn listar_arquivos(
    tenant_id: i32,
    tipo: Option<TipoArquivo>,
    usuario_id: Option<u32>,
    limite: u32,
) -> Result<Vec<Arquivo>, ErroArquivo> {
    Ok(repository::listar_arquivos(
        tenant_id, tipo, usuario_id, limite,
    )?)
}

pub fn obter(tenant_id: i32, id: u32) -> Result<Option<Arquivo>, ErroArquivo> {
    Ok(repository::obter_arquivo(tenant_id, id)?)
}

pub fn obter_por_hash(tenant_id: i32, hash: &str) -> Result<Option<Arquivo>, ErroArquivo> {
    Ok(repository::obter_arquivo_por_hash(tenant_id, hash)?)
}

pub fn listar_vinculos(
    tenant_id: i32,
    arquivo_id: u32,
) -> Result<Vec<ArquivoVinculo>, ErroArquivo> {
    Ok(repository::listar_vinculos_arquivo(tenant_id, arquivo_id)?)
}

pub fn listar_arquivos_por_entidade(
    tenant_id: i32,
    entidade_tipo: TipoEntidade,
    entidade_id: u32,
) -> Result<Vec<Arquivo>, ErroArquivo> {
    Ok(repository::listar_arquivos_por_entidade(
        tenant_id,
        entidade_tipo,
        entidade_id,
    )?)
}

pub fn vincular(
    tenant_id: i32,
    arquivo_id: u32,
    entidade_tipo: TipoEntidade,
    entidade_id: u32,
    ctx: &ContextoUpload,
    papel: Option<&str>,
    observacao: Option<&str>,
) -> Result<u32, ErroArquivo> {
    let arquivo =
        repository::obter_arquivo(tenant_id, arquivo_id)?.ok_or(ErroArquivo::NaoEncontrado)?;
    let id = repository::criar_vinculo(
        tenant_id,
        arquivo_id,
        entidade_tipo,
        entidade_id,
        ctx.usuario_id,
        papel,
        observacao,
    )?;
    // Timeline
    let payload = serde_json::json!({
        "arquivo_id": arquivo_id,
        "nome": arquivo.nome_original,
        "entidade_tipo": entidade_tipo.as_db_str(),
        "entidade_id": entidade_id,
    });
    let cliente_id_opt = tentar_resolver_cliente_id(tenant_id, entidade_tipo, entidade_id);
    if let Some(cliente_id) = cliente_id_opt {
        let _ = crm_repo::inserir_evento_timeline(
            tenant_id,
            cliente_id,
            "ARQUIVO_VINCULADO",
            &format!("arquivo anexado: {}", arquivo.nome_original),
            Some(&payload),
            Some(ctx.usuario_id),
            ctx.username.as_deref(),
        );
    }
    Ok(id)
}

pub fn desvincular(tenant_id: i32, vinculo_id: u32) -> Result<bool, ErroArquivo> {
    Ok(repository::remover_vinculo(tenant_id, vinculo_id)?)
}

pub fn remover(tenant_id: i32, id: u32, ctx: &ContextoUpload) -> Result<bool, ErroArquivo> {
    let _arquivo = repository::obter_arquivo(tenant_id, id)?.ok_or(ErroArquivo::NaoEncontrado)?;
    let removido = repository::marcar_removido(tenant_id, id)?;
    if removido {
        // Se era o último vínculo, podemos remover do storage também
        // (lazy delete — deixamos para uma rotina de GC)
        // Timeline
        let _ = ctx; // suprime warning
    }
    Ok(removido)
}

pub fn download(tenant_id: i32, id: u32) -> Result<Vec<u8>, ErroArquivo> {
    let arquivo = repository::obter_arquivo(tenant_id, id)?.ok_or(ErroArquivo::NaoEncontrado)?;
    let bytes = storage::ler(&Localizacao::nova(
        &arquivo.storage_backend,
        &arquivo.storage_chave,
    ))?;
    Ok(bytes)
}

// =============================================================================
// Helpers
// =============================================================================

fn tentar_resolver_cliente_id(
    tenant_id: i32,
    entidade_tipo: TipoEntidade,
    entidade_id: u32,
) -> Option<u32> {
    use crate::banco_de_dados::obter_conexao;
    use mysql::prelude::Queryable;
    let mut conn = obter_conexao().ok()?;
    let sql = match entidade_tipo {
        TipoEntidade::Cliente => Some("SELECT id FROM clientes WHERE tenant_id = ? AND id = ?"),
        TipoEntidade::Os => {
            Some("SELECT cliente_id FROM ordens_servico WHERE tenant_id = ? AND id = ?")
        }
        TipoEntidade::Cotacao => {
            Some("SELECT cliente_id FROM cotacoes WHERE tenant_id = ? AND id = ?")
        }
        TipoEntidade::Orcamento => {
            Some("SELECT cliente_id FROM orcamentos WHERE tenant_id = ? AND id = ?")
        }
        TipoEntidade::Equipamento => {
            Some("SELECT cliente_id FROM equipamentos WHERE tenant_id = ? AND id = ?")
        }
        _ => None,
    };
    let sql = sql?;
    let res: Option<u32> = conn
        .exec_first(sql, (tenant_id, entidade_id))
        .ok()
        .flatten();
    res
}

fn registrar_evento_upload(
    tenant_id: i32,
    arquivo: &Arquivo,
    vinculo_id: Option<u32>,
    novo_registro: bool,
    ctx: &ContextoUpload,
) {
    let tipo_evento = if arquivo.is_audio() {
        "AUDIO_ENVIADO"
    } else {
        "ARQUIVO_ENVIADO"
    };
    let mut titulo = match tipo_evento {
        "AUDIO_ENVIADO" => format!("áudio enviado: {}", arquivo.nome_original),
        _ => format!("arquivo enviado: {}", arquivo.nome_original),
    };
    if !novo_registro {
        titulo.push_str(" (deduplicado)");
    }
    let payload = serde_json::json!({
        "arquivo_id": arquivo.id,
        "tipo": arquivo.tipo.as_db_str(),
        "tamanho": arquivo.tamanho,
        "mime": arquivo.mime_type,
        "hash_sha256": arquivo.hash_sha256,
        "novo_registro": novo_registro,
        "vinculo_id": vinculo_id,
    });
    // Tenta achar cliente via vínculo
    let cliente_id = if let Some(vid) = vinculo_id {
        repository::listar_vinculos_arquivo(tenant_id, arquivo.id)
            .ok()
            .and_then(|vs| vs.into_iter().find(|v| v.id == vid))
            .and_then(|v| tentar_resolver_cliente_id(tenant_id, v.entidade_tipo, v.entidade_id))
    } else {
        None
    };
    if let Some(cid) = cliente_id {
        let _ = crm_repo::inserir_evento_timeline(
            tenant_id,
            cid,
            tipo_evento,
            &titulo,
            Some(&payload),
            Some(ctx.usuario_id),
            ctx.username.as_deref(),
        );
    }
}

// =============================================================================
// Extensões auxiliares
// =============================================================================

trait NumToU32 {
    fn num_to_u32(&self) -> u32;
}

impl NumToU32 for str {
    fn num_to_u32(&self) -> u32 {
        self.parse().unwrap_or(0)
    }
}

// =============================================================================
// Testes
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calcular_hash_estavel() {
        let h1 = calcular_sha256(b"hello world");
        let h2 = calcular_sha256(b"hello world");
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
    }

    #[test]
    fn calcular_hash_diferente_para_conteudo_diferente() {
        let h1 = calcular_sha256(b"hello");
        let h2 = calcular_sha256(b"world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn validar_nome_rejeita_path_traversal() {
        assert!(matches!(
            validar_nome("../etc/passwd"),
            Err(ErroArquivo::PathTraversal(_))
        ));
        assert!(matches!(
            validar_nome("foo/bar"),
            Err(ErroArquivo::PathTraversal(_))
        ));
    }

    #[test]
    fn validar_nome_rejeita_vazio() {
        assert!(matches!(validar_nome(""), Err(ErroArquivo::NomeInvalido)));
        assert!(matches!(
            validar_nome("   "),
            Err(ErroArquivo::NomeInvalido)
        ));
    }

    #[test]
    fn extrair_extensao_basico() {
        assert_eq!(extrair_extensao("foto.jpg").unwrap(), "jpg");
        assert_eq!(extrair_extensao("Documento.PDF").unwrap(), "pdf");
        assert_eq!(extrair_extensao("v1.0.0.tar.gz").unwrap(), "gz");
    }

    #[test]
    fn extrair_extensao_bloqueia_exe() {
        assert!(matches!(
            extrair_extensao("virus.exe"),
            Err(ErroArquivo::ExtensaoBloqueada(_))
        ));
    }

    #[test]
    fn extrair_extensao_rejeita_sem_extensao() {
        assert!(matches!(
            extrair_extensao("arquivo"),
            Err(ErroArquivo::ExtensaoInvalida)
        ));
    }

    #[test]
    fn mime_tipos_permitidos_bloqueiam_exe() {
        // .exe é bloqueado pela extensão ANTES de chegar no mime check.
        assert!(matches!(
            upload(
                crate::servicos::TENANT_LEGACY,
                ParametrosUpload {
                    nome_original: "x.exe".into(),
                    mime_type: "application/x-msdownload".into(),
                    bytes: vec![1, 2, 3],
                    entidade_tipo: None,
                    entidade_id: None,
                    papel: None,
                    observacao: None,
                    gerar_thumbnails: false,
                    transcrever: false,
                },
                &ContextoUpload::default(),
            ),
            Err(ErroArquivo::ExtensaoBloqueada(_))
        ));
        // mime inválido (zip) também bloqueia
        assert!(matches!(
            upload(
                crate::servicos::TENANT_LEGACY,
                ParametrosUpload {
                    nome_original: "x.zip".into(),
                    mime_type: "application/zip".into(),
                    bytes: vec![1, 2, 3],
                    entidade_tipo: None,
                    entidade_id: None,
                    papel: None,
                    observacao: None,
                    gerar_thumbnails: false,
                    transcrever: false,
                },
                &ContextoUpload::default(),
            ),
            Err(ErroArquivo::MimeNaoPermitido(_))
        ));
    }

    #[test]
    fn upload_vazio_rejeita() {
        assert!(matches!(
            upload(
                crate::servicos::TENANT_LEGACY,
                ParametrosUpload {
                    nome_original: "vazio.jpg".into(),
                    mime_type: "image/jpeg".into(),
                    bytes: vec![],
                    entidade_tipo: None,
                    entidade_id: None,
                    papel: None,
                    observacao: None,
                    gerar_thumbnails: false,
                    transcrever: false,
                },
                &ContextoUpload::default(),
            ),
            Err(ErroArquivo::Vazio)
        ));
    }

    #[test]
    fn extensao_bloqueada_lista_contem_principais() {
        for e in &["exe", "dll", "bat", "scr", "js", "com", "vbs", "ps1", "msi"] {
            assert!(EXTENSOES_BLOQUEADAS.contains(e), "faltou {}", e);
        }
    }
}
