// src/cotacao_orcamento/service.rs
//
// Regras de negócio do módulo Cotação/Orçamento.
// Orquestra repository + timeline CRM + audit log.

use super::models::{
    Aprovacao, CategoriaItem, Cotacao, CotacaoAnexo, CotacaoCompleta, CotacaoItem, Decisao,
    HistoricoEntrada, Orcamento, OrcamentoCompleto, OrcamentoItem, StatusCotacao, StatusOrcamento,
    TipoAnexo,
};
use super::repository;
use crate::banco_de_dados::audit::{self, Acao};
use crate::servicos::ErroAplicacao;
use mysql::prelude::Queryable;

/// Contexto de quem está fazendo a operação (mesmo padrão do CRM).
#[derive(Debug, Clone, Default)]
pub struct Contexto {
    pub usuario_id: Option<u32>,
    pub username: Option<String>,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<String>,
}

// =============================================================================
// Timeline (CRM)
// =============================================================================

fn registrar_evento_crm(
    tenant_id: i32,
    cliente_id: u32,
    tipo_db: &str,
    descricao: &str,
    payload: Option<&serde_json::Value>,
    ctx: &Contexto,
) {
    let res = crate::crm::repository::inserir_evento_timeline(
        tenant_id,
        cliente_id,
        tipo_db,
        descricao,
        payload,
        ctx.usuario_id,
        ctx.username.as_deref(),
    );
    if let Err(e) = res {
        tracing::warn!(
            "Falha ao registrar evento CRM (cliente={}, tipo={}): {:?}",
            cliente_id,
            tipo_db,
            e
        );
    }
}

fn registrar_audit(
    acao: Acao,
    entidade: &str,
    entity_id: Option<&str>,
    antes: Option<serde_json::Value>,
    depois: Option<serde_json::Value>,
    ctx: &Contexto,
) {
    let _ = audit::registrar_para(
        ctx.usuario_id,
        ctx.username.as_deref(),
        acao,
        entidade,
        entity_id,
        antes,
        depois,
        ctx.ip.as_deref(),
        ctx.user_agent.as_deref(),
        ctx.request_id.as_deref(),
    );
}

// =============================================================================
// Cotações
// =============================================================================

pub fn criar_cotacao(
    tenant_id: i32,
    os_id: Option<u32>,
    cliente_id: u32,
    descricao: &str,
    observacoes: Option<&str>,
    ctx: &Contexto,
) -> Result<Cotacao, ErroAplicacao> {
    if descricao.trim().is_empty() {
        return Err(ErroAplicacao::Desconhecido("descrição vazia".into()));
    }
    let cot = repository::criar_cotacao(
        tenant_id,
        os_id,
        cliente_id,
        descricao,
        observacoes,
        ctx.usuario_id,
        ctx.username.as_deref(),
    )?;
    let descricao = format!(
        "{} criou cotação #{} (cliente {})",
        ctx.username.as_deref().unwrap_or("anônimo"),
        cot.id,
        cliente_id
    );
    let payload = serde_json::json!({
        "cotacao_id": cot.id,
        "os_id": os_id,
    });
    registrar_evento_crm(
        tenant_id,
        cliente_id,
        "COTACAO_CRIADA",
        &descricao,
        Some(&payload),
        ctx,
    );
    registrar_audit(
        Acao::Create,
        "cotacoes",
        Some(&cot.id.to_string()),
        None,
        Some(serde_json::to_value(&cot).unwrap_or_default()),
        ctx,
    );
    Ok(cot)
}

pub fn obter_cotacao(tenant_id: i32, id: u32) -> Result<Option<Cotacao>, ErroAplicacao> {
    repository::obter_cotacao(tenant_id, id)
}

pub fn obter_cotacao_completa(
    tenant_id: i32,
    id: u32,
) -> Result<Option<CotacaoCompleta>, ErroAplicacao> {
    repository::obter_cotacao_completa(tenant_id, id)
}

