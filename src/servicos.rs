// src/servicos.rs

use mysql::prelude::Queryable;
use mysql::FromRowError;
use serde::{Deserialize, Serialize};
use thiserror::Error; // Necessário para a conversão de erros

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum StatusOS {
    Aberta,
    Orcamento,
    Aprovada,
    EmAndamento,
    AguardandoPeca,
    Finalizada,
    Cancelada,
}

impl StatusOS {
    pub fn iter() -> impl Iterator<Item = StatusOS> {
        [
            StatusOS::Aberta,
            StatusOS::Orcamento,
            StatusOS::Aprovada,
            StatusOS::EmAndamento,
            StatusOS::AguardandoPeca,
            StatusOS::Finalizada,
            StatusOS::Cancelada,
        ]
        .iter()
        .copied()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Eq)]
pub enum SituacaoOS {
    Orcamento,
    Aprovado,
    EmAndamento,
    AutorizadoAguardandoPeca,
    ServicoConcluido,
    AguardandoAutorizacao,
    AguardandoRetirada,
    Reprovado,
    AguardandoFaturar,
    Faturado,
}

impl SituacaoOS {
    pub fn iter() -> impl Iterator<Item = Self> {
        [
            SituacaoOS::Orcamento,
            SituacaoOS::Aprovado,
            SituacaoOS::EmAndamento,
            SituacaoOS::AutorizadoAguardandoPeca,
            SituacaoOS::ServicoConcluido,
            SituacaoOS::AguardandoAutorizacao,
            SituacaoOS::AguardandoRetirada,
            SituacaoOS::Reprovado,
            SituacaoOS::AguardandoFaturar,
            SituacaoOS::Faturado,
        ]
        .iter()
        .cloned()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricoEdicao {
    pub usuario: String,
    pub data_hora: String,
    pub campo_alterado: String,
    pub valor_antigo: String,
    pub valor_novo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Fornecedor {
    pub id: u32,
    pub nome: String,
    pub cnpj: String,
    pub contato: String,
    pub telefone: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Cliente {
    pub id: u32,
    pub nome: String,
    pub email: String,
    pub telefone: String,
    pub endereco: Option<String>,
    pub inscricao_estadual: Option<String>,
    pub cpf_cnpj: Option<String>,
    pub credito_disponivel: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Peca {
    pub id: u32,
    pub nome: String,
    pub codigo_interno: String,
    pub part_number: String,
    pub descricao: String,
    pub fabricante: String,
    pub localizacao: String,
    pub estoque_atual: i32,
    pub estoque_minimo: i32,
    pub preco_custo: f64,
    pub preco_venda: f64,
}

// Servico básico mantido em memória (sem persistência no banco por enquanto)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Servico {
    pub id: u32,
    pub nome: String,
    pub descricao: String,
    pub preco: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicoOS {
    pub id_servico: u32,
    pub nome: String,
    pub descricao: String,
    pub quantidade: u32,
    pub preco_unitario: f64,
    pub preco_total: f64,
}

// Persistência de serviços: delega ao módulo de banco_de_dados
// (P2.6.2a) tenant_id obrigatório.
pub fn listar_servicos_db(tenant_id: i32) -> Result<Vec<Servico>, ErroAplicacao> {
    crate::banco_de_dados::servico::listar_servicos(tenant_id)
}

pub fn listar_servicos(tenant_id: i32) -> Vec<Servico> {
    match listar_servicos_db(tenant_id) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Erro ao listar serviços do banco: {}", e);
            Vec::new()
        }
    }
}

pub fn criar_servico_db(tenant_id: i32, s: &Servico) -> Result<u32, ErroAplicacao> {
    crate::banco_de_dados::servico::criar_servico(tenant_id, s)
}

pub fn criar_servico(tenant_id: i32, s: &Servico) -> u32 {
    match criar_servico_db(tenant_id, s) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("Erro ao criar serviço no banco: {}", e);
            0
        }
    }
}

pub fn listar_pecas(tenant_id: i32) -> Result<Vec<Peca>, ErroAplicacao> {
    crate::banco_de_dados::estoque::listar_pecas(tenant_id)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PecaOS {
    pub id_peca: u32,
    pub codigo_interno: String,
    pub descricao: String,
    pub quantidade: u32,
    pub preco_venda_unitario: f64,
    pub preco_total: f64,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrdemServico {
    pub id: u32,
    pub cliente: String,
    pub equipamento: String,
    pub defeito_relatado: String,
    pub status: StatusOS,
    pub parecer_tecnico: String,
    pub situacao: SituacaoOS,
    pub numero_serie_equipamento: String,
    pub observacoes: String,
    pub nome_tecnico_responsavel: String,
    pub atendente: String,
    pub horario_abertura: String,
    pub telefone_cliente: String,
    pub data_chegada: String,
    pub prazo_entrega: String,
    pub historico_edicoes: Vec<HistoricoEdicao>,
    pub pecas: Vec<PecaOS>,
    pub total_pecas: f64,
    pub servicos: Vec<ServicoOS>,
    pub total_servicos: f64,
}

impl OrdemServico {
    /// Cria uma OrdemServico placeholder enquanto o carregamento do servidor não retorna
    pub fn placeholder(id: u32) -> Self {
        OrdemServico {
            id,
            cliente: String::new(),
            equipamento: String::new(),
            defeito_relatado: String::new(),
            status: StatusOS::Aberta,
            parecer_tecnico: String::new(),
            situacao: SituacaoOS::Orcamento,
            numero_serie_equipamento: String::new(),
            observacoes: String::new(),
            nome_tecnico_responsavel: String::new(),
            atendente: String::new(),
            horario_abertura: String::new(),
            telefone_cliente: String::new(),
            data_chegada: String::new(),
            prazo_entrega: String::new(),
            historico_edicoes: Vec::new(),
            pecas: Vec::new(),
            total_pecas: 0.0,
            servicos: Vec::new(),
            total_servicos: 0.0,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq, Serialize, Deserialize)]
pub enum PapelUsuario {
    Administrador,
    Gerencia,
    Tecnico,
    Financeiro,
    Comercial,
    Estoquista,
}

impl PapelUsuario {
    pub fn iter() -> impl Iterator<Item = &'static Self> {
        static PAPEIS: &[PapelUsuario] = &[
            PapelUsuario::Administrador,
            PapelUsuario::Gerencia,
            PapelUsuario::Tecnico,
            PapelUsuario::Financeiro,
            PapelUsuario::Comercial,
            PapelUsuario::Estoquista,
        ];
        PAPEIS.iter()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfoUsuario {
    pub id: i32,
    pub nome_usuario: String,
}

// --- [CORREÇÃO APLICADA AQUI] ---
// Adicionamos os tipos de erro que estavam faltando para a compilação.
#[derive(Error, Debug, Clone)]
pub enum ErroAplicacao {
    #[error("Não foi possível conectar ao banco de dados.")]
    BancoDeDadosConexao,
    #[error("Erro na consulta ao banco de dados: {0}")]
    BancoDeDadosQuery(String),
    #[error("Erro ao converter dados do banco: {0}")]
    Conversao(String),
    #[error("Erro de serialização de dados: {0}")]
    Serializacao(String),
    #[error("Usuário não encontrado.")]
    UsuarioNaoEncontrado,
    #[error("A senha fornecida é inválida.")]
    SenhaInvalida,
    #[error("Este nome de usuário já está em uso.")]
    UsuarioJaExiste,
    #[error("O usuário 'admin' não pode ser removido.")]
    NaoPodeRemoverAdmin,
    #[error("Falha ao gerar o hash da senha: {0}")]
    FalhaNoHash(String),
    #[error("OS não encontrada.")]
    OsNaoEncontrada,
    #[error("Erro desconhecido: {0}")]
    Desconhecido(String),
}

// Conversão para erros genéricos do MySQL
impl From<mysql::Error> for ErroAplicacao {
    fn from(err: mysql::Error) -> Self {
        ErroAplicacao::BancoDeDadosQuery(err.to_string())
    }
}

// Conversão para erros de conversão de linha (FromRow)
impl From<FromRowError> for ErroAplicacao {
    fn from(err: FromRowError) -> Self {
        ErroAplicacao::Conversao(err.to_string())
    }
}

// Conversão para erros de serialização do serde_json
impl From<serde_json::Error> for ErroAplicacao {
    fn from(err: serde_json::Error) -> Self {
        ErroAplicacao::Serializacao(err.to_string())
    }
}

// Funções de fachada (existentes)
pub fn inicializar() {
    crate::banco_de_dados::inicializar();
}
pub fn verificar_login(
    u: &str,
    s: &str,
) -> Result<crate::banco_de_dados::LoginResult, ErroAplicacao> {
    crate::banco_de_dados::verificar_senha_e_obter_papel(u, s)
}
pub fn criar_usuario(u: &str, s: &str, p: PapelUsuario) -> Result<(), ErroAplicacao> {
    crate::banco_de_dados::criar_usuario(u, s, p)
}
pub fn listar_usuarios() -> Vec<InfoUsuario> {
    crate::banco_de_dados::listar_todos_usuarios()
}
pub fn remover_usuario(u: &str) -> Result<(), ErroAplicacao> {
    crate::banco_de_dados::remover_usuario(u)
}
pub fn listar_ordens_servico(tenant_id: i32) -> Result<Vec<OrdemServico>, ErroAplicacao> {
    crate::banco_de_dados::listar_ordens_servico(tenant_id)
}
pub fn buscar_os_por_id(tenant_id: i32, id: u32) -> Result<OrdemServico, ErroAplicacao> {
    crate::banco_de_dados::buscar_os_por_id(tenant_id, id)
}
pub fn atualizar_os(
    tenant_id: i32,
    os: &OrdemServico,
    usuario_logado: &str,
) -> Result<(), ErroAplicacao> {
    crate::banco_de_dados::atualizar_os(tenant_id, os, usuario_logado)
}

pub fn criar_ordem_servico(tenant_id: i32, os: &mut OrdemServico) -> Result<u32, ErroAplicacao> {
    crate::banco_de_dados::ordem_servico::criar_os(tenant_id, os)
}

/// DTO leve para criar uma OS a partir do handler HTTP.
/// Usa IDs (FKs) em vez do OrdemServico inteiro (21 campos).
pub struct CriarOrdemDTO {
    pub cliente_id: u32,
    pub equipamento_id: u32,
    pub defeito_relatado: String,
    pub observacoes: Option<String>,
    pub parecer_tecnico: Option<String>,
    pub atendente: Option<String>,
    pub prazo_entrega: Option<String>,
    pub situacao: Option<SituacaoOS>,
}

pub fn criar_ordem_servico_dto(
    tenant_id: i32,
    dto: &CriarOrdemDTO,
) -> Result<u32, ErroAplicacao> {
    // Busca nome do cliente e telefone para preencher OrdemServico
    let cliente = crate::banco_de_dados::obter_cliente_por_id(tenant_id, dto.cliente_id)
        .map_err(|_| ErroAplicacao::BancoDeDadosQuery("cliente nao encontrado".into()))?;
    // Resolve nome do equipamento (exec_first retorna None se vazio)
    let equip_descricao: String = {
        let mut conn = crate::banco_de_dados::conexao::obter_conexao()?;
        let row: Option<(String, Option<String>)> = conn.exec_first(
            "SELECT descricao, numero_serie FROM equipamentos WHERE id = ? AND tenant_id = ?",
            (dto.equipamento_id, tenant_id),
        )?;
        row.ok_or_else(|| {
            ErroAplicacao::BancoDeDadosQuery("equipamento nao encontrado".into())
        })?
        .0
    };
    // Monta OrdemServico
    let now = chrono::Local::now().format("%d/%m/%Y %H:%M").to_string();
    // Converte prazo_entrega (DD/MM/YYYY) para ISO se nao for vazio
    let prazo_iso: Option<String> = match dto.prazo_entrega.as_deref() {
        Some(s) if !s.trim().is_empty() => {
            // Tenta parsear DD/MM/YYYY (com ou sem hora)
            let formats = ["%d/%m/%Y", "%Y-%m-%d", "%d-%m-%Y"];
            let mut parsed: Option<chrono::NaiveDate> = None;
            for fmt in formats {
                if let Ok(d) = chrono::NaiveDate::parse_from_str(s.trim(), fmt) {
                    parsed = Some(d);
                    break;
                }
            }
            parsed.map(|d| d.format("%Y-%m-%d").to_string())
        }
        _ => None,
    };
    let mut os = OrdemServico {
        id: 0,
        cliente: cliente.nome.clone(),
        equipamento: equip_descricao,
        defeito_relatado: dto.defeito_relatado.clone(),
        status: StatusOS::Aberta,
        parecer_tecnico: dto.parecer_tecnico.clone().unwrap_or_default(),
        situacao: dto.situacao.clone().unwrap_or(SituacaoOS::Orcamento),
        numero_serie_equipamento: String::new(),
        observacoes: dto.observacoes.clone().unwrap_or_default(),
        nome_tecnico_responsavel: String::new(),
        atendente: dto.atendente.clone().unwrap_or_default(),
        horario_abertura: now.clone(),
        telefone_cliente: cliente.telefone.clone(),
        data_chegada: now,
        prazo_entrega: prazo_iso.unwrap_or_default(),
        historico_edicoes: vec![],
        pecas: vec![],
        total_pecas: 0.0,
        servicos: vec![],
        total_servicos: 0.0,
    };
    crate::banco_de_dados::ordem_servico::criar_os(tenant_id, &mut os)
}

// Orçamentos: facades para criar/obter orçamentos (P2.6.2a)
pub fn criar_orcamento(tenant_id: i32, o: &Orcamento) -> Result<u32, ErroAplicacao> {
    crate::banco_de_dados::criar_orcamento(tenant_id, o)
}

pub fn obter_orcamento(tenant_id: i32, id: u32) -> Result<Orcamento, ErroAplicacao> {
    crate::banco_de_dados::obter_orcamento(tenant_id, id)
}

// --- Helper de tenant padrão (P2.6.2a) ---
// Em produção, extrair de `Claims::tenant_do_usuario()`.
// Para GUI/desktop single-user atual, mantém compat com tenant `1` (legacy).
pub const TENANT_LEGACY: i32 = 1;

pub fn tenant_padrao() -> i32 {
    TENANT_LEGACY
}

// --- Clientes: fachada para chamadas ao banco (P2.6.2a — tenant_id obrigatório) ---
pub fn listar_clientes(tenant_id: i32) -> Result<Vec<Cliente>, ErroAplicacao> {
    crate::banco_de_dados::listar_clientes(tenant_id)
}

pub fn obter_gastos_e_credito(
    tenant_id: i32,
    cliente_id: u32,
) -> Result<(f64, f64), ErroAplicacao> {
    let gastos = crate::banco_de_dados::obter_gastos_por_cliente(tenant_id, cliente_id)?;
    let credito = crate::banco_de_dados::obter_credito_cliente(tenant_id, cliente_id)?;
    Ok((gastos, credito))
}

pub fn criar_ou_atualizar_cliente(tenant_id: i32, c: &Cliente) -> Result<u32, ErroAplicacao> {
    crate::banco_de_dados::criar_ou_atualizar_cliente(tenant_id, c)
}
