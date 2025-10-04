// src/telas/painel_financeiro.rs
use crate::aplicacao::AppEvent; // Importar AppEvent
use eframe::egui;

// pub enum AcaoFinanceiro { Nenhuma } // Remover

pub struct TelaFinanceiro;

impl TelaFinanceiro {
    pub fn new() -> Self {
        Self {}
    }
    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Option<AppEvent> {
        egui::CentralPanel::default()
            .show(ctx, |ui| {
                ui.heading("Painel Financeiro");
                ui.label("Conteúdo do painel financeiro (placeholder).");
                if ui.button("Voltar").clicked() {
                    return Some(AppEvent::VoltarParaDashboard);
                }
                None
            })
            .inner
    }
}
