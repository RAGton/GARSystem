// src/lib.rs

// Em modo de desenvolvimento, alguns itens ficam não utilizados (esqueleto
// de APIs). Para reduzir o ruído nos builds de desenvolvimento, adicionamos
// allows condicionais. Em release, os warnings continuam ativos.
#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]

// Declaramos que os módulos `banco_de_dados` e `servicos`
// fazem parte desta biblioteca e podem ser usados por
// outros programas (como a GUI e o Servidor).
pub mod banco_de_dados;
pub mod executor;
pub mod http_client;
pub mod rate_limit;
pub mod servicos;
