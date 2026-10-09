use eframe::egui::{self, RichText, Ui, Color32, Layout, Align};
use crate::telas::theme;

pub fn cabecalho(
    ui: &mut Ui,
    titulo: &str,
    descricao: &str,
    texto_botao_primario: Option<&str>,
) -> bool {
    let mut clicou_primario = false;
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.heading(titulo);
            ui.label(RichText::new(descricao).color(ui.visuals().weak_text_color()));
        });
        
        if let Some(texto) = texto_botao_primario {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.add(
                    egui::Button::new(RichText::new(texto).strong().color(Color32::WHITE))
                        .fill(theme::PRIMARY_PRESSED)
                        .min_size(egui::vec2(130.0, 36.0))
                ).clicked() {
                    clicou_primario = true;
                }
            });
        }
    });
    ui.add_space(10.0);
    ui.separator();
    ui.add_space(8.0);
    
    clicou_primario
}

pub fn estado_vazio(
    ui: &mut Ui,
    icone: &str,
    titulo: &str,
    descricao: &str,
    texto_botao: Option<&str>,
) -> bool {
    let mut clicou = false;
    ui.add_space(18.0);
    ui.vertical_centered(|ui| {
        ui.label(RichText::new(icone).size(34.0).color(theme::PRIMARY));
        ui.add_space(6.0);
        ui.heading(titulo);
        ui.label(descricao);
        if let Some(texto) = texto_botao {
            ui.add_space(8.0);
            if ui.add(egui::Button::new(texto).fill(theme::PRIMARY_PRESSED)).clicked() {
                clicou = true;
            }
        }
    });
    ui.add_space(18.0);
    clicou
}
