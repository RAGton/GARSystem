// src/financial/service.rs
//
// Sprint P2.5 — Financeiro.
//
// Orquestra:
//   1. Validação de regras
//   2. Persistência (repository)
//   3. Lançamento automático (append-only)
//   4. Geração de ContaReceber a partir de Orçamento aprovado (P2.2)
//   5. Cálculo de fluxo de caixa + dashboard
//   6. Alertas financeiros
//   7. Timeline CRM

use super::models::{
    AlertaFinanceiro, AlertaFinanceiroOcorrencia, CentroCusto, ClienteInadimplente,
    ConfiguracaoFinanceira, ContaPagar, ContaReceber, DashboardFinanceiro, Lancamento, OrigemPagar,
    OrigemReceber, PlanoConta, ResumoFluxo, Severidade, StatusConta, TipoEntidadeAlerta,
    TipoLancamento, TipoPlanoConta,
};
use super::repository;
use crate::servicos::ErroAplicacao;
use chrono::{Datelike, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Contexto {
    pub usuario_id: u32,
    pub username: Option<String>,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ErroFinanceiro {
    #[error("configuração financeira não inicializada")]
    ConfigNaoInicializada,
    #[error("valor inválido: {0}")]
    ValorInvalido(String),
    #[error("conta não encontrada")]
    ContaNaoEncontrada,
    #[error("conta já finalizada (PAGO/CANCELADO)")]
    ContaJaFinalizada,
    #[error("valor pago excede saldo")]
    ValorPagoExcede,
    #[error("centro de custo não encontrado: {0}")]
    CentroCustoInvalido(u32),
    #[error("plano de contas não encontrado: {0}")]
    PlanoContaInvalido(u32),
    #[error("descrição obrigatória")]
    DescricaoObrigatoria,
    #[error("erro desconhecido: {0}")]
    Desconhecido(String),
}

impl From<ErroFinanceiro> for ErroAplicacao {
    fn from(e: ErroFinanceiro) -> Self {
        ErroAplicacao::Desconhecido(format!("{:?}", e))
    }
}

impl From<ErroAplicacao> for ErroFinanceiro {
    fn from(e: ErroAplicacao) -> Self {
        ErroFinanceiro::Desconhecido(format!("{:?}", e))
    }
}

// =============================================================================
// Configuração
// =============================================================================

pub fn obter_configuracao(tenant_id: i32) -> Result<ConfiguracaoFinanceira, ErroFinanceiro> {
    Ok(repository::obter_configuracao(tenant_id)?)
}

pub fn atualizar_configuracao(
    tenant_id: i32,
    dias: u32,
    cc_rec: Option<u32>,
    pc_rec: Option<u32>,
    cc_pag: Option<u32>,
    pc_pag: Option<u32>,
    moeda: &str,
) -> Result<(), ErroFinanceiro> {
    Ok(repository::atualizar_configuracao(
        tenant_id, dias, cc_rec, pc_rec, cc_pag, pc_pag, moeda,
    )?)
}

// =============================================================================
// Plano de Contas
// =============================================================================

pub fn listar_plano_contas(
    tenant_id: i32,
    ativo_apenas: bool,
) -> Result<Vec<PlanoConta>, ErroFinanceiro> {
    Ok(repository::listar_plano_contas(tenant_id, ativo_apenas)?)
}

pub fn obter_plano_conta(tenant_id: i32, id: u32) -> Result<Option<PlanoConta>, ErroFinanceiro> {
    Ok(repository::obter_plano_conta(tenant_id, id)?)
}

// =============================================================================
// Centro de Custo
// =============================================================================

pub fn listar_centros_custo(
    tenant_id: i32,
    ativo_apenas: bool,
) -> Result<Vec<CentroCusto>, ErroFinanceiro> {
    Ok(repository::listar_centros_custo(tenant_id, ativo_apenas)?)
}

pub fn obter_centro_custo(tenant_id: i32, id: u32) -> Result<Option<CentroCusto>, ErroFinanceiro> {
    Ok(repository::obter_centro_custo(tenant_id, id)?)
}

// =============================================================================
// Contas a Receber
// =============================================================================

pub fn criar_conta_receber(
    tenant_id: i32,
    cliente_id: u32,
    origem_tipo: OrigemReceber,
    origem_id: Option<u32>,
    descricao: &str,
    valor: i64,
    vencimento: NaiveDate,
    centro_custo_id: u32,
    plano_conta_id: u32,
    observacao: Option<&str>,
    ctx: &Contexto,
) -> Result<u32, ErroFinanceiro> {
    validar_basico(valor, descricao, centro_custo_id, plano_conta_id)?;
    let comp = competencia(vencimento);
    let id = repository::inserir_conta_receber(
        tenant_id,
        cliente_id,
        origem_tipo,
        origem_id,
        descricao,
        valor,
        vencimento,
        comp,
        centro_custo_id,
        plano_conta_id,
        observacao,
        ctx.usuario_id,
    )?;
    let _ = registrar_evento_crm_conta_receber(
        crate::servicos::TENANT_LEGACY,
        id,
        "CONTA_RECEBER_CRIADA",
        "Conta a receber criada",
        ctx,
    );
    Ok(id)
}

pub fn obter_conta_receber(
    tenant_id: i32,
    id: u32,
) -> Result<Option<ContaReceber>, ErroFinanceiro> {
    Ok(repository::obter_conta_receber(tenant_id, id)?)
}

pub fn listar_contas_receber(
    tenant_id: i32,
    cliente_id: Option<u32>,
    status: Option<StatusConta>,
    limite: u32,
) -> Result<Vec<ContaReceber>, ErroFinanceiro> {
    Ok(repository::listar_contas_receber(
        tenant_id, cliente_id, status, limite,
    )?)
}

pub fn pagar_conta_receber(
    tenant_id: i32,
    id: u32,
    valor_pago: i64,
    data_pagamento: NaiveDate,
    forma_pagamento: Option<&str>,
    ctx: &Contexto,
) -> Result<ContaReceber, ErroFinanceiro> {
    if valor_pago <= 0 {
        return Err(ErroFinanceiro::ValorInvalido(
            "valor_pago deve ser > 0".into(),
        ));
    }
    let conta = repository::obter_conta_receber(tenant_id, id)?
        .ok_or(ErroFinanceiro::ContaNaoEncontrada)?;
    if conta
        .calcular_status_efetivo(Utc::now().date_naive())
        .is_final()
    {
        return Err(ErroFinanceiro::ContaJaFinalizada);
    }
    let novo_total_pago = conta.valor_pago + valor_pago;
    if novo_total_pago > conta.valor {
        return Err(ErroFinanceiro::ValorPagoExcede);
    }
    let novo_status = if novo_total_pago == conta.valor {
        StatusConta::Pago
    } else {
        StatusConta::Parcial
    };
    repository::atualizar_pagamento_conta_receber(
        tenant_id,
        id,
        novo_total_pago,
        data_pagamento,
        novo_status,
        ctx.usuario_id,
    )?;
    // Lançamento de ENTRADA
    let _ = repository::inserir_lancamento(
        tenant_id,
        TipoLancamento::Entrada,
        Some(id),
        None,
        valor_pago,
        data_pagamento,
        competencia(data_pagamento),
        &format!("Recebimento de conta #{}", id),
        conta.plano_conta_id,
        conta.centro_custo_id,
        forma_pagamento,
        None,
        None,
        ctx.usuario_id,
    )?;
    let _ = registrar_evento_crm_conta_receber(
        crate::servicos::TENANT_LEGACY,
        id,
        "CONTA_RECEBER_PAGA",
        &format!("Recebido R$ {:.2}", valor_pago as f64 / 100.0),
        ctx,
    );
    let atualizada = repository::obter_conta_receber(tenant_id, id)?
        .ok_or(ErroFinanceiro::ContaNaoEncontrada)?;
    Ok(atualizada)
}

pub fn cancelar_conta_receber(
    tenant_id: i32,
    id: u32,
    ctx: &Contexto,
) -> Result<bool, ErroFinanceiro> {
    let conta = repository::obter_conta_receber(tenant_id, id)?
        .ok_or(ErroFinanceiro::ContaNaoEncontrada)?;
    if conta.calcular_status_efetivo(Utc::now().date_naive()) == StatusConta::Pago {
        return Err(ErroFinanceiro::ContaJaFinalizada);
    }
    let ok = repository::cancelar_conta_receber(tenant_id, id)?;
    if ok {
        let _ = registrar_evento_crm_conta_receber(
            crate::servicos::TENANT_LEGACY,
            id,
            "CONTA_RECEBER_CANCELADA",
            "Conta a receber cancelada",
            ctx,
        );
    }
    Ok(ok)
}

// =============================================================================
// Contas a Pagar
// =============================================================================

pub fn criar_conta_pagar(
    tenant_id: i32,
    fornecedor: &str,
    fornecedor_doc: Option<&str>,
    descricao: &str,
    valor: i64,
    vencimento: NaiveDate,
    centro_custo_id: u32,
    plano_conta_id: u32,
    origem_tipo: OrigemPagar,
    origem_id: Option<u32>,
    observacao: Option<&str>,
    ctx: &Contexto,
) -> Result<u32, ErroFinanceiro> {
    if fornecedor.trim().is_empty() {
        return Err(ErroFinanceiro::DescricaoObrigatoria);
    }
    validar_basico(valor, descricao, centro_custo_id, plano_conta_id)?;
    let comp = competencia(vencimento);
    let id = repository::inserir_conta_pagar(
        tenant_id,
        fornecedor,
        fornecedor_doc,
        descricao,
        valor,
        vencimento,
        comp,
        centro_custo_id,
        plano_conta_id,
        origem_tipo,
        origem_id,
        observacao,
        ctx.usuario_id,
    )?;
    Ok(id)
}

pub fn obter_conta_pagar(tenant_id: i32, id: u32) -> Result<Option<ContaPagar>, ErroFinanceiro> {
    Ok(repository::obter_conta_pagar(tenant_id, id)?)
}

pub fn listar_contas_pagar(
    tenant_id: i32,
    status: Option<StatusConta>,
    limite: u32,
) -> Result<Vec<ContaPagar>, ErroFinanceiro> {
    Ok(repository::listar_contas_pagar(tenant_id, status, limite)?)
}

pub fn pagar_conta_pagar(
    tenant_id: i32,
    id: u32,
    valor_pago: i64,
    data_pagamento: NaiveDate,
    forma_pagamento: Option<&str>,
    ctx: &Contexto,
) -> Result<ContaPagar, ErroFinanceiro> {
    if valor_pago <= 0 {
        return Err(ErroFinanceiro::ValorInvalido(
            "valor_pago deve ser > 0".into(),
        ));
    }
    let conta =
        repository::obter_conta_pagar(tenant_id, id)?.ok_or(ErroFinanceiro::ContaNaoEncontrada)?;
    if conta
        .calcular_status_efetivo(Utc::now().date_naive())
        .is_final()
    {
        return Err(ErroFinanceiro::ContaJaFinalizada);
    }
    let novo_total_pago = conta.valor_pago + valor_pago;
    if novo_total_pago > conta.valor {
        return Err(ErroFinanceiro::ValorPagoExcede);
    }
    let novo_status = if novo_total_pago == conta.valor {
        StatusConta::Pago
    } else {
        StatusConta::Parcial
    };
    repository::atualizar_pagamento_conta_pagar(
        tenant_id,
        id,
        novo_total_pago,
        data_pagamento,
        novo_status,
        ctx.usuario_id,
    )?;
    let _ = repository::inserir_lancamento(
        tenant_id,
        TipoLancamento::Saida,
        None,
        Some(id),
        valor_pago,
        data_pagamento,
        competencia(data_pagamento),
        &format!("Pagamento de conta #{}", id),
        conta.plano_conta_id,
        conta.centro_custo_id,
        forma_pagamento,
        None,
        None,
        ctx.usuario_id,
    )?;
    let atualizada =
        repository::obter_conta_pagar(tenant_id, id)?.ok_or(ErroFinanceiro::ContaNaoEncontrada)?;
    Ok(atualizada)
}

// =============================================================================
// Estorno (gera lançamento inverso)
// =============================================================================

pub fn estornar_lancamento(
    tenant_id: i32,
    lancamento_id: u32,
    motivo: &str,
    ctx: &Contexto,
) -> Result<u32, ErroFinanceiro> {
    let original = repository::obter_lancamento(tenant_id, lancamento_id)?
        .ok_or(ErroFinanceiro::ContaNaoEncontrada)?;
    if original.tipo == TipoLancamento::Estorno {
        return Err(ErroFinanceiro::ValorInvalido(
            "lançamento já é estorno".into(),
        ));
    }
    let novo_tipo = match original.tipo {
        TipoLancamento::Entrada => TipoLancamento::Saida,
        TipoLancamento::Saida => TipoLancamento::Entrada,
        _ => TipoLancamento::Estorno,
    };
    let id = repository::inserir_lancamento(
        tenant_id,
        novo_tipo,
        original.conta_receber_id,
        original.conta_pagar_id,
        original.valor,
        original.data_lancamento,
        original.data_competencia,
        &format!("ESTORNO de #{}: {}", lancamento_id, motivo),
        original.plano_conta_id,
        original.centro_custo_id,
        original.forma_pagamento.as_deref(),
        Some(lancamento_id),
        Some(lancamento_id),
        ctx.usuario_id,
    )?;
    Ok(id)
}

// =============================================================================
// Fluxo de Caixa
// =============================================================================

pub fn calcular_fluxo(
    tenant_id: i32,
    inicio: NaiveDate,
    fim: NaiveDate,
) -> Result<FluxoCaixaResultado, ErroFinanceiro> {
    use mysql::prelude::Queryable;
    let mut conn = crate::banco_de_dados::obter_conexao()?;

    // Saldo realizado = soma dos lançamentos até `fim`
    let ini_str = inicio.format("%Y-%m-%d").to_string();
    let fim_str = fim.format("%Y-%m-%d").to_string();
    let saldo_realizado: i64 = conn
        .exec_first(
            r#"SELECT COALESCE(SUM(CASE WHEN tipo='ENTRADA' THEN valor ELSE -valor END), 0)
               FROM lancamentos
               WHERE tenant_id = ? AND data_competencia <= ?"#,
            (tenant_id, &fim_str),
        )
        .map_err(|e| ErroFinanceiro::Desconhecido(e.to_string()))?
        .unwrap_or(0);

    // Receitas pendentes no período
    let receitas_pendentes: i64 = conn
        .exec_first(
            r#"SELECT COALESCE(SUM(valor - valor_pago), 0)
               FROM contas_receber
               WHERE tenant_id = ? AND status IN ('PENDENTE','PARCIAL') AND vencimento BETWEEN ? AND ?"#,
            (tenant_id, &ini_str, &fim_str),
        )
        .map_err(|e| ErroFinanceiro::Desconhecido(e.to_string()))?
        .unwrap_or(0);
    let despesas_pendentes: i64 = conn
        .exec_first(
            r#"SELECT COALESCE(SUM(valor - valor_pago), 0)
               FROM contas_pagar
               WHERE tenant_id = ? AND status IN ('PENDENTE','PARCIAL') AND vencimento BETWEEN ? AND ?"#,
            (tenant_id, &ini_str, &fim_str),
        )
        .map_err(|e| ErroFinanceiro::Desconhecido(e.to_string()))?
        .unwrap_or(0);

    Ok(FluxoCaixaResultado {
        saldo_realizado,
        saldo_projetado: saldo_realizado + receitas_pendentes - despesas_pendentes,
        receitas_pendentes,
        despesas_pendentes,
        periodo_inicio: inicio,
        periodo_fim: fim,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluxoCaixaResultado {
    pub saldo_realizado: i64,
    pub saldo_projetado: i64,
    pub receitas_pendentes: i64,
    pub despesas_pendentes: i64,
    pub periodo_inicio: NaiveDate,
    pub periodo_fim: NaiveDate,
}

// =============================================================================
// Dashboard Financeiro
// =============================================================================

pub fn dashboard(tenant_id: i32) -> Result<DashboardFinanceiro, ErroFinanceiro> {
    use mysql::prelude::Queryable;
    let mut conn = crate::banco_de_dados::obter_conexao()?;
    let mut dash = DashboardFinanceiro::default();
    let hoje = Utc::now().date_naive();
    let hoje_str = hoje.format("%Y-%m-%d").to_string();
    let mes = NaiveDate::from_ymd_opt(hoje.year(), hoje.month(), 1).unwrap();
    let mes_str = mes.format("%Y-%m-%d").to_string();
    let trinta_dias = hoje + chrono::Duration::days(30);
    let trinta_str = trinta_dias.format("%Y-%m-%d").to_string();

    // QUERY 1: Totais de receita/despesa em uma só consulta (CASE WHEN + GROUP BY)
    let totais: mysql::Row = conn.exec_first(
        r#"SELECT
              COALESCE(SUM(CASE WHEN tipo='CR' AND status IN ('PENDENTE','PARCIAL') THEN valor - valor_pago ELSE 0 END), 0) as receita_prevista,
              COALESCE(SUM(CASE WHEN tipo='CR' THEN valor_pago ELSE 0 END), 0) as receita_total,
              COALESCE(SUM(CASE WHEN tipo='CR' AND data_pagamento >= ? THEN valor_pago ELSE 0 END), 0) as receita_mes,
              COALESCE(SUM(CASE WHEN tipo='CP' AND status IN ('PENDENTE','PARCIAL') THEN valor - valor_pago ELSE 0 END), 0) as despesa_prevista,
              COALESCE(SUM(CASE WHEN tipo='CP' THEN valor_pago ELSE 0 END), 0) as despesa_total,
              COALESCE(SUM(CASE WHEN tipo='CP' AND data_pagamento >= ? THEN valor_pago ELSE 0 END), 0) as despesa_mes
           FROM (
             SELECT 'CR' as tipo, status, valor, valor_pago, data_pagamento FROM contas_receber WHERE tenant_id = ?
             UNION ALL
             SELECT 'CP' as tipo, status, valor, valor_pago, data_pagamento FROM contas_pagar WHERE tenant_id = ?
           ) t"#,
        (&mes_str, &mes_str, tenant_id, tenant_id),
    ).map_err(|e| ErroFinanceiro::Desconhecido(e.to_string()))?
     .ok_or_else(|| ErroFinanceiro::Desconhecido("totais vazio".into()))?;
    dash.receita_prevista = get_i64(&totais, 0);
    dash.receita_recebida_total = get_i64(&totais, 1);
    dash.receita_recebida_mes = get_i64(&totais, 2);
    dash.despesa_prevista = get_i64(&totais, 3);
    dash.despesa_paga_total = get_i64(&totais, 4);
    dash.despesa_paga_mes = get_i64(&totais, 5);

    // QUERY 2: Saldos e projeções
    let saldos: mysql::Row = conn.exec_first(
        r#"SELECT
              COALESCE((SELECT SUM(CASE WHEN tipo='ENTRADA' THEN valor ELSE -valor END) FROM lancamentos WHERE tenant_id = ?), 0) as saldo_atual,
              COALESCE((SELECT SUM(valor - valor_pago) FROM contas_receber WHERE tenant_id = ? AND status IN ('PENDENTE','PARCIAL') AND vencimento <= ?), 0) as cr_30d,
              COALESCE((SELECT SUM(valor - valor_pago) FROM contas_pagar WHERE tenant_id = ? AND status IN ('PENDENTE','PARCIAL') AND vencimento <= ?), 0) as cp_30d"#,
        (tenant_id, tenant_id, &trinta_str, tenant_id, &trinta_str),
    ).map_err(|e| ErroFinanceiro::Desconhecido(e.to_string()))?
     .ok_or_else(|| ErroFinanceiro::Desconhecido("saldos vazio".into()))?;
    let saldo_atual = get_i64(&saldos, 0);
    let cr_30d = get_i64(&saldos, 1);
    let cp_30d = get_i64(&saldos, 2);
    dash.saldo_atual = saldo_atual;
    dash.saldo_projetado_30_dias = saldo_atual + cr_30d - cp_30d;

    // QUERY 3: Contagens e inadimplência (CTE para evitar limite de params do MySQL)
    let counts: mysql::Row = conn.exec_first(
        r#"SELECT
              SUM(CASE WHEN tabela='cr' THEN 1 ELSE 0 END) as cr_total,
              SUM(CASE WHEN tabela='cr' AND status_pendente THEN 1 ELSE 0 END) as cr_pendentes,
              SUM(CASE WHEN tabela='cr' AND status_pendente AND atrasada THEN 1 ELSE 0 END) as cr_atrasadas,
              SUM(CASE WHEN tabela='cp' THEN 1 ELSE 0 END) as cp_total,
              SUM(CASE WHEN tabela='cp' AND status_pendente THEN 1 ELSE 0 END) as cp_pendentes,
              SUM(CASE WHEN tabela='cp' AND status_pendente AND atrasada THEN 1 ELSE 0 END) as cp_atrasadas,
              COALESCE(SUM(CASE WHEN tabela='cr' AND status_pendente THEN valor - valor_pago ELSE 0 END), 0) as valor_em_aberto,
              COALESCE(SUM(CASE WHEN tabela='cr' AND status_pendente AND atrasada THEN valor - valor_pago ELSE 0 END), 0) as valor_atrasado,
              SUM(CASE WHEN tabela='cr' AND recebida_mes THEN 1 ELSE 0 END) as qtd_mes
           FROM (
             SELECT 'cr' as tabela, status IN ('PENDENTE','PARCIAL') as status_pendente,
                    vencimento < ? as atrasada, valor, valor_pago,
                    data_pagamento >= ? AND valor_pago > 0 as recebida_mes
             FROM contas_receber WHERE tenant_id = ?
             UNION ALL
             SELECT 'cp' as tabela, status IN ('PENDENTE','PARCIAL') as status_pendente,
                    vencimento < ? as atrasada, valor, valor_pago, FALSE as recebida_mes
             FROM contas_pagar WHERE tenant_id = ?
           ) sub"#,
        (&hoje_str, &mes_str, tenant_id, &hoje_str, tenant_id),
    ).map_err(|e| ErroFinanceiro::Desconhecido(e.to_string()))?
     .ok_or_else(|| ErroFinanceiro::Desconhecido("counts vazio".into()))?;
    dash.contas_receber_total = get_u32(&counts, 0);
    dash.contas_receber_pendentes = get_u32(&counts, 1);
    dash.contas_receber_atrasadas = get_u32(&counts, 2);
    dash.contas_pagar_total = get_u32(&counts, 3);
    dash.contas_pagar_pendentes = get_u32(&counts, 4);
    dash.contas_pagar_atrasadas = get_u32(&counts, 5);
    let valor_em_aberto = get_i64(&counts, 6);
    let valor_atrasado = get_i64(&counts, 7);
    dash.inadimplencia_valor = valor_atrasado;
    if valor_em_aberto > 0 {
        dash.inadimplencia_percentual = (valor_atrasado as f32 / valor_em_aberto as f32) * 100.0;
    }
    let qtd_mes = get_i64(&counts, 8);
    if qtd_mes > 0 && dash.receita_recebida_mes > 0 {
        dash.ticket_medio = dash.receita_recebida_mes / qtd_mes;
    }

    // QUERY 4: Fluxo por centro de custo
    let centro_rows: Vec<mysql::Row> = conn
        .exec(
            r#"SELECT c.id, c.nome,
                  COALESCE(SUM(CASE WHEN l.tipo='ENTRADA' THEN l.valor ELSE 0 END), 0) as entradas,
                  COALESCE(SUM(CASE WHEN l.tipo='SAIDA' THEN l.valor ELSE 0 END), 0) as saidas
           FROM centros_custo c
           LEFT JOIN lancamentos l ON l.centro_custo_id = c.id AND l.tenant_id = c.tenant_id
           WHERE c.tenant_id = ?
           GROUP BY c.id, c.nome
           ORDER BY c.nome"#,
            (tenant_id,),
        )
        .map_err(|e| ErroFinanceiro::Desconhecido(e.to_string()))?;
    for r in centro_rows {
        let id = get_u32(&r, 0);
        let nome = get_string(&r, 1);
        let entradas = get_i64(&r, 2);
        let saidas = get_i64(&r, 3);
        dash.fluxo_por_centro_custo.push(ResumoFluxo {
            centro_custo_id: id,
            centro_custo_nome: nome,
            total_entradas: entradas,
            total_saidas: saidas,
            saldo: entradas - saidas,
        });
    }

    // QUERY 5: Top inadimplentes + alertas
    let top_rows: Vec<mysql::Row> = conn.exec(
        r#"SELECT c.id, c.nome, COUNT(*) as qtd, COALESCE(SUM(cr.valor - cr.valor_pago), 0) as valor
           FROM contas_receber cr
           INNER JOIN clientes c ON c.id = cr.cliente_id AND c.tenant_id = cr.tenant_id
           WHERE cr.tenant_id = ? AND cr.status IN ('PENDENTE','PARCIAL') AND cr.vencimento < ?
           GROUP BY c.id, c.nome
           ORDER BY valor DESC
           LIMIT 10"#,
        (tenant_id, &hoje_str),
    ).map_err(|e| ErroFinanceiro::Desconhecido(e.to_string()))?;
    for r in top_rows {
        dash.top_clientes_inadimplentes.push(ClienteInadimplente {
            cliente_id: get_u32(&r, 0),
            cliente_nome: get_string(&r, 1),
            valor_atrasado: get_i64(&r, 3),
            quantidade_contas: get_u32(&r, 2),
        });
    }
    let alerta: mysql::Row = conn.exec_first(
        r#"SELECT
              (SELECT COUNT(*) FROM alertas_financeiros_ocorrencias WHERE tenant_id = ? AND resolvido = FALSE),
              (SELECT COUNT(*) FROM alertas_financeiros_ocorrencias WHERE tenant_id = ? AND resolvido = FALSE AND severidade = 'CRITICAL')"#,
        (tenant_id, tenant_id),
    ).map_err(|e| ErroFinanceiro::Desconhecido(e.to_string()))?
     .ok_or_else(|| ErroFinanceiro::Desconhecido("alerta vazio".into()))?;
    dash.alertas_pendentes = get_u32(&alerta, 0);
    dash.alertas_criticos = get_u32(&alerta, 1);

    Ok(dash)
}

fn get_i64(row: &mysql::Row, idx: usize) -> i64 {
    match row.get::<mysql::Value, _>(idx) {
        Some(mysql::Value::Int(n)) => n,
        Some(mysql::Value::UInt(n)) => n as i64,
        _ => 0,
    }
}

fn get_u32(row: &mysql::Row, idx: usize) -> u32 {
    match row.get::<mysql::Value, _>(idx) {
        Some(mysql::Value::UInt(n)) => n as u32,
        Some(mysql::Value::Int(n)) => n as u32,
        _ => 0,
    }
}

fn get_string(row: &mysql::Row, idx: usize) -> String {
    match row.get::<mysql::Value, _>(idx) {
        Some(mysql::Value::Bytes(b)) => String::from_utf8_lossy(&b).into_owned(),
        _ => String::new(),
    }
}

// =============================================================================
// Alertas
// =============================================================================

pub fn listar_alertas_ativos(tenant_id: i32) -> Result<Vec<AlertaFinanceiro>, ErroFinanceiro> {
    Ok(repository::listar_alertas_financeiros_ativos(tenant_id)?)
}

pub fn listar_alertas_pendentes(
    tenant_id: i32,
    limite: u32,
) -> Result<Vec<AlertaFinanceiroOcorrencia>, ErroFinanceiro> {
    Ok(repository::listar_ocorrencias_financeiras_pendentes(
        tenant_id, limite,
    )?)
}

pub fn marcar_alerta_resolvido(tenant_id: i32, id: u32) -> Result<bool, ErroFinanceiro> {
    Ok(repository::marcar_ocorrencia_financeira_resolvida(
        tenant_id, id,
    )?)
}

pub fn verificar_alertas(tenant_id: i32) -> Result<u32, ErroFinanceiro> {
    use mysql::prelude::Queryable;
    let alertas = repository::listar_alertas_financeiros_ativos(tenant_id)?;
    let mut conn = crate::banco_de_dados::obter_conexao()?;
    let mut criadas = 0u32;
    let hoje = Utc::now().date_naive();
    let hoje_str = hoje.format("%Y-%m-%d").to_string();
    for alerta in alertas {
        match alerta.condicao_tipo.as_str() {
            "VENCIMENTO_PROXIMO" => {
                let dias = alerta.condicao_threshold_dias.unwrap_or(3) as i64;
                let limite = (hoje + chrono::Duration::days(dias))
                    .format("%Y-%m-%d")
                    .to_string();
                // Receber
                let cr_rows: Vec<mysql::Row> = conn.exec(
                    r#"SELECT id FROM contas_receber
                       WHERE tenant_id = ? AND status IN ('PENDENTE','PARCIAL') AND vencimento BETWEEN ? AND ?"#,
                    (tenant_id, &hoje_str, &limite),
                ).map_err(|e| ErroFinanceiro::Desconhecido(e.to_string()))?;
                for r in cr_rows {
                    let cid: u32 = match r.get::<mysql::Value, _>(0) {
                        Some(mysql::Value::UInt(n)) => n as u32,
                        _ => continue,
                    };
                    let _ = repository::inserir_ocorrencia_financeira(
                        tenant_id,
                        alerta.id,
                        TipoEntidadeAlerta::ContaReceber,
                        cid,
                        alerta.severidade,
                        &format!("{} (#{})", alerta.titulo, cid),
                        Some(&serde_json::json!({ "conta_receber_id": cid })),
                    );
                    criadas += 1;
                }
                // Pagar
                let cp_rows: Vec<mysql::Row> = conn.exec(
                    r#"SELECT id FROM contas_pagar
                       WHERE tenant_id = ? AND status IN ('PENDENTE','PARCIAL') AND vencimento BETWEEN ? AND ?"#,
                    (tenant_id, &hoje_str, &limite),
                ).map_err(|e| ErroFinanceiro::Desconhecido(e.to_string()))?;
                for r in cp_rows {
                    let cid: u32 = match r.get::<mysql::Value, _>(0) {
                        Some(mysql::Value::UInt(n)) => n as u32,
                        _ => continue,
                    };
                    let _ = repository::inserir_ocorrencia_financeira(
                        tenant_id,
                        alerta.id,
                        TipoEntidadeAlerta::ContaPagar,
                        cid,
                        alerta.severidade,
                        &format!("{} (#{})", alerta.titulo, cid),
                        Some(&serde_json::json!({ "conta_pagar_id": cid })),
                    );
                    criadas += 1;
                }
            }
            "VENCIMENTO_PASSADO" => {
                // Receber
                let cr_rows: Vec<mysql::Row> = conn.exec(
                    r#"SELECT id FROM contas_receber
                       WHERE tenant_id = ? AND status IN ('PENDENTE','PARCIAL') AND vencimento < ?"#,
                    (tenant_id, &hoje_str),
                ).map_err(|e| ErroFinanceiro::Desconhecido(e.to_string()))?;
                for r in cr_rows {
                    let cid: u32 = match r.get::<mysql::Value, _>(0) {
                        Some(mysql::Value::UInt(n)) => n as u32,
                        _ => continue,
                    };
                    let _ = repository::inserir_ocorrencia_financeira(
                        tenant_id,
                        alerta.id,
                        TipoEntidadeAlerta::ContaReceber,
                        cid,
                        alerta.severidade,
                        &format!("{} (#{})", alerta.titulo, cid),
                        Some(&serde_json::json!({ "conta_receber_id": cid })),
                    );
                    criadas += 1;
                }
                // Pagar
                let cp_rows: Vec<mysql::Row> = conn.exec(
                    r#"SELECT id FROM contas_pagar
                       WHERE tenant_id = ? AND status IN ('PENDENTE','PARCIAL') AND vencimento < ?"#,
                    (tenant_id, &hoje_str),
                ).map_err(|e| ErroFinanceiro::Desconhecido(e.to_string()))?;
                for r in cp_rows {
                    let cid: u32 = match r.get::<mysql::Value, _>(0) {
                        Some(mysql::Value::UInt(n)) => n as u32,
                        _ => continue,
                    };
                    let _ = repository::inserir_ocorrencia_financeira(
                        tenant_id,
                        alerta.id,
                        TipoEntidadeAlerta::ContaPagar,
                        cid,
                        alerta.severidade,
                        &format!("{} (#{})", alerta.titulo, cid),
                        Some(&serde_json::json!({ "conta_pagar_id": cid })),
                    );
                    criadas += 1;
                }
            }
            "FLUXO_NEGATIVO" => {
                let trinta = (hoje + chrono::Duration::days(30))
                    .format("%Y-%m-%d")
                    .to_string();
                let cr: i64 = conn.exec_first(
                    "SELECT COALESCE(SUM(valor - valor_pago), 0) FROM contas_receber WHERE tenant_id = ? AND status IN ('PENDENTE','PARCIAL') AND vencimento <= ?",
                    (tenant_id, &trinta),
                ).ok().flatten().unwrap_or(0);
                let cp: i64 = conn.exec_first(
                    "SELECT COALESCE(SUM(valor - valor_pago), 0) FROM contas_pagar WHERE tenant_id = ? AND status IN ('PENDENTE','PARCIAL') AND vencimento <= ?",
                    (tenant_id, &trinta),
                ).ok().flatten().unwrap_or(0);
                let saldo_atual: i64 = conn.exec_first(
                    "SELECT COALESCE(SUM(CASE WHEN tipo='ENTRADA' THEN valor ELSE -valor END), 0) FROM lancamentos WHERE tenant_id = ?",
                    (tenant_id,),
                ).ok().flatten().unwrap_or(0);
                let projetado = saldo_atual + cr - cp;
                if projetado < 0 {
                    let _ = repository::inserir_ocorrencia_financeira(
                        tenant_id,
                        alerta.id,
                        TipoEntidadeAlerta::Fluxo,
                        0,
                        alerta.severidade,
                        &format!("Saldo projetado: R$ {:.2}", projetado as f64 / 100.0),
                        Some(&serde_json::json!({ "saldo_projetado": projetado })),
                    );
                    criadas += 1;
                }
            }
            "RECEBIMENTO_PENDENTE" => {
                let threshold = alerta.condicao_threshold_valor.unwrap_or(1_000_000);
                let total: i64 = conn.exec_first(
                    "SELECT COALESCE(SUM(valor - valor_pago), 0) FROM contas_receber WHERE tenant_id = ? AND status IN ('PENDENTE','PARCIAL')",
                    (tenant_id,),
                ).ok().flatten().unwrap_or(0);
                if total > threshold {
                    let _ = repository::inserir_ocorrencia_financeira(
                        tenant_id,
                        alerta.id,
                        TipoEntidadeAlerta::Fluxo,
                        0,
                        alerta.severidade,
                        &format!("Total a receber: R$ {:.2}", total as f64 / 100.0),
                        Some(&serde_json::json!({ "total": total })),
                    );
                    criadas += 1;
                }
            }
            _ => {}
        }
    }
    Ok(criadas)
}

// =============================================================================
// Geração automática a partir de Orçamento (P2.2 → P2.5)
// =============================================================================

/// Chamado por `cotacao_orcamento::service::decidir_orcamento` quando
/// `decisao == APROVADO`. Cria automaticamente uma `ContaReceber`.
pub fn gerar_conta_receber_de_orcamento(
    tenant_id: i32,
    orcamento_cliente_id: u32,
    orcamento_id: u32,
    orcamento_total_centavos: i64,
    orcamento_descricao: &str,
    ctx: &Contexto,
) -> Result<u32, ErroFinanceiro> {
    let config = repository::obter_configuracao(tenant_id)?;
    let cc = config
        .centro_custo_padrao_receber_id
        .ok_or(ErroFinanceiro::ConfigNaoInicializada)?;
    let pc = config
        .plano_conta_padrao_receber_id
        .ok_or(ErroFinanceiro::ConfigNaoInicializada)?;
    let vencimento =
        Utc::now().date_naive() + chrono::Duration::days(config.dias_vencimento_padrao as i64);
    let descricao = format!("OS/Orçamento #{} — {}", orcamento_id, orcamento_descricao);
    let id = criar_conta_receber(
        tenant_id,
        orcamento_cliente_id,
        OrigemReceber::Orcamento,
        Some(orcamento_id),
        &descricao,
        orcamento_total_centavos,
        vencimento,
        cc,
        pc,
        Some("Gerado automaticamente a partir de orçamento aprovado"),
        ctx,
    )?;
    Ok(id)
}

// =============================================================================
// Helpers
// =============================================================================

fn competencia(data: NaiveDate) -> NaiveDate {
    NaiveDate::from_ymd_opt(data.year(), data.month(), 1).unwrap()
}

fn validar_basico(
    valor: i64,
    descricao: &str,
    _centro_custo_id: u32,
    _plano_conta_id: u32,
) -> Result<(), ErroFinanceiro> {
    if valor <= 0 {
        return Err(ErroFinanceiro::ValorInvalido("valor deve ser > 0".into()));
    }
    if descricao.trim().is_empty() {
        return Err(ErroFinanceiro::DescricaoObrigatoria);
    }
    Ok(())
}

/// Wrapper para chamadas async. Resolve cliente_id a partir do
/// conta_receber_id e insere evento na timeline CRM.
async fn registrar_evento_crm_conta_receber(
    tenant_id: i32,
    conta_receber_id: u32,
    tipo: &str,
    descricao: &str,
    _ctx: &Contexto,
) -> Result<(), ErroAplicacao> {
    use mysql::prelude::Queryable;
    let mut conn = crate::banco_de_dados::obter_conexao()?;
    let cid: Option<u32> = conn
        .exec_first(
            "SELECT cliente_id FROM contas_receber WHERE id = ? AND tenant_id = ?",
            (conta_receber_id, tenant_id),
        )
        .ok()
        .flatten();
    if let Some(cliente_id) = cid {
        let payload = serde_json::json!({ "conta_receber_id": conta_receber_id });
        let _ = crate::crm::repository::inserir_evento_timeline(
            tenant_id,
            cliente_id,
            tipo,
            descricao,
            Some(&payload),
            Some(_ctx.usuario_id),
            _ctx.username.as_deref(),
        );
    }
    Ok(())
}

// =============================================================================
// Testes
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn competencia_eh_primeiro_dia_do_mes() {
        let d = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        assert_eq!(competencia(d), NaiveDate::from_ymd_opt(2026, 6, 1).unwrap());
    }

    #[test]
    fn validar_basico_rejeita_valor_zero() {
        let r = validar_basico(0, "x", 1, 1);
        assert!(matches!(r, Err(ErroFinanceiro::ValorInvalido(_))));
    }

    #[test]
    fn validar_basico_rejeita_descricao_vazia() {
        let r = validar_basico(100, "  ", 1, 1);
        assert!(matches!(r, Err(ErroFinanceiro::DescricaoObrigatoria)));
    }
}
