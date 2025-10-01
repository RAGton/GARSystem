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
}

impl AplicativoPrincipal {
    pub fn new(
        envio_db: Sender<(String, String)>,
        recebimento_db: Receiver<Result<PapelUsuario, ErroAplicacao>>,
    ) -> Self {
        Self {
            estado_tela: EstadoTela::Login(TelaLogin::new(envio_db.clone())),
            envio_db,
            recebimento_db,
            tema_atual: Tema::Escuro,
        }
    }
}

impl eframe::App for AplicativoPrincipal {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        match self.tema_atual {
            Tema::Escuro => ctx.set_visuals(egui::Visuals::dark()),
            Tema::Claro => ctx.set_visuals(egui::Visuals::light()),
        };

        let mut acao_global = AcaoGlobal::Nenhuma;

        // Desenha a barra de menu apenas se o usuário estiver logado
        if !matches!(&self.estado_tela, EstadoTela::Login(_)) {
            egui::TopBottomPanel::top("barra_superior_principal").show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
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
                tela.update(ctx, frame, &self.recebimento_db);
                if let Some(Ok(papel)) = tela.obter_resultado_login() {
                    proximo_estado = Some(EstadoTela::Dashboard(TelaDashboard::new(papel)));
                }
            }
            EstadoTela::Dashboard(tela) => match tela.update(ctx, frame) {
                AcaoDashboard::NavegarPara(alvo) => match alvo {
                    AlvoNavegacao::Admin => {
                        proximo_estado = Some(EstadoTela::Admin(TelaAdmin::new()))
                    }
                },
                AcaoDashboard::Deslogar => acao_global = AcaoGlobal::Deslogar,
                AcaoDashboard::Nenhuma => {}
            },
            EstadoTela::Admin(tela) => match tela.update(ctx, frame) {
                AcaoAdmin::Voltar => {
                    proximo_estado =
                        Some(EstadoTela::Dashboard(TelaDashboard::new(PapelUsuario::ADM)))
                }
                AcaoAdmin::Deslogar => acao_global = AcaoGlobal::Deslogar,
                AcaoAdmin::Nenhuma => {}
            },
            EstadoTela::Tecnico(tela) => {
                tela.update(ctx, frame);
            }
            EstadoTela::Financeiro(tela) => {
                tela.update(ctx, frame);
            }
            EstadoTela::Vendedor(tela) => {
                tela.update(ctx, frame);
            }
            EstadoTela::Gerente(tela) => {
                tela.update(ctx, frame);
            }
            EstadoTela::Atendente(tela) => {
                tela.update(ctx, frame);
            }
        }

        if let Some(novo_estado) = proximo_estado {
            self.estado_tela = novo_estado;
        }

        if acao_global == AcaoGlobal::Deslogar {
            self.estado_tela = EstadoTela::Login(TelaLogin::new(self.envio_db.clone()));
        }
    }
}
