// src/telas/login.rs

use crate::servicos::{ErroAplicacao, PapelUsuario};
use eframe::egui::{self, Align2, Color32, TextureHandle};
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, PartialEq, Clone)]
pub enum EstadoLogin {
    Ocioso,
    EmProgresso,
    Sucesso(PapelUsuario),
    Falha { titulo: String, mensagem: String },
}

#[derive(Deserialize)]
struct LoginResponse {
    papel: PapelUsuario,
}

pub struct TelaLogin {
    nome_usuario: String,
    senha: String,
    lembrar_usuario: bool,
    last_lembrar: bool,
    estado: Arc<Mutex<EstadoLogin>>,
    endereco_servidor: Arc<Mutex<String>>,
    ir_para_configuracao: bool,
}

impl TelaLogin {
    // Assinatura atualizada
    pub fn new(
        endereco_servidor: Arc<Mutex<String>>,
        nome_usuario: String,
        lembrar_usuario: bool,
    ) -> Self {
        Self {
            nome_usuario,
            senha: String::new(),
            lembrar_usuario,
            last_lembrar: lembrar_usuario,
            estado: Arc::new(Mutex::new(EstadoLogin::Ocioso)),
            endereco_servidor,
            ir_para_configuracao: false,
        }
    }

    pub fn nome_usuario_atual(&self) -> &str {
        &self.nome_usuario
    }

    // Novo método para salvar o estado
    pub fn salvar_estado_login(&self, storage: &mut dyn eframe::Storage) {
        storage.set_string("lembrar_usuario", self.lembrar_usuario.to_string());
        if self.lembrar_usuario {
            storage.set_string("nome_usuario", self.nome_usuario.clone());
        } else {
            storage.set_string("nome_usuario", "".to_string());
        }
    }

    pub fn deve_ir_para_configuracao(&mut self) -> bool {
        if self.ir_para_configuracao {
            self.ir_para_configuracao = false;
            true
        } else {
            false
        }
    }

    pub fn obter_resultado_login(&mut self) -> Option<Result<PapelUsuario, ErroAplicacao>> {
        if let Ok(mut estado_guard) = self.estado.lock() {
            if let EstadoLogin::Sucesso(papel) = *estado_guard {
                let resultado = Some(Ok(papel));
                *estado_guard = EstadoLogin::Ocioso;
                return resultado;
            }
        }
        None
    }

