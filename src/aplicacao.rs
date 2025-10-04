// src/aplicacao.rs

use crate::servicos::{ErroAplicacao, PapelUsuario};
use crate::telas::{
    componentes::sidebar::{self, AcaoSidebar},
    login::TelaLogin,
    painel_adm::TelaAdmin,
    painel_os_edicao::TelaOsEdicao, // IMPORTA A NOVA TELA
    painel_principal::{self, TelaDashboard},
    painel_tecnico::TelaTecnico,
};
use eframe::egui;
use std::sync::mpsc::{Receiver, Sender};

// NOVO: Enum de eventos centralizado para toda a aplicação.
// Este enum substitui todas as `Acao...` individuais de cada tela.
#[derive(Debug)]
pub enum AppEvent {
    NavegarPara(TelaAtiva),
    AbrirEditorOS(u32),
    VoltarParaDashboard,
}

// ATUALIZADO: Adiciona o novo estado para a tela de edição.
pub enum EstadoTela {
    Login(TelaLogin),
    Dashboard(TelaDashboard),
    Admin(TelaAdmin),
    Tecnico(TelaTecnico),
    OsEdicao(TelaOsEdicao), // NOVO ESTADO
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Tema {
    Escuro,
    Claro,
}

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq)]
pub enum TelaAtiva {
    Dashboard,
    Admin,
    Tecnico,
    Financeiro,
    Comercial,
    Gerencia,
}

pub struct AplicativoPrincipal {
    estado_tela: EstadoTela,
    envio_db: Sender<(String, String)>,
    recebimento_db: Receiver<Result<PapelUsuario, ErroAplicacao>>,
    tema_atual: Tema,
    papel_usuario_logado: Option<PapelUsuario>,
    sidebar_aberto: bool,
    tela_ativa: TelaAtiva,
}

fn definir_estilo_azul(ctx: &egui::Context, tema: Tema) {
    let mut visuals = if tema == Tema::Escuro {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    let azul_destaque = egui::Color32::from_rgb(0, 120, 215);
    visuals.widgets.active.bg_fill = azul_destaque;
    visuals.selection.bg_fill = azul_destaque;
    ctx.set_visuals(visuals);
}

impl AplicativoPrincipal {
    pub fn new(
        envio_db: Sender<(String, String)>,
        recebimento_db: Receiver<Result<PapelUsuario, ErroAplicacao>>,
    ) -> Self {
        Self {
            estado_tela: TelaLogin::new(envio_db.clone()).into(),
            envio_db,
            recebimento_db,
            tema_atual: Tema::Escuro,
            papel_usuario_logado: None,
            sidebar_aberto: true,
            tela_ativa: TelaAtiva::Dashboard,
        }
    }
}

impl eframe::App for AplicativoPrincipal {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        definir_estilo_azul(ctx, self.tema_atual);

        if self.papel_usuario_logado.is_some() {
            self.mostrar_ui_principal(ctx, frame);
        } else {
            if let EstadoTela::Login(tela) = &mut self.estado_tela {
                tela.update(ctx, frame, &self.recebimento_db);
                if let Some(Ok(papel)) = tela.obter_resultado_login() {
                    self.papel_usuario_logado = Some(papel);
                    // Navega para o Dashboard ao logar
                    self.processar_evento(AppEvent::NavegarPara(TelaAtiva::Dashboard));
                    ctx.send_viewport_cmd(egui::ViewportCommand::Resizable(true));
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize([1280.0, 720.0].into()));
                    ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(
                        [1024.0, 600.0].into(),
                    ));
                }
            }
        }
    }
}

