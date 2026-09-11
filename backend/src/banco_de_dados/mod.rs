// src/banco_de_dados/mod.rs

// 1. Declara os outros arquivos como submódulos.
pub mod audit;
pub mod cliente;
pub mod conexao;
pub mod estoque;
pub mod init;
pub mod migrations;
pub mod orcamento;
pub mod ordem_servico;
pub mod pagination;
pub mod servico;
pub mod usuario;

// 2. Re-exporta as funções públicas que serão usadas pelo resto da aplicação.
#[allow(unused_imports)]
pub use cliente::{
    criar_ou_atualizar_cliente, listar_clientes, obter_cliente_por_id, obter_credito_cliente,
    obter_gastos_por_cliente,
};
#[allow(unused_imports)]
pub use conexao::obter_conexao;
#[allow(unused_imports)]
pub use init::inicializar;
#[allow(unused_imports)]
pub use orcamento::{criar_orcamento, obter_orcamento};
#[allow(unused_imports)]
pub use ordem_servico::{atualizar_os, buscar_os_por_id, criar_os, listar_ordens_servico};
#[allow(unused_imports)]
pub use servico::{criar_servico as criar_servico_db, listar_servicos as listar_servicos_db};
pub use usuario::{
    criar_usuario, listar_todos_usuarios, remover_usuario, verificar_senha_e_obter_papel,
    LoginResult,
};
