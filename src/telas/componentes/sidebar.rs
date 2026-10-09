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

pub fn mostrar(
    ctx: &egui::Context,
    papel_usuario: PapelUsuario,
    sidebar_aberto: bool,
    logo: Option<&TextureHandle>,
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
                // --- GRUPO DE NAVEGAÇÃO PRINCIPAL ---
                ui.collapsing("🏠 Navegação Principal", |ui| {
                    // Dashboard
                    if matches!(
                        papel_usuario,
                        PapelUsuario::Administrador
                            | PapelUsuario::Gerencia
                            | PapelUsuario::Comercial
                    ) {
                        if ui.button("📊 Dashboard").clicked() {
                            evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Dashboard));
                        }
                    }

                    // Ordens de Serviço
                    if permissoes.contains(&TelaAtiva::Admin)
                        || permissoes.contains(&TelaAtiva::Orcamentos)
                        || permissoes.contains(&TelaAtiva::Tecnico)
                    {
                        if ui.button("📝 Ordens de Serviço").clicked() {
                            evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Ordens));
                        }
                    }

                    // Clientes
                    if ui.button("👥 Clientes").clicked() {
                        evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Clientes));
                    }
                });

                // --- GRUPO DE MÓDULOS DE GESTÃO ---
                ui.collapsing("💼 Módulos de Gestão", |ui| {
                    if permissoes.contains(&TelaAtiva::Admin)
                        && ui.button("🛡️ Administrativo").clicked()
                    {
                        evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Admin));
                    }
                    if permissoes.contains(&TelaAtiva::Admin) && ui.button("🛠️ Serviços").clicked()
                    {
                        evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Servicos));
                    }
                    if permissoes.contains(&TelaAtiva::Financeiro)
                        && ui.button("💰 Financeiro").clicked()
                    {
                        evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Financeiro));
                    }
                    if permissoes.contains(&TelaAtiva::Orcamentos)
                        && ui.button("📈 Orçamentos").clicked()
                    {
                        evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Orcamentos));
                    }
                    if permissoes.contains(&TelaAtiva::Gerencia)
                        && ui.button("📊 Gerência").clicked()
                    {
                        evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Gerencia));
                    }
                    if permissoes.contains(&TelaAtiva::Estoque) && ui.button("📦 Estoque").clicked()
                    {
                        evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Estoque));
                    }
                });

                // --- GRUPO DE FUNCIONALIDADES FUTURAS ---
                ui.collapsing("🚀 Funcionalidades Futuras", |ui| {
                    if permissoes.contains(&TelaAtiva::Orcamentos) {
                        ui.add_enabled(false, egui::Button::new("Clientes (avançado)"));
                        ui.add_enabled(false, egui::Button::new("🧾 Vendas"));
                    }
                    if permissoes.contains(&TelaAtiva::Gerencia)
                        || permissoes.contains(&TelaAtiva::Admin)
                    {
                        ui.add_enabled(false, egui::Button::new("📑 Relatórios"));
                        ui.add_enabled(false, egui::Button::new("📆 Calendário"));
                    }
                    if permissoes.contains(&TelaAtiva::Financeiro) {
                        ui.add_enabled(false, egui::Button::new("Relatórios Financeiros"));
                    }
                    if permissoes.contains(&TelaAtiva::Estoque) {
                        ui.add_enabled(false, egui::Button::new("Inventário Avançado"));
                    }
                    if permissoes.contains(&TelaAtiva::Admin) {
                        ui.add_enabled(false, egui::Button::new("⚙️ Configurações"));
                        ui.add_enabled(false, egui::Button::new("🔌 Integrações"));
                    }
                    ui.add_enabled(false, egui::Button::new("🔔 Notificações"));
                    ui.add_enabled(false, egui::Button::new("🆘 Suporte"));
                    ui.add_enabled(false, egui::Button::new("❓ Ajuda"));
                });
            });
            // --- FIM DA ÁREA DE ROLAGEM ---

            // --- SEÇÃO INFERIOR: VERSÃO (fora da área de rolagem para ficar fixa) ---
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
