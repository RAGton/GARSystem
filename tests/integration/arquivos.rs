// tests/integration/arquivos.rs
//
// Testes de integração do módulo Arquivos — Sprint P2.2.1.
// Requer MySQL real + storage local acessível (./var/storage).

#[path = "common.rs"]
mod common;

use gar_system::arquivos::{self as arq, TipoEntidade, TipoArquivo};
use gar_system::storage;

fn ctx_padrao() -> arq::ContextoUpload {
    arq::ContextoUpload {
        usuario_id: 1,
        username: Some("teste".into()),
        ip: Some("127.0.0.1".into()),
        user_agent: Some("integration-test".into()),
        request_id: Some("test-1".into()),
    }
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn upload_basico_persiste_arquivo() {
    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");
    storage::auto_configurar().expect("storage");

    let bytes = b"conteudo de teste do arquivo".to_vec();
    let params = arq::ParametrosUpload {
        nome_original: "teste.txt".to_string(),
        // txt não passa whitelist — usamos jpg fictício
        mime_type: "image/jpeg".to_string(),
        bytes,
        entidade_tipo: None,
        entidade_id: None,
        papel: None,
        observacao: None,
        gerar_thumbnails: false,
        transcrever: false,
    };
    let resultado = arq::upload(params, &ctx_padrao()).expect("upload");
    assert!(resultado.arquivo.id > 0);
    assert_eq!(resultado.arquivo.tipo, TipoArquivo::Imagem);
    assert!(resultado.arquivo.hash_sha256.len() == 64);
    assert!(resultado.novo_registro);
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn deduplicacao_por_hash_reutiliza_arquivo() {
    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");
    storage::auto_configurar().expect("storage");

    let bytes = b"mesmo conteudo exato".to_vec();
    let p1 = arq::ParametrosUpload {
        nome_original: "a.jpg".into(),
        mime_type: "image/jpeg".into(),
        bytes: bytes.clone(),
        entidade_tipo: None,
        entidade_id: None,
        papel: None,
        observacao: None,
        gerar_thumbnails: false,
        transcrever: false,
    };
    let r1 = arq::upload(p1, &ctx_padrao()).expect("upload1");
    assert!(r1.novo_registro);

    let p2 = arq::ParametrosUpload {
        nome_original: "b.jpg".into(),
        mime_type: "image/jpeg".into(),
        bytes,
        entidade_tipo: None,
        entidade_id: None,
        papel: None,
        observacao: None,
        gerar_thumbnails: false,
        transcrever: false,
    };
    let r2 = arq::upload(p2, &ctx_padrao()).expect("upload2");
    assert!(!r2.novo_registro);
    assert_eq!(r1.arquivo.id, r2.arquivo.id, "deve deduplicar");
    assert_eq!(r1.arquivo.hash_sha256, r2.arquivo.hash_sha256);
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn upload_bloqueia_extensao_exe() {
    let _pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    storage::auto_configurar().expect("storage");

    let p = arq::ParametrosUpload {
        nome_original: "virus.exe".into(),
        mime_type: "application/octet-stream".into(),
        bytes: b"MZ...".to_vec(),
        entidade_tipo: None,
        entidade_id: None,
        papel: None,
        observacao: None,
        gerar_thumbnails: false,
        transcrever: false,
    };
    let r = arq::upload(p, &ctx_padrao());
    assert!(matches!(r, Err(arq::ErroArquivo::ExtensaoBloqueada(_))));
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn upload_bloqueia_mime_invalido() {
    let _pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    storage::auto_configurar().expect("storage");

    let p = arq::ParametrosUpload {
        nome_original: "x.html".into(),
        mime_type: "text/html".into(),
        bytes: b"<script>".to_vec(),
        entidade_tipo: None,
        entidade_id: None,
        papel: None,
        observacao: None,
        gerar_thumbnails: false,
        transcrever: false,
    };
    let r = arq::upload(p, &ctx_padrao());
    assert!(matches!(r, Err(arq::ErroArquivo::MimeNaoPermitido(_))));
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn upload_rejeita_path_traversal() {
    let _pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    storage::auto_configurar().expect("storage");

    let p = arq::ParametrosUpload {
        nome_original: "../../../etc/passwd".into(),
        mime_type: "image/jpeg".into(),
        bytes: b"x".to_vec(),
        entidade_tipo: None,
        entidade_id: None,
        papel: None,
        observacao: None,
        gerar_thumbnails: false,
        transcrever: false,
    };
    let r = arq::upload(p, &ctx_padrao());
    assert!(matches!(r, Err(arq::ErroArquivo::PathTraversal(_))));
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn vinculacao_a_entidade_registra_e_lista() {
    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");
    storage::auto_configurar().expect("storage");

    use mysql::prelude::Queryable;
    let mut conn = pool.get_conn().expect("conn");
    conn.exec_drop("INSERT INTO clientes (nome) VALUES (?)", ("Cliente Files",))
        .expect("insert cliente");
    let cid = conn.last_insert_id() as u32;

    let p = arq::ParametrosUpload {
        nome_original: "foto.jpg".into(),
        mime_type: "image/jpeg".into(),
        bytes: b"jpeg bytes".to_vec(),
        entidade_tipo: Some(TipoEntidade::Cliente),
        entidade_id: Some(cid),
        papel: Some("foto_antes".into()),
        observacao: None,
        gerar_thumbnails: false,
        transcrever: false,
    };
    let r = arq::upload(p, &ctx_padrao()).expect("upload");
    assert!(r.vinculo_id.is_some());

    let arquivos = arq::listar_arquivos_por_entidade(TipoEntidade::Cliente, cid)
        .expect("listar");
    assert_eq!(arquivos.len(), 1);
    assert_eq!(arquivos[0].id, r.arquivo.id);
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn download_retorna_bytes_originais() {
    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");
    storage::auto_configurar().expect("storage");

    let original = b"conteudo original para download".to_vec();
    let p = arq::ParametrosUpload {
        nome_original: "doc.pdf".into(),
        mime_type: "application/pdf".into(),
        bytes: original.clone(),
        entidade_tipo: None,
        entidade_id: None,
        papel: None,
        observacao: None,
        gerar_thumbnails: false,
        transcrever: false,
    };
    let r = arq::upload(p, &ctx_padrao()).expect("upload");

    let baixado = arq::download(r.arquivo.id).expect("download");
    assert_eq!(baixado, original);
}

#[tokio::test]
#[ignore = "requer MySQL real"]
async fn transcricao_mock_registra_para_audio() {
    let pool = common::setup_pool().await.expect("MySQL");
    common::aplicar_migrations().expect("migrations");
    common::truncate_all(&pool).expect("truncate");
    storage::auto_configurar().expect("storage");
    gar_system::transcription::auto_configurar();

    let p = arq::ParametrosUpload {
        nome_original: "audio.mp3".into(),
        mime_type: "audio/mpeg".into(),
        bytes: b"fake mp3 bytes".to_vec(),
        entidade_tipo: None,
        entidade_id: None,
        papel: None,
        observacao: None,
        gerar_thumbnails: false,
        transcrever: true,
    };
    let r = arq::upload(p, &ctx_padrao()).expect("upload");
    assert!(r.transcricao.is_some());
    let t = r.transcricao.unwrap();
    assert_eq!(t.status, arq::StatusTranscricao::Concluida);
    assert!(t.texto.unwrap().contains("transcrição mock"));
}
