// src/financial/handlers.rs
//! Handlers HTTP do módulo Financeiro.
//!
//! Compilado apenas pelo binário `senior-system-server` (declarado em
//! `server.rs` via `#[path]`).
//!
//! ## Contrato comum
//!
//! - `Claims` obrigatório + `check_perm` por rota.
//! - `tenant_id` sempre de `auth::tenant_do_usuario(&claims)`.
//! - **Valores monetários trafegam em centavos (`i64`)**, igual ao domínio.
//!   Nenhum handler faz aritmética de ponto flutuante com dinheiro.
//! - Listagens paginam via `Paginacao::sanitizar()` e devolvem
//!   `Pagina<T> { items, page, limit, total, total_paginas }`.

use axum::{
    extract::{Json, Path, Query},
    http::StatusCode,
    response::Json as AxJson,
};
use chrono::{NaiveDate, Utc};
use serde::Deserialize;

use crate::auth::{self, Claims};
use crate::banco_de_dados::pagination::{Pagina, Paginacao};
use crate::check_perm;
use crate::financial::models::{
    AlertaFinanceiro, CentroCusto, ContaPagar, ContaReceber, DashboardFinanceiro, OrigemPagar,
    OrigemReceber, PlanoConta, StatusConta,
};
use crate::financial::service::{self, Contexto, ErroFinanceiro, FluxoCaixaResultado};
use crate::{erro_padrao, ErroApi};

type Resposta<T> = Result<AxJson<T>, (StatusCode, AxJson<ErroApi>)>;
type RespostaStatus = Result<StatusCode, (StatusCode, AxJson<ErroApi>)>;

/// Teto de registros lidos do banco antes de paginar em memória.
///
/// O `service::listar_contas_*` recebe um `limite` simples (sem OFFSET), então
/// buscamos até este teto e paginamos aqui. Vale para as telas de gestão
/// (milhares de títulos); acima disso o repositório precisa de OFFSET nativo.
const TETO_LEITURA: u32 = 5_000;

fn contexto(claims: &Claims) -> Contexto {
    Contexto {
        usuario_id: if claims.uid > 0 { claims.uid as u32 } else { 0 },
        username: Some(claims.sub.clone()),
        ip: None,
        user_agent: None,
        request_id: None,
    }
}

fn map_erro_financeiro(e: ErroFinanceiro) -> (StatusCode, AxJson<ErroApi>) {
    let (code, msg): (&'static str, String) = match &e {
        ErroFinanceiro::ConfigNaoInicializada => (
            "NOT_FOUND",
            "Configuração financeira não inicializada para este tenant".to_string(),
        ),
        ErroFinanceiro::ContaNaoEncontrada => ("NOT_FOUND", "Conta não encontrada".to_string()),
        ErroFinanceiro::ContaJaFinalizada => (
            "CONFLICT",
            "Conta já está paga ou cancelada".to_string(),
        ),
        ErroFinanceiro::ValorPagoExcede => (
            "VALIDATION",
            "Valor pago excede o saldo da conta".to_string(),
        ),
        ErroFinanceiro::ValorInvalido(m) => ("VALIDATION", format!("Valor inválido: {}", m)),
        ErroFinanceiro::CentroCustoInvalido(id) => (
            "VALIDATION",
            format!("Centro de custo {} não encontrado", id),
        ),
        ErroFinanceiro::PlanoContaInvalido(id) => (
            "VALIDATION",
            format!("Plano de contas {} não encontrado", id),
        ),
        ErroFinanceiro::DescricaoObrigatoria => {
            ("VALIDATION", "Descrição é obrigatória".to_string())
        }
        ErroFinanceiro::Desconhecido(m) => {
            tracing::error!(target: "api", "ErroFinanceiro::Desconhecido: {}", m);
            ("INTERNAL", "Erro interno no módulo financeiro".to_string())
        }
    };
    tracing::warn!(target: "api", code = code, "financial: {:?}", e);
    erro_padrao(code, msg, None)
}

