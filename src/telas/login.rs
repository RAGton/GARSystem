// src/telas/login.rs
//
// MVP refactor (2026-10-09): visual redesenhado com tema GAR + background customizado.
// Lógica de autenticação preservada integralmente.
// Spec: docs/superpowers/specs/2026-10-09-mvp-login-tema.md

#[allow(unused_imports)]
use crate::http_client::{STORAGE_KEY_PAPEL, STORAGE_KEY_TOKEN};
use crate::servicos::{ErroAplicacao, PapelUsuario};
use crate::telas::theme::{
    cores as cores_tema,
    espacamento::{SP_LG, SP_MD, SP_SM, SP_XL, SP_XS, SP_XXL},
    tipografia,
};
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
    token: String,
    papel: PapelUsuario,
    #[allow(dead_code)]
    expira_em: i64,
}

pub struct TelaLogin {
    nome_usuario: String,
    senha: String,
    lembrar_usuario: bool,
    last_lembrar: bool,
    estado: Arc<Mutex<EstadoLogin>>,
    endereco_servidor: Arc<Mutex<String>>,
    /// Token JWT. Setado após login bem-sucedido e persistido no storage.
    token_jwt: Arc<Mutex<Option<String>>>,
    ir_para_configuracao: bool,
}

impl TelaLogin {
    pub fn new(
        endereco_servidor: Arc<Mutex<String>>,
        token_jwt: Arc<Mutex<Option<String>>>,
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
            token_jwt,
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
        let form_disabled = matches!(estado_atual, EstadoLogin::EmProgresso);

        // === PAINEL ESQUERDO: branding + background gradient ===
        // O gradient é aplicado via Frame::fill() — isso garante cobertura total.
        // Detalhe do gradient: da cor azul profundo (topo) pro ink (base) com leve vinheta.
        let bg_azul = Color32::from_rgb(0, 26, 77);   // #001A4D blue-900
        let bg_ink = Color32::from_rgb(10, 10, 15);   // #0A0A0F ink

        let side_response = egui::SidePanel::left("painel_branding_login")
            .resizable(false)
            .exact_width(380.0)
            .frame(egui::Frame::default().fill(bg_ink))
            .show(ctx, |ui| {
                // Pega a área VISÍVEL do painel (todo o retângulo, não só o disponivel)
                let rect = ui.max_rect();
                let painter = ui.painter_at(rect);

                // Gradient vertical: 100% azul no topo, 100% ink no fundo.
                // Implementação: pintamos 50 "fatias" horizontais com cor interpolada.
                let slices = 60;
                for i in 0..slices {
                    let t = i as f32 / slices as f32;
                    let r = (bg_azul.r() as f32 * (1.0 - t) + bg_ink.r() as f32 * t) as u8;
                    let g = (bg_azul.g() as f32 * (1.0 - t) + bg_ink.g() as f32 * t) as u8;
                    let b = (bg_azul.b() as f32 * (1.0 - t) + bg_ink.b() as f32 * t) as u8;
                    let slice = egui::Rect::from_min_max(
                        egui::pos2(rect.left(), rect.top() + t * rect.height()),
                        egui::pos2(rect.right(), rect.top() + (t + 1.0 / slices as f32) * rect.height()),
                    );
                    painter.rect_filled(slice, 0.0, Color32::from_rgb(r, g, b));
                }

                // Vinheta: cantos escurecidos (4 retângulos com alpha)
                let vinheta = Color32::from_black_alpha(100);
                let cs = 150.0;
                for (x, y, w, h) in [
                    (rect.left(), rect.top(), cs, cs),
                    (rect.right() - cs, rect.top(), cs, cs),
                    (rect.left(), rect.bottom() - cs, cs, cs),
                    (rect.right() - cs, rect.bottom() - cs, cs, cs),
                ] {
                    painter.rect_filled(
                        egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(w, h)),
                        0.0,
                        vinheta,
                    );
                }

                ui.allocate_ui_with_layout(
                    rect.size(),
                    egui::Layout::top_down(egui::Align::Center),
                    |ui| {
                        ui.add_space(SP_XXL * 2.0);
                        if let Some(tex) = logo {
                            ui.add(
                                egui::Image::new(tex)
                                    .fit_to_exact_size(egui::vec2(280.0, 158.0))
                                    .maintain_aspect_ratio(true),
                            );
                        }
                        ui.add_space(SP_XL);
                        ui.label(
                            egui::RichText::new("Sistema de gestão integrado")
                                .color(cores_tema::TEXT_SECONDARY)
                                .size(tipografia::FONT_BODY),
                        );
                        ui.label(
                            egui::RichText::new("para assistência técnica")
                                .color(cores_tema::TEXT_SECONDARY)
                                .size(tipografia::FONT_BODY),
                        );
                        // Versão fica só no footer do painel direito (com copyright)
                    },
                );
            });
        let _ = side_response; // silence unused

