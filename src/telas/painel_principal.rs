// src/telas/painel_principal.rs

// 1. REMOVA a definição local de `AppEvent`
// 2. IMPORTE o `AppEvent` central
use crate::aplicacao::AppEvent;
use crate::servicos::PapelUsuario;
use eframe::egui;

pub struct TelaDashboard {
    papel: PapelUsuario,
}

impl TelaDashboard {
    pub fn new(papel: PapelUsuario) -> Self {
        Self { papel }
    }

    // 3. O tipo de retorno de `update` agora é `Option<AppEvent>`
    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Option<AppEvent> {
        let mut evento_emitido: Option<AppEvent> = None;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Dashboard Principal");
            ui.label(format!(
                "Bem-vindo! Você está logado como: {:?}",
                self.papel
            ));

            // Exemplo de como emitir um evento
            if ui.button("Abrir OS de teste (ID: 1)").clicked() {
                evento_emitido = Some(AppEvent::AbrirEditorOS(1));
            }
        });

        evento_emitido
    }
}