pub fn listar_cotacoes(
    tenant_id: i32,
    status: Option<&str>,
    cliente_id: Option<u32>,
    limite: u32,
) -> Result<Vec<Cotacao>, ErroAplicacao> {
    repository::listar_cotacoes(tenant_id, status, cliente_id, limite)
}

pub fn transicionar_status_cotacao(
    tenant_id: i32,
    id: u32,
    novo: StatusCotacao,
    ctx: &Contexto,
) -> Result<Cotacao, ErroAplicacao> {
    let atual = repository::obter_cotacao(tenant_id, id)?
        .ok_or(ErroAplicacao::Desconhecido("cotação não encontrada".into()))?;
    let status_atual = StatusCotacao::from_db_str(&atual.status)
        .ok_or_else(|| ErroAplicacao::Desconhecido(format!("status inválido: {}", atual.status)))?;
    if !status_atual.pode_transicionar_para(novo) {
        return Err(ErroAplicacao::Desconhecido(format!(
            "transição inválida: {} → {}",
            status_atual.as_db_str(),
            novo.as_db_str()
        )));
    }
    repository::atualizar_status_cotacao(tenant_id, id, novo)?;
    let cot = repository::obter_cotacao(tenant_id, id)?
        .ok_or(ErroAplicacao::Desconhecido("cotação sumiu".into()))?;
    let descricao = format!(
        "{} mudou status: {} → {}",
        ctx.username.as_deref().unwrap_or("anônimo"),
        status_atual.as_db_str(),
        novo.as_db_str()
    );
    let payload = serde_json::json!({
        "cotacao_id": id,
        "status_anterior": status_atual.as_db_str(),
        "status_novo": novo.as_db_str(),
    });
    registrar_evento_crm(
        tenant_id,
        atual.cliente_id,
        "COTACAO_STATUS_ALTERADO",
        &descricao,
        Some(&payload),
        ctx,
    );
    Ok(cot)
}

pub fn remover_cotacao(tenant_id: i32, id: u32, ctx: &Contexto) -> Result<bool, ErroAplicacao> {
    let cot = repository::obter_cotacao(tenant_id, id)?
        .ok_or(ErroAplicacao::Desconhecido("cotação não encontrada".into()))?;
    let removed = repository::remover_cotacao(tenant_id, id)?;
    if removed {
        let descricao = format!(
            "{} removeu cotação #{}",
            ctx.username.as_deref().unwrap_or("anônimo"),
            id
        );
        registrar_evento_crm(
            tenant_id,
            cot.cliente_id,
            "COTACAO_REMOVIDA",
            &descricao,
            None,
            ctx,
        );
        registrar_audit(
            Acao::Delete,
            "cotacoes",
            Some(&id.to_string()),
            Some(serde_json::to_value(&cot).unwrap_or_default()),
            None,
            ctx,
        );
    }
    Ok(removed)
}

// =============================================================================
// Itens de cotação
// =============================================================================

pub fn adicionar_item_cotacao(
    tenant_id: i32,
    cotacao_id: u32,
    nome: &str,
    quantidade: f64,
    valor_estimado: f64,
    observacao: Option<&str>,
    ctx: &Contexto,
) -> Result<CotacaoItem, ErroAplicacao> {
    if nome.trim().is_empty() {
        return Err(ErroAplicacao::Desconhecido("nome vazio".into()));
    }
    if quantidade <= 0.0 {
        return Err(ErroAplicacao::Desconhecido(
            "quantidade deve ser > 0".into(),
        ));
    }
    let item = repository::adicionar_item_cotacao(
        tenant_id,
        cotacao_id,
        nome,
        quantidade,
        valor_estimado,
        observacao,
    )?;
    let cot = repository::obter_cotacao(tenant_id, cotacao_id)?
        .ok_or(ErroAplicacao::Desconhecido("cotação não encontrada".into()))?;
    let descricao = format!(
        "{} adicionou item à cotação #{}: {} x{}",
        ctx.username.as_deref().unwrap_or("anônimo"),
        cotacao_id,
        nome,
        quantidade
    );
    let payload = serde_json::json!({
        "cotacao_id": cotacao_id,
        "item_id": item.id,
        "nome": nome,
    });
    registrar_evento_crm(
        tenant_id,
        cot.cliente_id,
        "COTACAO_ITEM_ADICIONADO",
        &descricao,
        Some(&payload),
        ctx,
    );
    Ok(item)
}

