// src/aplicacao.rs

use crate::servicos::PapelUsuario;
use crate::telas::{
    componentes::sidebar, configuracao::TelaConfiguracao, login::TelaLogin, painel_adm::TelaAdmin,
    painel_os_edicao::TelaOsEdicao, painel_principal::TelaDashboard, painel_tecnico::TelaTecnico,
};
use eframe::egui::{self, TextureHandle};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub enum AppEvent {
    NavegarPara(TelaAtiva),
    AbrirEditorOS(u32),
    VoltarParaDashboard,
}

pub enum EstadoTela {
    Login(TelaLogin),
    Configuracao(TelaConfiguracao),
    Dashboard(TelaDashboard),
    Admin(TelaAdmin),
    Tecnico(TelaTecnico),
    OsEdicao(TelaOsEdicao),
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
    tema_atual: Tema,
    papel_usuario_logado: Option<PapelUsuario>,
    sidebar_aberto: bool,
    tela_ativa: TelaAtiva,
    logo: Option<TextureHandle>,
    endereco_servidor: Arc<Mutex<String>>,
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
    pub fn new() -> Self {
        let endereco_servidor = Arc::new(Mutex::new("http://localhost:3000".to_string()));
        Self {
            estado_tela: TelaLogin::new(Arc::clone(&endereco_servidor)).into(),
            tema_atual: Tema::Escuro,
            papel_usuario_logado: None,
            sidebar_aberto: true,
            tela_ativa: TelaAtiva::Dashboard,
            logo: None,
            endereco_servidor,
        }
    }
}

impl eframe::App for AplicativoPrincipal {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        definir_estilo_azul(ctx, self.tema_atual);

        if self.logo.is_none() {
            if let Ok(imagem) = sidebar::carregar_logo() {
                self.logo = Some(ctx.load_texture("logo_empresa", imagem, Default::default()));
            }
        }

        let mut proximo_estado: Option<EstadoTela> = None;
        let mut login_sucesso: Option<PapelUsuario> = None;
        let mut deslogar_pedido = false;

        match &mut self.estado_tela {
            EstadoTela::Login(tela) => {
                tela.update(ctx, frame, self.logo.as_ref());
                if tela.deve_ir_para_configuracao() {
                    let endereco_atual = self.endereco_servidor.lock().unwrap().clone();
                    proximo_estado = Some(TelaConfiguracao::new(&endereco_atual).into());
                }
                if let Some(Ok(papel)) = tela.obter_resultado_login() {
                    login_sucesso = Some(papel);
                }
            }
            EstadoTela::Configuracao(tela) => {
                tela.update(ctx);
                if tela.foi_salvo() {
                    *self.endereco_servidor.lock().unwrap() = tela.endereco_servidor.clone();
                    proximo_estado =
                        Some(TelaLogin::new(Arc::clone(&self.endereco_servidor)).into());
                }
            }
            _ => {
                if self.papel_usuario_logado.is_some() {
                    // A função `mostrar_ui_principal` agora nos diz se o deslog foi pedido.
                    if self.mostrar_ui_principal(ctx, frame) {
                        deslogar_pedido = true;
                    }
                } else {
                    proximo_estado =
                        Some(TelaLogin::new(Arc::clone(&self.endereco_servidor)).into());
                }
            }
        }

        if let Some(novo_estado) = proximo_estado {
            self.estado_tela = novo_estado;
        }

        if let Some(papel) = login_sucesso {
            self.papel_usuario_logado = Some(papel);
            self.processar_evento(AppEvent::NavegarPara(TelaAtiva::Dashboard));
            ctx.send_viewport_cmd(egui::ViewportCommand::Resizable(true));
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize([1280.0, 720.0].into()));
            ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize([1024.0, 600.0].into()));
        }

        // A ação de deslogar acontece aqui, no final do ciclo de `update`.
        if deslogar_pedido {
            self.deslogar(ctx);
        }
    }
}

impl AplicativoPrincipal {
    /// Agora retorna `bool` para indicar se o deslog foi solicitado.
    fn mostrar_ui_principal(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) -> bool {
        let mut deslogar_clicado = false;

        egui::TopBottomPanel::top("barra_superior").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                if ui.button("☰").clicked() {
                    self.sidebar_aberto = !self.sidebar_aberto;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Deslogar 📴").clicked() {
                        deslogar_clicado = true; // Apenas sinaliza, não executa
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

        if let Some(evento) = sidebar::mostrar(ctx, papel, self.sidebar_aberto, self.logo.as_ref())
        {
            self.processar_evento(evento);
        }

        let mut evento_emitido: Option<AppEvent> = None;
        egui::CentralPanel::default().show(ctx, |_| {
            evento_emitido = match &mut self.estado_tela {
                EstadoTela::Dashboard(tela) => tela.update(ctx, frame),
                EstadoTela::Admin(tela) => tela.update(ctx, frame),
                EstadoTela::Tecnico(tela) => tela.update(ctx, frame),
                EstadoTela::OsEdicao(tela) => tela.update(ctx, frame),
                _ => None,
            };
        });

        if let Some(evento) = evento_emitido {
            self.processar_evento(evento);
        }

        deslogar_clicado // Retorna a sinalização
    }

    fn processar_evento(&mut self, evento: AppEvent) {
        match evento {
            AppEvent::NavegarPara(tela) => {
                if let Some(papel) = self.papel_usuario_logado {
                    self.tela_ativa = tela;
                    self.estado_tela = match tela {
                        TelaAtiva::Dashboard => TelaDashboard::new(papel).into(),
                        TelaAtiva::Admin => TelaAdmin::new().into(),
                        TelaAtiva::Tecnico => TelaTecnico::new().into(),
                        _ => TelaDashboard::new(papel).into(),
                    };
                }
            }
            AppEvent::AbrirEditorOS(os_id) => {
                self.estado_tela = TelaOsEdicao::new(os_id).into();
            }
            AppEvent::VoltarParaDashboard => {
                if let Some(papel) = self.papel_usuario_logado {
                    self.tela_ativa = TelaAtiva::Dashboard;
                    self.estado_tela = TelaDashboard::new(papel).into();
                }
            }
        }
    }

    fn deslogar(&mut self, ctx: &egui::Context) {
        self.papel_usuario_logado = None;
        self.estado_tela = TelaLogin::new(Arc::clone(&self.endereco_servidor)).into();
        ctx.send_viewport_cmd(egui::ViewportCommand::Resizable(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize([0.0, 0.0].into()));
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize([800.0, 600.0].into()));
    }
}

impl From<TelaLogin> for EstadoTela {
    fn from(t: TelaLogin) -> Self {
        Self::Login(t)
    }
}
impl From<TelaConfiguracao> for EstadoTela {
    fn from(t: TelaConfiguracao) -> Self {
        Self::Configuracao(t)
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
impl From<TelaOsEdicao> for EstadoTela {
    fn from(t: TelaOsEdicao) -> Self {
        Self::OsEdicao(t)
    }
}
