// src/telas/painel_principal.rs

use crate::aplicacao::{AppEvent, TelaAtiva};
use crate::servicos::PapelUsuario;
use eframe::egui;
use std::sync::{Arc, Mutex};

// Este enum ainda é útil para a sidebar e para a própria tela.
#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug)]
pub enum AlvoNavegacao {
    Admin,
    Tecnico,
    Financeiro,
    Comercial,
    Gerencia,
}

pub struct TelaDashboard {
    papel_usuario: PapelUsuario,
}

impl TelaDashboard {
    pub fn new(papel_usuario: PapelUsuario, _endereco_servidor: Arc<Mutex<String>>) -> Self {
        Self { papel_usuario }
    }

    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Option<AppEvent> {
        let mut evento_emitido = None;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Menu Principal");
            ui.label("Selecione uma opção para começar.");
            ui.separator();

            egui::Grid::new("grid_menu")
                .num_columns(3)
                .spacing([40.0, 40.0])
                .striped(false)
                .show(ui, |ui| {
                    // Lógica para exibir botões com base no papel do usuário
                    match self.papel_usuario {
                        PapelUsuario::Administrador => {
                            if botao_menu(ui, "💼", "Administrativo").clicked() {
                                evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Admin));
                            }
                            if botao_menu(ui, "🔧", "Técnico").clicked() {
                                evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Tecnico));
                            }
                            if botao_menu(ui, "💰", "Financeiro").clicked() {
                                evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Financeiro));
                            }
                            if botao_menu(ui, "📈", "Comercial").clicked() {
                                evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Comercial));
                            }
                            if botao_menu(ui, "📊", "Gerência").clicked() {
                                evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Gerencia));
                            }
                        }
                        PapelUsuario::Tecnico => {
                            if botao_menu(ui, "🔧", "Técnico").clicked() {
                                evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Tecnico));
                            }
                        }
                        PapelUsuario::Financeiro => {
                            if botao_menu(ui, "💰", "Financeiro").clicked() {
                                evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Financeiro));
                            }
                        }
                        PapelUsuario::Comercial => {
                            if botao_menu(ui, "📈", "Comercial").clicked() {
                                evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Comercial));
                            }
                        }
                        PapelUsuario::Gerencia => {
                            if botao_menu(ui, "📊", "Gerência").clicked() {
                                evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Gerencia));
                            }
                        }
                    }
                });
        });

        evento_emitido
    }
}

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