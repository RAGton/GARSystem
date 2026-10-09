// src/bootstrap.rs
//
// Bootstrap seguro de empresa + admin (Sprint P2.6.1).
//
// ## Contexto
//
// Em P0, o admin era criado com `admin/admin` (problema de segurança).
// Em P2.6.1, **PROIBIDO** criar admin/admin. O admin é gerado em runtime
// com senha aleatória (ou usa `GAR_BOOTSTRAP_PASSWORD` se setada).
//
// ## Fluxo
//
// 1. Na inicialização do servidor, após migrations:
//    a) Garante que existe pelo menos 1 empresa (id=1, "Empresa Padrão").
//    b) Garante que existe o user `admin@local` (NÃO `admin`!) com role ADMIN
//       e senha aleatória/forte (32 chars alfanuméricos).
//    c) Imprime a senha UMA VEZ via `tracing::warn!`.
//
// 2. Se a empresa 1 não existe → cria.
// 3. Se admin@local já existe com bootstrap_done → pula.
// 4. Se admin@local não existe OU empresa_configuracao.bootstrap_done = FALSE
//    → executa bootstrap_seguro().

use crate::servicos::ErroAplicacao;

/// Executa o bootstrap seguro. Chamado na inicialização do servidor.
///
/// **Idempotente**: se já foi feito, não repete.
pub fn garantir_bootstrap() {
    if let Err(e) = executar_bootstrap() {
        tracing::error!("❌ Falha no bootstrap seguro: {}", e);
    }
}

fn executar_bootstrap() -> Result<(), ErroAplicacao> {
    use gar_system::empresa;

    // 1) Verificar se empresa 1 existe
    if !empresa::repository::empresa_existe()? {
        tracing::info!("🌱 Bootstrap: nenhuma empresa encontrada. Criando empresa 1 (Padrão).");
        let ctx = empresa::models::ContextoEmpresa::system();
        let id = empresa::service::criar_empresa(
            "Empresa Padrão",
            "Empresa Padrão LTDA",
            Some("00.000.000/0001-00"),
            Some("admin@local"),
            None,
            empresa::models::Plano::Business,
            &ctx,
        )?;
        tracing::info!("✅ Empresa 1 criada (id={}).", id);
    }

    // 2) Verificar se bootstrap já foi feito
    if let Some(cfg) = empresa::repository::obter_configuracao(1)? {
        if cfg.bootstrap_done {
            tracing::info!("✅ Bootstrap já realizado para empresa 1. Pulando.");
            return Ok(());
        }
    }

    // 3) Bootstrap seguro
    tracing::info!("🌱 Bootstrap: executando geração segura de admin para empresa 1.");
    let _ = empresa::service::bootstrap_seguro(1)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    // Testes completos requerem MySQL. Apenas sanity de tipos.

    #[test]
    fn garantir_bootstrap_nao_panica() {
        // Em sandbox sem MySQL, o `empresa_existe` falha e a função
        // loga o erro mas não panica — comportamento desejado.
        super::garantir_bootstrap();
    }
}
