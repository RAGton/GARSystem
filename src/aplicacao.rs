// src/aplicacao.rs

use crate::servicos::PapelUsuario;
use crate::telas::{
    componentes::sidebar, configuracao::TelaConfiguracao, login::TelaLogin, painel_adm::TelaAdmin,
    painel_clientes::TelaClientes, painel_estoque::TelaEstoque, painel_financeiro::TelaFinanceiro,
    painel_gerencia::TelaGerencia, painel_orcamentos::TelaOrcamentos, painel_ordens::TelaOrdens,
    painel_os_criar::TelaCriarOs, painel_os_edicao::TelaOsEdicao, painel_principal::TelaDashboard,
    painel_tecnico::TelaTecnico,
};
use eframe::egui::{self, ColorImage, TextureHandle};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub enum AppEvent {
    NavegarPara(TelaAtiva),
    AbrirEditorOS(u32),
    Repaint,
    VoltarParaDashboard,
    FecharOverlayCriarOs,
}

pub enum EstadoTela {
    Login(TelaLogin),
    Configuracao(TelaConfiguracao),
    Dashboard(TelaDashboard),
    Clientes(TelaClientes),
    Admin(TelaAdmin),
    CriarOs(TelaCriarOs),
    Tecnico(TelaTecnico),
    Ordens(TelaOrdens),
    OsEdicao(TelaOsEdicao),
    Financeiro(TelaFinanceiro),
    Orcamentos(TelaOrcamentos),
    Gerencia(TelaGerencia),
    Servicos(crate::telas::painel_servicos::TelaServicos),
    Estoque(TelaEstoque),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Tema {
    Escuro,
    Claro,
}

#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq)]
pub enum TelaAtiva {
    Dashboard,
    Clientes,
    Admin,
    Tecnico,
    CriarOs,
    Financeiro,
    Orcamentos,
    Gerencia,
    Servicos,
    Ordens,
    Estoque,
}

pub struct AplicativoPrincipal {
    estado_tela: Option<EstadoTela>,
    tema_atual: Tema,
    papel_usuario_logado: Option<PapelUsuario>,
    sidebar_aberto: bool,
    tela_ativa: TelaAtiva,
    logo: Option<TextureHandle>,
    logo_data: Option<ColorImage>,
    endereco_servidor: Arc<Mutex<String>>,
    usuario_logado: Option<String>,
    // Quando preenchido, exibe a tela de criação de OS como janela flutuante
    overlay_criar_os: Option<TelaCriarOs>,
    evento_tx: Sender<AppEvent>,
    evento_rx: Receiver<AppEvent>,
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
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut endereco_servidor_str = "http://localhost:3000".to_string();
        let mut nome_usuario = String::new();
        let mut lembrar_usuario = false;
        let mut tema_atual = Tema::Escuro;

        if let Some(storage) = cc.storage {
            if let Some(addr) = storage.get_string("endereco_servidor") {
                if !addr.is_empty() {
                    endereco_servidor_str = addr;
                }
            }
            if let Some(lembrar) = storage.get_string("lembrar_usuario") {
                if let Ok(val) = lembrar.parse::<bool>() {
                    lembrar_usuario = val;
                    if lembrar_usuario {
                        if let Some(guardado) = storage.get_string("nome_usuario") {
                            nome_usuario = guardado;
                        }
                    }
                }
            }
            if let Some(tema_str) = storage.get_string("tema") {
                tema_atual = if tema_str.to_lowercase() == "claro" {
                    Tema::Claro
                } else {
                    Tema::Escuro
                };
            }
        }

        let endereco_servidor = Arc::new(Mutex::new(endereco_servidor_str));

        let (tx, rx) = mpsc::channel::<AppEvent>();

        let estado_tela = Some(EstadoTela::Login(TelaLogin::new(
            Arc::clone(&endereco_servidor),
            nome_usuario,
            lembrar_usuario,
        )));
        // Evitar preloads síncronos aqui para não travar a UI na inicialização.

        let logo_data = sidebar::carregar_logo().ok();

        Self {
            estado_tela,
            tema_atual,
            papel_usuario_logado: None,
            sidebar_aberto: true,
            tela_ativa: TelaAtiva::Dashboard,
            logo: None,
            logo_data,
            endereco_servidor,
            overlay_criar_os: None,
            usuario_logado: None,
            evento_tx: tx,
            evento_rx: rx,
        }
    }
}

