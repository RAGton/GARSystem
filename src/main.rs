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
// MVP (2026-10-09): Inter como Proportional (sans principal), JetBrains Mono como Monospace.
fn configurar_fontes(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Inter (sans principal) — 4 pesos embedados
    fonts.font_data.insert(
        "Inter-Regular".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/inter/Inter-400.ttf")).into(),
    );
    fonts.font_data.insert(
        "Inter-Medium".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/inter/Inter-500.ttf")).into(),
    );
    fonts.font_data.insert(
        "Inter-SemiBold".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/inter/Inter-600.ttf")).into(),
    );
    fonts.font_data.insert(
        "Inter-Bold".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/inter/Inter-700.ttf")).into(),
    );

    // JetBrains Mono (mono) — já estava embedado
    fonts.font_data.insert(
        "JetBrainsMono".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf"))
            .into(),
    );

    // Emojis
    fonts.font_data.insert(
        "NotoEmoji".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/NotoColorEmoji-Regular.ttf"))
            .into(),
    );

    // Proportional (texto normal) = Inter Regular como base
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .splice(0..0, [
            "Inter-Regular".to_owned(),
            "Inter-Medium".to_owned(),
            "Inter-SemiBold".to_owned(),
            "Inter-Bold".to_owned(),
        ]);

    // Monospace (valores numéricos, código) = JetBrains Mono
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "JetBrainsMono".to_owned());

    // Emojis no final
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .push("NotoEmoji".to_owned());
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
            .with_title("GAR System - RAG"),
        centered: true,
        ..Default::default()
    };

    println!("Iniciando a interface gráfica (Cliente)...");

    // A lógica de canais (mpsc) foi completamente removida daqui.
    eframe::run_native(
        "GAR System - RAG",
        opcoes_janela,
        Box::new(|cc: &CreationContext| {
            configurar_fontes(&cc.egui_ctx);
            // O tema é aplicado pelo aplicacao.rs (definir_estilo_azul) em cada
            // update. Ver src/aplicacao.rs:84. O palette GAR é configurado lá.

            // Chamamos `AplicativoPrincipal::new()` sem argumentos.
            Ok(Box::new(AplicativoPrincipal::new(cc)))
        }),
    )
}
