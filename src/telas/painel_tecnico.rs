// src/telas/painel_tecnico.rs
use eframe::egui;

pub enum AcaoTecnico {
    Nenhuma,
}

pub struct TelaTecnico {}

impl TelaTecnico {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> AcaoTecnico {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Painel do Técnico");
            ui.label("Conteúdo da tela do técnico aqui.");
        });
        AcaoTecnico::Nenhuma
    }
}
