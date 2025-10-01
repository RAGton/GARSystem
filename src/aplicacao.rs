// src/aplicacao.rs

use crate::servicos::{ErroAplicacao, PapelUsuario};
use crate::telas::{
    login::TelaLogin,
    painel_adm::{AcaoAdmin, TelaAdmin},
    painel_atendente::TelaAtendente,
    painel_financeiro::TelaFinanceiro,
    painel_gerente::TelaGerente,
    painel_principal::{AcaoDashboard, AlvoNavegacao, TelaDashboard},
    painel_tecnico::TelaTecnico,
    painel_vendedor::TelaVendedor,
};
use eframe::egui;
use std::sync::mpsc::{Receiver, Sender};

#[derive(PartialEq)]
pub enum AcaoGlobal {
    Nenhuma,
    Deslogar,
}

pub enum EstadoTela {
    Login(TelaLogin),
    Dashboard(TelaDashboard),
    Admin(TelaAdmin),
    Tecnico(TelaTecnico),
    Financeiro(TelaFinanceiro),
    Vendedor(TelaVendedor),
    Gerente(TelaGerente),
    Atendente(TelaAtendente),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Tema {
    Escuro,
    Claro,
}

pub struct AplicativoPrincipal {
    estado_tela: EstadoTela,
    envio_db: Sender<(String, String)>,
    recebimento_db: Receiver<Result<PapelUsuario, ErroAplicacao>>,
    tema_atual: Tema,
    papel_usuario_logado: PapelUsuario,
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
            papel_usuario_logado: PapelUsuario::ADM,
        }
    }

    fn voltar_para_dashboard(&mut self) {
        self.estado_tela = TelaDashboard::new(self.papel_usuario_logado).into();
    }
}

impl eframe::App for AplicativoPrincipal {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.tema_atual {
            Tema::Escuro => ctx.set_visuals(egui::Visuals::dark()),
            Tema::Claro => ctx.set_visuals(egui::Visuals::light()),
        };

        let mut acao_global = AcaoGlobal::Nenhuma;

        if !matches!(&self.estado_tela, EstadoTela::Login(_)) {
            egui::TopBottomPanel::top("barra_superior_principal").show(ctx, |ui| {
                // CORRIGIDO: Esta é a API moderna para criar uma barra de menu
                egui::menu::bar(ui, |ui| {
                    if !matches!(&self.estado_tela, EstadoTela::Dashboard(_)) {
                        if ui.button("⬅ Menu Principal").clicked() {
                            self.voltar_para_dashboard();
                        }
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Deslogar").clicked() {
                            acao_global = AcaoGlobal::Deslogar;
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
        }

        let mut proximo_estado = None;
        match &mut self.estado_tela {
            EstadoTela::Login(tela) => {
                tela.update(ctx, _frame, &self.recebimento_db);
                if let Some(Ok(papel)) = tela.obter_resultado_login() {
                    self.papel_usuario_logado = papel;
                    proximo_estado = Some(TelaDashboard::new(papel).into());

                    ctx.send_viewport_cmd(egui::ViewportCommand::Resizable(true));
                    ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(
                        [800.0, 600.0].into(),
                    ));
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize([1024.0, 768.0].into()));
                }
            }
            EstadoTela::Dashboard(tela) => match tela.update(ctx, _frame) {
                AcaoDashboard::NavegarPara(alvo) => match alvo {
                    AlvoNavegacao::Admin => proximo_estado = Some(TelaAdmin::new().into()),
                    AlvoNavegacao::Tecnico => proximo_estado = Some(TelaTecnico::new().into()),
                    AlvoNavegacao::Financeiro => {
                        proximo_estado = Some(TelaFinanceiro::new().into())
                    }
                    AlvoNavegacao::Vendedor => proximo_estado = Some(TelaVendedor::new().into()),
                    AlvoNavegacao::Gerente => proximo_estado = Some(TelaGerente::new().into()),
                    AlvoNavegacao::Atendente => proximo_estado = Some(TelaAtendente::new().into()),
                },
                AcaoDashboard::Nenhuma => {}
            },
            EstadoTela::Admin(tela) => match tela.update(ctx, _frame) {
                AcaoAdmin::Voltar => self.voltar_para_dashboard(),
                // O botão de deslogar na tela de admin agora dispara a ação global
                AcaoAdmin::Deslogar => acao_global = AcaoGlobal::Deslogar,
                AcaoAdmin::Nenhuma => {}
            },
            EstadoTela::Tecnico(tela) => {
                tela.update(ctx, _frame);
            }
            EstadoTela::Financeiro(tela) => {
                tela.update(ctx, _frame);
            }
            EstadoTela::Vendedor(tela) => {
                tela.update(ctx, _frame);
            }
            EstadoTela::Gerente(tela) => {
                tela.update(ctx, _frame);
            }
            EstadoTela::Atendente(tela) => {
                tela.update(ctx, _frame);
            }
        }

        if let Some(novo_estado) = proximo_estado {
            self.estado_tela = novo_estado;
        }

        if acao_global == AcaoGlobal::Deslogar {
            self.estado_tela = TelaLogin::new(self.envio_db.clone()).into();
            ctx.send_viewport_cmd(egui::ViewportCommand::Resizable(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize([624.0, 468.0].into()));
        }
    }
}

// Conversões `From`
impl From<TelaLogin> for EstadoTela {
    fn from(tela: TelaLogin) -> Self {
        Self::Login(tela)
    }
}
impl From<TelaDashboard> for EstadoTela {
    fn from(tela: TelaDashboard) -> Self {
        Self::Dashboard(tela)
    }
}
impl From<TelaAdmin> for EstadoTela {
    fn from(tela: TelaAdmin) -> Self {
        Self::Admin(tela)
    }
}
impl From<TelaTecnico> for EstadoTela {
    fn from(tela: TelaTecnico) -> Self {
        Self::Tecnico(tela)
    }
}
impl From<TelaFinanceiro> for EstadoTela {
    fn from(tela: TelaFinanceiro) -> Self {
        Self::Financeiro(tela)
    }
}
impl From<TelaVendedor> for EstadoTela {
    fn from(tela: TelaVendedor) -> Self {
        Self::Vendedor(tela)
    }
}
impl From<TelaGerente> for EstadoTela {
    fn from(tela: TelaGerente) -> Self {
        Self::Gerente(tela)
    }
}
impl From<TelaAtendente> for EstadoTela {
    fn from(tela: TelaAtendente) -> Self {
        Self::Atendente(tela)
    }
}
