// src/telas/painel_principal.rs

use crate::servicos::PapelUsuario;
use eframe::egui;

pub enum AcaoDashboard {
    Nenhuma,
    NavegarPara(AlvoNavegacao),
}

// Variantes de navegação finais: Admin, Tecnico, Financeiro, Comercial, Gerencia
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
    pub fn new(papel_usuario: PapelUsuario) -> Self {
        Self { papel_usuario }
    }

    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> AcaoDashboard {
        let mut acao = AcaoDashboard::Nenhuma;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Menu Principal");
            ui.label("Selecione uma opção para começar.");
            ui.separator();

            egui::Grid::new("grid_menu")
                .num_columns(3)
                .spacing([40.0, 40.0])
                .striped(false)
                .show(ui, |ui| {
                    if self.papel_usuario == PapelUsuario::Administrador {
                        if botao_menu(ui, "👤", "Gerenciar Usuários").clicked() {
                            acao = AcaoDashboard::NavegarPara(AlvoNavegacao::Admin);
                        }
                    }
                    if matches!(
                        self.papel_usuario,
                        PapelUsuario::Administrador | PapelUsuario::Tecnico
                    ) {
                        if botao_menu(ui, "🔧", "Ordens de Serviço").clicked() {
                            acao = AcaoDashboard::NavegarPara(AlvoNavegacao::Tecnico);
                        }
                    }
                    if matches!(
                        self.papel_usuario,
                        PapelUsuario::Administrador | PapelUsuario::Comercial
                    ) {
                        if botao_menu(ui, "🛒", "Comercial").clicked() {
                            acao = AcaoDashboard::NavegarPara(AlvoNavegacao::Comercial);
                        }
                    }
                    if matches!(
                        self.papel_usuario,
                        PapelUsuario::Administrador | PapelUsuario::Financeiro
                    ) {
                        if botao_menu(ui, "💳", "Financeiro").clicked() {
                            acao = AcaoDashboard::NavegarPara(AlvoNavegacao::Financeiro);
                        }
                    }
                    if matches!(
                        self.papel_usuario,
                        PapelUsuario::Administrador | PapelUsuario::Gerencia
                    ) {
                        if botao_menu(ui, "📈", "Gerência").clicked() {
                            acao = AcaoDashboard::NavegarPara(AlvoNavegacao::Gerencia);
                        }
                    }
                });
        });

        acao
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

        let _ = ui.put(rect, |ui: &mut egui::Ui| {
            frame
                .show(ui, |ui: &mut egui::Ui| {
                    ui.centered_and_justified(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(egui::RichText::new(icone).size(40.0));
                            ui.add_space(5.0);
                            ui.label(egui::RichText::new(texto).strong());
                        });
                    });
                })
                .response
        });
    }
    response
}
