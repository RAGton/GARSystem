// src/telas/painel_atendente.rs
use eframe::egui;

pub enum AcaoAtendente {
    Nenhuma,
}

pub struct TelaAtendente {}

impl TelaAtendente {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> AcaoAtendente {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Painel do Atendente");
            ui.label("Conteúdo da tela do atendente aqui.");
        });
        AcaoAtendente::Nenhuma
    }
}
