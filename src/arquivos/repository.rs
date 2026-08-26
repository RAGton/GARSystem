// src/arquivos/repository.rs
//
// Camada de acesso a dados — Sprint P2.2.1.
//
// Operações:
//   - CRUD de arquivos
//   - Vínculos
//   - Thumbnails
//   - Transcrições
//
// Tudo via SQL direto (mysql). Nenhuma regra de negócio aqui —
// só persistência e leitura.

use super::models::{
    Arquivo, ArquivoVinculo, StatusTranscricao, TamanhoThumb, Thumbnail, TipoArquivo, TipoEntidade,
    Transcricao,
};
use crate::banco_de_dados::obter_conexao;
use crate::servicos::ErroAplicacao;
use chrono::{DateTime, Utc};
use mysql::prelude::Queryable;
use mysql::{params, Params, Row};

fn err<E: std::fmt::Display>(e: E) -> ErroAplicacao {
    ErroAplicacao::Desconhecido(e.to_string())
}

// =============================================================================
// Arquivos
// =============================================================================

pub fn inserir_arquivo(
    tenant_id: i32,
    nome_original: &str,
    nome_armazenado: &str,
    extensao: &str,
    mime_type: &str,
    tamanho: u64,
    hash_sha256: &str,
    tipo: TipoArquivo,
    storage_backend: &str,
    storage_chave: &str,
    usuario_upload_id: u32,
) -> Result<u32, crate::servicos::ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        r#"INSERT INTO arquivos
            (nome_original, nome_armazenado, extensao, mime_type, tamanho,
             hash_sha256, tipo, storage_backend, storage_chave, usuario_upload_id, tenant_id)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        (
            nome_original,
            nome_armazenado,
            extensao,
            mime_type,
            tamanho,
            hash_sha256,
            tipo.as_db_str(),
            storage_backend,
            storage_chave,
            usuario_upload_id,
            tenant_id,
        ),
    )
    .map_err(err)?;
    Ok(conn.last_insert_id() as u32)
}

/// Insere ou reutiliza arquivo baseado no hash.
/// Retorna `(id, novo_registro)` — se hash já existe (e não foi removido),
/// retorna o id existente e `novo_registro = false`.
pub fn inserir_ou_obter_por_hash(
    tenant_id: i32,
    nome_original: &str,
    nome_armazenado: &str,
    extensao: &str,
    mime_type: &str,
    tamanho: u64,
    hash_sha256: &str,
    tipo: TipoArquivo,
    storage_backend: &str,
    storage_chave: &str,
    usuario_upload_id: u32,
) -> Result<(u32, bool), crate::servicos::ErroAplicacao> {
    // Tenta encontrar existente
    let mut conn = obter_conexao()?;
    let existing: Option<u32> = conn
        .exec_first(
            "SELECT id FROM arquivos WHERE tenant_id = ? AND hash_sha256 = ? AND removido_em IS NULL LIMIT 1",
            (tenant_id, hash_sha256),
        )
        .map_err(err)?;
    if let Some(id) = existing {
        return Ok((id, false));
    }
    let id = inserir_arquivo(
        tenant_id,
        nome_original,
        nome_armazenado,
        extensao,
        mime_type,
        tamanho,
        hash_sha256,
        tipo,
        storage_backend,
        storage_chave,
        usuario_upload_id,
    )?;
    Ok((id, true))
}

pub fn obter_arquivo(
    tenant_id: i32,
    id: u32,
) -> Result<Option<Arquivo>, crate::servicos::ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<Row> = conn
        .exec_first(
            r#"SELECT id, nome_original, nome_armazenado, extensao, mime_type, tamanho,
                      hash_sha256, tipo, storage_backend, storage_chave,
                      duracao_segundos, codec, taxa_amostral_hz,
                      largura, altura, duracao_video_segundos,
                      usuario_upload_id, data_upload, removido_em
               FROM arquivos WHERE id = ? AND tenant_id = ? AND removido_em IS NULL"#,
            (id, tenant_id),
        )
        .map_err(err)?;
    match row {
        Some(r) => Ok(Some(row_para_arquivo(r)?)),
        None => Ok(None),
    }
}