pub fn listar_itens_cotacao(
    tenant_id: i32,
    cotacao_id: u32,
) -> Result<Vec<CotacaoItem>, ErroAplicacao> {
    repository::listar_itens_cotacao(tenant_id, cotacao_id)
}

pub fn remover_item_cotacao(
    tenant_id: i32,
    id: u32,
    cotacao_id: u32,
    ctx: &Contexto,
) -> Result<bool, ErroAplicacao> {
    let removed = repository::remover_item_cotacao(tenant_id, id)?;
    if removed {
        let cot = repository::obter_cotacao(tenant_id, cotacao_id)?;
        if let Some(cot) = cot {
            let descricao = format!(
                "{} removeu item #{} da cotação #{}",
                ctx.username.as_deref().unwrap_or("anônimo"),
                id,
                cotacao_id
            );
            registrar_evento_crm(
                tenant_id,
                cot.cliente_id,
                "COTACAO_ITEM_REMOVIDO",
                &descricao,
                None,
                ctx,
            );
        }
    }
    Ok(removed)
}

// =============================================================================
// Anexos
// =============================================================================

pub fn adicionar_anexo(
    tenant_id: i32,
    cotacao_id: u32,
    tipo: TipoAnexo,
    nome: &str,
    arquivo_path: &str,
    tamanho_bytes: Option<i64>,
    mime_type: Option<&str>,
    hash_sha256: Option<&str>,
    ctx: &Contexto,
) -> Result<CotacaoAnexo, ErroAplicacao> {
    if nome.trim().is_empty() || arquivo_path.trim().is_empty() {
        return Err(ErroAplicacao::Desconhecido("nome/path vazio".into()));
    }
    let anexo = repository::adicionar_anexo(
        tenant_id,
        cotacao_id,
        tipo,
        nome,
        arquivo_path,
        tamanho_bytes,
        mime_type,
        hash_sha256,
        ctx.usuario_id,
        ctx.username.as_deref(),
    )?;
    let cot = repository::obter_cotacao(tenant_id, cotacao_id)?;
    if let Some(cot) = cot {
        let descricao = format!(
            "{} anexou {} em cotação #{}",
            ctx.username.as_deref().unwrap_or("anônimo"),
            tipo.as_db_str(),
            cotacao_id
        );
        let payload = serde_json::json!({
            "cotacao_id": cotacao_id,
            "anexo_id": anexo.id,
            "tipo": tipo.as_db_str(),
        });
        registrar_evento_crm(
            tenant_id,
            cot.cliente_id,
            "ANEXO_ADICIONADO",
            &descricao,
            Some(&payload),
            ctx,
        );
    }
    Ok(anexo)
}

pub fn listar_anexos(tenant_id: i32, cotacao_id: u32) -> Result<Vec<CotacaoAnexo>, ErroAplicacao> {
    repository::listar_anexos(tenant_id, cotacao_id)
}

pub fn remover_anexo(
    tenant_id: i32,
    id: u32,
    cotacao_id: u32,
    ctx: &Contexto,
) -> Result<bool, ErroAplicacao> {
    let removed = repository::remover_anexo(tenant_id, id)?;
    if removed {
        let cot = repository::obter_cotacao(tenant_id, cotacao_id)?;
        if let Some(cot) = cot {
            let descricao = format!(
                "{} removeu anexo #{} da cotação #{}",
                ctx.username.as_deref().unwrap_or("anônimo"),
                id,
                cotacao_id
            );
            registrar_evento_crm(
                tenant_id,
                cot.cliente_id,
                "ANEXO_REMOVIDO",
                &descricao,
                None,
                ctx,
            );
        }
    }
    Ok(removed)
}

