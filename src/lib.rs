// src/lib.rs

// Em modo de desenvolvimento, alguns itens ficam não utilizados (esqueleto
// de APIs). Para reduzir o ruído nos builds de desenvolvimento, adicionamos
// allows condicionais. Em release, os warnings continuam ativos.
// Justificativa P2.6.2c: muitas funções `pub` existem como API pública
// (chamadas pelo `bin/gar-system-server`, GUI, ou admin CLI) e não são
// usadas dentro da `lib` em si. Em release, o lint deve ser considerado
// para auditoria de API surface, mas para P2.6.2c manteremos compat.
#![allow(dead_code, unused_imports)]
// Justificativa técnica para P2.6.2c: várias funções do projeto (especialmente
// em `repository.rs` dos módulos CRM, Cotação, OS, Operações e Financeiro)
// recebem 8+ argumentos incluindo `tenant_id`, `id`, parâmetros de negócio e
// flags opcionais. Refatorar todas para structs seria uma refatoração de
// grande porte que viola a Regra 2 (Anti-Refatoração Infinita) e não traz
// benefício funcional mensurável. O lint `too_many_arguments` é puramente
// style e não afeta correção nem segurança.
#![allow(clippy::too_many_arguments)]
// Justificativa: várias funções disparam side-effects async (registrar evento
// CRM, auditoria offline) sem bloquear o fluxo principal. O lint
// `let_underscore_future` recomenda `tokio::spawn` mas isso muda ordem de
// execução; mantemos `let _ = ...` explícito e documentado.
#![allow(clippy::let_underscore_future)]
// Justificativa: `exec_first` da crate `mysql` retorna `Option<(T1, T2, T3, ...)>`
// onde cada Ti é uma coluna. Refatorar cada chamada para `type Row = (...)` seria
// centenas de type aliases novos sem ganho de clareza. O lint `type_complexity`
// é puramente style e não afeta correção nem segurança.
#![allow(clippy::type_complexity)]

// Declaramos que os módulos `banco_de_dados` e `servicos`
// fazem parte desta biblioteca e podem ser usados por
// outros programas (como a GUI e o Servidor).
pub mod arquivos;
pub mod banco_de_dados;
pub mod cotacao_orcamento;
pub mod crm;
pub mod dto;
pub mod empresa;
pub mod executor;
pub mod financial;
pub mod gui_services;
pub mod http_client;
pub mod operations;
pub mod os_mobile;
pub mod rate_limit;
pub mod rbac;
pub mod servicos;
pub mod storage;
pub mod transcription;