pub fn obter_arquivo_por_hash(
    tenant_id: i32,
    hash: &str,
) -> Result<Option<Arquivo>, crate::servicos::ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<Row> = conn
        .exec_first(
            r#"SELECT id, nome_original, nome_armazenado, extensao, mime_type, tamanho,
                      hash_sha256, tipo, storage_backend, storage_chave,
                      duracao_segundos, codec, taxa_amostral_hz,
                      largura, altura, duracao_video_segundos,
                      usuario_upload_id, data_upload, removido_em
               FROM arquivos WHERE tenant_id = ? AND hash_sha256 = ? AND removido_em IS NULL LIMIT 1"#,
            (tenant_id, hash),
        )
        .map_err(err)?;
    match row {
        Some(r) => Ok(Some(row_para_arquivo(r)?)),
        None => Ok(None),
    }
}

pub fn listar_arquivos(
    tenant_id: i32,
    tipo: Option<TipoArquivo>,
    usuario_id: Option<u32>,
    limite: u32,
) -> Result<Vec<Arquivo>, crate::servicos::ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let mut sql = String::from(
        r#"SELECT id, nome_original, nome_armazenado, extensao, mime_type, tamanho,
                  hash_sha256, tipo, storage_backend, storage_chave,
                  duracao_segundos, codec, taxa_amostral_hz,
                  largura, altura, duracao_video_segundos,
                  usuario_upload_id, data_upload, removido_em
           FROM arquivos WHERE tenant_id = ? AND removido_em IS NULL"#,
    );
    let mut p: Vec<String> = Vec::new();
    p.push(tenant_id.to_string());
    if let Some(t) = tipo {
        sql.push_str(" AND tipo = ?");
        p.push(t.as_db_str().to_string());
    }
    if let Some(uid) = usuario_id {
        sql.push_str(" AND usuario_upload_id = ?");
        p.push(uid.to_string());
    }
    sql.push_str(" ORDER BY data_upload DESC LIMIT ?");
    p.push(limite.to_string());

    let rows: Vec<Row> = conn
        .exec(
            &sql,
            Params::Positional(p.iter().map(|s| s.as_str().into()).collect::<Vec<_>>()),
        )
        .map_err(err)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(row_para_arquivo(r)?);
    }
    Ok(out)
}

/// Soft delete — marca como removido. Vínculos em cascade (FK manual).
pub fn marcar_removido(tenant_id: i32, id: u32) -> Result<bool, crate::servicos::ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        "UPDATE arquivos SET removido_em = NOW() WHERE tenant_id = ? AND id = ? AND removido_em IS NULL",
        (tenant_id, id),
    )
    .map_err(err)?;
    Ok(conn.affected_rows() > 0)
}

