// src/telas/painel_principal.rs

use crate::aplicacao::AppEvent;
use crate::servicos::PapelUsuario;
use eframe::egui;
use std::sync::{Arc, Mutex};

// O enum `AcaoDashboard` foi removido, pois agora usamos AppEvent.

// Este enum ainda é útil para a sidebar e para a própria tela.
#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub enum AlvoNavegacao {
    Admin,
    Tecnico,
    Financeiro,
    Comercial,
    Gerencia,
    Estoque,
}

pub struct TelaDashboard {
    papel_usuario: PapelUsuario,
}

impl TelaDashboard {
    pub fn new(papel_usuario: PapelUsuario, _endereco_servidor: Arc<Mutex<String>>) -> Self {
        Self { papel_usuario }
    }

    // [CORREÇÃO] A função `update` agora retorna `Option<AppEvent>`.
    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Option<AppEvent> {
        let evento_emitido = None;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Dashboard");
            ui.label("Relatórios e gráficos da empresa.");
            ui.separator();

            // TODO: Adicionar gráficos aqui
            ui.label("Gráficos em breve...");
        });

        evento_emitido
    }
}

// A função `botao_menu` permanece exatamente a mesma.
fn botao_menu(ui: &mut egui::Ui, icone: &str, texto: &str) -> egui::Response {
    let tamanho_botao = egui::vec2(160.0, 100.0);
    let (rect, response) = ui.allocate_exact_size(tamanho_botao, egui::Sense::click());

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let frame = egui::Frame::new()
            .inner_margin(10.0)
            .corner_radius(10.0)
            .fill(visuals.bg_fill)
            .stroke(visuals.bg_stroke);

        frame.show(ui, |ui| {
            ui.allocate_ui_at_rect(rect, |ui| {
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(egui::RichText::new(icone).size(40.0));
                        ui.add_space(5.0);
                        ui.label(egui::RichText::new(texto).strong());
                    });
                });
            });
        });
    }
    response
}
