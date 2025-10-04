// src/telas/login.rs

use crate::servicos::{ErroAplicacao, PapelUsuario};
use eframe::egui::{self, TextureHandle};
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, PartialEq, Clone)]
pub enum EstadoLogin {
    Ocioso,
    EmProgresso,
    Sucesso(PapelUsuario),
    Falha(String),
}

#[derive(Deserialize)]
struct LoginResponse {
    papel: PapelUsuario,
}

pub struct TelaLogin {
    nome_usuario: String,
    senha: String,
    estado: Arc<Mutex<EstadoLogin>>,
}

impl TelaLogin {
    pub fn new() -> Self {
        Self {
            nome_usuario: String::new(),
            senha: String::new(),
            estado: Arc::new(Mutex::new(EstadoLogin::Ocioso)),
        }
    }

    pub fn obter_resultado_login(&mut self) -> Option<Result<PapelUsuario, ErroAplicacao>> {
        let mut estado_guard = self.estado.lock().unwrap();
        match &*estado_guard {
            EstadoLogin::Sucesso(papel) => {
                let resultado = Some(Ok(*papel));
                *estado_guard = EstadoLogin::Ocioso;
                resultado
            }
            EstadoLogin::Falha(msg) => {
                let resultado = Some(Err(ErroAplicacao::Desconhecido(msg.clone())));
                *estado_guard = EstadoLogin::Ocioso;
                resultado
            }
            _ => None,
        }
    }

    pub fn update(
        &mut self,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        logo: Option<&TextureHandle>,
    ) {
        let estado_atual = self.estado.lock().unwrap().clone();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.add_space(ui.available_height() * 0.1);
                
                if let Some(logo_texture) = logo {
                    ui.add(egui::Image::new(logo_texture).max_size(egui::vec2(280.0, 158.0)));
                }

                ui.add_space(20.0);
                ui.label(egui::RichText::new("Bem-vindo! Faça o login para continuar.").italics().size(16.0));
                ui.add_space(30.0);

                ui.add_enabled_ui(!matches!(estado_atual, EstadoLogin::EmProgresso), |ui| {
                    egui::Frame::new().show(ui, |ui| {
                        ui.set_max_width(300.0);
                        ui.label("Usuário:");
                        ui.text_edit_singleline(&mut self.nome_usuario);
                        ui.add_space(10.0);
                        ui.label("Senha:");
                        ui.add(egui::TextEdit::singleline(&mut self.senha).password(true));
                        ui.add_space(20.0);

                        ui.horizontal(|ui| {
                            if ui.button("   Entrar   ").clicked() {
                                *self.estado.lock().unwrap() = EstadoLogin::EmProgresso;

                                let estado_clone = self.estado.clone();
                                let nome_usuario_clone = self.nome_usuario.clone();
                                let senha_clone = self.senha.clone();
                                let ctx_clone = ctx.clone();

                                thread::spawn(move || {
                                    let client = reqwest::blocking::Client::new();
                                    let response = client
                                        .post("http://localhost:3000/login")
                                        .json(&serde_json::json!({ "usuario": nome_usuario_clone, "senha": senha_clone }))
                                        .send();

                                    let mut estado = estado_clone.lock().unwrap();
                                    match response {
                                        Ok(res) => {
                                            if res.status().is_success() {
                                                match res.json::<LoginResponse>() {
                                                    Ok(login_res) => *estado = EstadoLogin::Sucesso(login_res.papel),
                                                    Err(e) => *estado = EstadoLogin::Falha(format!("Erro ao processar resposta: {}", e)),
                                                }
                                            } else {
                                                *estado = EstadoLogin::Falha("Usuário ou senha inválidos.".to_string());
                                            }
                                        }
                                        Err(e) => *estado = EstadoLogin::Falha(format!("Erro de conexão com o servidor: {}", e)),
                                    }
                                    ctx_clone.request_repaint();
                                });
                            }

                            if ui.button("Cancelar").clicked() {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        });
                    });
                });
                
                match estado_atual {
                    EstadoLogin::EmProgresso => {
                        ui.add_space(10.0);
                        ui.horizontal(|ui| { ui.spinner(); ui.label("Entrando..."); });
                    }
                    EstadoLogin::Falha(msg_erro) => {
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new(msg_erro).color(egui::Color32::RED));
                    }
                    _ => {}
                }
                
                ui.add_space(ui.available_height() - 40.0);
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("© RAG - 2025").color(egui::Color32::GRAY).size(12.0));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(format!("Versão: {}", env!("CARGO_PKG_VERSION"))).color(egui::Color32::GRAY).size(12.0));
                    });
                });
            });
        });
    }
}