fn row_para_arquivo(r: Row) -> Result<Arquivo, crate::servicos::ErroAplicacao> {
    use mysql::Value;
    let s = |i: usize| -> Result<String, crate::servicos::ErroAplicacao> {
        match r.get::<Value, _>(i) {
            Some(Value::Bytes(b)) => Ok(String::from_utf8_lossy(&b).into_owned()),
            Some(Value::NULL) => Ok(String::new()),
            _ => Err(err("tipo errado")),
        }
    };
    let i = |i: usize| -> Result<i64, crate::servicos::ErroAplicacao> {
        match r.get::<Value, _>(i) {
            Some(Value::Int(n)) => Ok(n),
            Some(Value::UInt(n)) => Ok(n as i64),
            Some(Value::NULL) => Ok(0),
            _ => Err(err("tipo errado")),
        }
    };
    let opt_str = |i: usize| -> Result<Option<String>, crate::servicos::ErroAplicacao> {
        match r.get::<Value, _>(i) {
            Some(Value::Bytes(b)) => Ok(Some(String::from_utf8_lossy(&b).into_owned())),
            Some(Value::NULL) | None => Ok(None),
            _ => Err(err("tipo errado")),
        }
    };
    let opt_u32 = |i: usize| -> Result<Option<u32>, crate::servicos::ErroAplicacao> {
        match r.get::<Value, _>(i) {
            Some(Value::Int(n)) => Ok(Some(n as u32)),
            Some(Value::UInt(n)) => Ok(Some(n as u32)),
            Some(Value::NULL) | None => Ok(None),
            _ => Err(err("tipo errado")),
        }
    };
    let opt_dt = |i: usize| -> Result<Option<DateTime<Utc>>, crate::servicos::ErroAplicacao> {
        match r.get::<Value, _>(i) {
            Some(Value::Date(y, m, d, h, mi, s, _us)) => {
                use chrono::NaiveDate;
                let nd = NaiveDate::from_ymd_opt(y as i32, m as u32, d as u32).unwrap();
                let nt = nd.and_hms_opt(h as u32, mi as u32, s as u32).unwrap();
                Ok(Some(DateTime::<Utc>::from_naive_utc_and_offset(nt, Utc)))
            }
            Some(Value::NULL) | None => Ok(None),
            _ => Err(err("tipo errado")),
        }
    };
    let dt = |i: usize| -> Result<DateTime<Utc>, crate::servicos::ErroAplicacao> {
        opt_dt(i)?.ok_or_else(|| err("data obrigatória nula"))
    };

    Ok(Arquivo {
        id: i(0)? as u32,
        nome_original: s(1)?,
        nome_armazenado: s(2)?,
        extensao: s(3)?,
        mime_type: s(4)?,
        tamanho: i(5)? as u64,
        hash_sha256: s(6)?,
        tipo: TipoArquivo::from_db_str(&s(7)?),
        storage_backend: s(8)?,
        storage_chave: s(9)?,
        duracao_segundos: opt_u32(10)?,
        codec: opt_str(11)?,
        taxa_amostral_hz: opt_u32(12)?,
        largura: opt_u32(13)?,
        altura: opt_u32(14)?,
        duracao_video_segundos: opt_u32(15)?,
        usuario_upload_id: i(16)? as u32,
        data_upload: dt(17)?,
        removido_em: opt_dt(18)?,
    })
}

// =============================================================================
// Vínculos
// =============================================================================

pub fn criar_vinculo(
    tenant_id: i32,
    arquivo_id: u32,
    entidade_tipo: TipoEntidade,
    entidade_id: u32,
    usuario_id: u32,
    papel: Option<&str>,
    observacao: Option<&str>,
) -> Result<u32, crate::servicos::ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        r#"INSERT INTO arquivo_vinculos
            (tenant_id, arquivo_id, entidade_tipo, entidade_id, usuario_id, papel, observacao)
           VALUES (?, ?, ?, ?, ?, ?, ?)"#,
        (
            tenant_id,
            arquivo_id,
            entidade_tipo.as_db_str(),
            entidade_id,
            usuario_id,
            papel,
            observacao,
        ),
    )
    .map_err(err)?;
    Ok(conn.last_insert_id() as u32)
}

pub fn listar_vinculos_arquivo(
    tenant_id: i32,
    arquivo_id: u32,
) -> Result<Vec<ArquivoVinculo>, crate::servicos::ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let rows: Vec<Row> = conn
        .exec(
            r#"SELECT id, arquivo_id, entidade_tipo, entidade_id, usuario_id,
                      data_vinculo, papel, observacao
               FROM arquivo_vinculos WHERE tenant_id = ? AND arquivo_id = ?"#,
            (tenant_id, arquivo_id),
        )
        .map_err(err)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(row_para_vinculo(r)?);
    }
    Ok(out)
}

pub fn listar_arquivos_por_entidade(
    tenant_id: i32,
    entidade_tipo: TipoEntidade,
    entidade_id: u32,
) -> Result<Vec<Arquivo>, crate::servicos::ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let rows: Vec<Row> = conn
        .exec(
            r#"SELECT a.id, a.nome_original, a.nome_armazenado, a.extensao, a.mime_type, a.tamanho,
                      a.hash_sha256, a.tipo, a.storage_backend, a.storage_chave,
                      a.duracao_segundos, a.codec, a.taxa_amostral_hz,
                      a.largura, a.altura, a.duracao_video_segundos,
                      a.usuario_upload_id, a.data_upload, a.removido_em
               FROM arquivos a
               INNER JOIN arquivo_vinculos v ON v.arquivo_id = a.id AND v.tenant_id = a.tenant_id
               WHERE a.tenant_id = ? AND v.entidade_tipo = ? AND v.entidade_id = ? AND a.removido_em IS NULL
               ORDER BY a.data_upload DESC"#,
            (tenant_id, entidade_tipo.as_db_str(), entidade_id),
        )
        .map_err(err)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(row_para_arquivo(r)?);
    }
    Ok(out)
}

