// src/banco_de_dados/conexao.rs

use crate::servicos::ErroAplicacao;
use dotenvy::dotenv;
use mysql::{OptsBuilder, Pool, PooledConn};
use once_cell::sync::Lazy;
use std::env;
use std::sync::Mutex;

// A única instância estática do Pool de Conexões fica aqui.
static POOL_DB: Lazy<Mutex<Pool>> = Lazy::new(|| {
    dotenv().ok();
    let url_db =
        env::var("DATABASE_URL").expect("A variável de ambiente DATABASE_URL não foi definida.");
    let opts = OptsBuilder::from_opts(mysql::Opts::from_url(&url_db).expect("URL do DB inválida"));
    let pool = Pool::new(opts).expect("Não foi possível criar o pool de conexões.");
    Mutex::new(pool)
});

// A função que fornece uma conexão para os outros submódulos (`init.rs`, `usuario.rs`).
// `pub(super)` a torna visível apenas para o módulo pai (`banco_de_dados`) e seus irmãos.
pub(super) fn obter_conexao() -> Result<PooledConn, ErroAplicacao> {
    POOL_DB
        .lock()
        .unwrap()
        .get_conn()
        .map_err(|_| ErroAplicacao::BancoDeDadosConexao)
}
