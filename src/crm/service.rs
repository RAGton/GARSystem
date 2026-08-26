// src/crm/service.rs
//
// Camada de serviço do CRM. Orquestra repository + timeline + audit.
//
// Toda mutação pública aqui:
//   1. Executa a operação no repository
//   2. Registra evento na timeline (best-effort)
//   3. Registra no audit log (best-effort)
//
// O dashboard agrega dados de várias tabelas numa única response para
// a UI do cliente.

use super::models::TipoEventoTimeline;
use super::models::{Contato, DashboardCliente, Equipamento, Observacao, Tag, TimelineEvento};
use super::repository;
use crate::banco_de_dados::audit::{self, Acao};
use crate::servicos::ErroAplicacao;

// =============================================================================
// Helpers para o dashboard
// =============================================================================

/// Struct intermediária para query ao DB de OS recentes.
#[derive(Debug, mysql::prelude::FromRow)]
struct OsRecenteRow {
    id: u32,
    cliente: Option<String>,
    equipamento: String,
    defeito: Option<String>,
    status: String,
    parecer_tecnico: Option<String>,
    situacao: String,
    numero_serie: Option<String>,
    observacoes: Option<String>,
    tecnico_responsavel: Option<String>,
    atendente: Option<String>,
    telefone: Option<String>,
    data_chegada: String,
    prazo_entrega: String,
}

impl From<OsRecenteRow> for serde_json::Value {
    fn from(r: OsRecenteRow) -> Self {
        serde_json::json!({
            "id": r.id,
            "cliente": r.cliente,
            "equipamento": r.equipamento,
            "defeito": r.defeito,
            "status": r.status,
            "parecer_tecnico": r.parecer_tecnico,
            "situacao": r.situacao,
            "numero_serie": r.numero_serie,
            "observacoes": r.observacoes,
            "tecnico_responsavel": r.tecnico_responsavel,
            "atendente": r.atendente,
            "telefone": r.telefone,
            "data_chegada": r.data_chegada,
            "prazo_entrega": r.prazo_entrega,
        })
    }
}

