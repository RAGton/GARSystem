// src/telas/painel_gerencia.rs
use eframe::egui;

pub enum AcaoGerencia {
    Nenhuma,
}

pub struct TelaGerencia {}

impl TelaGerencia {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> AcaoGerencia {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Painel de Gerencia");
            ui.label("Conteúdo da tela de gerencia aqui.");
        });
        AcaoGerencia::Nenhuma
    }
}
