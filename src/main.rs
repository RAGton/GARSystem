// src/main.rs
// Ponto de entrada para o cliente GUI (Interface Gráfica do Usuário)
//
// Justificativa P2.6.2c: binário GUI, permite dead_code e unused_imports.
#![allow(
    dead_code,
    unused_imports,
    unused_variables,
    unused_mut,
    clippy::collapsible_if,
    clippy::needless_return,
    clippy::assigning_clones,
    clippy::derive_partial_eq_without_eq,
    clippy::len_zero,
    clippy::useless_vec,
    clippy::cast_possible_truncation,
    clippy::field_reassign_with_default,
    float_literal_f32_fallback
)]

mod aplicacao;
mod banco_de_dados;
mod executor;
mod gui_services;
mod http_client;
mod servicos;
mod telas;
use aplicacao::AplicativoPrincipal;
use eframe::{egui, CreationContext};

// Esta função é responsável por carregar e configurar as fontes customizadas.
fn configurar_fontes(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Carrega os dados da fonte principal (JetBrains Mono).
    // A API do egui agora espera um `Arc<FontData>`, e `.into()` faz a conversão.
    fonts.font_data.insert(
        "JetBrainsMono".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf"))
            .into(),
    );

    // Carrega os dados da fonte de emojis.
    fonts.font_data.insert(
        "NotoEmoji".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/NotoColorEmoji-Regular.ttf"))
            .into(),
    );

    // Define a ordem de prioridade para as fontes, garantindo que os símbolos apareçam.
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "JetBrainsMono".to_owned());
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .push("NotoEmoji".to_owned());

    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "JetBrainsMono".to_owned());
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("NotoEmoji".to_owned());

    ctx.set_fonts(fonts);
}

fn main() -> Result<(), eframe::Error> {
    let opcoes_janela = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([850.0, 500.0])
            .with_resizable(true)
            .with_title("Senior System - RAG"),
        centered: true,
        ..Default::default()
    };

    println!("Iniciando a interface gráfica (Cliente)...");

    // A lógica de canais (mpsc) foi completamente removida daqui.
    eframe::run_native(
        "Senior System - RAG",
        opcoes_janela,
        Box::new(|cc: &CreationContext| {
            configurar_fontes(&cc.egui_ctx);

            // Chamamos `AplicativoPrincipal::new()` sem argumentos.
            Ok(Box::new(AplicativoPrincipal::new(cc)))
        }),
    )
}
