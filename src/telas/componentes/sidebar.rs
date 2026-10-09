// src/telas/componentes/sidebar.rs

use crate::aplicacao::{AppEvent, TelaAtiva};
use crate::servicos::PapelUsuario;
use eframe::egui::{self, ColorImage, TextureHandle};
use std::collections::HashSet;

pub fn carregar_logo() -> Result<ColorImage, image::ImageError> {
    let caminho = std::path::Path::new("./assets/logo.png");
    let imagem = image::ImageReader::open(caminho)?.decode()?;
    let tamanho = [imagem.width() as _, imagem.height() as _];
    let buffer_imagem = imagem.to_rgba8();
    let pixels = buffer_imagem.as_flat_samples();
    Ok(ColorImage::from_rgba_unmultiplied(
        tamanho,
        pixels.as_slice(),
    ))
}

fn montar_permissoes(papel_usuario: PapelUsuario) -> HashSet<TelaAtiva> {
    let mut permissoes = HashSet::new();
    use PapelUsuario::*;
    match papel_usuario {
        Administrador => {
            permissoes.extend([
                TelaAtiva::Admin,
                TelaAtiva::Tecnico,
                TelaAtiva::Financeiro,
                TelaAtiva::Orcamentos,
                TelaAtiva::Gerencia,
                TelaAtiva::Estoque,
                TelaAtiva::Clientes,
                TelaAtiva::Ordens,
                TelaAtiva::Servicos,
            ]);
        }
        Tecnico => {
            permissoes.insert(TelaAtiva::Tecnico);
            permissoes.insert(TelaAtiva::Ordens);
        }
        Financeiro => {
            permissoes.insert(TelaAtiva::Financeiro);
            permissoes.insert(TelaAtiva::Clientes);
        }
        Comercial => {
            permissoes.insert(TelaAtiva::Orcamentos);
            permissoes.insert(TelaAtiva::Clientes);
        }
        Gerencia => {
            permissoes.insert(TelaAtiva::Gerencia);
            permissoes.insert(TelaAtiva::Ordens);
        }
        Estoquista => {
            permissoes.insert(TelaAtiva::Estoque);
        }
    }
    permissoes
}


fn item_menu(ui: &mut egui::Ui, icone: &str, texto: &str, ativo: bool) -> bool {
    let padding = egui::vec2(8.0, 6.0);
    let mut bg_color = egui::Color32::TRANSPARENT;
    let text_color = if ativo {
        crate::telas::theme::PRIMARY
    } else {
        crate::telas::theme::TEXT_PRIMARY
    };

    if ativo {
        bg_color = crate::telas::theme::PRIMARY.gamma_multiply(0.15);
    }

    let response = ui.allocate_response(
        egui::vec2(ui.available_width(), 32.0),
        egui::Sense::click(),
    );

    if response.hovered() && !ativo {
        bg_color = crate::telas::theme::SURFACE_ELEV;
    }

    if bg_color != egui::Color32::TRANSPARENT {
        ui.painter().rect_filled(
            response.rect,
            4.0, // corner radius
            bg_color,
        );
    }
    
    if ativo {
        ui.painter().rect_filled(
            egui::Rect::from_min_size(
                response.rect.min + egui::vec2(0.0, 4.0),
                egui::vec2(3.0, 24.0)
            ),
            1.5,
            crate::telas::theme::PRIMARY,
        );
    }

    let gal = ui.painter().layout_no_wrap(
        format!("{}  {}", icone, texto),
        eframe::egui::FontId::proportional(14.0),
        text_color,
    );
    let text_pos = response.rect.min + egui::vec2(12.0, (32.0 - gal.rect.height()) / 2.0);
    ui.painter().galley(text_pos, gal, text_color);

    response.clicked()
}

