// src/os_mobile/service.rs
//
// Camada de regras — Sprint P2.3.
//
// Orquestra:
//   1. Validações
//   2. Persistência (repository)
//   3. Timeline CRM (crm::repository::inserir_evento_timeline)
//   4. Auditoria offline (dedupe por device+evento)

use super::models::{
    Assinatura, AuditoriaCampo, Checklist, ChecklistCompleto, ChecklistItem, ChecklistTemplate,
    ChecklistTemplateCompleto, ChecklistTemplateItem, DashboardTecnico, Evolucao, OsResumo,
    TipoAcaoAuditoria, TipoEvolucao,
};
use super::repository;
use crate::crm::repository as crm_repo;
use crate::servicos::ErroAplicacao;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Contexto {
    pub usuario_id: u32,
    pub username: Option<String>,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<String>,
    /// ID único do device mobile — permite dedupe de eventos offline.
    pub cliente_device_id: Option<String>,
}

// =============================================================================
// Templates
// =============================================================================

pub fn criar_template(
    tenant_id: i32,
    nome: &str,
    descricao: Option<&str>,
    ctx: &Contexto,
) -> Result<u32, ErroAplicacao> {
    if nome.trim().is_empty() {
        return Err(ErroAplicacao::Desconhecido("nome obrigatório".into()));
    }
    let id = repository::criar_template(tenant_id, nome, descricao, ctx.usuario_id)?;
    Ok(id)
}

pub fn adicionar_item_template(
    tenant_id: i32,
    template_id: u32,
    texto: &str,
    obrigatorio: bool,
) -> Result<u32, ErroAplicacao> {
    if texto.trim().is_empty() {
        return Err(ErroAplicacao::Desconhecido("texto obrigatório".into()));
    }
    let id = repository::adicionar_item_template(tenant_id, template_id, 0, texto, obrigatorio)?;
    Ok(id)
}

pub fn listar_templates(
    tenant_id: i32,
    ativo_apenas: bool,
) -> Result<Vec<ChecklistTemplate>, ErroAplicacao> {
    repository::listar_templates(tenant_id, ativo_apenas)
}

pub fn obter_template(
    tenant_id: i32,
    id: u32,
) -> Result<Option<ChecklistTemplateCompleto>, ErroAplicacao> {
    repository::obter_template(tenant_id, id)
}

// =============================================================================
// Checklists aplicados
// =============================================================================

pub fn criar_checklist(
    tenant_id: i32,
    os_id: u32,
    template_id: Option<u32>,
    titulo: &str,
    ctx: &Contexto,
) -> Result<u32, ErroAplicacao> {
    if os_id == 0 {
        return Err(ErroAplicacao::Desconhecido("os_id obrigatório".into()));
    }
    if titulo.trim().is_empty() {
        return Err(ErroAplicacao::Desconhecido("título obrigatório".into()));
    }
    let id = repository::criar_checklist(tenant_id, os_id, template_id, titulo)?;

    // Timeline
    let payload = serde_json::json!({
        "checklist_id": id,
        "template_id": template_id,
        "titulo": titulo,
    });
    let _ = inserir_evento_timeline_para_os(
        tenant_id,
        os_id,
        "CHECKLIST_INICIADO",
        &format!("Checklist '{}' iniciado", titulo),
        Some(&payload),
        Some(ctx.usuario_id),
        ctx.username.as_deref(),
    );
    let _ = repository::registrar_auditoria_campo(
        tenant_id,
        ctx.cliente_device_id.as_deref(),
        ctx.usuario_id,
        TipoAcaoAuditoria::ChecklistItemConcluido, // placeholder; será detalhado em P2.3.x
        Some(os_id),
        Some(id),
        None,
        None,
        None,
        None,
        Utc::now(),
        Some(&payload),
    );
    Ok(id)
}

pub fn adicionar_item_checklist(
    tenant_id: i32,
    checklist_id: u32,
    texto: &str,
    obrigatorio: bool,
) -> Result<u32, ErroAplicacao> {
    if texto.trim().is_empty() {
        return Err(ErroAplicacao::Desconhecido("texto obrigatório".into()));
    }
    let id = repository::adicionar_item_checklist(tenant_id, checklist_id, texto, obrigatorio)?;
    Ok(id)
}

