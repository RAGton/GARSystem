// src/telas/painel_principal.rs

use crate::servicos::PapelUsuario;
use eframe::egui;

pub enum AcaoDashboard {
    Nenhuma,
    Deslogar,
    NavegarPara(AlvoNavegacao),
}

pub enum AlvoNavegacao {
    Admin,
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
                    if self.papel_usuario == PapelUsuario::ADM {
                        if botao_menu(ui, "👤", "Gerenciar Usuários").clicked() {
                            acao = AcaoDashboard::NavegarPara(AlvoNavegacao::Admin);
                        }
                    }
                    if matches!(
                        self.papel_usuario,
                        PapelUsuario::ADM | PapelUsuario::Gerente | PapelUsuario::Vendedor
                    ) {
                        if botao_menu(ui, "📦", "Estoque").clicked() { /* Em breve... */ }
                    }
                    if matches!(
                        self.papel_usuario,
                        PapelUsuario::ADM | PapelUsuario::Gerente | PapelUsuario::Tecnico
                    ) {
                        if botao_menu(ui, "🔧", "Ordens de Serviço").clicked() { /* Em breve... */
                        }
                    }
                    ui.end_row();
                    if botao_menu(ui, "📝", "Requisição").clicked() { /* Em breve... */ }
                    if botao_menu(ui, "➕", "Adicionar Produto").clicked() { /* Em breve... */ }
                    if botao_menu(ui, "📊", "Relatórios").clicked() { /* Em breve... */ }
                });
        });

        acao
    }
}

/// Função auxiliar para desenhar um botão de menu grande e estilizado.
fn botao_menu(ui: &mut egui::Ui, icone: &str, texto: &str) -> egui::Response {
    let tamanho_botao = egui::vec2(160.0, 100.0);

    // CORREÇÃO: Alocamos o retângulo primeiro e obtemos a resposta da interação.
    let (rect, response) = ui.allocate_exact_size(tamanho_botao, egui::Sense::click());

    // Desenha o conteúdo do botão apenas se a área estiver visível.
    if ui.is_rect_visible(rect) {
        // Obtém o estilo visual baseado na interação (hover, clique, etc.).
        let visuals = ui.style().interact(&response);

        // Cria a moldura (fundo) do botão com o estilo correto.
        let frame = egui::Frame::none()
            .inner_margin(egui::style::Margin::same(10.0))
            .fill(visuals.bg_fill) // Usa a cor de fundo do estilo de interação
            .rounding(egui::Rounding::from(10.0))
            .stroke(visuals.bg_stroke); // Usa a borda do estilo de interação

        // Desenha a moldura e o conteúdo dentro do retângulo alocado.
        frame.show(ui, |ui| {
            ui.set_clip_rect(rect);
            ui.allocate_ui_at_rect(rect, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new(icone).size(40.0));
                    ui.add_space(5.0);
                    ui.label(egui::RichText::new(texto).strong());
                });
            });
        });
    }

    // Retorna a resposta da interação para que `.clicked()` funcione.
    response
}
