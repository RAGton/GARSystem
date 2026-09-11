// src/financial/models.rs
//
// Sprint P2.5 — Financeiro (fundação).
//
// Entidades:
//   - ConfiguracaoFinanceira
//   - PlanoContas
//   - CentroCusto
//   - ContaReceber / StatusConta
//   - ContaPagar
//   - Lancamento / TipoLancamento
//   - AlertaFinanceiro / AlertaFinanceiroOcorrencia
//   - DashboardFinanceiro
//
// Valores monetários: sempre BIGINT em centavos.
// JSON expõe como f64 (cliente é responsável por precisão).

use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StatusConta {
    Pendente,
    Parcial,
    Pago,
    Atrasado, // derivado (calculado)
    Cancelado,
}

impl StatusConta {
    pub fn as_db_str(self) -> &'static str {
        match self {
            StatusConta::Pendente => "PENDENTE",
            StatusConta::Parcial => "PARCIAL",
            StatusConta::Pago => "PAGO",
            StatusConta::Atrasado => "ATRASADO",
            StatusConta::Cancelado => "CANCELADO",
        }
    }
    pub fn from_db_str(s: &str) -> Self {
        match s {
            "PARCIAL" => StatusConta::Parcial,
            "PAGO" => StatusConta::Pago,
            "ATRASADO" => StatusConta::Atrasado,
            "CANCELADO" => StatusConta::Cancelado,
            _ => StatusConta::Pendente,
        }
    }
    pub fn is_final(self) -> bool {
        matches!(self, StatusConta::Pago | StatusConta::Cancelado)
    }
}

impl fmt::Display for StatusConta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_db_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrigemReceber {
    Orcamento,
    Os,
    Manual,
    Outro,
}