pub fn marcar_item(
    tenant_id: i32,
    item_id: u32,
    concluido: bool,
    observacao: Option<&str>,
    ctx: &Contexto,
    data_evento_device: Option<DateTime<Utc>>,
) -> Result<bool, ErroAplicacao> {
    let updated = repository::marcar_item(tenant_id, item_id, concluido, observacao)?;
    if updated {
        // Timeline
        let payload = serde_json::json!({
            "item_id": item_id,
            "concluido": concluido,
            "observacao": observacao,
        });
        // Descobre a OS para registrar timeline
        if let Some(checklist) = repository::obter_checklist_direto(tenant_id, item_id)? {
            let _ = inserir_evento_timeline_para_os(
                tenant_id,
                checklist.os_id,
                if concluido {
                    "CHECKLIST_ITEM_OK"
                } else {
                    "CHECKLIST_ITEM_REABERTO"
                },
                &format!(
                    "Item #{} {}",
                    item_id,
                    if concluido { "concluído" } else { "reaberto" }
                ),
                Some(&payload),
                Some(ctx.usuario_id),
                ctx.username.as_deref(),
            );

            // Se o checklist inteiro ficou completo, registra evento
            if let Some(c) = repository::obter_checklist(tenant_id, checklist.id)? {
                if c.checklist.is_completo() {
                    let _ = inserir_evento_timeline_para_os(
                        tenant_id,
                        checklist.os_id,
                        "CHECKLIST_CONCLUIDO",
                        &format!("Checklist '{}' 100% completo", c.checklist.titulo),
                        Some(&payload),
                        Some(ctx.usuario_id),
                        ctx.username.as_deref(),
                    );
                }
            }
        }
        // Auditoria offline
        let _ = repository::registrar_auditoria_campo(
            tenant_id,
            ctx.cliente_device_id.as_deref(),
            ctx.usuario_id,
            TipoAcaoAuditoria::ChecklistItemConcluido,
            None,
            None,
            Some(item_id),
            None,
            None,
            None,
            data_evento_device.unwrap_or_else(Utc::now),
            Some(&payload),
        );
    }
    Ok(updated)
}

pub fn obter_checklist(
    tenant_id: i32,
    id: u32,
) -> Result<Option<ChecklistCompleto>, ErroAplicacao> {
    repository::obter_checklist(tenant_id, id)
}

pub fn listar_checklists_os(
    tenant_id: i32,
    os_id: u32,
) -> Result<Vec<ChecklistCompleto>, ErroAplicacao> {
    repository::listar_checklists_os(tenant_id, os_id)
}

pub fn remover_checklist(tenant_id: i32, id: u32) -> Result<bool, ErroAplicacao> {
    repository::remover_checklist(tenant_id, id)
}

// =============================================================================
// Evoluções
// =============================================================================

pub fn registrar_evolucao(
    tenant_id: i32,
    os_id: u32,
    texto: &str,
    tipo: TipoEvolucao,
    latitude: Option<f64>,
    longitude: Option<f64>,
    ctx: &Contexto,
    data_evento_device: Option<DateTime<Utc>>,
) -> Result<u32, ErroAplicacao> {
    if texto.trim().is_empty() {
        return Err(ErroAplicacao::Desconhecido("texto obrigatório".into()));
    }
    let id = repository::criar_evolucao(
        tenant_id,
        os_id,
        ctx.usuario_id,
        texto,
        tipo,
        latitude,
        longitude,
        data_evento_device,
    )?;
    // Timeline CRM
    let payload = serde_json::json!({
        "evolucao_id": id,
        "tipo": tipo.as_db_str(),
        "latitude": latitude,
        "longitude": longitude,
    });
    let _ = inserir_evento_timeline_para_os(
        tenant_id,
        os_id,
        "OS_EVOLUCAO",
        &format!("[{}] {}", tipo, texto),
        Some(&payload),
        Some(ctx.usuario_id),
        ctx.username.as_deref(),
    );
    // Auditoria offline
    let _ = repository::registrar_auditoria_campo(
        tenant_id,
        ctx.cliente_device_id.as_deref(),
        ctx.usuario_id,
        TipoAcaoAuditoria::EvolucaoRegistrada,
        Some(os_id),
        None,
        None,
        Some(id),
        None,
        None,
        data_evento_device.unwrap_or_else(Utc::now),
        Some(&payload),
    );
    Ok(id)
}

pub fn listar_evolucoes(tenant_id: i32, os_id: u32) -> Result<Vec<Evolucao>, ErroAplicacao> {
    repository::listar_evolucoes_os(tenant_id, os_id)
}

// =============================================================================
// Assinaturas
// =============================================================================