// =============================================================================
// Conversão Cotação → Orçamento
// =============================================================================

/// Converte uma cotação (status COTADO) em um orçamento.
/// Cria o orçamento, copia os itens, calcula totais.
pub fn converter_cotacao_para_orcamento(
    tenant_id: i32,
    cotacao_id: u32,
    desconto: f64,
    impostos_estimado: f64,
    ctx: &Contexto,
) -> Result<Orcamento, ErroAplicacao> {
    if desconto < 0.0 {
        return Err(ErroAplicacao::Desconhecido(
            "desconto não pode ser negativo".into(),
        ));
    }
    if impostos_estimado < 0.0 {
        return Err(ErroAplicacao::Desconhecido(
            "impostos não pode ser negativo".into(),
        ));
    }
    let cot = repository::obter_cotacao(tenant_id, cotacao_id)?
        .ok_or(ErroAplicacao::Desconhecido("cotação não encontrada".into()))?;
    let status = StatusCotacao::from_db_str(&cot.status)
        .ok_or_else(|| ErroAplicacao::Desconhecido(format!("status inválido: {}", cot.status)))?;
    if !matches!(
        status,
        StatusCotacao::Cotado | StatusCotacao::AguardandoAprovacao
    ) {
        return Err(ErroAplicacao::Desconhecido(format!(
            "cotação precisa estar COTADO ou AGUARDANDO_APROVACAO (atual: {})",
            status.as_db_str()
        )));
    }

    // 1. Criar orçamento
    let orc = repository::criar_orcamento(
        tenant_id,
        cot.cliente_id,
        Some(cotacao_id),
        cot.observacoes.as_deref(),
        ctx.usuario_id,
        ctx.username.as_deref(),
    )?;

    // 2. Copiar itens
    let itens = repository::listar_itens_cotacao(tenant_id, cotacao_id)?;
    let mut subtotal = 0.0_f64;
    for item in &itens {
        // Item de cotação → item de orçamento (categoria OUTRO pois é estimativa)
        let peca_id: Option<u32> = None;
        let servico_id: Option<u32> = None;
        repository::adicionar_item_orcamento(
            tenant_id,
            orc.id,
            &item.nome,
            item.quantidade.round() as u32, // arredonda
            item.valor_estimado,
            CategoriaItem::Outro,
            peca_id,
            servico_id,
            item.observacao.as_deref(),
        )?;
        subtotal += item.quantidade * item.valor_estimado;
    }
    let total = subtotal - desconto + impostos_estimado;
    repository::atualizar_totais_orcamento(
        tenant_id,
        orc.id,
        subtotal,
        desconto,
        impostos_estimado,
        total,
    )?;

    // 3. Atualizar status do orçamento para AGUARDANDO_APROVACAO
    repository::atualizar_status_orcamento(
        tenant_id,
        orc.id,
        StatusOrcamento::AguardandoAprovacao,
    )?;
    repository::registrar_historico(
        tenant_id,
        orc.id,
        Some(StatusOrcamento::Rascunho),
        StatusOrcamento::AguardandoAprovacao,
        Some("Gerado a partir de cotação"),
        ctx.usuario_id,
        ctx.username.as_deref(),
    )?;

    // 4. Vincular cotação ao orçamento
    repository::definir_orcamento_da_cotacao(tenant_id, cotacao_id, orc.id)?;

    // 5. Timeline + audit
    let descricao = format!(
        "{} gerou orçamento #{} a partir da cotação #{}",
        ctx.username.as_deref().unwrap_or("anônimo"),
        orc.id,
        cotacao_id
    );
    let payload = serde_json::json!({
        "cotacao_id": cotacao_id,
        "orcamento_id": orc.id,
        "total": total,
    });
    registrar_evento_crm(
        tenant_id,
        cot.cliente_id,
        "ORCAMENTO_CRIADO",
        &descricao,
        Some(&payload),
        ctx,
    );
    registrar_audit(
        Acao::Create,
        "orcamentos",
        Some(&orc.id.to_string()),
        None,
        Some(serde_json::to_value(&orc).unwrap_or_default()),
        ctx,
    );

    repository::obter_orcamento(tenant_id, orc.id)?
        .ok_or(ErroAplicacao::Desconhecido("orçamento sumiu".into()))
}

