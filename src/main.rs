// src/main.rs
// Ponto de entrada para o cliente GUI (Interface Gráfica do Usuário)

mod aplicacao;
mod banco_de_dados;
mod servicos;
mod telas;

use aplicacao::AplicativoPrincipal;
use eframe::{egui, CreationContext};

// Esta função é responsável por carregar e configurar as fontes customizadas.
fn configurar_fontes(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // [CORREÇÃO 1] Corrigido o caminho para a pasta 'fontes' que você está usando.
    // [CORREÇÃO 2] Adicionado `.into()` para a nova API do egui.
    fonts.font_data.insert(
        "JetBrainsMono".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/fontes/JetBrainsMono-Regular.ttf"))
            .into(),
    );

    fonts.font_data.insert(
        "NotoEmoji".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../assets/fontes/NotoColorEmoji-Regular.ttf"
        ))
        .into(),
    );

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
            .with_inner_size([400.0, 500.0])
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

            // [CORREÇÃO 3] Chamamos `AplicativoPrincipal::new()` sem argumentos.
            Ok(Box::new(AplicativoPrincipal::new()))
        }),
    )
}