pub fn mostrar(

    ctx: &egui::Context,
    papel_usuario: PapelUsuario,
    sidebar_aberto: bool,
    logo: Option<&TextureHandle>,
    tela_ativa: &TelaAtiva,
) -> Option<AppEvent> {
    let mut evento_emitido: Option<AppEvent> = None;
    let permissoes = montar_permissoes(papel_usuario);

    egui::SidePanel::left("sidebar")
        .resizable(true)
        .default_width(220.0)
        .frame(
            egui::Frame::default()
                .fill(crate::telas::theme::cores::BG)
                .stroke(egui::Stroke::new(1.0, crate::telas::theme::cores::BORDER))
                .inner_margin(egui::Margin::same(8)),
        )
        .show_animated(ctx, sidebar_aberto, |ui| {
            // --- SEÇÃO SUPERIOR: LOGO E VERSÃO NO FINAL ---
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.add_space(12.0);
                if let Some(logo_texture) = logo {
                    ui.add(egui::Image::new(logo_texture).max_width(180.0));
                }
                ui.add_space(12.0);
            });

            ui.separator();

            // --- [NOVO] INÍCIO DA ÁREA DE ROLAGEM ---
            // A barra de rolagem aparecerá automaticamente se o conteúdo abaixo for muito grande.
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(8.0);
                
                // --- GERAL ---
                ui.label(egui::RichText::new("GERAL").small().strong().color(crate::telas::theme::TEXT_MUTED));
                ui.add_space(4.0);
                if matches!(papel_usuario, PapelUsuario::Administrador | PapelUsuario::Gerencia | PapelUsuario::Comercial) {
                    if item_menu(ui, "📊", "Dashboard", *tela_ativa == TelaAtiva::Dashboard) {
                        evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Dashboard));
                    }
                }
                ui.add_space(12.0);

                // --- OPERAÇÃO ---
                ui.label(egui::RichText::new("OPERAÇÃO").small().strong().color(crate::telas::theme::TEXT_MUTED));
                ui.add_space(4.0);
                if permissoes.contains(&TelaAtiva::Admin) || permissoes.contains(&TelaAtiva::Orcamentos) || permissoes.contains(&TelaAtiva::Tecnico) {
                    if item_menu(ui, "📝", "Ordens de Serviço", *tela_ativa == TelaAtiva::Ordens) {
                        evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Ordens));
                    }
                }
                if item_menu(ui, "👥", "Clientes", *tela_ativa == TelaAtiva::Clientes) {
                    evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Clientes));
                }
                if permissoes.contains(&TelaAtiva::Estoque) {
                    if item_menu(ui, "📦", "Estoque", *tela_ativa == TelaAtiva::Estoque) {
                        evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Estoque));
                    }
                }
                if permissoes.contains(&TelaAtiva::Admin) {
                    if item_menu(ui, "🛠️", "Serviços", *tela_ativa == TelaAtiva::Servicos) {
                        evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Servicos));
                    }
                }
                ui.add_space(12.0);

                // --- COMERCIAL & FINANCEIRO ---
                ui.label(egui::RichText::new("COMERCIAL & FINANCEIRO").small().strong().color(crate::telas::theme::TEXT_MUTED));
                ui.add_space(4.0);
                if permissoes.contains(&TelaAtiva::Orcamentos) {
                    if item_menu(ui, "📈", "Orçamentos", *tela_ativa == TelaAtiva::Orcamentos) {
                        evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Orcamentos));
                    }
                }
                if permissoes.contains(&TelaAtiva::Financeiro) {
                    if item_menu(ui, "💰", "Financeiro", *tela_ativa == TelaAtiva::Financeiro) {
                        evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Financeiro));
                    }
                }
                ui.add_space(12.0);

                // --- GESTÃO ---
                if permissoes.contains(&TelaAtiva::Admin) || permissoes.contains(&TelaAtiva::Gerencia) {
                    ui.label(egui::RichText::new("GESTÃO").small().strong().color(crate::telas::theme::TEXT_MUTED));
                    ui.add_space(4.0);
                    if permissoes.contains(&TelaAtiva::Admin) {
                        if item_menu(ui, "🛡️", "Administrativo", *tela_ativa == TelaAtiva::Admin) {
                            evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Admin));
                        }
                    }
                    if permissoes.contains(&TelaAtiva::Gerencia) {
                        if item_menu(ui, "📊", "Gerência", *tela_ativa == TelaAtiva::Gerencia) {
                            evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Gerencia));
                        }
                    }
                }
            });            // --- SEÇÃO INFERIOR: VERSÃO (fora da área de rolagem para ficar fixa) ---
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(format!("v {}", env!("CARGO_PKG_VERSION")))
                        .color(crate::telas::theme::cores::TEXT_MUTED)
                        .size(11.0),
                );
            });
        });

    evento_emitido
}