// =============================================================================
// Orçamentos
// =============================================================================

pub fn obter_orcamento(tenant_id: i32, id: u32) -> Result<Option<Orcamento>, ErroAplicacao> {
    repository::obter_orcamento(tenant_id, id)
}

pub fn obter_orcamento_completo(
    tenant_id: i32,
    id: u32,
) -> Result<Option<OrcamentoCompleto>, ErroAplicacao> {
    repository::obter_orcamento_completo(tenant_id, id)
}

pub fn listar_orcamentos(
    tenant_id: i32,
    status: Option<&str>,
    cliente_id: Option<u32>,
    limite: u32,
) -> Result<Vec<Orcamento>, ErroAplicacao> {
    repository::listar_orcamentos(tenant_id, status, cliente_id, limite)
}

pub fn adicionar_item_orcamento(
    tenant_id: i32,
    orcamento_id: u32,
    descricao: &str,
    quantidade: u32,
    preco_unitario: f64,
    categoria: CategoriaItem,
    peca_id: Option<u32>,
    servico_id: Option<u32>,
    observacao: Option<&str>,
    ctx: &Contexto,
) -> Result<OrcamentoItem, ErroAplicacao> {
    if descricao.trim().is_empty() {
        return Err(ErroAplicacao::Desconhecido("descrição vazia".into()));
    }
    if quantidade == 0 {
        return Err(ErroAplicacao::Desconhecido(
            "quantidade deve ser > 0".into(),
        ));
    }
    if preco_unitario < 0.0 {
        return Err(ErroAplicacao::Desconhecido(
            "preço não pode ser negativo".into(),
        ));
    }
    let item = repository::adicionar_item_orcamento(
        tenant_id,
        orcamento_id,
        descricao,
        quantidade,
        preco_unitario,
        categoria,
        peca_id,
        servico_id,
        observacao,
    )?;
    recalcular_totais(tenant_id, orcamento_id)?;
    let orc = repository::obter_orcamento(tenant_id, orcamento_id)?.ok_or(
        ErroAplicacao::Desconhecido("orçamento não encontrado".into()),
    )?;
    let descricao = format!(
        "{} adicionou item ao orçamento #{}: {} x{}",
        ctx.username.as_deref().unwrap_or("anônimo"),
        orcamento_id,
        descricao,
        quantidade
    );
    let payload = serde_json::json!({
        "orcamento_id": orcamento_id,
        "item_id": item.id,
    });
    registrar_evento_crm(
        tenant_id,
        orc.cliente_id,
        "ORCAMENTO_ITEM_ADICIONADO",
        &descricao,
        Some(&payload),
        ctx,
    );
    Ok(item)
}

/// Recalcula subtotal, total, etc. baseado nos itens atuais.
fn recalcular_totais(tenant_id: i32, orcamento_id: u32) -> Result<(), ErroAplicacao> {
    let orc = repository::obter_orcamento(tenant_id, orcamento_id)?.ok_or(
        ErroAplicacao::Desconhecido("orçamento não encontrado".into()),
    )?;
    let itens = repository::listar_itens_orcamento(tenant_id, orcamento_id)?;
    let subtotal: f64 = itens.iter().map(|i| i.preco_total).sum();
    let total = subtotal - orc.desconto + orc.impostos_estimado;
    repository::atualizar_totais_orcamento(
        tenant_id,
        orcamento_id,
        subtotal,
        orc.desconto,
        orc.impostos_estimado,
        total,
    )?;
    Ok(())
}

