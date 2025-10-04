// src/telas/painel_gerencia.rs
use crate::aplicacao::AppEvent; // Importar AppEvent
use eframe::egui;

// pub enum AcaoGerencia { Nenhuma } // Remover

pub struct TelaGerencia {}

impl TelaGerencia {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Option<AppEvent> {
        egui::CentralPanel::default()
            .show(ctx, |ui| {
                ui.heading("Painel de Gerencia");
                ui.label("Conteúdo da tela de gerencia aqui.");
                if ui.button("Voltar").clicked() {
                    return Some(AppEvent::VoltarParaDashboard);
                }
                None
            })
            .inner
    }
}
