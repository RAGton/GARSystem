// src/telas/componentes/sidebar.rs

use crate::aplicacao::{AppEvent, TelaAtiva};
use crate::servicos::PapelUsuario;
use crate::telas::painel_principal::AlvoNavegacao;
use eframe::egui::{self, ColorImage, TextureHandle};
use std::collections::HashSet;

// A função para carregar a imagem do arquivo.
fn carregar_imagem_de_arquivo(caminho: &std::path::Path) -> Result<ColorImage, image::ImageError> {
    let imagem = image::io::Reader::open(caminho)?.decode()?;
    let tamanho = [imagem.width() as _, imagem.height() as _];
    let buffer_imagem = imagem.to_rgba8();
    let pixels = buffer_imagem.as_flat_samples();
    Ok(ColorImage::from_rgba_unmultiplied(
        tamanho,
        pixels.as_slice(),
    ))
}

// O enum `AcaoSidebar` foi removido.

/// Desenha a sidebar e retorna um `AppEvent` se o usuário clicar em um botão de navegação.
pub fn mostrar(
    ctx: &egui::Context,
    papel_usuario: PapelUsuario,
    sidebar_aberto: bool,
) -> Option<AppEvent> {
    let mut evento_emitido: Option<AppEvent> = None;

    // A lógica de permissões permanece a mesma.
    let mut permissoes: HashSet<AlvoNavegacao> = HashSet::new();
    use PapelUsuario::*;
    match papel_usuario {
        Administrador => {
            permissoes.insert(AlvoNavegacao::Admin);
            permissoes.insert(AlvoNavegacao::Tecnico);
            permissoes.insert(AlvoNavegacao::Financeiro);
            permissoes.insert(AlvoNavegacao::Comercial);
            permissoes.insert(AlvoNavegacao::Gerencia);
        }
        Gerencia => { /* ... */ }
        Tecnico => {
            permissoes.insert(AlvoNavegacao::Tecnico);
        }
        Financeiro => { /* ... */ }
        Comercial => { /* ... */ }
    }

    egui::SidePanel::left("sidebar")
        .resizable(true)
        .default_width(200.0)
        .width_range(150.0..=300.0)
        .show_animated(ctx, sidebar_aberto, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                // --- [NOVO] LÓGICA PARA CARREGAR E EXIBIR O LOGO ---
                // Esta é uma forma idiomática no egui de carregar uma textura apenas uma vez.
                // Ele usa o armazenamento de dados do próprio contexto para guardar a textura.
                let logo_texture: &TextureHandle = ui.ctx().data_mut(|d| {
                    d.get_persisted(egui::Id::new("logo_texture"))
                        .unwrap_or_else(|| {
                            let imagem = carregar_imagem_de_arquivo(std::path::Path::new(
                                "./assets/logo.png",
                            ))
                            .expect("Não foi possível carregar o logo.");
                            ui.ctx()
                                .load_texture("logo_empresa", imagem, Default::default())
                        })
                });

                ui.add_space(10.0);
                ui.add(egui::Image::new(logo_texture).max_width(180.0));
                ui.add_space(10.0);
                ui.separator();
                // --- FIM DA LÓGICA DO LOGO ---

                ui.collapsing("📊 Relatório de OS", |ui| {
                    ui.label("Em aberto: 5");
                    ui.label("Para faturar: 2");
                    ui.label("A realizar: 8");
                });
                ui.separator();

                ui.label("Navegação");

                // [CORREÇÃO] Os botões agora emitem `AppEvent` diretamente e usam `Button::new`.
                if ui.add(egui::Button::new("🏠 Dashboard")).clicked() {
                    evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Dashboard));
                }

                if permissoes.contains(&AlvoNavegacao::Admin)
                    && ui.add(egui::Button::new("⚙️ Admin")).clicked()
                {
                    evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Admin));
                }

                if permissoes.contains(&AlvoNavegacao::Tecnico)
                    && ui.add(egui::Button::new("🔧 Técnico")).clicked()
                {
                    evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Tecnico));
                }

                if permissoes.contains(&AlvoNavegacao::Financeiro)
                    && ui.add(egui::Button::new("💳 Financeiro")).clicked()
                {
                    evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Financeiro));
                }

                if permissoes.contains(&AlvoNavegacao::Comercial)
                    && ui.add(egui::Button::new("🛒 Comercial")).clicked()
                {
                    evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Comercial));
                }

                if permissoes.contains(&AlvoNavegacao::Gerencia)
                    && ui.add(egui::Button::new("📈 Gerência")).clicked()
                {
                    evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Gerencia));
                }
            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(format!("Versão: {}", env!("CARGO_PKG_VERSION")))
                        .color(egui::Color32::GRAY)
                        .size(12.0),
                );
            });
        });

    evento_emitido
}