        // === PAINEL DIREITO: formulário ===
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(cores_tema::BG))
            .show(ctx, |ui| {
                // Botão de configuração (canto superior direito)
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    if ui
                        .add(
                            egui::Button::new(
                                egui::RichText::new("⚙")
                                    .size(tipografia::FONT_HEAD)
                                    .color(cores_tema::TEXT_SECONDARY),
                            )
                            .frame(false),
                        )
                        .on_hover_text("Configurar Servidor")
                        .clicked()
                    {
                        self.ir_para_configuracao = true;
                    }
                });
                ui.add_space(SP_XL);

                // Container do form (centrado verticalmente)
                ui.vertical_centered(|ui| {
                    ui.set_max_width(360.0);
                    // Centraliza vertical: usa min_height igual à área disponível
                    // e adiciona o form no centro
                    let total_h = ui.available_height() - 60.0; // reserva pro footer
                    let top_pad = total_h * 0.18;
                    ui.add_space(top_pad);

                    ui.label(
                        egui::RichText::new("Bem-vindo de volta")
                            .color(cores_tema::TEXT_PRIMARY)
                            .size(tipografia::FONT_DISPLAY)
                            .strong(),
                    );
                    ui.add_space(SP_SM);
                    ui.label(
                        egui::RichText::new("Acesse sua conta para continuar")
                            .color(cores_tema::TEXT_SECONDARY)
                            .size(tipografia::FONT_BODY),
                    );
                    ui.add_space(SP_XXL);

                    // === CAMPOS ===
                    ui.add_enabled_ui(!form_disabled, |ui| {
                        // Usuário
                        ui.label(
                            egui::RichText::new("Usuário")
                                .color(cores_tema::TEXT_SECONDARY)
                                .size(tipografia::FONT_LABEL),
                        );
                        ui.add_space(SP_XS);
                        let user_resp = ui.add(
                            egui::TextEdit::singleline(&mut self.nome_usuario)
                                .hint_text("seu usuário")
                                .desired_width(f32::INFINITY)
                                .margin(egui::Margin::same(SP_MD as i8)),
                        );

                        ui.add_space(SP_LG);

                        // Senha
                        ui.label(
                            egui::RichText::new("Senha")
                                .color(cores_tema::TEXT_SECONDARY)
                                .size(tipografia::FONT_LABEL),
                        );
                        ui.add_space(SP_XS);
                        let pass_resp = ui.add(
                            egui::TextEdit::singleline(&mut self.senha)
                                .password(true)
                                .hint_text("••••••••")
                                .desired_width(f32::INFINITY)
                                .margin(egui::Margin::same(SP_MD as i8)),
                        );

                        ui.add_space(SP_MD);

                        // Lembrar de mim
                        let resp = ui.checkbox(
                            &mut self.lembrar_usuario,
                            egui::RichText::new(" Lembrar de mim")
                                .color(cores_tema::TEXT_SECONDARY)
                                .size(tipografia::FONT_BODY),
                        );
                        if resp.changed() {
                            if let Some(storage) = frame.storage_mut() {
                                self.salvar_estado_login(storage);
                                self.last_lembrar = self.lembrar_usuario;
                            }
                        }

                        // Tab nav
                        if user_resp.lost_focus()
                            && ui.input(|i| i.key_pressed(egui::Key::Enter))
                        {
                            ui.memory_mut(|m| m.request_focus(pass_resp.id));
                        }
                        if pass_resp.lost_focus()
                            && ui.input(|i| i.key_pressed(egui::Key::Enter))
                        {
                            self.iniciar_processo_login(ctx);
                        }

                        ui.add_space(SP_XL);

                        // === BOTÃO ENTRAR (primário, full width) ===
                        let btn = egui::Button::new(
                            egui::RichText::new("Entrar")
                                .color(Color32::WHITE)
                                .size(tipografia::FONT_SUBHEAD)
                                .strong(),
                        )
                        .fill(cores_tema::PRIMARY)
                        .min_size(egui::vec2(0.0, 44.0));
                        if ui.add(btn).clicked() {
                            self.iniciar_processo_login(ctx);
                        }
                    });

                    // Spinner durante progresso
                    if let EstadoLogin::EmProgresso = estado_atual {
                        ui.add_space(SP_LG);
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.add_space(SP_SM);
                            ui.label(
                                egui::RichText::new("Entrando...")
                                    .color(cores_tema::TEXT_SECONDARY)
                                    .size(tipografia::FONT_BODY),
                            );
                        });
                    }

                    // === ERRO ===
                    if let EstadoLogin::Falha {
                        ref titulo,
                        ref mensagem,
                    } = estado_atual
                    {
                        let mut is_open = true;
                        egui::Window::new(
                            egui::RichText::new(titulo)
                                .color(cores_tema::ERROR)
                                .strong()
                                .size(tipografia::FONT_SUBHEAD),
                        )
                        .open(&mut is_open)
                        .collapsible(false)
                        .resizable(false)
                        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
                        .frame(
                            egui::Frame::window(&ctx.style())
                                .fill(cores_tema::SURFACE)
                                .stroke(egui::Stroke::new(1.0, cores_tema::BORDER_FOCUS.gamma_multiply(0.3)))
                                .corner_radius(egui::CornerRadius::same(12))
                                .inner_margin(egui::Margin::same(SP_XL as i8)),
                        )
                        .show(ctx, |ui| {
                            ui.set_max_width(360.0);
                            ui.label(
                                egui::RichText::new(mensagem)
                                    .color(cores_tema::TEXT_PRIMARY)
                                    .size(tipografia::FONT_BODY),
                            );
                            ui.add_space(SP_LG);
                            ui.horizontal(|ui| {
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new("Fechar")
                                                    .color(Color32::WHITE)
                                                    .size(tipografia::FONT_BODY),
                                            )
                                            .fill(cores_tema::PRIMARY)
                                            .min_size(egui::vec2(80.0, 32.0)),
                                        )
                                        .clicked()
                                    {
                                        *self.estado.lock().unwrap() = EstadoLogin::Ocioso;
                                    }
                                });
                            });
                        });
                        if !is_open {
                            *self.estado.lock().unwrap() = EstadoLogin::Ocioso;
                        }
                    }
                });

                // === FOOTER ===
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.add_space(SP_XXL);
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("© 2026 RAGton")
                                .color(cores_tema::TEXT_MUTED)
                                .size(tipografia::FONT_LABEL),
                        );
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                ui.label(
                                    egui::RichText::new(format!("v {}", env!("CARGO_PKG_VERSION")))
                                        .color(cores_tema::TEXT_MUTED)
                                        .size(tipografia::FONT_LABEL),
                                );
                            },
                        );
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
        let token_jwt_clone = self.token_jwt.clone();

        // Use o executor compartilhado para evitar spawn ilimitado de threads.
        crate::executor::spawn(move || {
            let url = format!("{}/login", endereco_servidor);
            let body = serde_json::json!({
                "usuario": nome_usuario_clone,
                "senha": senha_clone,
            });
            let response = crate::http_client::get_client()
                .post(&url)
                .json(&body)
                .timeout(Duration::from_secs(5))
                .send();

            let mut estado = estado_clone.lock().unwrap();
            *estado = match response {
                Ok(res) => {
                    if res.status().is_success() {
                        match res.json::<LoginResponse>() {
                            Ok(login_res) => {
                                // ✅ LOGIN OK: guarda o token em memória. O
                                // aplicacao.rs cuida da persistência no storage
                                // quando detecta o estado `Sucesso`.
                                if let Ok(mut g) = token_jwt_clone.lock() {
                                    *g = Some(login_res.token.clone());
                                }
                                EstadoLogin::Sucesso(login_res.papel)
                            }
                            Err(e) => EstadoLogin::Falha {
                                titulo: "Erro de Resposta".to_string(),
                                mensagem: format!(
                                    "Falha ao processar a resposta do servidor: {}",
                                    e
                                ),
                            },
                        }
                    } else {
                        // Anti-enumeração: mesma mensagem para qualquer 4xx do login.
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
