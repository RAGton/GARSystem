// src/dto.rs
//
// DTOs (Data Transfer Objects) do GAR System.
//
// São apenas structs de serialização. NÃO têm lógica de negócio.
// Movidos de `src/server.rs` no Sprint P1.5 (Fase 1 - refatoração)
// para reduzir o monolito e tornar o código navegável.

use serde::{Deserialize, Serialize};

use crate::servicos::PapelUsuario;

// ============================================================================
// Auth
// ============================================================================

#[derive(Deserialize)]
pub struct LoginPayload {
    pub usuario: String,
    pub senha: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub papel: PapelUsuario,
    /// Tempo de expiração do token, em segundos (epoch).
    pub expira_em: i64,
}

/// Payload público para criação de usuário.
/// NÃO inclui `papel` (separação: criação vs. atribuição administrativa).
#[derive(Deserialize)]
pub struct CriarUsuarioPayload {
    pub nome_usuario: String,
    pub senha: String,
}

/// Payload interno para mudança de papel. Exige privilégio de Administrador.
#[derive(Deserialize)]
pub struct AlterarPapelPayload {
    pub novo_papel: PapelUsuario,
}

// ============================================================================
// Cliente
// ============================================================================

#[derive(Deserialize)]
pub struct ClientePayload {
    pub nome: String,
    pub email: String,
    pub telefone: String,
    pub endereco: Option<String>,
    pub cpf_cnpj: Option<String>,
}

#[derive(Serialize)]
pub struct ResumoClienteResponse {
    pub gastos_totais: f64,
    pub credito_disponivel: f64,
}

// ============================================================================
// Ordem de Serviço
// ============================================================================

#[derive(Deserialize)]
pub struct AtualizarOrdemPayload {
    pub os: crate::servicos::OrdemServico,
    pub usuario: String,
}

// ============================================================================
// Orçamento
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrcamentoItem {
    pub descricao: String,
    pub quantidade: u32,
    pub preco_unitario: f64,
    pub preco_total: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Orcamento {
    pub id: u32,
    pub cliente_id: u32,
    pub items: Vec<OrcamentoItem>,
    pub total: f64,
}