impl OrigemReceber {
    pub fn as_db_str(self) -> &'static str {
        match self {
            OrigemReceber::Orcamento => "ORCAMENTO",
            OrigemReceber::Os => "OS",
            OrigemReceber::Manual => "MANUAL",
            OrigemReceber::Outro => "OUTRO",
        }
    }
    pub fn from_db_str(s: &str) -> Self {
        match s {
            "ORCAMENTO" => OrigemReceber::Orcamento,
            "OS" => OrigemReceber::Os,
            "MANUAL" => OrigemReceber::Manual,
            _ => OrigemReceber::Outro,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrigemPagar {
    Cotacao,
    Manual,
    Outro,
}

impl OrigemPagar {
    pub fn as_db_str(self) -> &'static str {
        match self {
            OrigemPagar::Cotacao => "COTACAO",
            OrigemPagar::Manual => "MANUAL",
            OrigemPagar::Outro => "OUTRO",
        }
    }
    pub fn from_db_str(s: &str) -> Self {
        match s {
            "COTACAO" => OrigemPagar::Cotacao,
            "MANUAL" => OrigemPagar::Manual,
            _ => OrigemPagar::Outro,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TipoPlanoConta {
    Receita,
    Despesa,
    Interno,
    Imobilizado,
}

impl TipoPlanoConta {
    pub fn as_db_str(self) -> &'static str {
        match self {
            TipoPlanoConta::Receita => "RECEITA",
            TipoPlanoConta::Despesa => "DESPESA",
            TipoPlanoConta::Interno => "INTERNO",
            TipoPlanoConta::Imobilizado => "IMOBILIZADO",
        }
    }
    pub fn from_db_str(s: &str) -> Self {
        match s {
            "RECEITA" => TipoPlanoConta::Receita,
            "DESPESA" => TipoPlanoConta::Despesa,
            "IMOBILIZADO" => TipoPlanoConta::Imobilizado,
            _ => TipoPlanoConta::Interno,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TipoLancamento {
    Entrada,
    Saida,
    Transferencia,
    Estorno,
}

impl TipoLancamento {
    pub fn as_db_str(self) -> &'static str {
        match self {
            TipoLancamento::Entrada => "ENTRADA",
            TipoLancamento::Saida => "SAIDA",
            TipoLancamento::Transferencia => "TRANSFERENCIA",
            TipoLancamento::Estorno => "ESTORNO",
        }
    }
    pub fn from_db_str(s: &str) -> Self {
        match s {
            "SAIDA" => TipoLancamento::Saida,
            "TRANSFERENCIA" => TipoLancamento::Transferencia,
            "ESTORNO" => TipoLancamento::Estorno,
            _ => TipoLancamento::Entrada,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TipoEntidadeAlerta {
    ContaReceber,
    ContaPagar,
    Fluxo,
}

impl TipoEntidadeAlerta {
    pub fn as_db_str(self) -> &'static str {
        match self {
            TipoEntidadeAlerta::ContaReceber => "CONTA_RECEBER",
            TipoEntidadeAlerta::ContaPagar => "CONTA_PAGAR",
            TipoEntidadeAlerta::Fluxo => "FLUXO",
        }
    }
    pub fn from_db_str(s: &str) -> Self {
        match s {
            "CONTA_PAGAR" => TipoEntidadeAlerta::ContaPagar,
            "FLUXO" => TipoEntidadeAlerta::Fluxo,
            _ => TipoEntidadeAlerta::ContaReceber,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Severidade {
    Info,
    Warning,
    Critical,
}

impl Severidade {
    pub fn as_db_str(self) -> &'static str {
        match self {
            Severidade::Info => "INFO",
            Severidade::Warning => "WARNING",
            Severidade::Critical => "CRITICAL",
        }
    }
    pub fn from_db_str(s: &str) -> Self {
        match s {
            "INFO" => Severidade::Info,
            "CRITICAL" => Severidade::Critical,
            _ => Severidade::Warning,
        }
    }
}

impl fmt::Display for Severidade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_db_str())
    }
}

// =============================================================================
// Configuração
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfiguracaoFinanceira {
    pub id: u32,
    pub dias_vencimento_padrao: u32,
    pub centro_custo_padrao_receber_id: Option<u32>,
    pub plano_conta_padrao_receber_id: Option<u32>,
    pub centro_custo_padrao_pagar_id: Option<u32>,
    pub plano_conta_padrao_pagar_id: Option<u32>,
    pub moeda_padrao: String,
    pub data_atualizacao: DateTime<Utc>,
}

// =============================================================================
// Plano de Contas
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanoConta {
    pub id: u32,
    pub codigo: String,
    pub nome: String,
    pub tipo: TipoPlanoConta,
    pub pai_id: Option<u32>,
    pub ativo: bool,
    pub descricao: Option<String>,
    pub data_criacao: DateTime<Utc>,
}

// =============================================================================
// Centro de Custo
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CentroCusto {
    pub id: u32,
    pub codigo: String,
    pub nome: String,
    pub descricao: Option<String>,
    pub ativo: bool,
    pub data_criacao: DateTime<Utc>,
}

// =============================================================================
// Contas
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContaReceber {
    pub id: u32,
    pub cliente_id: u32,
    pub origem_tipo: OrigemReceber,
    pub origem_id: Option<u32>,
    pub descricao: String,
    /// Centavos.
    pub valor: i64,
    pub valor_pago: i64,
    pub vencimento: NaiveDate,
    pub data_pagamento: Option<NaiveDate>,
    pub data_competencia: NaiveDate,
    pub status: StatusConta,
    pub centro_custo_id: u32,
    pub plano_conta_id: u32,
    pub observacao: Option<String>,
    pub usuario_criacao_id: u32,
    pub usuario_pagamento_id: Option<u32>,
    pub data_criacao: DateTime<Utc>,
    pub data_atualizacao: DateTime<Utc>,
}

impl ContaReceber {
    pub fn calcular_status_efetivo(&self, hoje: NaiveDate) -> StatusConta {
        if self.status == StatusConta::Pago || self.status == StatusConta::Cancelado {
            return self.status;
        }
        if self.vencimento < hoje {
            return StatusConta::Atrasado;
        }
        self.status
    }
    pub fn saldo_restante(&self) -> i64 {
        self.valor - self.valor_pago
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContaPagar {
    pub id: u32,
    pub fornecedor: String,
    pub fornecedor_doc: Option<String>,
    pub descricao: String,
    pub valor: i64,
    pub valor_pago: i64,
    pub vencimento: NaiveDate,
    pub data_pagamento: Option<NaiveDate>,
    pub data_competencia: NaiveDate,
    pub status: StatusConta,
    pub centro_custo_id: u32,
    pub plano_conta_id: u32,
    pub origem_tipo: OrigemPagar,
    pub origem_id: Option<u32>,
    pub observacao: Option<String>,
    pub usuario_criacao_id: u32,
    pub usuario_pagamento_id: Option<u32>,
    pub data_criacao: DateTime<Utc>,
    pub data_atualizacao: DateTime<Utc>,
}

impl ContaPagar {
    pub fn calcular_status_efetivo(&self, hoje: NaiveDate) -> StatusConta {
        if self.status == StatusConta::Pago || self.status == StatusConta::Cancelado {
            return self.status;
        }
        if self.vencimento < hoje {
            return StatusConta::Atrasado;
        }
        self.status
    }
    pub fn saldo_restante(&self) -> i64 {
        self.valor - self.valor_pago
    }
}

// =============================================================================
// Lançamentos
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lancamento {
    pub id: u32,
    pub tipo: TipoLancamento,
    pub conta_receber_id: Option<u32>,
    pub conta_pagar_id: Option<u32>,
    pub valor: i64,
    pub data_lancamento: NaiveDate,
    pub data_competencia: NaiveDate,
    pub descricao: String,
    pub plano_conta_id: u32,
    pub centro_custo_id: u32,
    pub forma_pagamento: Option<String>,
    pub lancamento_estornado_id: Option<u32>,
    pub lancamento_origem_id: Option<u32>,
    pub usuario_id: u32,
    pub data_criacao: DateTime<Utc>,
}

// =============================================================================
// Alertas
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertaFinanceiro {
    pub id: u32,
    pub codigo: String,
    pub titulo: String,
    pub descricao: Option<String>,
    pub severidade: Severidade,
    pub condicao_tipo: String,
    pub condicao_threshold_dias: Option<u32>,
    pub condicao_threshold_valor: Option<i64>,
    pub ativo: bool,
    pub data_criacao: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertaFinanceiroOcorrencia {
    pub id: u32,
    pub alerta_id: u32,
    pub entidade_tipo: TipoEntidadeAlerta,
    pub entidade_id: u32,
    pub data_ocorrencia: DateTime<Utc>,
    pub severidade: Severidade,
    pub mensagem: String,
    pub contexto_json: Option<serde_json::Value>,
    pub visualizado: bool,
    pub data_visualizacao: Option<DateTime<Utc>>,
    pub resolvido: bool,
    pub data_resolucao: Option<DateTime<Utc>>,
}

// =============================================================================
// Dashboard
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DashboardFinanceiro {
    pub receita_prevista: i64,
    pub receita_recebida_mes: i64,
    pub receita_recebida_total: i64,
    pub despesa_prevista: i64,
    pub despesa_paga_mes: i64,
    pub despesa_paga_total: i64,
    pub saldo_atual: i64,
    pub saldo_projetado_30_dias: i64,
    pub inadimplencia_percentual: f32,
    pub inadimplencia_valor: i64,
    pub ticket_medio: i64,
    pub contas_receber_total: u32,
    pub contas_receber_pendentes: u32,
    pub contas_receber_atrasadas: u32,
    pub contas_pagar_total: u32,
    pub contas_pagar_pendentes: u32,
    pub contas_pagar_atrasadas: u32,
    pub fluxo_por_centro_custo: Vec<ResumoFluxo>,
    pub top_clientes_inadimplentes: Vec<ClienteInadimplente>,
    pub alertas_pendentes: u32,
    pub alertas_criticos: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumoFluxo {
    pub centro_custo_id: u32,
    pub centro_custo_nome: String,
    pub total_entradas: i64,
    pub total_saidas: i64,
    pub saldo: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClienteInadimplente {
    pub cliente_id: u32,
    pub cliente_nome: String,
    pub valor_atrasado: i64,
    pub quantidade_contas: u32,
}

// =============================================================================
// Helpers
// =============================================================================

pub fn parse_date(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

pub fn parse_dt(s: &str) -> DateTime<Utc> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return dt.with_timezone(&Utc);
    }
    if let Ok(ndt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
        return DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc);
    }
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return DateTime::<Utc>::from_naive_utc_and_offset(d.and_hms_opt(0, 0, 0).unwrap(), Utc);
    }
    Utc::now()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn status_conta_roundtrip() {
        for s in [
            StatusConta::Pendente,
            StatusConta::Parcial,
            StatusConta::Pago,
            StatusConta::Atrasado,
            StatusConta::Cancelado,
        ] {
            assert_eq!(StatusConta::from_db_str(s.as_db_str()), s);
        }
    }

    #[test]
    fn status_final_pago_ou_cancelado() {
        assert!(StatusConta::Pago.is_final());
        assert!(StatusConta::Cancelado.is_final());
        assert!(!StatusConta::Pendente.is_final());
        assert!(!StatusConta::Parcial.is_final());
        assert!(!StatusConta::Atrasado.is_final());
    }

    #[test]
    fn tipo_lancamento_roundtrip() {
        for t in [
            TipoLancamento::Entrada,
            TipoLancamento::Saida,
            TipoLancamento::Transferencia,
            TipoLancamento::Estorno,
        ] {
            assert_eq!(TipoLancamento::from_db_str(t.as_db_str()), t);
        }
    }

    #[test]
    fn status_efetivo_vira_atrasado_quando_vence() {
        let hoje = date(2026, 1, 15);
        let c = ContaReceber {
            id: 1,
            cliente_id: 1,
            origem_tipo: OrigemReceber::Manual,
            origem_id: None,
            descricao: "x".into(),
            valor: 1000,
            valor_pago: 0,
            vencimento: date(2026, 1, 10), // 5 dias atrás
            data_pagamento: None,
            data_competencia: date(2026, 1, 1),
            status: StatusConta::Pendente,
            centro_custo_id: 1,
            plano_conta_id: 1,
            observacao: None,
            usuario_criacao_id: 1,
            usuario_pagamento_id: None,
            data_criacao: Utc::now(),
            data_atualizacao: Utc::now(),
        };
        assert_eq!(c.calcular_status_efetivo(hoje), StatusConta::Atrasado);
    }

    #[test]
    fn status_efetivo_pendente_quando_nao_venceu() {
        let hoje = date(2026, 1, 15);
        let c = ContaReceber {
            vencimento: date(2026, 1, 20), // daqui 5 dias
            status: StatusConta::Pendente,
            ..conta_test()
        };
        assert_eq!(c.calcular_status_efetivo(hoje), StatusConta::Pendente);
    }

    #[test]
    fn status_efetivo_pago_mesmo_vencido() {
        let hoje = date(2026, 1, 15);
        let c = ContaReceber {
            vencimento: date(2026, 1, 10),
            status: StatusConta::Pago,
            ..conta_test()
        };
        assert_eq!(c.calcular_status_efetivo(hoje), StatusConta::Pago);
    }

    #[test]
    fn saldo_restante() {
        let c = ContaReceber {
            valor: 1000,
            valor_pago: 300,
            ..conta_test()
        };
        assert_eq!(c.saldo_restante(), 700);
    }

    fn conta_test() -> ContaReceber {
        ContaReceber {
            id: 1,
            cliente_id: 1,
            origem_tipo: OrigemReceber::Manual,
            origem_id: None,
            descricao: "x".into(),
            valor: 1000,
            valor_pago: 0,
            vencimento: date(2026, 1, 10),
            data_pagamento: None,
            data_competencia: date(2026, 1, 1),
            status: StatusConta::Pendente,
            centro_custo_id: 1,
            plano_conta_id: 1,
            observacao: None,
            usuario_criacao_id: 1,
            usuario_pagamento_id: None,
            data_criacao: Utc::now(),
            data_atualizacao: Utc::now(),
        }
    }
}
