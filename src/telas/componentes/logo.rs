// src/telas/componentes/logo.rs
// Carrega e renderiza a logo GAR System (assets/logo.png) com transparência.
// Substitui o render anterior que aparecia com fundo branco.
//
// Prática egui (Emil Ernerfeldt): `let-else` em vez de unwrap/match aninhado,
// fallback gracioso sem panic.

use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions};
use std::sync::OnceLock;

static LOGO_TEXTURE: OnceLock<Result<TextureHandle, String>> = OnceLock::new();

/// Carrega a logo PNG do disco (uma vez) e retorna a texture do egui.
///
/// Usa `OnceLock` para garantir leitura única (I/O no disco). Em caso
/// de erro (arquivo não existe, PNG inválido), cacheia o `Err` para
/// evitar re-tentativas custosas.
fn obter_texture(ctx: &egui::Context) -> Result<TextureHandle, String> {
    LOGO_TEXTURE
        .get_or_init(|| {
            let caminho = std::path::Path::new("./assets/logo.png");
            let img = image::ImageReader::open(caminho)
                .map_err(|e| format!("abrir logo.png: {e}"))?
                .decode()
                .map_err(|e| format!("decodificar logo.png: {e}"))?;
            let tamanho = [img.width() as _, img.height() as _];
            let rgba = img.to_rgba8();
            let color = ColorImage::from_rgba_unmultiplied(tamanho, rgba.as_flat_samples().as_slice());
            Ok(ctx.load_texture("gar_logo", color, TextureOptions::LINEAR))
        })
        .clone()
}

/// Renderiza a logo GAR com tamanho máximo (largura, altura em px).
///
/// Preserva aspect ratio. Fundo transparente (PNG RGBA).
///
/// # Panics
/// Não panica. Em caso de erro de carregamento, mostra
/// `[logo: <motivo>]` em cinza no lugar (fallback visual).
pub fn mostrar(ui: &mut egui::Ui, ctx: &egui::Context, largura_max: f32, altura_max: f32) {
    let Ok(tex) = obter_texture(ctx) else {
        // Fallback gracioso — não crasha a GUI se a logo não carregar
        ui.colored_label(
            super::super::theme::cores::TEXT_SECONDARY,
            "[logo indisponível]",
        );
        return;
    };

    let size = tex.size_vec2();
    let ratio = size.x / size.y;
    let (w, h) = if size.x / largura_max > size.y / altura_max {
        (largura_max, largura_max / ratio)
    } else {
        (altura_max * ratio, altura_max)
    };
    ui.add(
        egui::Image::new(&tex)
            .fit_to_exact_size(egui::vec2(w, h))
            .maintain_aspect_ratio(true),
    );
}

