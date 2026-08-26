// src/banco_de_dados/conexao.rs

use crate::servicos::ErroAplicacao;
use dotenvy::dotenv;
use mysql::prelude::Queryable;
use mysql::{OptsBuilder, Pool, PooledConn};
use once_cell::sync::Lazy;
use std::env;
use std::sync::Mutex;

// Em vez de criar o Pool no momento da carga do binário (o que causa panic
// se o host não estiver resolvível, por exemplo quando rodamos somente a GUI),
// mantemos um Option e inicializamos o pool sob demanda.
static POOL_DB: Lazy<Mutex<Option<Pool>>> = Lazy::new(|| Mutex::new(None));

/// Inicializa o pool explicitamente. O servidor deve chamar isso durante
/// sua inicialização. Retorna um ErroAplicacao caso não consiga conectar.
pub fn inicializar_pool() -> Result<(), ErroAplicacao> {
    dotenv().ok();
    let url_db = env::var("DATABASE_URL").map_err(|_| ErroAplicacao::BancoDeDadosConexao)?;
    let opts = mysql::Opts::from_url(&url_db).map_err(|_| ErroAplicacao::BancoDeDadosConexao)?;

    // Tentativas com backoff para aguardar o banco ficar pronto.
    let mut tentativas = 0u8;
    let max_tentativas: u8 = 12; // ~12 * 2s = 24s de espera
    loop {
        match Pool::new(OptsBuilder::from_opts(opts.clone())) {
            Ok(p) => {
                *POOL_DB.lock().unwrap() = Some(p);
                println!("Pool do banco inicializado com sucesso.");
                return Ok(());
            }
            Err(err) => {
                tentativas = tentativas.saturating_add(1);
                eprintln!(
                    "Tentativa {}: falha ao criar pool do DB: {}",
                    tentativas, err
                );
                if tentativas >= max_tentativas {
                    return Err(ErroAplicacao::BancoDeDadosConexao);
                }
                std::thread::sleep(std::time::Duration::from_secs(2));
            }
        }
    }
}

// A função que fornece uma conexão para os outros submódulos (`init.rs`, `usuario.rs`).
// Se o pool ainda não foi inicializado, tentamos inicializá-lo de forma preguiçosa
// e retornamos um erro claro em caso de falha, em vez de panic.
pub fn obter_conexao() -> Result<PooledConn, ErroAplicacao> {
    let mut guard = POOL_DB.lock().unwrap();
    if guard.is_none() {
        // tentativa de inicialização preguiçosa
        dotenv().ok();
        let url_db = env::var("DATABASE_URL").map_err(|_| ErroAplicacao::BancoDeDadosConexao)?;
        let opts = OptsBuilder::from_opts(
            mysql::Opts::from_url(&url_db).map_err(|_| ErroAplicacao::BancoDeDadosConexao)?,
        );
        let pool = Pool::new(opts).map_err(|_| ErroAplicacao::BancoDeDadosConexao)?;
        *guard = Some(pool);
    }

    guard
        .as_mut()
        .unwrap()
        .get_conn()
        .map_err(|_| ErroAplicacao::BancoDeDadosConexao)
}

/// Verifica se o banco está respondendo. Usado pelo /readyz.
/// Faz um `SELECT 1` simples para confirmar a conexão de fato funciona.
pub fn ping_banco() -> Result<(), ErroAplicacao> {
    let mut conn = obter_conexao()?;
    let one: Option<i32> = conn.query_first("SELECT 1").map_err(ErroAplicacao::from)?;
    if one == Some(1) {
        Ok(())
    } else {
        Err(ErroAplicacao::BancoDeDadosConexao)
    }
}