pub fn aplicar_desconto(
    tenant_id: i32,
    orcamento_id: u32,
    desconto: f64,
    ctx: &Contexto,
) -> Result<Orcamento, ErroAplicacao> {
    if desconto < 0.0 {
        return Err(ErroAplicacao::Desconhecido(
            "desconto não pode ser negativo".into(),
        ));
    }
    let orc_antes = repository::obter_orcamento(tenant_id, orcamento_id)?.ok_or(
        ErroAplicacao::Desconhecido("orçamento não encontrado".into()),
    )?;
    recalcular_totais(tenant_id, orcamento_id)?;
    // Atualiza desconto
    let mut conn = crate::banco_de_dados::conexao::obter_conexao()?;
    use mysql::params;
    conn.exec_drop(
        "UPDATE orcamentos SET desconto = :d WHERE tenant_id = :tid AND id = :id",
        params! { "d" => desconto, "tid" => tenant_id, "id" => orcamento_id },
    )
    .map_err(ErroAplicacao::from)?;
    recalcular_totais(tenant_id, orcamento_id)?;
    let orc = repository::obter_orcamento(tenant_id, orcamento_id)?.ok_or(
        ErroAplicacao::Desconhecido("orçamento não encontrado".into()),
    )?;
    let descricao = format!(
        "{} alterou desconto de R$ {:.2} para R$ {:.2}",
        ctx.username.as_deref().unwrap_or("anônimo"),
        orc_antes.desconto,
        desconto
    );
    let payload = serde_json::json!({
        "orcamento_id": orcamento_id,
        "desconto_anterior": orc_antes.desconto,
        "desconto_novo": desconto,
    });
    registrar_evento_crm(
        tenant_id,
        orc.cliente_id,
        "ORCAMENTO_DESCONTO_ALTERADO",
        &descricao,
        Some(&payload),
        ctx,
    );
    Ok(orc)
}

pub fn remover_item_orcamento(
    tenant_id: i32,
    id: u32,
    orcamento_id: u32,
    ctx: &Contexto,
) -> Result<bool, ErroAplicacao> {
    let removed = repository::remover_item_orcamento(tenant_id, id)?;
    if removed {
        recalcular_totais(tenant_id, orcamento_id)?;
        if let Some(orc) = repository::obter_orcamento(tenant_id, orcamento_id)? {
            let descricao = format!(
                "{} removeu item #{} do orçamento #{}",
                ctx.username.as_deref().unwrap_or("anônimo"),
                id,
                orcamento_id
            );
            registrar_evento_crm(
                tenant_id,
                orc.cliente_id,
                "ORCAMENTO_ITEM_REMOVIDO",
                &descricao,
                None,
                ctx,
            );
        }
    }
    Ok(removed)
}

// =============================================================================
// Aprovação
// =============================================================================