    pub fn update(
        &mut self,
        ctx: &egui::Context,
        frame: &mut eframe::Frame,
        logo: Option<&TextureHandle>,
    ) {
        let estado_atual = self.estado.lock().unwrap().clone();
        let is_dark_mode = ctx.style().visuals.dark_mode;

        let left_panel_color = if is_dark_mode {
            egui::Color32::from_rgb(20, 25, 40)
        } else {
            egui::Color32::from_rgb(30, 80, 180)
        };

        let gradient_end_color = {
            let mut color = ctx.style().visuals.panel_fill;
            color = color.linear_multiply(0.0); // Torna transparente
            color
        };

        let side_panel_response = egui::SidePanel::left("painel_branding")
            .resizable(false)
            .exact_width(ctx.available_rect().width() / 2.5)
            .frame(egui::Frame::default().fill(left_panel_color))
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.add_space(ui.available_height() * 0.2);
                    if let Some(logo_texture) = logo {
                        ui.add(egui::Image::new(logo_texture).max_size(egui::vec2(280.0, 158.0)));
                    }
                    ui.add_space(20.0);
                    ui.add_space(20.0);
                    ui.label(
                        egui::RichText::new("Bem-vindo! Faça o login para continuar.")
                            .color(egui::Color32::WHITE)
                            .italics()
                            .size(16.0),
                    );
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                ui.painter()
                    .rect_filled(ui.clip_rect(), 0.0, ui.visuals().panel_fill);

                let panel_rect = side_panel_response.response.rect;
                let gradient_width = 20.0;
                let gradient_rect = egui::Rect::from_min_max(
                    egui::pos2(panel_rect.right(), panel_rect.top()),
                    egui::pos2(panel_rect.right() + gradient_width, panel_rect.bottom()),
                );

                use egui::epaint::{Shape, Vertex};
                let mut mesh = egui::Mesh::default();
                mesh.vertices = vec![
                    Vertex {
                        pos: gradient_rect.left_top(),
                        color: left_panel_color,
                        uv: egui::pos2(0.0, 0.0),
                    },
                    Vertex {
                        pos: gradient_rect.right_top(),
                        color: gradient_end_color,
                        uv: egui::pos2(1.0, 0.0),
                    },
                    Vertex {
                        pos: gradient_rect.right_bottom(),
                        color: gradient_end_color,
                        uv: egui::pos2(1.0, 1.0),
                    },
                    Vertex {
                        pos: gradient_rect.left_bottom(),
                        color: left_panel_color,
                        uv: egui::pos2(0.0, 1.0),
                    },
                ];
                mesh.indices = vec![0, 1, 2, 0, 2, 3];
                ui.painter().add(Shape::Mesh(mesh.into()));

                egui::Frame::default()
                    .fill(Color32::TRANSPARENT)
                    .show(ui, |ui| {
                        ui.with_layout(egui::Layout::top_down(egui::Align::Max), |ui| {
                            if ui
                                .button("⚙")
                                .on_hover_text("Configurar Servidor")
                                .clicked()
                            {
                                self.ir_para_configuracao = true;
                            }
                        });

                        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                            ui.add_space(ui.available_height() * 0.15);
                            ui.heading(egui::RichText::new("Acesse sua Conta").size(24.0));
                            ui.add_space(30.0);

                            let formulario_habilitado =
                                !matches!(estado_atual, EstadoLogin::EmProgresso);
                            ui.add_enabled_ui(formulario_habilitado, |ui| {
                                egui::Frame::new()
                                    .outer_margin(egui::Margin::symmetric(10, 0))
                                    .show(ui, |ui| {
                                        ui.set_max_width(320.0);
                                        ui.label("Usuário:");
                                        ui.add_space(4.0);
                                        let user_input_response =
                                            ui.text_edit_singleline(&mut self.nome_usuario);

                                        ui.add_space(15.0);
                                        ui.label("Senha:");
                                        ui.add_space(4.0);
                                        let password_input_response = ui.add(
                                            egui::TextEdit::singleline(&mut self.senha)
                                                .password(true),
                                        );

                                        // Adiciona o checkbox (salva somente quando o usuário alterar)
                                        ui.add_space(10.0);
                                        let resp = ui
                                            .checkbox(&mut self.lembrar_usuario, "Lembrar de mim");
                                        if resp.changed() {
                                            if let Some(storage) = frame.storage_mut() {
                                                self.salvar_estado_login(storage);
                                                self.last_lembrar = self.lembrar_usuario;
                                            }
                                        }

                                        if user_input_response.lost_focus()
                                            && ui.input(|i| i.key_pressed(egui::Key::Enter))
                                        {
                                            ui.memory_mut(|m| {
                                                m.request_focus(password_input_response.id)
                                            });
                                        }

                                        if password_input_response.lost_focus()
                                            && ui.input(|i| i.key_pressed(egui::Key::Enter))
                                        {
                                            self.iniciar_processo_login(ctx);
                                        }

                                        ui.add_space(25.0);
                                        ui.horizontal(|ui| {
                                            if ui
                                                .button(
                                                    egui::RichText::new("   Entrar   ").size(14.0),
                                                )
                                                .clicked()
                                            {
                                                self.iniciar_processo_login(ctx);
                                            }
                                            if ui
                                                .button(
                                                    egui::RichText::new(" Cancelar ").size(14.0),
                                                )
                                                .clicked()
                                            {
                                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                            }
                                        });
                                    });
                            });

                            if let EstadoLogin::EmProgresso = estado_atual {
                                ui.add_space(15.0);
                                ui.horizontal(|ui| {
                                    ui.spinner();
                                    ui.label("Entrando...");
                                });
                            }

                            if let EstadoLogin::Falha {
                                ref titulo,
                                ref mensagem,
                            } = estado_atual
                            {
                                let mut is_open = true;
                                egui::Window::new(
                                    egui::RichText::new(titulo)
                                        .color(egui::Color32::RED)
                                        .strong(),
                                )
                                .open(&mut is_open)
                                .collapsible(false)
                                .resizable(false)
                                .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
                                .show(ctx, |ui| {
                                    ui.label(mensagem);
                                    ui.add_space(20.0);
                                    ui.horizontal_centered(|ui| {
                                        if ui.button("Fechar").clicked() {
                                            *self.estado.lock().unwrap() = EstadoLogin::Ocioso;
                                        }
                                    });
                                });
                                if !is_open {
                                    *self.estado.lock().unwrap() = EstadoLogin::Ocioso;
                                }
                            }

                            ui.add_space(ui.available_height() - 40.0);
                            ui.separator();
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("© RAG - 2025")
                                        .color(egui::Color32::GRAY)
                                        .size(12.0),
                                );
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "Versão: {}",
                                                env!("CARGO_PKG_VERSION")
                                            ))
                                            .color(egui::Color32::GRAY)
                                            .size(12.0),
                                        );
                                    },
                                );
                            });
                        });
                    });
            });
    }

    fn iniciar_processo_login(&mut self, ctx: &egui::Context) {
        if self.nome_usuario.is_empty() || self.senha.is_empty() {
            *self.estado.lock().unwrap() = EstadoLogin::Falha {
                titulo: "Campos Inválidos".to_string(),
                mensagem: "Usuário e senha não podem estar vazios.".to_string(),
            };
            return;
        }

        *self.estado.lock().unwrap() = EstadoLogin::EmProgresso;

        let estado_clone = self.estado.clone();
        let nome_usuario_clone = self.nome_usuario.clone();
        let senha_clone = self.senha.clone();
        let ctx_clone = ctx.clone();
        let endereco_servidor = self.endereco_servidor.lock().unwrap().clone();

        // Use o executor compartilhado para evitar spawn ilimitado de threads.
        crate::executor::spawn(move || {
            let url = format!("{}/login", endereco_servidor);
            let client = crate::http_client::get_client();
            let response = client
                .post(&url)
                .json(&serde_json::json!({ "usuario": nome_usuario_clone, "senha": senha_clone }))
                .timeout(Duration::from_secs(5))
                .send();

            let mut estado = estado_clone.lock().unwrap();
            *estado = match response {
                Ok(res) => {
                    if res.status().is_success() {
                        match res.json::<LoginResponse>() {
                            Ok(login_res) => EstadoLogin::Sucesso(login_res.papel),
                            Err(e) => EstadoLogin::Falha {
                                titulo: "Erro de Resposta".to_string(),
                                mensagem: format!(
                                    "Falha ao processar a resposta do servidor: {}",
                                    e
                                ),
                            },
                        }
                    } else {
                        EstadoLogin::Falha {
                            titulo: "Erro de Autenticação".to_string(),
                            mensagem: "Usuário ou senha inválidos.".to_string(),
                        }
                    }
                }
                Err(e) => EstadoLogin::Falha {
                    titulo: "Erro de Conexão".to_string(),
                    mensagem: format!("Não foi possível conectar ao servidor em '{}': {}", url, e),
                },
            };
            ctx_clone.request_repaint();
        });
    }
}
