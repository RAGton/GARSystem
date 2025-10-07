// src/banco_de_dados/mod.rs

// 1. Declara os outros arquivos como submódulos.
//    O compilador do Rust vai procurar por `conexao.rs`, `init.rs`, `usuario.rs`, etc.
pub mod conexao;
pub mod estoque;
pub mod init;
pub mod ordem_servico;
pub mod usuario;

// 2. Re-exporta as funções públicas que serão usadas pelo resto da aplicação.
//    Isso permite que você continue chamando `crate::banco_de_dados::inicializar()`
//    em vez de ter que chamar `crate::banco_de_dados::init::inicializar()`.
pub use init::inicializar;
pub use ordem_servico::{atualizar_os, buscar_os_por_id, listar_ordens_servico};
pub use usuario::{
    criar_usuario, listar_todos_usuarios, remover_usuario, verificar_senha_e_obter_papel,
};