impl eframe::App for AplicativoPrincipal {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Processar eventos vindos de threads (ex: solicitações de repaint)
        while let Ok(ev) = self.evento_rx.try_recv() {
            match ev {
                AppEvent::Repaint => ctx.request_repaint(),
                _ => self.processar_evento(ev),
            }
        }

        // Sincronizar tema se tiver sido alterado via Login/Storage
        if let Some(storage) = frame.storage() {
            if let Some(tema_str) = storage.get_string("tema") {
                let novo = if tema_str.to_lowercase() == "claro" {
                    Tema::Claro
                } else {
                    Tema::Escuro
                };
                if novo != self.tema_atual {
                    self.tema_atual = novo;
                }
            }
        }
        definir_estilo_azul(ctx, self.tema_atual);

        if self.logo.is_none() {
            if let Some(image) = self.logo_data.take() {
                self.logo = Some(ctx.load_texture("logo_empresa", image, Default::default()));
            }
        }

        let mut proximo_estado: Option<EstadoTela> = None;
        let mut login_sucesso: Option<PapelUsuario> = None;
        let mut deslogar_pedido = false;
        let mut evento_processado = false;

        if let Some(mut estado_atual) = self.estado_tela.take() {
            match &mut estado_atual {
                EstadoTela::Login(tela) => {
                    tela.update(ctx, frame, self.logo.as_ref());
                    if tela.deve_ir_para_configuracao() {
                        let endereco_atual = self.endereco_servidor.lock().unwrap().clone();
                        proximo_estado = Some(TelaConfiguracao::new(&endereco_atual).into());
                    }
                    if let Some(Ok(papel)) = tela.obter_resultado_login() {
                        // Persistir o estado 'lembrar usuário' quando o login for bem sucedido
                        if let Some(storage) = frame.storage_mut() {
                            tela.salvar_estado_login(storage);
                        }
                        login_sucesso = Some(papel);
                        self.usuario_logado = Some(tela.nome_usuario_atual().to_string());
                    }
                }
                EstadoTela::Configuracao(tela) => {
                    tela.update(ctx);
                    if tela.foi_salvo() {
                        let nova_url = tela.endereco_servidor.clone();
                        *self.endereco_servidor.lock().unwrap() = nova_url.clone();
                        if let Some(storage) = frame.storage_mut() {
                            storage.set_string("endereco_servidor", nova_url);
                        }
                    }
                    if tela.deve_voltar() {
                        // Ao voltar da configuração, repassa o usuário salvo e o flag 'lembrar'
                        if let Some(storage) = frame.storage() {
                            let mut nome_usuario = String::new();
                            let mut lembrar_usuario = false;
                            if let Some(lembrar) = storage.get_string("lembrar_usuario") {
                                if let Ok(val) = lembrar.parse::<bool>() {
                                    lembrar_usuario = val;
                                    if lembrar_usuario {
                                        if let Some(guardado) = storage.get_string("nome_usuario") {
                                            nome_usuario = guardado;
                                        }
                                    }
                                }
                            }
                            proximo_estado = Some(
                                TelaLogin::new(
                                    Arc::clone(&self.endereco_servidor),
                                    nome_usuario,
                                    lembrar_usuario,
                                )
                                .into(),
                            );
                        } else {
                            proximo_estado = Some(
                                TelaLogin::new(
                                    Arc::clone(&self.endereco_servidor),
                                    String::new(),
                                    false,
                                )
                                .into(),
                            );
                        }
                    }
                }
                _ => {
                    if self.papel_usuario_logado.is_some() {
                        let (deslogar, evento) =
                            self.mostrar_ui_principal(ctx, frame, &mut estado_atual);
                        deslogar_pedido = deslogar;
                        evento_processado = evento;
                    } else {
                        // O estado será `None`, recriando a tela de login
                    }
                }
            }
            if proximo_estado.is_none() && !deslogar_pedido && !evento_processado {
                self.estado_tela = Some(estado_atual);
            }
        }

        if let Some(novo_estado) = proximo_estado {
            self.estado_tela = Some(novo_estado);
        }

