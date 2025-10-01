// src/servicos.rs
use crate::banco_de_dados;
use std::fmt;

#[derive(Clone, Debug)]
pub struct InfoUsuario {
    pub id: i32,
    pub nome_usuario: String,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum PapelUsuario {
    ADM,
    Tecnico,
    Financeiro,
    Vendedor,
    Gerente,
    Atendente,
}

impl PapelUsuario {
    pub fn todos() -> &'static [PapelUsuario] {
        &[
            Self::ADM,
            Self::Tecnico,
            Self::Financeiro,
            Self::Vendedor,
            Self::Gerente,
            Self::Atendente,
        ]
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErroAplicacao {
    BancoDeDadosConexao,
    BancoDeDadosQuery(String),
    UsuarioNaoEncontrado,
    SenhaInvalida,
    UsuarioJaExiste,
    NaoPodeRemoverAdmin,
    Validacao(String),
    Desconhecido(String),
}

impl fmt::Display for ErroAplicacao {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mensagem = match self {
            Self::BancoDeDadosConexao => {
                "[BD-01] Falha de conexão com o banco de dados.".to_string()
            }
            Self::BancoDeDadosQuery(e) => format!("[BD-02] Erro interno no banco de dados: {}", e),
            Self::UsuarioNaoEncontrado | Self::SenhaInvalida => {
                "[AUTH-01] Usuário ou senha inválidos.".to_string()
            }
            Self::UsuarioJaExiste => "[USR-01] Este nome de usuário já está em uso.".to_string(),
            Self::NaoPodeRemoverAdmin => {
                "[USR-02] O usuário 'admin' não pode ser removido.".to_string()
            }
            Self::Validacao(msg) => format!("[VAL-01] {}", msg),
            Self::Desconhecido(msg) => format!("[ERR-99] Ocorreu um erro inesperado: {}", msg),
        };
        write!(f, "{}", mensagem)
    }
}

pub fn inicializar() {
    banco_de_dados::inicializar();
}
pub fn verificar_login(nome_usuario: &str, senha: &str) -> Result<PapelUsuario, ErroAplicacao> {
    banco_de_dados::verificar_senha_e_obter_papel(nome_usuario, senha)
}
pub fn criar_usuario(
    nome_usuario: &str,
    senha: &str,
    papel: PapelUsuario,
) -> Result<(), ErroAplicacao> {
    if nome_usuario.len() < 3 {
        return Err(ErroAplicacao::Validacao(
            "Usuário deve ter 3+ caracteres.".to_string(),
        ));
    }
    if senha.len() < 8 {
        return Err(ErroAplicacao::Validacao(
            "Senha deve ter 8+ caracteres.".to_string(),
        ));
    }
    banco_de_dados::criar_usuario(nome_usuario, senha, papel)
}
pub fn listar_usuarios() -> Vec<InfoUsuario> {
    banco_de_dados::listar_todos_usuarios()
}
pub fn remover_usuario(nome_usuario: &str) -> Result<(), ErroAplicacao> {
    banco_de_dados::remover_usuario(nome_usuario)
}
