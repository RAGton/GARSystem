// src/servicos.rs

use serde::{Deserialize, Serialize}; // ADICIONE ESTA LINHA
use thiserror::Error;

// --- ESTRUTURAS DE DADOS ADICIONADAS ---

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)] // Adicionado Serialize/Deserialize
pub enum StatusOS {
    Aberta,
    EmAndamento,
    AguardandoPeca,
    Finalizada,
    Cancelada,
}

// Representa uma Ordem de Serviço.
#[derive(Debug, Clone, Serialize, Deserialize)] // Adicionado Serialize/Deserialize
pub struct OrdemServico {
    pub id: u32,
    pub cliente: String,
    pub equipamento: String,
    pub defeito_relatado: String,
    pub status: StatusOS,
}

// --- FIM DAS ESTRUTURAS DE DADOS ADICIONADAS ---

// [CORREÇÃO] Adicionamos os derives de Serialize e Deserialize
#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq, Serialize, Deserialize)]
pub enum PapelUsuario {
    Administrador,
    Gerencia,
    Tecnico,
    Financeiro,
    Comercial,
}

impl PapelUsuario {
    pub fn iter() -> impl Iterator<Item = &'static Self> {
        static PAPEIS: &[PapelUsuario] = &[
            PapelUsuario::Administrador,
            PapelUsuario::Gerencia,
            PapelUsuario::Tecnico,
            PapelUsuario::Financeiro,
            PapelUsuario::Comercial,
        ];
        PAPEIS.iter()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)] // Adicionado Serialize/Deserialize
pub struct InfoUsuario {
    pub id: i32,
    pub nome_usuario: String,
}

#[derive(Error, Debug, Clone)]
pub enum ErroAplicacao {
    #[error("Não foi possível conectar ao banco de dados.")]
    BancoDeDadosConexao,
    #[error("Erro na consulta ao banco de dados: {0}")]
    BancoDeDadosQuery(String),
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
    #[error("Erro desconhecido: {0}")]
    Desconhecido(String),
}

impl From<mysql::Error> for ErroAplicacao {
    fn from(err: mysql::Error) -> Self {
        ErroAplicacao::BancoDeDadosQuery(err.to_string())
    }
}

// Funções de fachada
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