        if let Some(papel) = login_sucesso {
            self.papel_usuario_logado = Some(papel);
            self.processar_evento(AppEvent::NavegarPara(TelaAtiva::Dashboard));
            ctx.send_viewport_cmd(egui::ViewportCommand::Resizable(true));
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize([1280.0, 720.0].into()));
            ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize([1024.0, 600.0].into()));
        }

        if deslogar_pedido {
            self.deslogar(ctx);
        }

        if self.estado_tela.is_none() {
            // Ao criar a tela de login por falta de estado, repassar os valores salvos no storage
            if let Some(storage) = frame.storage() {
                let mut nome_usuario = String::new();
                let mut lembrar_usuario = false;
                if let Some(lembrar) = storage.get_string("lembrar_usuario") {
                    if let Ok(val) = lembrar.parse::<bool>() {
                        lembrar_usuario = val;
                        if lembrar_usuario {
                            if let Some(guardado) = storage.get_string("nome_usuario") {
                                nome_usuario = guardado;
                            }
                        }
                    }
                }
                self.estado_tela = Some(
                    TelaLogin::new(
                        Arc::clone(&self.endereco_servidor),
                        nome_usuario,
                        lembrar_usuario,
                    )
                    .into(),
                );
            } else {
                self.estado_tela = Some(
                    TelaLogin::new(Arc::clone(&self.endereco_servidor), String::new(), false)
                        .into(),
                );
            }
        }
    }
}
impl AplicativoPrincipal {
    fn mostrar_ui_principal(
        &mut self,
        ctx: &egui::Context,
        frame: &mut eframe::Frame,
        estado_tela: &mut EstadoTela,
    ) -> (bool, bool) {
        let mut deslogar_clicado = false;
        let mut evento_processado = false;

        egui::TopBottomPanel::top("barra_superior").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("☰").clicked() {
                    self.sidebar_aberto = !self.sidebar_aberto;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Deslogar 📴").clicked() {
                        deslogar_clicado = true;
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
                        if let Some(storage) = frame.storage_mut() {
                            let tema_str = if self.tema_atual == Tema::Claro {
                                "claro".to_string()
                            } else {
                                "escuro".to_string()
                            };
                            storage.set_string("tema", tema_str);
                        }
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
            evento_processado = true;
        }

        let mut evento_emitido: Option<AppEvent> = None;
        egui::CentralPanel::default().show(ctx, |_| {
            evento_emitido = match estado_tela {
                EstadoTela::Dashboard(tela) => tela.update(ctx, frame),
                EstadoTela::Clientes(tela) => tela.update(ctx, frame),
                EstadoTela::Admin(tela) => tela.update(ctx, frame),
                EstadoTela::Tecnico(tela) => tela.update(ctx, frame),
                EstadoTela::CriarOs(tela) => tela.update(ctx, frame),
                EstadoTela::Ordens(tela) => tela.update(ctx, frame),
                EstadoTela::OsEdicao(tela) => tela.update(ctx, frame),
                EstadoTela::Financeiro(tela) => tela.update(ctx, frame),
                // Comercial removido (substituído por Orçamentos)
                EstadoTela::Orcamentos(tela) => tela.update(ctx, frame),
                EstadoTela::Gerencia(tela) => tela.update(ctx, frame),
                EstadoTela::Estoque(tela) => tela.update(ctx, frame),
                EstadoTela::Servicos(tela) => tela.update(ctx, frame),
                _ => None,
            };
        });

        if let Some(evento) = evento_emitido {
            self.processar_evento(evento);
            evento_processado = true;
        }

        // Se houver uma janela flutuante de criação de OS, exibi-la por cima da tela atual.
        if let Some(overlay) = &mut self.overlay_criar_os {
            let evento_overlay = overlay.show_as_window(ctx, frame);
            if let Some(ev) = evento_overlay {
                // fechar overlay antes de processar para evitar reentrância/confusão de estado
                self.overlay_criar_os = None;
                self.processar_evento(ev);
                evento_processado = true;
            }
        }

        (deslogar_clicado, evento_processado)
    }

