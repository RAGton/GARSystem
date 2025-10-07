// src/telas/componentes/sidebar.rs

// REMOVA qualquer definição local de `AppEvent`
// IMPORTE o AppEvent central
use crate::aplicacao::{AppEvent, TelaAtiva};
use crate::servicos::PapelUsuario;
use eframe::egui::{self, ColorImage, TextureHandle};
use std::path::Path;

pub fn carregar_logo() -> Result<ColorImage, image::ImageError> {
    let caminho_logo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("logo.png");
    let imagem = image::io::Reader::open(caminho_logo)?.decode()?;
    let tamanho = [imagem.width() as _, imagem.height() as _];
    let pixels_rgba = imagem.to_rgba8().into_raw();
    Ok(ColorImage::from_rgba_unmultiplied(tamanho, &pixels_rgba))
}

// A função `mostrar` agora retorna `Option<AppEvent>`
pub fn mostrar(
    ctx: &egui::Context,
    papel: PapelUsuario,
    aberto: bool,
    logo: Option<&TextureHandle>,
) -> Option<AppEvent> {
    if !aberto {
        return None;
    }

    let mut proximo_evento: Option<AppEvent> = None;

    egui::SidePanel::left("sidebar")
        .default_width(200.0)
        .show(ctx, |ui| {
            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    if let Some(logo_texture) = logo {
                        ui.image(logo_texture);
                    } else {
                        ui.heading("Senior System");
                    }
                },
            );

            ui.separator();

            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
                if ui.button("Dashboard").clicked() {
                    proximo_evento = Some(AppEvent::NavegarPara(TelaAtiva::Dashboard));
                }
                if ui.button("Admin").clicked() {
                    proximo_evento = Some(AppEvent::NavegarPara(TelaAtiva::Admin));
                }
                if ui.button("Técnico").clicked() {
                    proximo_evento = Some(AppEvent::NavegarPara(TelaAtiva::Tecnico));
                }
            });
        });

    proximo_evento
}
