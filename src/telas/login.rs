// src/telas/login.rs

use crate::servicos::{ErroAplicacao, PapelUsuario};
use eframe::egui::{self, Align2, Color32, TextureHandle};
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use std::thread;
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

#[derive(Deserialize)]
struct UsuariosResponse {
    usuarios: Vec<String>,
}

pub struct TelaLogin {
    nome_usuario: String,
    senha: String,
    lembrar_usuario: bool,
    estado: Arc<Mutex<EstadoLogin>>,
    endereco_servidor: Arc<Mutex<String>>,
    ir_para_configuracao: bool,
    usuarios_sugeridos: Arc<Mutex<Option<Vec<String>>>>,
    busca_em_progresso: Arc<Mutex<bool>>,
}

impl TelaLogin {
    pub fn new(
        endereco_servidor: Arc<Mutex<String>>,
        nome_usuario: String,
        lembrar_usuario: bool,
    ) -> Self {
        Self {
            nome_usuario,
            senha: String::new(),
            lembrar_usuario,
            estado: Arc::new(Mutex::new(EstadoLogin::Ocioso)),
            endereco_servidor,
            ir_para_configuracao: false,
            usuarios_sugeridos: Arc::new(Mutex::new(None)),
            busca_em_progresso: Arc::new(Mutex::new(false)),
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
        let (left_panel_color, gradient_end_color) = if is_dark_mode {
            (
                Color32::from_rgb(20, 25, 40),
                Color32::from_rgba_premultiplied(20, 25, 40, 0),
            )
        } else {
            (
                Color32::from_rgb(30, 80, 180),
                Color32::from_rgba_premultiplied(30, 80, 180, 0),
            )
        };
        let panel_width = ctx.available_rect().width() / 2.5;

        egui::SidePanel::left("painel_branding")
            .resizable(false)
            .exact_width(panel_width)
            .frame(egui::Frame::default().fill(left_panel_color))
            .show(ctx, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.add_space(ui.available_height() * 0.2);
                    if let Some(logo_texture) = logo {
                        ui.add(egui::Image::new(logo_texture).max_size(egui::vec2(280.0, 158.0)));
                    }
                    ui.add_space(20.0);
                    ui.heading(
                        egui::RichText::new("Senior System")
                            .color(Color32::WHITE)
                            .size(32.0)
                            .strong(),
                    );
                    ui.add_space(10.0);
                    ui.label(
                        egui::RichText::new("Bem-vindo! Faça o login para continuar.")
                            .color(Color32::WHITE)
                            .italics()
                            .size(16.0),
                    );
                });

                let panel_rect = ui.min_rect();
                let gradient_width = 20.0;
                let gradient_rect = egui::Rect::from_min_max(
                    egui::pos2(panel_rect.right(), panel_rect.top()),
                    egui::pos2(panel_rect.right() + gradient_width, panel_rect.bottom()),
                );

                // CORREÇÃO: `add_quad` não existe mais. `add_colored_rect` com 4 cores é a forma moderna.
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
                // CORREÇÃO: `Shape::Mesh` espera um `Arc<Mesh>`. Convertemos com `.into()`.
                ui.painter().add(Shape::Mesh(mesh.into()));
            });

        egui::CentralPanel::default().show(ctx, |ui| {
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

                let formulario_habilitado = !matches!(estado_atual, EstadoLogin::EmProgresso);
                ui.add_enabled_ui(formulario_habilitado, |ui| {
                    egui::Frame::new()
                        .outer_margin(egui::Margin::symmetric(10, 0))
                        .show(ui, |ui| {
                            ui.set_max_width(320.0);
                            ui.label("Usuário:");
                            ui.add_space(4.0);
                            let user_input_response =
                                ui.text_edit_singleline(&mut self.nome_usuario);

                            if user_input_response.changed() {
                                if self.nome_usuario.len() > 1 {
                                    self.buscar_usuarios_por_prefixo(ctx);
                                } else {
                                    *self.usuarios_sugeridos.lock().unwrap() = None;
                                }
                            }
                            if user_input_response.gained_focus() && self.nome_usuario.len() > 1 {
                                self.buscar_usuarios_por_prefixo(ctx);
                            }

                            let usuarios_sugeridos_guard =
                                self.usuarios_sugeridos.lock().unwrap();
                            if let Some(sugestoes) = usuarios_sugeridos_guard.as_deref() {
                                if !sugestoes.is_empty() && user_input_response.has_focus() {
                                    // CORREÇÃO: `Area::new` espera um `Id`, que pode ser criado a partir de um `&str`.
                                    egui::Area::new("popup_sugestoes_area".into())
                                        .order(egui::Order::Tooltip)
                                        .fixed_pos(user_input_response.rect.left_bottom())
                                        .show(ctx, |ui| {
                                            egui::Frame::popup(ui.style()).show(ui, |ui| {
                                                ui.set_max_width(user_input_response.rect.width());
                                                egui::ScrollArea::vertical()
                                                    .max_height(100.0)
                                                    .show(ui, |ui| {
                                                        for sugestao in sugestoes {
                                                            if ui
                                                                .selectable_label(
                                                                    self.nome_usuario == *sugestao,
                                                                    sugestao,
                                                                )
                                                                .clicked()
                                                            {
                                                                self.nome_usuario =
                                                                    sugestao.clone();
                                                                *self
                                                                    .usuarios_sugeridos
                                                                    .lock()
                                                                    .unwrap() = None;
                                                            }
                                                        }
                                                    });
                                            });
                                        });
                                }
                            }
                            drop(usuarios_sugeridos_guard);

                            ui.add_space(15.0);
                            ui.label("Senha:");
                            ui.add_space(4.0);
                            let password_input_response =
                                ui.add(egui::TextEdit::singleline(&mut self.senha).password(true));

                            if user_input_response.lost_focus()
                                && ui.input(|i| i.key_pressed(egui::Key::Enter))
                            {
                                ui.memory_mut(|m| m.request_focus(password_input_response.id));
                            }
                            if password_input_response.lost_focus()
                                && ui.input(|i| i.key_pressed(egui::Key::Enter))
                            {
                                self.iniciar_processo_login(ctx, frame);
                            }

                            ui.add_space(10.0);
                            ui.checkbox(&mut self.lembrar_usuario, "Lembrar usuário");
                            ui.add_space(25.0);

                            ui.horizontal(|ui| {
                                if ui
                                    .button(egui::RichText::new("   Entrar   ").size(14.0))
                                    .clicked()
                                {
                                    self.iniciar_processo_login(ctx, frame);
                                }
                                if ui
                                    .button(egui::RichText::new(" Cancelar ").size(14.0))
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
                    egui::Window::new(egui::RichText::new(titulo).color(Color32::RED).strong())
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
                            .color(Color32::GRAY)
                            .size(12.0),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("Versão: {}", env!("CARGO_PKG_VERSION")))
                                .color(Color32::GRAY)
                                .size(12.0),
                        );
                    });
                });
            });
        });
    }

    fn iniciar_processo_login(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if self.nome_usuario.is_empty() || self.senha.is_empty() {
            *self.estado.lock().unwrap() = EstadoLogin::Falha {
                titulo: "Campos Inválidos".to_string(),
                mensagem: "Usuário e senha não podem estar vazios.".to_string(),
            };
            return;
        }

        if let Some(storage) = frame.storage_mut() {
            storage.set_string("lembrar_usuario", self.lembrar_usuario.to_string());
            if self.lembrar_usuario {
                storage.set_string("nome_usuario", self.nome_usuario.clone());
            } else {
                storage.set_string("nome_usuario", "".to_string());
            }
        }

        *self.estado.lock().unwrap() = EstadoLogin::EmProgresso;

        let estado_clone = self.estado.clone();
        let nome_usuario_clone = self.nome_usuario.clone();
        let senha_clone = self.senha.clone();
        let ctx_clone = ctx.clone();
        let endereco_servidor = self.endereco_servidor.lock().unwrap().clone();

        thread::spawn(move || {
            let url = format!("{}/login", endereco_servidor);
            let client = reqwest::blocking::Client::new();
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
                                mensagem: format!("Falha ao processar a resposta: {}", e),
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
                    mensagem: format!("Não foi possível conectar ao servidor: {}", e),
                },
            };
            ctx_clone.request_repaint();
        });
    }

    fn buscar_usuarios_por_prefixo(&self, ctx: &egui::Context) {
        if *self.busca_em_progresso.lock().unwrap() {
            return;
        }
        *self.busca_em_progresso.lock().unwrap() = true;

        let busca_clone = self.busca_em_progresso.clone();
        let sugestoes_clone = self.usuarios_sugeridos.clone();
        let ctx_clone = ctx.clone();
        let endereco_servidor = self.endereco_servidor.lock().unwrap().clone();
        let prefixo = self.nome_usuario.clone();

        thread::spawn(move || {
            let url = format!("{}/usuarios/{}", endereco_servidor, prefixo);
            let response = reqwest::blocking::get(&url);
            match response {
                Ok(res) => {
                    if res.status().is_success() {
                        if let Ok(data) = res.json::<UsuariosResponse>() {
                            *sugestoes_clone.lock().unwrap() = Some(data.usuarios);
                        }
                    }
                }
                Err(_) => { /* Ignora erros de conexão para não poluir a UI */ }
            }
            *busca_clone.lock().unwrap() = false;
            ctx_clone.request_repaint();
        });
    }
}
