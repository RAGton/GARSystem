// src/telas/login_empresas.rs
//
// MVP: lista hardcoded de tenants disponíveis.
// Em produção, isso vira uma chamada a GET /tenants/active (endpoint novo).
//
// Cada empresa é identificada por um "slug" que o backend usa pra rotear
// o login pro database correto (database-per-tenant).

pub struct EmpresaOpcao {
    pub slug: &'static str,
    pub cnpj: &'static str,
    pub razao_social: &'static str,
}

/// Lista de empresas demo (dev/preview).
/// Quando o backend tiver endpoint de listagem, substituir essa constante
/// por `let empresas = cliente.listar_tenants().await?;` no handler de init.
pub const EMPRESAS_DEMO: &[EmpresaOpcao] = &[
    EmpresaOpcao {
        slug: "techfix",
        cnpj: "12.345.678/0001-90",
        razao_social: "TechFix Assistência",
    },
    EmpresaOpcao {
        slug: "demo",
        cnpj: "00.000.000/0001-00",
        razao_social: "Empresa Demonstração",
    },
];