/// Busca as últimas N ordens de serviço de um cliente.
pub fn crm_service_buscar_os_recentes(
    tenant_id: i32,
    cliente_id: u32,
    limite: u32,
) -> Vec<serde_json::Value> {
    use mysql::params;
    use mysql::prelude::Queryable;
    let mut conn = match crate::banco_de_dados::conexao::obter_conexao() {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let rows: Result<Vec<OsRecenteRow>, _> = conn.exec(
        r#"SELECT os.id, c.nome AS cliente, e.descricao AS equipamento, os.defeito_relatado AS defeito,
                  os.status, os.parecer_tecnico, os.situacao, e.numero_serie,
                  os.observacoes, os.tecnico_responsavel, os.atendente, c.telefone,
                  DATE_FORMAT(os.data_chegada, '%d/%m/%Y %H:%i') AS data_chegada,
                  DATE_FORMAT(os.prazo_entrega, '%d/%m/%Y') AS prazo_entrega
           FROM ordens_servico os
           LEFT JOIN clientes c ON os.cliente_id = c.id AND c.tenant_id = os.tenant_id
           JOIN equipamentos e ON os.equipamento_id = e.id AND e.tenant_id = os.tenant_id
           WHERE os.tenant_id = :tid AND os.cliente_id = :cid
           ORDER BY os.id DESC
           LIMIT :lim"#,
        params! { "tid" => tenant_id, "cid" => cliente_id, "lim" => limite },
    );
    rows.unwrap_or_default()
        .into_iter()
        .map(Into::into)
        .collect()
}

/// Busca os últimos N orçamentos de um cliente.
pub fn crm_service_buscar_orcamentos_recentes(
    tenant_id: i32,
    cliente_id: u32,
    limite: u32,
) -> Vec<serde_json::Value> {
    use mysql::params;
    use mysql::prelude::Queryable;
    let mut conn = match crate::banco_de_dados::conexao::obter_conexao() {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let rows: Result<Vec<(u32, u32, f64, String)>, _> = conn.exec(
        r#"SELECT id, cliente_id, total,
                  DATE_FORMAT(created_at, '%d/%m/%Y %H:%i') AS data
           FROM orcamentos
           WHERE tenant_id = :tid AND cliente_id = :cid
           ORDER BY id DESC
           LIMIT :lim"#,
        params! { "tid" => tenant_id, "cid" => cliente_id, "lim" => limite },
    );
    rows.unwrap_or_default()
        .into_iter()
        .map(|(id, cid, total, data)| {
            serde_json::json!({
                "id": id,
                "cliente_id": cid,
                "total": total,
                "data": data,
            })
        })
        .collect()
}

/// Contexto de quem está fazendo a operação. Vem do JWT/Claims.
#[derive(Debug, Clone, Default)]
pub struct Contexto {
    pub usuario_id: Option<u32>,
    pub username: Option<String>,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<String>,
}

impl Contexto {
    pub fn anonimo() -> Self {
        Self::default()
    }
}

// =============================================================================
// Timeline helpers
// =============================================================================

fn registrar_timeline(
    tenant_id: i32,
    cliente_id: u32,
    tipo: TipoEventoTimeline,
    descricao: &str,
    payload: Option<&serde_json::Value>,
    ctx: &Contexto,
) {
    let res = repository::inserir_evento_timeline(
        tenant_id,
        cliente_id,
        tipo.as_db_str(),
        descricao,
        payload,
        ctx.usuario_id,
        ctx.username.as_deref(),
    );
    if let Err(e) = res {
        tracing::warn!(
            "Falha ao registrar evento de timeline (cliente={}, tipo={:?}): {:?}",
            cliente_id,
            tipo,
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
// Observações
// =============================================================================

pub fn adicionar_observacao(
    tenant_id: i32,
    cliente_id: u32,
    conteudo: &str,
    ctx: &Contexto,
) -> Result<Observacao, ErroAplicacao> {
    if conteudo.trim().is_empty() {
        return Err(ErroAplicacao::Desconhecido("conteúdo vazio".into()));
    }
    let obs = repository::criar_observacao(
        tenant_id,
        cliente_id,
        conteudo,
        ctx.usuario_id,
        ctx.username.as_deref(),
    )?;
    let descricao = format!(
        "{} adicionou observação (id={})",
        ctx.username.as_deref().unwrap_or("anônimo"),
        obs.id
    );
    let payload = serde_json::json!({
        "observacao_id": obs.id,
        "preview": conteudo.chars().take(100).collect::<String>(),
    });
    registrar_timeline(
        tenant_id,
        cliente_id,
        TipoEventoTimeline::ObservacaoAdicionada,
        &descricao,
        Some(&payload),
        ctx,
    );
    registrar_audit(
        Acao::Create,
        "cliente_observacoes",
        Some(&obs.id.to_string()),
        None,
        Some(serde_json::to_value(&obs).unwrap_or_default()),
        ctx,
    );
    Ok(obs)
}

pub fn editar_observacao(
    tenant_id: i32,
    id: u32,
    conteudo_novo: &str,
    ctx: &Contexto,
) -> Result<Observacao, ErroAplicacao> {
    if conteudo_novo.trim().is_empty() {
        return Err(ErroAplicacao::Desconhecido("conteúdo vazio".into()));
    }
    let obs = repository::editar_observacao(
        tenant_id,
        id,
        conteudo_novo,
        ctx.usuario_id,
        ctx.username.as_deref(),
    )?;
    let descricao = format!(
        "{} editou observação (id={}) — {}ª edição",
        ctx.username.as_deref().unwrap_or("anônimo"),
        obs.id,
        obs.vezes_editada
    );
    let payload = serde_json::json!({
        "observacao_id": obs.id,
        "vezes_editada": obs.vezes_editada,
    });
    registrar_timeline(
        tenant_id,
        obs.cliente_id,
        TipoEventoTimeline::ObservacaoEditada,
        &descricao,
        Some(&payload),
        ctx,
    );
    Ok(obs)
}

pub fn listar_observacoes(
    tenant_id: i32,
    cliente_id: u32,
    limite: u32,
) -> Result<Vec<Observacao>, ErroAplicacao> {
    repository::listar_observacoes(tenant_id, cliente_id, limite)
}

// =============================================================================
// Tags
// =============================================================================

pub fn listar_tags(tenant_id: i32) -> Result<Vec<Tag>, ErroAplicacao> {
    repository::listar_tags(tenant_id)
}

pub fn criar_tag(
    tenant_id: i32,
    nome: &str,
    cor: &str,
    descricao: Option<&str>,
) -> Result<Tag, ErroAplicacao> {
    if nome.trim().is_empty() {
        return Err(ErroAplicacao::Desconhecido("nome vazio".into()));
    }
    repository::criar_tag(tenant_id, nome, cor, descricao)
}

pub fn listar_tags_do_cliente(tenant_id: i32, cliente_id: u32) -> Result<Vec<Tag>, ErroAplicacao> {
    repository::listar_tags_do_cliente(tenant_id, cliente_id)
}

pub fn atribuir_tag(
    tenant_id: i32,
    cliente_id: u32,
    tag_id: u32,
    ctx: &Contexto,
) -> Result<(), ErroAplicacao> {
    repository::atribuir_tag(
        tenant_id,
        cliente_id,
        tag_id,
        ctx.usuario_id,
        ctx.username.as_deref(),
    )?;
    let descricao = format!(
        "{} atribuiu tag id={} ao cliente",
        ctx.username.as_deref().unwrap_or("anônimo"),
        tag_id
    );
    registrar_timeline(
        tenant_id,
        cliente_id,
        TipoEventoTimeline::TagAdicionada,
        &descricao,
        Some(&serde_json::json!({ "tag_id": tag_id })),
        ctx,
    );
    Ok(())
}

pub fn remover_tag(
    tenant_id: i32,
    cliente_id: u32,
    tag_id: u32,
    ctx: &Contexto,
) -> Result<bool, ErroAplicacao> {
    let removed = repository::remover_tag(tenant_id, cliente_id, tag_id)?;
    if removed {
        let descricao = format!(
            "{} removeu tag id={}",
            ctx.username.as_deref().unwrap_or("anônimo"),
            tag_id
        );
        registrar_timeline(
            tenant_id,
            cliente_id,
            TipoEventoTimeline::TagRemovida,
            &descricao,
            Some(&serde_json::json!({ "tag_id": tag_id })),
            ctx,
        );
    }
    Ok(removed)
}

// =============================================================================
// Contatos
// =============================================================================

pub fn adicionar_contato(
    tenant_id: i32,
    cliente_id: u32,
    tipo: &str,
    valor: &str,
    rotulo: Option<&str>,
    principal: bool,
    observacao: Option<&str>,
    ctx: &Contexto,
) -> Result<Contato, ErroAplicacao> {
    if valor.trim().is_empty() {
        return Err(ErroAplicacao::Desconhecido("valor vazio".into()));
    }
    let contato = repository::criar_contato(
        tenant_id, cliente_id, tipo, valor, rotulo, principal, observacao,
    )?;
    let descricao = format!(
        "{} adicionou contato {} ({})",
        ctx.username.as_deref().unwrap_or("anônimo"),
        tipo,
        valor
    );
    registrar_timeline(
        tenant_id,
        cliente_id,
        TipoEventoTimeline::ContatoAdicionado,
        &descricao,
        Some(&serde_json::json!({ "contato_id": contato.id, "tipo": tipo })),
        ctx,
    );
    Ok(contato)
}

pub fn listar_contatos(tenant_id: i32, cliente_id: u32) -> Result<Vec<Contato>, ErroAplicacao> {
    repository::listar_contatos(tenant_id, cliente_id)
}

pub fn remover_contato(
    tenant_id: i32,
    id: u32,
    cliente_id: u32,
    ctx: &Contexto,
) -> Result<bool, ErroAplicacao> {
    let removed = repository::remover_contato(tenant_id, id)?;
    if removed {
        let descricao = format!(
            "{} removeu contato id={}",
            ctx.username.as_deref().unwrap_or("anônimo"),
            id
        );
        registrar_timeline(
            tenant_id,
            cliente_id,
            TipoEventoTimeline::ContatoRemovido,
            &descricao,
            Some(&serde_json::json!({ "contato_id": id })),
            ctx,
        );
    }
    Ok(removed)
}

// =============================================================================
// Equipamentos
// =============================================================================

pub fn adicionar_equipamento(
    tenant_id: i32,
    cliente_id: u32,
    descricao: &str,
    marca: Option<&str>,
    modelo: Option<&str>,
    numero_serie: Option<&str>,
    patrimonio: Option<&str>,
    observacao: Option<&str>,
    tipo: &str,
    ctx: &Contexto,
) -> Result<Equipamento, ErroAplicacao> {
    if descricao.trim().is_empty() {
        return Err(ErroAplicacao::Desconhecido("descrição vazia".into()));
    }
    let eq = repository::criar_equipamento(
        tenant_id,
        cliente_id,
        descricao,
        marca,
        modelo,
        numero_serie,
        patrimonio,
        observacao,
        tipo,
    )?;
    let descricao = format!(
        "{} adicionou equipamento {} ({})",
        ctx.username.as_deref().unwrap_or("anônimo"),
        tipo,
        descricao
    );
    registrar_timeline(
        tenant_id,
        cliente_id,
        TipoEventoTimeline::EquipamentoAdicionado,
        &descricao,
        Some(&serde_json::json!({ "equipamento_id": eq.id, "tipo": tipo })),
        ctx,
    );
    Ok(eq)
}

pub fn listar_equipamentos(
    tenant_id: i32,
    cliente_id: u32,
) -> Result<Vec<Equipamento>, ErroAplicacao> {
    repository::listar_equipamentos(tenant_id, cliente_id)
}

pub fn remover_equipamento(
    tenant_id: i32,
    id: u32,
    cliente_id: u32,
    ctx: &Contexto,
) -> Result<bool, ErroAplicacao> {
    let removed = repository::remover_equipamento(tenant_id, id)?;
    if removed {
        let descricao = format!(
            "{} removeu equipamento id={}",
            ctx.username.as_deref().unwrap_or("anônimo"),
            id
        );
        registrar_timeline(
            tenant_id,
            cliente_id,
            TipoEventoTimeline::EquipamentoRemovido,
            &descricao,
            Some(&serde_json::json!({ "equipamento_id": id })),
            ctx,
        );
    }
    Ok(removed)
}

// =============================================================================
// Timeline
// =============================================================================

pub fn listar_timeline(
    tenant_id: i32,
    cliente_id: u32,
    limite: u32,
) -> Result<Vec<TimelineEvento>, ErroAplicacao> {
    repository::listar_timeline(tenant_id, cliente_id, limite)
}

// =============================================================================
// Busca global
// =============================================================================

pub fn buscar_clientes(
    tenant_id: i32,
    query: &str,
    limite: u32,
) -> Result<Vec<serde_json::Value>, ErroAplicacao> {
    if query.trim().is_empty() {
        return Ok(Vec::new());
    }
    repository::buscar_clientes(tenant_id, query, limite)
}

// =============================================================================
// Dashboard agregado
// =============================================================================

/// Retorna o dashboard completo de um cliente: dados, tags, contatos,
/// equipamentos, observações recentes, últimas OS, últimos orçamentos,
/// timeline.
pub fn dashboard_cliente(
    tenant_id: i32,
    cliente_id: u32,
) -> Result<DashboardCliente, ErroAplicacao> {
    // Cliente (reusa função do módulo cliente) — defense in depth (P2.6.2a)
    let cliente = crate::banco_de_dados::obter_cliente_por_id(tenant_id, cliente_id)?;
    let cliente_json = serde_json::to_value(&cliente).unwrap_or(serde_json::Value::Null);

    // Tags, contatos, equipamentos
    let tags = repository::listar_tags_do_cliente(tenant_id, cliente_id)?;
    let contatos = repository::listar_contatos(tenant_id, cliente_id)?;
    let equipamentos = repository::listar_equipamentos(tenant_id, cliente_id)?;
    let observacoes_recentes = repository::listar_observacoes(tenant_id, cliente_id, 10)?;

    // OS recentes (últimas 5) — delega para o módulo de OS
    let ultimas_os = crm_service_buscar_os_recentes(tenant_id, cliente_id, 5);

    // Orçamentos recentes
    let ultimos_orcamentos = crm_service_buscar_orcamentos_recentes(tenant_id, cliente_id, 5);

    // Timeline (últimos 50 eventos)
    let timeline = repository::listar_timeline(tenant_id, cliente_id, 50)?;

    Ok(DashboardCliente {
        cliente: cliente_json,
        tags,
        contatos,
        equipamentos,
        observacoes_recentes,
        ultimas_os,
        ultimos_orcamentos,
        timeline,
    })
}
