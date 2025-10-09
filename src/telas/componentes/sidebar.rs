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
            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                ui.add_space(10.0);
                if let Some(logo_texture) = logo {
                    ui.add(egui::Image::new(logo_texture).max_width(180.0));
                }
                ui.add_space(10.0);
                ui.separator();

                ui.label("Navegação");

                // Dashboard: somente para papéis com acesso (Admin, Gerencia, Comercial)
                if matches!(
                    papel_usuario,
                    PapelUsuario::Administrador | PapelUsuario::Gerencia | PapelUsuario::Comercial
                ) {
                    if ui.add(egui::Button::new("🏠 Dashboard")).clicked() {
                        evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Dashboard));
                    }
                }

                if permissoes.contains(&AlvoNavegacao::Admin)
                    && ui.add(egui::Button::new("💼 Administrativo")).clicked()
                {
                    evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Admin));
                }

                // Ordens de Serviço: disponível para Admin, Comercial e Técnico
                if permissoes.contains(&AlvoNavegacao::Admin)
                    || permissoes.contains(&AlvoNavegacao::Comercial)
                    || permissoes.contains(&AlvoNavegacao::Tecnico)
                {
                    if ui.add(egui::Button::new("📝 Ordens de Serviço")).clicked() {
                        evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Ordens));
                    }
                }

                if permissoes.contains(&AlvoNavegacao::Financeiro)
                    && ui.add(egui::Button::new("💰 Financeiro")).clicked()
                {
                    evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Financeiro));
                }
                if permissoes.contains(&AlvoNavegacao::Comercial)
                    && ui.add(egui::Button::new("📈 Orçamentos")).clicked()
                {
                    evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Orcamentos));
                }
                if permissoes.contains(&AlvoNavegacao::Gerencia)
                    && ui.add(egui::Button::new("📊 Gerência")).clicked()
                {
                    evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Gerencia));
                }
                if permissoes.contains(&AlvoNavegacao::Estoque)
                    && ui.add(egui::Button::new("📦 Estoque")).clicked()
                {
                    evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Estoque));
                }

                // Clientes: acessível a todos os usuários
                if ui.add(egui::Button::new("👥 Clientes")).clicked() {
                    evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Clientes));
                }

                ui.add_space(8.0);
                ui.separator();
                ui.label("Funcionalidades futuras (Em breve)");

                if permissoes.contains(&AlvoNavegacao::Comercial) {
                    ui.add_enabled(
                        false,
                        egui::Button::new("👥 Clientes (avançado) (Em breve)"),
                    );
                    ui.add_enabled(false, egui::Button::new("🧾 Vendas (Em breve)"));
                }

                if permissoes.contains(&AlvoNavegacao::Gerencia)
                    || permissoes.contains(&AlvoNavegacao::Admin)
                {
                    ui.add_enabled(false, egui::Button::new("📑 Relatórios (Em breve)"));
                    ui.add_enabled(false, egui::Button::new("📆 Calendário (Em breve)"));
                }

                if permissoes.contains(&AlvoNavegacao::Financeiro) {
                    ui.add_enabled(
                        false,
                        egui::Button::new("📊 Relatórios Financeiros (Em breve)"),
                    );
                }

                if permissoes.contains(&AlvoNavegacao::Estoque) {
                    ui.add_enabled(
                        false,
                        egui::Button::new("🔁 Inventário Avançado (Em breve)"),
                    );
                }

                if permissoes.contains(&AlvoNavegacao::Admin) {
                    ui.add_enabled(false, egui::Button::new("⚙️ Configurações (Em breve)"));
                    ui.add_enabled(false, egui::Button::new("🔌 Integrações (Em breve)"));
                }

                ui.add_enabled(false, egui::Button::new("🔔 Notificações (Em breve)"));
                ui.add_enabled(false, egui::Button::new("🆘 Suporte (Em breve)"));
                ui.add_enabled(false, egui::Button::new("❓ Ajuda (Em breve)"));
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