async fn executar<T, F>(f: F, contexto_erro: &'static str) -> Resposta<T>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, ErroFinanceiro> + Send + 'static,
{
    match tokio::task::spawn_blocking(f).await {
        Ok(Ok(v)) => Ok(AxJson(v)),
        Ok(Err(e)) => Err(map_erro_financeiro(e)),
        Err(_) => Err(erro_padrao("INTERNAL", contexto_erro, None)),
    }
}

/// Aceita os nomes de status do front (`aberta`, `paga`, `cancelada`) e os do
/// domínio (`PENDENTE`, `PARCIAL`, `PAGO`, `ATRASADO`, `CANCELADO`).
fn parse_status(bruto: &str) -> Option<StatusConta> {
    match bruto.trim().to_uppercase().as_str() {
        "" => None,
        "ABERTA" | "ABERTO" | "PENDENTE" => Some(StatusConta::Pendente),
        "PARCIAL" => Some(StatusConta::Parcial),
        "PAGA" | "PAGO" => Some(StatusConta::Pago),
        "ATRASADA" | "ATRASADO" => Some(StatusConta::Atrasado),
        "CANCELADA" | "CANCELADO" => Some(StatusConta::Cancelado),
        _ => None,
    }
}

fn parse_data_obrigatoria(
    bruto: &str,
    campo: &'static str,
) -> Result<NaiveDate, (StatusCode, AxJson<ErroApi>)> {
    NaiveDate::parse_from_str(bruto.trim(), "%Y-%m-%d").map_err(|_| {
        erro_padrao(
            "VALIDATION",
            format!("campo '{}' deve estar em YYYY-MM-DD", campo),
            None,
        )
    })
}

/// Recorta uma página de uma lista já carregada.
fn paginar<T>(items: Vec<T>, pag: Paginacao) -> Pagina<T> {
    let total = items.len() as u64;
    let inicio = pag.offset() as usize;
    let pagina: Vec<T> = items
        .into_iter()
        .skip(inicio)
        .take(pag.limit as usize)
        .collect();
    Pagina::from_items(pagina, pag.page, pag.limit, total)
}

// =============================================================================
// DTOs
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct ListagemQuery {
    pub status: Option<String>,
    pub cliente_id: Option<u32>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct FluxoQuery {
    pub de: Option<String>,
    pub ate: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AlertasQuery {
    pub severidade: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CriarContaReceberPayload {
    pub cliente_id: u32,
    pub descricao: String,
    /// Centavos.
    pub valor: i64,
    /// `YYYY-MM-DD`.
    pub vencimento: String,
    pub centro_custo_id: u32,
    pub plano_conta_id: u32,
    pub origem_tipo: Option<String>,
    pub origem_id: Option<u32>,
    pub observacao: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CriarContaPagarPayload {
    pub fornecedor: String,
    pub fornecedor_doc: Option<String>,
    pub descricao: String,
    pub valor: i64,
    pub vencimento: String,
    pub centro_custo_id: u32,
    pub plano_conta_id: u32,
    pub origem_tipo: Option<String>,
    pub origem_id: Option<u32>,
    pub observacao: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PagarPayload {
    /// Centavos. Ausente = quita o saldo restante.
    pub valor_pago: Option<i64>,
    /// `YYYY-MM-DD`. Ausente = hoje.
    pub data_pagamento: Option<String>,
    pub forma_pagamento: Option<String>,
}

// =============================================================================
// Dashboard / fluxo de caixa
// =============================================================================

/// `GET /financeiro/dashboard` — perm `financeiro.view`.
pub async fn dashboard(claims: Claims) -> Resposta<DashboardFinanceiro> {
    check_perm(&claims, "financeiro.view")?;
    let tenant = auth::tenant_do_usuario(&claims);
    executar(
        move || service::dashboard(tenant),
        "Erro ao montar dashboard financeiro",
    )
    .await
}

/// `GET /financeiro/fluxo-caixa?de=YYYY-MM-DD&ate=YYYY-MM-DD` —
/// perm `financeiro.view`. Sem parâmetros, usa o mês corrente.
pub async fn fluxo_caixa(
    claims: Claims,
    Query(q): Query<FluxoQuery>,
) -> Resposta<FluxoCaixaResultado> {
    check_perm(&claims, "financeiro.view")?;
    let tenant = auth::tenant_do_usuario(&claims);
    let hoje = Utc::now().date_naive();
    let inicio = match q.de.as_deref() {
        Some(s) => parse_data_obrigatoria(s, "de")?,
        None => {
            use chrono::Datelike;
            NaiveDate::from_ymd_opt(hoje.year(), hoje.month(), 1).unwrap_or(hoje)
        }
    };
    let fim = match q.ate.as_deref() {
        Some(s) => parse_data_obrigatoria(s, "ate")?,
        None => hoje,
    };
    if fim < inicio {
        return Err(erro_padrao("VALIDATION", "'ate' deve ser >= 'de'", None));
    }
    executar(
        move || service::calcular_fluxo(tenant, inicio, fim),
        "Erro ao calcular fluxo de caixa",
    )
    .await
}

// =============================================================================
// Contas a receber
// =============================================================================

/// `GET /financeiro/contas-receber?status=&cliente_id=&page=&limit=` —
/// perm `financeiro.view`.
pub async fn listar_contas_receber(
    claims: Claims,
    Query(q): Query<ListagemQuery>,
) -> Resposta<Pagina<ContaReceber>> {
    check_perm(&claims, "financeiro.view")?;
    let tenant = auth::tenant_do_usuario(&claims);
    let pag = Paginacao {
        page: q.page.unwrap_or(1),
        limit: q.limit.unwrap_or(50),
    }
    .sanitizar();
    let status = q.status.as_deref().and_then(parse_status);
    let cliente_id = q.cliente_id;
    executar(
        move || {
            let contas = service::listar_contas_receber(tenant, cliente_id, status, TETO_LEITURA)?;
            Ok(paginar(contas, pag))
        },
        "Erro ao listar contas a receber",
    )
    .await
}

/// `POST /financeiro/contas-receber` — perm `financeiro.create`.
pub async fn criar_conta_receber(
    claims: Claims,
    Json(p): Json<CriarContaReceberPayload>,
) -> Result<(StatusCode, AxJson<serde_json::Value>), (StatusCode, AxJson<ErroApi>)> {
    check_perm(&claims, "financeiro.create")?;
    let tenant = auth::tenant_do_usuario(&claims);
    let ctx = contexto(&claims);
    let vencimento = parse_data_obrigatoria(&p.vencimento, "vencimento")?;
    let origem = OrigemReceber::from_db_str(
        &p.origem_tipo
            .clone()
            .unwrap_or_else(|| "MANUAL".to_string())
            .to_uppercase(),
    );
    let resultado = tokio::task::spawn_blocking(move || {
        service::criar_conta_receber(
            tenant,
            p.cliente_id,
            origem,
            p.origem_id,
            &p.descricao,
            p.valor,
            vencimento,
            p.centro_custo_id,
            p.plano_conta_id,
            p.observacao.as_deref(),
            &ctx,
        )
    })
    .await;
    match resultado {
        Ok(Ok(id)) => Ok((
            StatusCode::CREATED,
            AxJson(serde_json::json!({ "id": id })),
        )),
        Ok(Err(e)) => Err(map_erro_financeiro(e)),
        Err(_) => Err(erro_padrao(
            "INTERNAL",
            "Erro ao criar conta a receber",
            None,
        )),
    }
}

/// `POST /financeiro/contas-receber/{id}/pagar` —
/// perm `financeiro.conta_receber.pagar`.
pub async fn pagar_conta_receber(
    claims: Claims,
    Path(id): Path<u32>,
    Json(p): Json<PagarPayload>,
) -> Resposta<ContaReceber> {
    check_perm(&claims, "financeiro.conta_receber.pagar")?;
    let tenant = auth::tenant_do_usuario(&claims);
    let ctx = contexto(&claims);
    let data = match p.data_pagamento.as_deref() {
        Some(s) => parse_data_obrigatoria(s, "data_pagamento")?,
        None => Utc::now().date_naive(),
    };
    executar(
        move || {
            // Sem `valor_pago` explícito: quita o saldo restante.
            let valor = match p.valor_pago {
                Some(v) => v,
                None => service::obter_conta_receber(tenant, id)?
                    .ok_or(ErroFinanceiro::ContaNaoEncontrada)?
                    .saldo_restante(),
            };
            service::pagar_conta_receber(tenant, id, valor, data, p.forma_pagamento.as_deref(), &ctx)
        },
        "Erro ao registrar pagamento",
    )
    .await
}

/// `POST /financeiro/contas-receber/{id}/cancelar` — perm `financeiro.edit`.
pub async fn cancelar_conta_receber(claims: Claims, Path(id): Path<u32>) -> RespostaStatus {
    check_perm(&claims, "financeiro.edit")?;
    let tenant = auth::tenant_do_usuario(&claims);
    let ctx = contexto(&claims);
    match tokio::task::spawn_blocking(move || service::cancelar_conta_receber(tenant, id, &ctx))
        .await
    {
        Ok(Ok(true)) => Ok(StatusCode::NO_CONTENT),
        Ok(Ok(false)) => Err(erro_padrao("NOT_FOUND", "Conta não encontrada", None)),
        Ok(Err(e)) => Err(map_erro_financeiro(e)),
        Err(_) => Err(erro_padrao("INTERNAL", "Erro ao cancelar conta", None)),
    }
}

// =============================================================================
// Contas a pagar
// =============================================================================

/// `GET /financeiro/contas-pagar?status=&page=&limit=` — perm `financeiro.view`.
pub async fn listar_contas_pagar(
    claims: Claims,
    Query(q): Query<ListagemQuery>,
) -> Resposta<Pagina<ContaPagar>> {
    check_perm(&claims, "financeiro.view")?;
    let tenant = auth::tenant_do_usuario(&claims);
    let pag = Paginacao {
        page: q.page.unwrap_or(1),
        limit: q.limit.unwrap_or(50),
    }
    .sanitizar();
    let status = q.status.as_deref().and_then(parse_status);
    executar(
        move || {
            let contas = service::listar_contas_pagar(tenant, status, TETO_LEITURA)?;
            Ok(paginar(contas, pag))
        },
        "Erro ao listar contas a pagar",
    )
    .await
}

/// `POST /financeiro/contas-pagar` — perm `financeiro.create`.
pub async fn criar_conta_pagar(
    claims: Claims,
    Json(p): Json<CriarContaPagarPayload>,
) -> Result<(StatusCode, AxJson<serde_json::Value>), (StatusCode, AxJson<ErroApi>)> {
    check_perm(&claims, "financeiro.create")?;
    let tenant = auth::tenant_do_usuario(&claims);
    let ctx = contexto(&claims);
    let vencimento = parse_data_obrigatoria(&p.vencimento, "vencimento")?;
    let origem = OrigemPagar::from_db_str(
        &p.origem_tipo
            .clone()
            .unwrap_or_else(|| "MANUAL".to_string())
            .to_uppercase(),
    );
    let resultado = tokio::task::spawn_blocking(move || {
        service::criar_conta_pagar(
            tenant,
            &p.fornecedor,
            p.fornecedor_doc.as_deref(),
            &p.descricao,
            p.valor,
            vencimento,
            p.centro_custo_id,
            p.plano_conta_id,
            origem,
            p.origem_id,
            p.observacao.as_deref(),
            &ctx,
        )
    })
    .await;
    match resultado {
        Ok(Ok(id)) => Ok((
            StatusCode::CREATED,
            AxJson(serde_json::json!({ "id": id })),
        )),
        Ok(Err(e)) => Err(map_erro_financeiro(e)),
        Err(_) => Err(erro_padrao("INTERNAL", "Erro ao criar conta a pagar", None)),
    }
}

/// `POST /financeiro/contas-pagar/{id}/pagar` — perm `financeiro.conta_pagar.pagar`.
pub async fn pagar_conta_pagar(
    claims: Claims,
    Path(id): Path<u32>,
    Json(p): Json<PagarPayload>,
) -> Resposta<ContaPagar> {
    check_perm(&claims, "financeiro.conta_pagar.pagar")?;
    let tenant = auth::tenant_do_usuario(&claims);
    let ctx = contexto(&claims);
    let data = match p.data_pagamento.as_deref() {
        Some(s) => parse_data_obrigatoria(s, "data_pagamento")?,
        None => Utc::now().date_naive(),
    };
    executar(
        move || {
            let valor = match p.valor_pago {
                Some(v) => v,
                None => {
                    let conta = service::obter_conta_pagar(tenant, id)?
                        .ok_or(ErroFinanceiro::ContaNaoEncontrada)?;
                    conta.valor - conta.valor_pago
                }
            };
            service::pagar_conta_pagar(tenant, id, valor, data, p.forma_pagamento.as_deref(), &ctx)
        },
        "Erro ao registrar pagamento",
    )
    .await
}

// =============================================================================
// Plano de contas / centros de custo
// =============================================================================

/// `GET /financeiro/plano-contas` — perm `financeiro.view`.
pub async fn listar_plano_contas(claims: Claims) -> Resposta<Vec<PlanoConta>> {
    check_perm(&claims, "financeiro.view")?;
    let tenant = auth::tenant_do_usuario(&claims);
    executar(
        move || service::listar_plano_contas(tenant, true),
        "Erro ao listar plano de contas",
    )
    .await
}

/// `GET /financeiro/centros-custo` — perm `financeiro.view`.
pub async fn listar_centros_custo(claims: Claims) -> Resposta<Vec<CentroCusto>> {
    check_perm(&claims, "financeiro.view")?;
    let tenant = auth::tenant_do_usuario(&claims);
    executar(
        move || service::listar_centros_custo(tenant, true),
        "Erro ao listar centros de custo",
    )
    .await
}

// =============================================================================
// Alertas financeiros
// =============================================================================

/// `GET /financeiro/alertas?severidade=critical` — perm `financeiro.view`.
pub async fn listar_alertas(
    claims: Claims,
    Query(q): Query<AlertasQuery>,
) -> Resposta<Vec<AlertaFinanceiro>> {
    check_perm(&claims, "financeiro.view")?;
    let tenant = auth::tenant_do_usuario(&claims);
    let filtro = q.severidade.map(|s| s.trim().to_uppercase());
    executar(
        move || {
            let mut alertas = service::listar_alertas_ativos(tenant)?;
            if let Some(sev) = &filtro {
                alertas.retain(|a| a.severidade.as_db_str() == sev.as_str());
            }
            Ok(alertas)
        },
        "Erro ao listar alertas financeiros",
    )
    .await
}

/// `POST /financeiro/alertas/{id}/resolver` — perm `financeiro.edit`.
pub async fn resolver_alerta(claims: Claims, Path(id): Path<u32>) -> RespostaStatus {
    check_perm(&claims, "financeiro.edit")?;
    let tenant = auth::tenant_do_usuario(&claims);
    match tokio::task::spawn_blocking(move || service::marcar_alerta_resolvido(tenant, id)).await {
        Ok(Ok(true)) => Ok(StatusCode::NO_CONTENT),
        Ok(Ok(false)) => Err(erro_padrao("NOT_FOUND", "Alerta não encontrado", None)),
        Ok(Err(e)) => Err(map_erro_financeiro(e)),
        Err(_) => Err(erro_padrao("INTERNAL", "Erro ao resolver alerta", None)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_do_front_mapeia_para_dominio() {
        assert_eq!(parse_status("aberta"), Some(StatusConta::Pendente));
        assert_eq!(parse_status("paga"), Some(StatusConta::Pago));
        assert_eq!(parse_status("cancelada"), Some(StatusConta::Cancelado));
        assert_eq!(parse_status("PENDENTE"), Some(StatusConta::Pendente));
    }

    #[test]
    fn status_invalido_vira_none() {
        assert_eq!(parse_status("qualquer"), None);
        assert_eq!(parse_status(""), None);
    }

    #[test]
    fn paginacao_recorta_a_pagina_certa() {
        let items: Vec<u32> = (1..=25).collect();
        let pag = Paginacao { page: 2, limit: 10 }.sanitizar();
        let p = paginar(items, pag);
        assert_eq!(p.total, 25);
        assert_eq!(p.total_paginas, 3);
        assert_eq!(p.items, vec![11, 12, 13, 14, 15, 16, 17, 18, 19, 20]);
    }

    #[test]
    fn paginacao_pagina_alem_do_fim_vem_vazia() {
        let items: Vec<u32> = (1..=5).collect();
        let pag = Paginacao { page: 9, limit: 10 }.sanitizar();
        let p = paginar(items, pag);
        assert!(p.items.is_empty());
        assert_eq!(p.total, 5);
    }

    #[test]
    fn erro_conta_finalizada_e_409() {
        let (status, _) = map_erro_financeiro(ErroFinanceiro::ContaJaFinalizada);
        assert_eq!(status, StatusCode::CONFLICT);
    }

    #[test]
    fn erro_valor_excede_e_400() {
        let (status, _) = map_erro_financeiro(ErroFinanceiro::ValorPagoExcede);
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }
}
