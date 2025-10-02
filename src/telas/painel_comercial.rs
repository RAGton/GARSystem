// src/telas/painel_comercial.rs
use eframe::egui;

pub enum AcaoComercial {
    Nenhuma,
}

pub struct TelaComercial {}

impl TelaComercial {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> AcaoComercial {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Painel Comercial");
            ui.label("Conteúdo da tela comercial aqui.");
        });
        AcaoComercial::Nenhuma
    }
}
