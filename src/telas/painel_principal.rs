// src/telas/painel_principal.rs

use crate::aplicacao::{AppEvent, TelaAtiva}; // IMPORTA O NOVO ENUM
use crate::servicos::PapelUsuario;
use eframe::egui;

// O enum `AcaoDashboard` pode ser removido.

// ... (enum AlvoNavegacao e struct TelaDashboard permanecem iguais) ...
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

    // ATUALIZADO: A assinatura da função agora retorna Option<AppEvent>
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
                    if self.papel_usuario == PapelUsuario::Administrador {
                        if botao_menu(ui, "👤", "Gerenciar Usuários").clicked() {
                            // ATUALIZADO: Emite um evento de navegação
                            evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Admin));
                        }
                    }
                    if matches!(
                        self.papel_usuario,
                        PapelUsuario::Administrador | PapelUsuario::Tecnico
                    ) {
                        if botao_menu(ui, "🔧", "Ordens de Serviço").clicked() {
                            evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Tecnico));
                        }
                    }
                    if matches!(
                        self.papel_usuario,
                        PapelUsuario::Administrador | PapelUsuario::Comercial
                    ) {
                        if botao_menu(ui, "🛒", "Comercial").clicked() {
                            evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Comercial));
                        }
                    }
                    if matches!(
                        self.papel_usuario,
                        PapelUsuario::Administrador | PapelUsuario::Financeiro
                    ) {
                        if botao_menu(ui, "💳", "Financeiro").clicked() {
                            evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Financeiro));
                        }
                    }
                    if matches!(
                        self.papel_usuario,
                        PapelUsuario::Administrador | PapelUsuario::Gerencia
                    ) {
                        if botao_menu(ui, "📈", "Gerência").clicked() {
                            evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Gerencia));
                        }
                    }
                });
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
