// src/telas/painel_financeiro.rs
use eframe::egui;

pub enum AcaoFinanceiro {
    Nenhuma,
}

pub struct TelaFinanceiro;

impl TelaFinanceiro {
    pub fn new() -> Self {
        Self {}
    }
    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> AcaoFinanceiro {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Painel Financeiro");
            ui.label("Conteúdo do painel financeiro (placeholder).");
        });
        AcaoFinanceiro::Nenhuma
    }
}