pub fn remover_vinculo(tenant_id: i32, id: u32) -> Result<bool, crate::servicos::ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        "DELETE FROM arquivo_vinculos WHERE tenant_id = ? AND id = ?",
        (tenant_id, id),
    )
    .map_err(err)?;
    Ok(conn.affected_rows() > 0)
}

fn row_para_vinculo(r: Row) -> Result<ArquivoVinculo, crate::servicos::ErroAplicacao> {
    use mysql::Value;
    let s = |i: usize| -> Result<String, crate::servicos::ErroAplicacao> {
        match r.get::<Value, _>(i) {
            Some(Value::Bytes(b)) => Ok(String::from_utf8_lossy(&b).into_owned()),
            Some(Value::NULL) => Ok(String::new()),
            _ => Err(err("tipo errado")),
        }
    };
    let i = |i: usize| -> Result<i64, crate::servicos::ErroAplicacao> {
        match r.get::<Value, _>(i) {
            Some(Value::Int(n)) => Ok(n),
            Some(Value::UInt(n)) => Ok(n as i64),
            Some(Value::NULL) => Ok(0),
            _ => Err(err("tipo errado")),
        }
    };
    let opt_str = |i: usize| -> Result<Option<String>, crate::servicos::ErroAplicacao> {
        match r.get::<Value, _>(i) {
            Some(Value::Bytes(b)) => Ok(Some(String::from_utf8_lossy(&b).into_owned())),
            Some(Value::NULL) | None => Ok(None),
            _ => Err(err("tipo errado")),
        }
    };
    let opt_dt = |i: usize| -> Result<Option<DateTime<Utc>>, crate::servicos::ErroAplicacao> {
        match r.get::<Value, _>(i) {
            Some(Value::Date(y, m, d, h, mi, s, _us)) => {
                use chrono::NaiveDate;
                let nd = NaiveDate::from_ymd_opt(y as i32, m as u32, d as u32).unwrap();
                let nt = nd.and_hms_opt(h as u32, mi as u32, s as u32).unwrap();
                Ok(Some(DateTime::<Utc>::from_naive_utc_and_offset(nt, Utc)))
            }
            Some(Value::NULL) | None => Ok(None),
            _ => Err(err("tipo errado")),
        }
    };
    let dt = |i: usize| -> Result<DateTime<Utc>, crate::servicos::ErroAplicacao> {
        opt_dt(i)?.ok_or_else(|| err("data obrigatória nula"))
    };
    Ok(ArquivoVinculo {
        id: i(0)? as u32,
        arquivo_id: i(1)? as u32,
        entidade_tipo: TipoEntidade::from_db_str(&s(2)?),
        entidade_id: i(3)? as u32,
        usuario_id: i(4)? as u32,
        data_vinculo: dt(5)?,
        papel: opt_str(6)?,
        observacao: opt_str(7)?,
    })
}

// =============================================================================
// Thumbnails
// =============================================================================

pub fn inserir_thumbnail(
    tenant_id: i32,
    arquivo_id: u32,
    tamanho: TamanhoThumb,
    largura: u32,
    altura: u32,
    tamanho_bytes: u64,
    storage_chave: &str,
) -> Result<u32, crate::servicos::ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        r#"INSERT INTO arquivo_thumbnails
            (arquivo_id, tamanho, largura, altura, tamanho_bytes, storage_chave, tenant_id)
           VALUES (?, ?, ?, ?, ?, ?, ?)
           ON DUPLICATE KEY UPDATE
            largura=VALUES(largura), altura=VALUES(altura),
            tamanho_bytes=VALUES(tamanho_bytes), storage_chave=VALUES(storage_chave),
            data_geracao=NOW(), tenant_id=VALUES(tenant_id)"#,
        (
            arquivo_id,
            tamanho.as_db_str(),
            largura,
            altura,
            tamanho_bytes,
            storage_chave,
            tenant_id,
        ),
    )
    .map_err(err)?;
    Ok(conn.last_insert_id() as u32)
}

