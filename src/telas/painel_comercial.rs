// src/telas/painel_comercial.rs
use crate::aplicacao::AppEvent; // Importar AppEvent
use eframe::egui;

// pub enum AcaoComercial { Nenhuma } // Remover

pub struct TelaComercial {}

impl TelaComercial {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Option<AppEvent> {
        egui::CentralPanel::default()
            .show(ctx, |ui| {
                ui.heading("Painel Comercial");
                ui.label("Conteúdo da tela comercial aqui.");
                // Adicionar um botão de voltar como exemplo
                if ui.button("Voltar").clicked() {
                    return Some(AppEvent::VoltarParaDashboard);
                }
                None
            })
            .inner
    }
}
