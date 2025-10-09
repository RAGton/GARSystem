// src/telas/componentes/sidebar.rs

use crate::aplicacao::{AppEvent, TelaAtiva};
use crate::servicos::PapelUsuario;
use crate::telas::painel_principal::AlvoNavegacao;
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

fn montar_permissoes(papel_usuario: PapelUsuario) -> HashSet<AlvoNavegacao> {
    let mut permissoes = HashSet::new();
    use PapelUsuario::*;
    match papel_usuario {
        Administrador => {
            permissoes.extend([
                AlvoNavegacao::Admin,
                AlvoNavegacao::Tecnico,
                AlvoNavegacao::Financeiro,
                AlvoNavegacao::Comercial,
                AlvoNavegacao::Gerencia,
                AlvoNavegacao::Estoque,
            ]);
        }
        Tecnico => {
            permissoes.insert(AlvoNavegacao::Tecnico);
        }
        Financeiro => {
            permissoes.insert(AlvoNavegacao::Financeiro);
        }
        Comercial => {
            permissoes.insert(AlvoNavegacao::Comercial);
        }
        Gerencia => {
            permissoes.insert(AlvoNavegacao::Gerencia);
        }
        Estoquista => {
            permissoes.insert(AlvoNavegacao::Estoque);
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
        .default_width(200.0)
        .show_animated(ctx, sidebar_aberto, |ui| {
            // --- SEÇÃO SUPERIOR: LOGO E VERSÃO NO FINAL ---
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.add_space(10.0);
                if let Some(logo_texture) = logo {
                    ui.add(egui::Image::new(logo_texture).max_width(180.0));
                }
                ui.add_space(10.0);
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
                        if ui.button("Dashboard").clicked() {
                            evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Dashboard));
                        }
                    }

                    // Ordens de Serviço
                    if permissoes.contains(&AlvoNavegacao::Admin)
                        || permissoes.contains(&AlvoNavegacao::Comercial)
                        || permissoes.contains(&AlvoNavegacao::Tecnico)
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
                    if permissoes.contains(&AlvoNavegacao::Admin)
                        && ui.button("Administrativo").clicked()
                    {
                        evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Admin));
                    }
                    if permissoes.contains(&AlvoNavegacao::Admin)
                        && ui.button("🛠️ Serviços").clicked()
                    {
                        evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Servicos));
                    }
                    if permissoes.contains(&AlvoNavegacao::Financeiro)
                        && ui.button("💰 Financeiro").clicked()
                    {
                        evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Financeiro));
                    }
                    if permissoes.contains(&AlvoNavegacao::Comercial)
                        && ui.button("📈 Orçamentos").clicked()
                    {
                        evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Orcamentos));
                    }
                    if permissoes.contains(&AlvoNavegacao::Gerencia)
                        && ui.button("📊 Gerência").clicked()
                    {
                        evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Gerencia));
                    }
                    if permissoes.contains(&AlvoNavegacao::Estoque)
                        && ui.button("📦 Estoque").clicked()
                    {
                        evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Estoque));
                    }
                });

                // --- GRUPO DE FUNCIONALIDADES FUTURAS ---
                ui.collapsing("🚀 Funcionalidades Futuras", |ui| {
                    if permissoes.contains(&AlvoNavegacao::Comercial) {
                        ui.add_enabled(false, egui::Button::new("Clientes (avançado)"));
                        ui.add_enabled(false, egui::Button::new("🧾 Vendas"));
                    }
                    if permissoes.contains(&AlvoNavegacao::Gerencia)
                        || permissoes.contains(&AlvoNavegacao::Admin)
                    {
                        ui.add_enabled(false, egui::Button::new("📑 Relatórios"));
                        ui.add_enabled(false, egui::Button::new("📆 Calendário"));
                    }
                    if permissoes.contains(&AlvoNavegacao::Financeiro) {
                        ui.add_enabled(false, egui::Button::new("Relatórios Financeiros"));
                    }
                    if permissoes.contains(&AlvoNavegacao::Estoque) {
                        ui.add_enabled(false, egui::Button::new("Inventário Avançado"));
                    }
                    if permissoes.contains(&AlvoNavegacao::Admin) {
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
                    egui::RichText::new(format!("Versão: {}", env!("CARGO_PKG_VERSION")))
                        .color(egui::Color32::GRAY)
                        .size(12.0),
                );
            });
        });

    evento_emitido
}