pub fn obter_thumbnail(
    tenant_id: i32,
    arquivo_id: u32,
    tamanho: TamanhoThumb,
) -> Result<Option<Thumbnail>, crate::servicos::ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<Row> = conn
        .exec_first(
            r#"SELECT id, arquivo_id, tamanho, largura, altura, tamanho_bytes,
                      storage_chave, data_geracao
               FROM arquivo_thumbnails WHERE tenant_id = ? AND arquivo_id = ? AND tamanho = ?"#,
            (tenant_id, arquivo_id, tamanho.as_db_str()),
        )
        .map_err(err)?;
    match row {
        Some(r) => Ok(Some(row_para_thumb(r)?)),
        None => Ok(None),
    }
}

fn row_para_thumb(r: Row) -> Result<Thumbnail, crate::servicos::ErroAplicacao> {
    use mysql::Value;
    let s = |i: usize| -> Result<String, crate::servicos::ErroAplicacao> {
        match r.get::<Value, _>(i) {
            Some(Value::Bytes(b)) => Ok(String::from_utf8_lossy(&b).into_owned()),
            Some(Value::NULL) => Ok(String::new()),
            _ => Err(err("tipo errado")),
        }
    };
    let i = |i: usize| -> Result<i64, crate::servicos::ErroAplicacao> {
        match r.get::<Value, _>(i) {
            Some(Value::Int(n)) => Ok(n),
            Some(Value::UInt(n)) => Ok(n as i64),
            Some(Value::NULL) => Ok(0),
            _ => Err(err("tipo errado")),
        }
    };
    let opt_dt = |i: usize| -> Result<Option<DateTime<Utc>>, crate::servicos::ErroAplicacao> {
        match r.get::<Value, _>(i) {
            Some(Value::Date(y, m, d, h, mi, s, _us)) => {
                use chrono::NaiveDate;
                let nd = NaiveDate::from_ymd_opt(y as i32, m as u32, d as u32).unwrap();
                let nt = nd.and_hms_opt(h as u32, mi as u32, s as u32).unwrap();
                Ok(Some(DateTime::<Utc>::from_naive_utc_and_offset(nt, Utc)))
            }
            Some(Value::NULL) | None => Ok(None),
            _ => Err(err("tipo errado")),
        }
    };
    Ok(Thumbnail {
        id: i(0)? as u32,
        arquivo_id: i(1)? as u32,
        tamanho: TamanhoThumb::from_db_str(&s(2)?),
        largura: i(3)? as u32,
        altura: i(4)? as u32,
        tamanho_bytes: i(5)? as u64,
        storage_chave: s(6)?,
        data_geracao: opt_dt(7)?.unwrap_or_else(Utc::now),
    })
}

// =============================================================================
// Transcrições
// =============================================================================

pub fn criar_transcricao(
    tenant_id: i32,
    arquivo_id: u32,
    engine: &str,
) -> Result<u32, crate::servicos::ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        r#"INSERT INTO transcricoes (arquivo_id, engine, status, data_inicio, tenant_id)
           VALUES (?, ?, 'PENDENTE', NOW(), ?)"#,
        (arquivo_id, engine, tenant_id),
    )
    .map_err(err)?;
    Ok(conn.last_insert_id() as u32)
}

pub fn atualizar_status_transcricao(
    tenant_id: i32,
    id: u32,
    status: StatusTranscricao,
    erro: Option<&str>,
) -> Result<bool, crate::servicos::ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        r#"UPDATE transcricoes SET status = ?, erro_mensagem = ? WHERE tenant_id = ? AND id = ?"#,
        (status.as_db_str(), erro, tenant_id, id),
    )
    .map_err(err)?;
    Ok(conn.affected_rows() > 0)
}

