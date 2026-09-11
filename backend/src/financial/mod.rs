// src/financial/mod.rs
//
// Módulo Financeiro — Sprint P2.5 (fundação).
//
// NÃO implementa (ainda):
// - integração bancária
// - PIX real
// - boleto
// - Open Finance
// - NF-e
// - multiempresa (P2.6)
//
// Apenas arquitetura + fundação:
//   - Configuração financeira (singleton)
//   - Plano de contas
//   - Centro de custo
//   - Contas a receber (criação manual + auto via orçamento)
//   - Contas a pagar (criação manual; gancho para cotação em P2.5.x)
//   - Lançamentos (append-only, imutável)
//   - Fluxo de caixa
//   - Alertas financeiros
//   - Dashboard
//
// Ver `docs/FINANCIAL-ARCHITECTURE.md` para a visão completa.

pub mod models;
pub mod repository;
pub mod service;

pub use models::{
    AlertaFinanceiro, AlertaFinanceiroOcorrencia, CentroCusto, ClienteInadimplente,
    ConfiguracaoFinanceira, ContaPagar, ContaReceber, DashboardFinanceiro, Lancamento, OrigemPagar,
    OrigemReceber, PlanoConta, ResumoFluxo, Severidade, StatusConta, TipoEntidadeAlerta,
    TipoLancamento, TipoPlanoConta,
};
pub use service::{
    atualizar_configuracao, calcular_fluxo, cancelar_conta_receber, criar_conta_pagar,
    criar_conta_receber, dashboard, estornar_lancamento, gerar_conta_receber_de_orcamento,
    listar_alertas_ativos, listar_alertas_pendentes, listar_centros_custo, listar_contas_pagar,
    listar_contas_receber, listar_plano_contas, marcar_alerta_resolvido, obter_centro_custo,
    obter_configuracao, obter_conta_pagar, obter_conta_receber, obter_plano_conta,
    pagar_conta_pagar, pagar_conta_receber, verificar_alertas, Contexto, ErroFinanceiro,
    FluxoCaixaResultado,
};
