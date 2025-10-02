// src/telas/componentes/sidebar.rs
use crate::servicos::PapelUsuario;
use crate::telas::painel_principal::AlvoNavegacao;
use eframe::egui;
use std::collections::HashSet;

/// Ações que a sidebar pode requisitar.
pub enum AcaoSidebar {
    NavegarPara(AlvoNavegacao),
}

/// Desenha a sidebar e retorna uma ação se o usuário clicar em um botão de navegação.
///
/// Parâmetros:
/// - ctx: contexto do egui
/// - papel_usuario: papel do usuário logado (usado para permitir/ocultar botões)
/// - sidebar_aberto: controla se a sidebar está animada/visível
pub fn mostrar(
    ctx: &egui::Context,
    papel_usuario: PapelUsuario,
    sidebar_aberto: bool,
) -> Option<AcaoSidebar> {
    let mut acao_requisicao: Option<AcaoSidebar> = None;

    // Definir permissões de forma explícita (baseado no papel)
    let mut permissoes: HashSet<AlvoNavegacao> = HashSet::new();
    use PapelUsuario::*;
    match papel_usuario {
        Administrador => {
            // insere manualmente para evitar problemas de array IntoIterator generics
            permissoes.insert(AlvoNavegacao::Admin);
            permissoes.insert(AlvoNavegacao::Tecnico);
            permissoes.insert(AlvoNavegacao::Financeiro);
            permissoes.insert(AlvoNavegacao::Comercial);
            permissoes.insert(AlvoNavegacao::Gerencia);
        }
        Gerencia => {
            permissoes.insert(AlvoNavegacao::Gerencia);
            permissoes.insert(AlvoNavegacao::Tecnico);
            permissoes.insert(AlvoNavegacao::Financeiro);
            permissoes.insert(AlvoNavegacao::Comercial);
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
    }

    egui::SidePanel::left("sidebar")
        .resizable(true)
        .default_width(200.0)
        .width_range(150.0..=300.0)
        .show_animated(ctx, sidebar_aberto, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                ui.heading("Senior System");
                ui.separator();

                ui.collapsing("📊 Relatório de OS", |ui| {
                    ui.label("Em aberto: 5");
                    ui.label("Para faturar: 2");
                    ui.label("A realizar: 8");
                });
                ui.separator();

                ui.label("Navegação");

                // Dashboard / Admin
                if ui
                    .add(egui::Button::selectable(false, "🏠 Dashboard"))
                    .clicked()
                {
                    if permissoes.contains(&AlvoNavegacao::Admin) {
                        acao_requisicao = Some(AcaoSidebar::NavegarPara(AlvoNavegacao::Admin));
                    } else if let Some(&primeira) = permissoes.iter().next() {
                        acao_requisicao = Some(AcaoSidebar::NavegarPara(primeira));
                    }
                }

                if permissoes.contains(&AlvoNavegacao::Admin)
                    && ui
                        .add(egui::Button::selectable(false, "⚙️ Admin"))
                        .clicked()
                {
                    acao_requisicao = Some(AcaoSidebar::NavegarPara(AlvoNavegacao::Admin));
                }

                if permissoes.contains(&AlvoNavegacao::Tecnico)
                    && ui
                        .add(egui::Button::selectable(false, "🔧 Técnico"))
                        .clicked()
                {
                    acao_requisicao = Some(AcaoSidebar::NavegarPara(AlvoNavegacao::Tecnico));
                }

                if permissoes.contains(&AlvoNavegacao::Financeiro)
                    && ui
                        .add(egui::Button::selectable(false, "💳 Financeiro"))
                        .clicked()
                {
                    acao_requisicao = Some(AcaoSidebar::NavegarPara(AlvoNavegacao::Financeiro));
                }

                if permissoes.contains(&AlvoNavegacao::Comercial)
                    && ui
                        .add(egui::Button::selectable(false, "🛒 Comercial"))
                        .clicked()
                {
                    acao_requisicao = Some(AcaoSidebar::NavegarPara(AlvoNavegacao::Comercial));
                }

                if permissoes.contains(&AlvoNavegacao::Gerencia)
                    && ui
                        .add(egui::Button::selectable(false, "📈 Gerência"))
                        .clicked()
                {
                    acao_requisicao = Some(AcaoSidebar::NavegarPara(AlvoNavegacao::Gerencia));
                }
            });

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.label("Versão 1.0.0");
            });
        });

    acao_requisicao
}