    fn processar_evento(&mut self, evento: AppEvent) {
        match evento {
            AppEvent::NavegarPara(tela) => {
                if let Some(papel) = self.papel_usuario_logado {
                    // Se pedirem para abrir a tela de criação de OS enquanto estamos
                    // na lista de Ordens, abrimos como overlay para manter a lista visível.
                    if tela == TelaAtiva::CriarOs {
                        if self.tela_ativa == TelaAtiva::Ordens {
                            self.overlay_criar_os = Some(TelaCriarOs::new(
                                Arc::clone(&self.endereco_servidor),
                                self.usuario_logado.clone(),
                            ));
                            // Garantir que continuamos com uma tela ativa (Ordens)
                            self.estado_tela = Some(
                                TelaOrdens::new(papel, Arc::clone(&self.endereco_servidor)).into(),
                            );
                            return;
                        }
                    }

                    self.tela_ativa = tela;
                    let novo_estado = match tela {
                        TelaAtiva::Dashboard => {
                            TelaDashboard::new(papel, Arc::clone(&self.endereco_servidor)).into()
                        }
                        TelaAtiva::Clientes => {
                            TelaClientes::new(Arc::clone(&self.endereco_servidor)).into()
                        }
                        TelaAtiva::Admin => {
                            TelaAdmin::new(Arc::clone(&self.endereco_servidor)).into()
                        }
                        TelaAtiva::Tecnico => {
                            TelaTecnico::new(Arc::clone(&self.endereco_servidor)).into()
                        }
                        TelaAtiva::Ordens => {
                            TelaOrdens::new(papel, Arc::clone(&self.endereco_servidor)).into()
                        }
                        TelaAtiva::CriarOs => TelaCriarOs::new(
                            Arc::clone(&self.endereco_servidor),
                            self.usuario_logado.clone(),
                        )
                        .into(),
                        TelaAtiva::Financeiro => TelaFinanceiro::new().into(),
                        TelaAtiva::Orcamentos => {
                            TelaOrcamentos::new(Arc::clone(&self.endereco_servidor)).into()
                        }
                        TelaAtiva::Gerencia => TelaGerencia::new().into(),
                        TelaAtiva::Servicos => {
                            crate::telas::painel_servicos::TelaServicos::new().into()
                        }
                        TelaAtiva::Estoque => TelaEstoque::new().into(),
                    };
                    self.estado_tela = Some(novo_estado);
                }
            }
            AppEvent::Repaint => {
                // ignorado aqui, já tratado no loop de recebimento
            }
            AppEvent::AbrirEditorOS(os_id) => {
                self.estado_tela = Some(
                    TelaOsEdicao::new(
                        os_id,
                        Arc::clone(&self.endereco_servidor),
                        self.usuario_logado
                            .clone()
                            .unwrap_or_else(|| "sistema".to_string()),
                        self.evento_tx.clone(),
                    )
                    .into(),
                );
            }
            AppEvent::FecharOverlayCriarOs => {
                self.overlay_criar_os = None;
            }
            AppEvent::VoltarParaDashboard => {
                if let Some(papel) = self.papel_usuario_logado {
                    self.tela_ativa = TelaAtiva::Dashboard;
                    self.estado_tela =
                        Some(TelaDashboard::new(papel, Arc::clone(&self.endereco_servidor)).into());
                }
            }
        }
    }

    fn deslogar(&mut self, ctx: &egui::Context) {
        self.papel_usuario_logado = None;
        self.usuario_logado = None;
        self.estado_tela = None;
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

impl From<TelaClientes> for EstadoTela {
    fn from(t: TelaClientes) -> Self {
        Self::Clientes(t)
    }
}

impl From<TelaCriarOs> for EstadoTela {
    fn from(t: TelaCriarOs) -> Self {
        Self::CriarOs(t)
    }
}

impl From<TelaOrdens> for EstadoTela {
    fn from(t: TelaOrdens) -> Self {
        Self::Ordens(t)
    }
}

impl From<TelaFinanceiro> for EstadoTela {
    fn from(t: TelaFinanceiro) -> Self {
        Self::Financeiro(t)
    }
}
impl From<TelaGerencia> for EstadoTela {
    fn from(t: TelaGerencia) -> Self {
        Self::Gerencia(t)
    }
}

impl From<crate::telas::painel_servicos::TelaServicos> for EstadoTela {
    fn from(t: crate::telas::painel_servicos::TelaServicos) -> Self {
        Self::Servicos(t)
    }
}

impl From<TelaEstoque> for EstadoTela {
    fn from(t: TelaEstoque) -> Self {
        Self::Estoque(t)
    }
}