impl AplicativoPrincipal {
    fn mostrar_ui_principal(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("barra_superior").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                if ui.button("☰").clicked() {
                    self.sidebar_aberto = !self.sidebar_aberto;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Deslogar 📴").clicked() {
                        self.deslogar(ctx);
                    }
                    if ui
                        .button(if self.tema_atual == Tema::Escuro {
                            "☀️"
                        } else {
                            "🌙"
                        })
                        .clicked()
                    {
                        self.tema_atual = if self.tema_atual == Tema::Escuro {
                            Tema::Claro
                        } else {
                            Tema::Escuro
                        };
                    }
                });
            });
        });

        let papel = self
            .papel_usuario_logado
            .expect("Usuário deveria estar logado");

        // Sidebar agora também emite AppEvent
        if let Some(acao) = sidebar::mostrar(ctx, papel, self.sidebar_aberto) {
            let AcaoSidebar::NavegarPara(nova_tela) = acao;
            let tela_ativa = match nova_tela {
                painel_principal::AlvoNavegacao::Admin => TelaAtiva::Admin,
                painel_principal::AlvoNavegacao::Tecnico => TelaAtiva::Tecnico,
                painel_principal::AlvoNavegacao::Financeiro => TelaAtiva::Financeiro,
                painel_principal::AlvoNavegacao::Comercial => TelaAtiva::Comercial,
                painel_principal::AlvoNavegacao::Gerencia => TelaAtiva::Gerencia,
            };
            self.processar_evento(AppEvent::NavegarPara(tela_ativa));
        }

        let mut evento_emitido: Option<AppEvent> = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            // As telas agora retornam `Option<AppEvent>`
            let evento_da_tela = match &mut self.estado_tela {
                EstadoTela::Dashboard(tela) => tela.update(ctx, frame),
                EstadoTela::Admin(tela) => tela.update(ctx, frame),
                EstadoTela::Tecnico(tela) => tela.update(ctx, frame),
                EstadoTela::OsEdicao(tela) => tela.update(ctx, frame),
                // O login não é chamado aqui, então não precisa retornar evento
                EstadoTela::Login(_) => None,
            };
            evento_emitido = evento_da_tela;
        });

        // Se a tela ativa emitiu um evento, processe-o.
        if let Some(evento) = evento_emitido {
            self.processar_evento(evento);
        }
    }

    // NOVA FUNÇÃO CENTRAL: O roteador de eventos.
    fn processar_evento(&mut self, evento: AppEvent) {
        match evento {
            AppEvent::NavegarPara(tela) => {
                let papel = self.papel_usuario_logado.unwrap();
                self.tela_ativa = tela;
                self.estado_tela = match tela {
                    TelaAtiva::Dashboard => TelaDashboard::new(papel).into(),
                    TelaAtiva::Admin => TelaAdmin::new().into(),
                    TelaAtiva::Tecnico => TelaTecnico::new().into(),
                    // Adicione aqui as outras telas quando forem criadas
                    _ => TelaDashboard::new(papel).into(),
                };
            }
            AppEvent::AbrirEditorOS(os_id) => {
                // Muda o estado para a nova tela de edição, passando o ID.
                self.estado_tela = TelaOsEdicao::new(os_id).into();
            }
            AppEvent::VoltarParaDashboard => {
                let papel = self.papel_usuario_logado.unwrap();
                self.tela_ativa = TelaAtiva::Dashboard;
                self.estado_tela = TelaDashboard::new(papel).into();
            }
        }
    }

    fn deslogar(&mut self, ctx: &egui::Context) {
        self.papel_usuario_logado = None;
        self.estado_tela = TelaLogin::new(self.envio_db.clone()).into();
        ctx.send_viewport_cmd(egui::ViewportCommand::Resizable(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize([0.0, 0.0].into()));
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize([400.0, 600.0].into()));
    }
}

// Conversões `From`
impl From<TelaLogin> for EstadoTela {
    fn from(t: TelaLogin) -> Self {
        Self::Login(t)
    }
}
impl From<TelaDashboard> for EstadoTela {
    fn from(t: TelaDashboard) -> Self {
        Self::Dashboard(t)
    }
}
impl From<TelaAdmin> for EstadoTela {
    fn from(t: TelaAdmin) -> Self {
        Self::Admin(t)
    }
}
impl From<TelaTecnico> for EstadoTela {
    fn from(t: TelaTecnico) -> Self {
        Self::Tecnico(t)
    }
}
// ADICIONE A CONVERSÃO PARA A NOVA TELA
impl From<TelaOsEdicao> for EstadoTela {
    fn from(t: TelaOsEdicao) -> Self {
        Self::OsEdicao(t)
    }
}