/// Decide um orçamento: aprova ou rejeita.
/// Também atualiza o status, registra no audit e na timeline.
pub fn decidir_orcamento(
    tenant_id: i32,
    orcamento_id: u32,
    decisao: Decisao,
    observacao: Option<&str>,
    ctx: &Contexto,
) -> Result<Orcamento, ErroAplicacao> {
    let orc_antes = repository::obter_orcamento(tenant_id, orcamento_id)?.ok_or(
        ErroAplicacao::Desconhecido("orçamento não encontrado".into()),
    )?;
    let status_antes = StatusOrcamento::from_db_str(&orc_antes.status).ok_or_else(|| {
        ErroAplicacao::Desconhecido(format!("status inválido: {}", orc_antes.status))
    })?;
    if !matches!(status_antes, StatusOrcamento::AguardandoAprovacao) {
        return Err(ErroAplicacao::Desconhecido(format!(
            "orçamento precisa estar AGUARDANDO_APROVACAO (atual: {})",
            status_antes.as_db_str()
        )));
    }
    let novo_status = match decisao {
        Decisao::Aprovado => StatusOrcamento::Aprovado,
        Decisao::Rejeitado => StatusOrcamento::Rejeitado,
    };
    repository::registrar_decisao_orcamento(
        tenant_id,
        orcamento_id,
        decisao,
        observacao,
        ctx.usuario_id,
        ctx.username.as_deref(),
    )?;
    repository::registrar_historico(
        tenant_id,
        orcamento_id,
        Some(status_antes),
        novo_status,
        observacao,
        ctx.usuario_id,
        ctx.username.as_deref(),
    )?;
    let orc = repository::obter_orcamento(tenant_id, orcamento_id)?.ok_or(
        ErroAplicacao::Desconhecido("orçamento não encontrado".into()),
    )?;
    let descricao = format!(
        "{} {} orçamento #{} (cliente {}, total R$ {:.2})",
        ctx.username.as_deref().unwrap_or("anônimo"),
        decisao.as_db_str().to_lowercase(),
        orcamento_id,
        orc.cliente_id,
        orc.total
    );
    let tipo_evento = match decisao {
        Decisao::Aprovado => "ORCAMENTO_APROVADO",
        Decisao::Rejeitado => "ORCAMENTO_REJEITADO",
    };
    let payload = serde_json::json!({
        "orcamento_id": orcamento_id,
        "decisao": decisao.as_db_str(),
        "total": orc.total,
    });
    registrar_evento_crm(
        tenant_id,
        orc.cliente_id,
        tipo_evento,
        &descricao,
        Some(&payload),
        ctx,
    );

    // Hook P2.5 — Geração automática de ContaReceber quando APROVADO
    if matches!(decisao, Decisao::Aprovado) {
        let fin_ctx = crate::financial::Contexto {
            usuario_id: ctx.usuario_id.unwrap_or(0),
            username: ctx.username.clone(),
            ip: ctx.ip.clone(),
            user_agent: ctx.user_agent.clone(),
            request_id: ctx.request_id.clone(),
        };
        // Converte total para centavos (orc.total está em reais f64)
        let total_centavos = (orc.total * 100.0).round() as i64;
        let _ = crate::financial::service::gerar_conta_receber_de_orcamento(
            tenant_id,
            orc.cliente_id,
            orcamento_id,
            total_centavos,
            &format!("OS do orçamento #{}", orcamento_id),
            &fin_ctx,
        );
    }
    Ok(orc)
}

pub fn finalizar_orcamento(
    tenant_id: i32,
    orcamento_id: u32,
    ctx: &Contexto,
) -> Result<Orcamento, ErroAplicacao> {
    let orc = repository::obter_orcamento(tenant_id, orcamento_id)?.ok_or(
        ErroAplicacao::Desconhecido("orçamento não encontrado".into()),
    )?;
    let status_atual = StatusOrcamento::from_db_str(&orc.status)
        .ok_or_else(|| ErroAplicacao::Desconhecido(format!("status inválido: {}", orc.status)))?;
    if !matches!(status_atual, StatusOrcamento::Aprovado) {
        return Err(ErroAplicacao::Desconhecido(format!(
            "orçamento precisa estar APROVADO (atual: {})",
            status_atual.as_db_str()
        )));
    }
    repository::atualizar_status_orcamento(tenant_id, orcamento_id, StatusOrcamento::Finalizado)?;
    repository::registrar_historico(
        tenant_id,
        orcamento_id,
        Some(status_atual),
        StatusOrcamento::Finalizado,
        None,
        ctx.usuario_id,
        ctx.username.as_deref(),
    )?;
    let orc = repository::obter_orcamento(tenant_id, orcamento_id)?.ok_or(
        ErroAplicacao::Desconhecido("orçamento não encontrado".into()),
    )?;
    let descricao = format!(
        "{} finalizou orçamento #{}",
        ctx.username.as_deref().unwrap_or("anônimo"),
        orcamento_id
    );
    let payload = serde_json::json!({ "orcamento_id": orcamento_id });
    registrar_evento_crm(
        tenant_id,
        orc.cliente_id,
        "ORCAMENTO_FINALIZADO",
        &descricao,
        Some(&payload),
        ctx,
    );
    Ok(orc)
}

