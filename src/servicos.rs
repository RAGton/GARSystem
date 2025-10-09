// src/servicos.rs

use mysql::FromRowError;
use serde::{Deserialize, Serialize};
use thiserror::Error; // Necessário para a conversão de erros

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StatusOS {
    Aberta,
    EmAndamento,
    AguardandoPeca,
    Finalizada,
    Cancelada,
}

// Sua adição foi mantida
impl StatusOS {
    pub fn iter() -> impl Iterator<Item = Self> {
        [
            StatusOS::Aberta,
            StatusOS::EmAndamento,
            StatusOS::AguardandoPeca,
            StatusOS::Finalizada,
            StatusOS::Cancelada,
        ]
        .iter()
        .cloned()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

// Sua adição foi mantida
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
pub fn verificar_login(u: &str, s: &str) -> Result<PapelUsuario, ErroAplicacao> {
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
pub fn listar_ordens_servico() -> Result<Vec<OrdemServico>, ErroAplicacao> {
    crate::banco_de_dados::listar_ordens_servico()
}
pub fn buscar_os_por_id(id: u32) -> Result<OrdemServico, ErroAplicacao> {
    crate::banco_de_dados::buscar_os_por_id(id)
}
pub fn atualizar_os(os: &OrdemServico, usuario_logado: &str) -> Result<(), ErroAplicacao> {
    crate::banco_de_dados::atualizar_os(os, usuario_logado)
}

pub fn criar_ordem_servico(os: &mut OrdemServico) -> Result<u32, ErroAplicacao> {
    crate::banco_de_dados::ordem_servico::criar_os(os)
}

// Orçamentos: facades para criar/obter orçamentos
pub fn criar_orcamento(o: &Orcamento) -> Result<u32, ErroAplicacao> {
    crate::banco_de_dados::criar_orcamento(o)
}

pub fn obter_orcamento(id: u32) -> Result<Orcamento, ErroAplicacao> {
    crate::banco_de_dados::obter_orcamento(id)
}

// --- Clientes: fachada para chamadas ao banco ---
pub fn listar_clientes() -> Result<Vec<Cliente>, ErroAplicacao> {
    crate::banco_de_dados::listar_clientes()
}

pub fn obter_gastos_e_credito(cliente_id: u32) -> Result<(f64, f64), ErroAplicacao> {
    let gastos = crate::banco_de_dados::obter_gastos_por_cliente(cliente_id)?;
    let credito = crate::banco_de_dados::obter_credito_cliente(cliente_id)?;
    Ok((gastos, credito))
}

pub fn criar_ou_atualizar_cliente(c: &Cliente) -> Result<u32, ErroAplicacao> {
    crate::banco_de_dados::criar_ou_atualizar_cliente(c)
}