pub fn atualizar_resultado_transcricao(
    tenant_id: i32,
    id: u32,
    texto: &str,
    idioma: Option<&str>,
    confianca: Option<f32>,
    tempo_ms: u32,
) -> Result<bool, crate::servicos::ErroAplicacao> {
    let mut conn = obter_conexao()?;
    conn.exec_drop(
        r#"UPDATE transcricoes
           SET status = 'CONCLUIDA', texto = ?, idioma = ?, confianca = ?,
               tempo_processamento_ms = ?, data_fim = NOW()
           WHERE tenant_id = ? AND id = ?"#,
        (texto, idioma, confianca, tempo_ms, tenant_id, id),
    )
    .map_err(err)?;
    Ok(conn.affected_rows() > 0)
}

pub fn obter_transcricao(
    tenant_id: i32,
    arquivo_id: u32,
) -> Result<Option<Transcricao>, crate::servicos::ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let row: Option<Row> = conn
        .exec_first(
            r#"SELECT id, arquivo_id, engine, engine_modelo, texto, idioma, confianca,
                      tempo_processamento_ms, data_inicio, data_fim, status, erro_mensagem
               FROM transcricoes WHERE tenant_id = ? AND arquivo_id = ? LIMIT 1"#,
            (tenant_id, arquivo_id),
        )
        .map_err(err)?;
    match row {
        Some(r) => Ok(Some(row_para_transcricao(r)?)),
        None => Ok(None),
    }
}

fn row_para_transcricao(r: Row) -> Result<Transcricao, crate::servicos::ErroAplicacao> {
    use mysql::Value;
    let s = |i: usize| -> Result<String, crate::servicos::ErroAplicacao> {
        match r.get::<Value, _>(i) {
            Some(Value::Bytes(b)) => Ok(String::from_utf8_lossy(&b).into_owned()),
            Some(Value::NULL) => Ok(String::new()),
            _ => Err(err("tipo errado")),
        }
    };
    let i = |i: usize| -> Result<i64, crate::servicos::ErroAplicacao> {
        match r.get::<Value, _>(i) {
            Some(Value::Int(n)) => Ok(n),
            Some(Value::UInt(n)) => Ok(n as i64),
            Some(Value::NULL) => Ok(0),
            _ => Err(err("tipo errado")),
        }
    };
    let opt_str = |i: usize| -> Result<Option<String>, crate::servicos::ErroAplicacao> {
        match r.get::<Value, _>(i) {
            Some(Value::Bytes(b)) => Ok(Some(String::from_utf8_lossy(&b).into_owned())),
            Some(Value::NULL) | None => Ok(None),
            _ => Err(err("tipo errado")),
        }
    };
    let opt_f32 = |i: usize| -> Result<Option<f32>, crate::servicos::ErroAplicacao> {
        match r.get::<Value, _>(i) {
            Some(Value::Float(f)) => Ok(Some(f)),
            Some(Value::Double(f)) => Ok(Some(f as f32)),
            Some(Value::NULL) | None => Ok(None),
            _ => Err(err("tipo errado")),
        }
    };
    let opt_dt = |i: usize| -> Result<Option<DateTime<Utc>>, crate::servicos::ErroAplicacao> {
        match r.get::<Value, _>(i) {
            Some(Value::Date(y, m, d, h, mi, s, _us)) => {
                use chrono::NaiveDate;
                let nd = NaiveDate::from_ymd_opt(y as i32, m as u32, d as u32).unwrap();
                let nt = nd.and_hms_opt(h as u32, mi as u32, s as u32).unwrap();
                Ok(Some(DateTime::<Utc>::from_naive_utc_and_offset(nt, Utc)))
            }
            Some(Value::NULL) | None => Ok(None),
            _ => Err(err("tipo errado")),
        }
    };
    Ok(Transcricao {
        id: i(0)? as u32,
        arquivo_id: i(1)? as u32,
        engine: s(2)?,
        engine_modelo: opt_str(3)?,
        texto: opt_str(4)?,
        idioma: opt_str(5)?,
        confianca: opt_f32(6)?,
        tempo_processamento_ms: opt_str(7).ok().flatten().and_then(|s| s.parse().ok()),
        data_inicio: opt_dt(8)?,
        data_fim: opt_dt(9)?,
        status: StatusTranscricao::from_db_str(&s(10)?),
        erro_mensagem: opt_str(11)?,
    })
}