pub fn listar_aprovacoes(
    tenant_id: i32,
    orcamento_id: u32,
) -> Result<Vec<Aprovacao>, ErroAplicacao> {
    repository::listar_aprovacoes(tenant_id, orcamento_id)
}

pub fn listar_historico(
    tenant_id: i32,
    orcamento_id: u32,
) -> Result<Vec<HistoricoEntrada>, ErroAplicacao> {
    repository::listar_historico(tenant_id, orcamento_id)
}

pub fn remover_orcamento(tenant_id: i32, id: u32, ctx: &Contexto) -> Result<bool, ErroAplicacao> {
    let orc = repository::obter_orcamento(tenant_id, id)?.ok_or(ErroAplicacao::Desconhecido(
        "orçamento não encontrado".into(),
    ))?;
    let removed = repository::remover_orcamento(tenant_id, id)?;
    if removed {
        let descricao = format!(
            "{} removeu orçamento #{}",
            ctx.username.as_deref().unwrap_or("anônimo"),
            id
        );
        registrar_evento_crm(
            tenant_id,
            orc.cliente_id,
            "ORCAMENTO_REMOVIDO",
            &descricao,
            None,
            ctx,
        );
        registrar_audit(
            Acao::Delete,
            "orcamentos",
            Some(&id.to_string()),
            Some(serde_json::to_value(&orc).unwrap_or_default()),
            None,
            ctx,
        );
    }
    Ok(removed)
}

// =============================================================================
// Dashboard
// =============================================================================

pub fn dashboard_cotacoes(tenant_id: i32) -> super::models::DashboardCotaoes {
    let (abertas, aguardando, aprovadas, rejeitadas, finalizadas) =
        repository::contar_cotacoes_por_status(tenant_id).unwrap_or((0, 0, 0, 0, 0));
    super::models::DashboardCotaoes {
        cotacoes_abertas: abertas,
        aguardando_aprovacao: aguardando,
        aprovadas,
        rejeitadas,
        finalizadas,
    }
}

// =============================================================================
// Wrappers de criação (para o handler que não precisa expor repository)
// =============================================================================

/// Cria um orçamento vazio (sem itens). Usado pelo handler `POST /orcamentos`.
pub fn criar_orcamento_vazio(
    tenant_id: i32,
    cliente_id: u32,
    cotacao_origem_id: Option<u32>,
    observacoes: Option<&str>,
    ctx: &Contexto,
) -> Result<Orcamento, ErroAplicacao> {
    let orc = repository::criar_orcamento(
        tenant_id,
        cliente_id,
        cotacao_origem_id,
        observacoes,
        ctx.usuario_id,
        ctx.username.as_deref(),
    )?;
    repository::registrar_historico(
        tenant_id,
        orc.id,
        None,
        StatusOrcamento::Rascunho,
        Some("Orçamento criado"),
        ctx.usuario_id,
        ctx.username.as_deref(),
    )?;
    let payload = serde_json::json!({
        "orcamento_id": orc.id,
        "cliente_id": cliente_id,
    });
    registrar_evento_crm(
        tenant_id,
        cliente_id,
        "ORCAMENTO_CRIADO",
        "orçamento criado",
        Some(&payload),
        ctx,
    );
    Ok(orc)
}