/// Registra assinatura do cliente. `arquivo_id` aponta para a imagem
/// (PNG do canvas) que foi feito upload via módulo Arquivos.
pub fn registrar_assinatura(
    tenant_id: i32,
    os_id: u32,
    arquivo_id: u32,
    nome_assinante: &str,
    documento_assinante: Option<&str>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    observacao: Option<&str>,
    ctx: &Contexto,
) -> Result<u32, ErroAplicacao> {
    if nome_assinante.trim().is_empty() {
        return Err(ErroAplicacao::Desconhecido(
            "nome do assinante obrigatório".into(),
        ));
    }
    if arquivo_id == 0 {
        return Err(ErroAplicacao::Desconhecido("arquivo_id obrigatório".into()));
    }
    let id = repository::criar_assinatura(
        tenant_id,
        os_id,
        arquivo_id,
        nome_assinante,
        documento_assinante,
        ctx.ip.as_deref(),
        latitude,
        longitude,
        observacao,
    )?;
    let payload = serde_json::json!({
        "assinatura_id": id,
        "arquivo_id": arquivo_id,
        "nome_assinante": nome_assinante,
    });
    let _ = inserir_evento_timeline_para_os(
        tenant_id,
        os_id,
        "ASSINATURA_REGISTRADA",
        &format!("Assinatura registrada por {}", nome_assinante),
        Some(&payload),
        Some(ctx.usuario_id),
        ctx.username.as_deref(),
    );
    let _ = repository::registrar_auditoria_campo(
        tenant_id,
        ctx.cliente_device_id.as_deref(),
        ctx.usuario_id,
        TipoAcaoAuditoria::AssinaturaRegistrada,
        Some(os_id),
        None,
        None,
        None,
        None,
        Some(id),
        Utc::now(),
        Some(&payload),
    );
    Ok(id)
}

pub fn obter_assinatura(tenant_id: i32, os_id: u32) -> Result<Option<Assinatura>, ErroAplicacao> {
    repository::obter_assinatura(tenant_id, os_id)
}

// =============================================================================
// Dashboard do técnico
// =============================================================================

pub fn dashboard_tecnico(
    tenant_id: i32,
    usuario_id: Option<u32>,
) -> Result<DashboardTecnico, ErroAplicacao> {
    let mut dash = repository::dashboard_tecnico(tenant_id, usuario_id)?;
    // Próximas OS (top 10)
    dash.proximas_os = repository::listar_os_resumo(tenant_id, usuario_id, 10)?;
    // Checklists recentes com algum item pendente
    dash.checklists_recentes = repository::listar_checklists_pendentes_recentes(tenant_id, 5)?;
    Ok(dash)
}

pub fn listar_os_resumo(
    tenant_id: i32,
    usuario_id: Option<u32>,
    limite: u32,
) -> Result<Vec<OsResumo>, ErroAplicacao> {
    repository::listar_os_resumo(tenant_id, usuario_id, limite)
}

// =============================================================================
// Auditoria
// =============================================================================

pub fn listar_auditoria_os(
    tenant_id: i32,
    os_id: u32,
) -> Result<Vec<AuditoriaCampo>, ErroAplicacao> {
    repository::listar_auditoria_os(tenant_id, os_id)
}

// =============================================================================
// Helpers
// =============================================================================

/// Resolve o cliente_id a partir do os_id.
fn resolver_cliente_id_por_os(tenant_id: i32, os_id: u32) -> Option<u32> {
    use crate::banco_de_dados::obter_conexao;
    use mysql::prelude::Queryable;
    let mut conn = obter_conexao().ok()?;
    let res: Option<u32> = conn
        .exec_first(
            "SELECT cliente_id FROM ordens_servico WHERE tenant_id = ? AND id = ?",
            (tenant_id, os_id),
        )
        .ok()
        .flatten();
    res
}

/// Insere evento na timeline CRM do cliente da OS. Silencioso em caso de
/// erro (timeline é "best-effort" — não bloqueia operação principal).
pub fn inserir_evento_timeline_para_os(
    tenant_id: i32,
    os_id: u32,
    tipo: &str,
    descricao: &str,
    payload: Option<&serde_json::Value>,
    usuario_id: Option<u32>,
    username: Option<&str>,
) -> Result<(), ErroAplicacao> {
    let cliente_id = match resolver_cliente_id_por_os(tenant_id, os_id) {
        Some(c) => c,
        None => return Ok(()), // OS sem cliente — não trava
    };
    crm_repo::inserir_evento_timeline(
        tenant_id, cliente_id, tipo, descricao, payload, usuario_id, username,
    )?;
    Ok(())
}

// =============================================================================
// Testes unit
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contexto_default() {
        let c = Contexto::default();
        assert_eq!(c.usuario_id, 0);
        assert!(c.cliente_device_id.is_none());
    }
}
