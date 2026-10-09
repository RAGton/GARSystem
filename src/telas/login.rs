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
    empresa: String,
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
            empresa: String::new(),
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

        // === PAINEL ESQUERDO: branding + background premium ===
        // Spec "Redesign do painel esquerdo" (2026-10-09): gradiente azul-marinho
        // com transições suaves + iluminação radial cyan sutil + logo + hierarquia
        // tipográfica + rodapé institucional. Painel DIREITO não é tocado.
        let bg_azul = Color32::from_rgb(0, 26, 77);    // #001A4D blue-900
        let bg_meio = Color32::from_rgb(8, 16, 40);   // intermediário
        let bg_ink = Color32::from_rgb(10, 10, 15);    // #0A0A0F ink

        let side_response = egui::SidePanel::left("painel_branding_login")
            .resizable(false)
            .exact_width(420.0)
            .frame(egui::Frame::default().fill(bg_ink))
            .show(ctx, |ui| {
                // Pega a área VISÍVEL do painel (todo o retângulo, não só o disponivel)
                let rect = ui.max_rect();
                let painter = ui.painter_at(rect);

                // === GRADIENTE AZUL-MARINHO ===
                // 3 estágios: blue-900 no topo → meio no centro → ink no fundo.
                // Implementação: 80 fatias horizontais com interpolação em 2 segmentos.
                let slices = 80;
                for i in 0..slices {
                    let t = i as f32 / (slices - 1) as f32;
                    // interpola entre (azul → meio) nos primeiros 60% e (meio → ink) nos 40% finais
                    let (r, g, b) = if t < 0.6 {
                        let u = t / 0.6;
                        (
                            lerp_u8(bg_azul.r(), bg_meio.r(), u),
                            lerp_u8(bg_azul.g(), bg_meio.g(), u),
                            lerp_u8(bg_azul.b(), bg_meio.b(), u),
                        )
                    } else {
                        let u = (t - 0.6) / 0.4;
                        (
                            lerp_u8(bg_meio.r(), bg_ink.r(), u),
                            lerp_u8(bg_meio.g(), bg_ink.g(), u),
                            lerp_u8(bg_meio.b(), bg_ink.b(), u),
                        )
                    };
                    let slice = egui::Rect::from_min_max(
                        egui::pos2(rect.left(), rect.top() + t * rect.height()),
                        egui::pos2(rect.right(), rect.top() + (t + 1.0 / slices as f32) * rect.height()),
                    );
                    painter.rect_filled(slice, 0.0, Color32::from_rgb(r, g, b));
                }

                // === ILUMINAÇÃO RADIAL CYAN SUTIL ===
                // Halo cyan no topo-centro (alpha 5%), cria profundidade.
                let glow_center = egui::pos2(rect.center().x, rect.top() + 80.0);
                let glow_radius = 280.0;
                let glow_color = cores_tema::CYAN_GLOW.linear_multiply(0.05);
                painter.add(egui::Shape::circle_filled(glow_center, glow_radius, glow_color));

                // === VINHETA MULTI-CAMADA ===
                // 4 cantos escurecidos, cada canto com 3 retângulos de alpha decrescente
                // (efeito de fade radial aproximado sem shader).
                let cs = 180.0;
                for (x, y) in [
                    (rect.left(), rect.top()),
                    (rect.right() - cs, rect.top()),
                    (rect.left(), rect.bottom() - cs),
                    (rect.right() - cs, rect.bottom() - cs),
                ] {
                    for i in 0..3 {
                        let sz = cs - (i as f32) * 50.0;
                        let off = (i as f32) * 50.0;
                        let alpha = 110 - (i * 35);
                        let c = Color32::from_black_alpha(alpha.clamp(0, 255) as u8);
                        let corner = egui::Rect::from_min_size(
                            egui::pos2(x + off, y + off),
                            egui::vec2(sz, sz),
                        );
                        painter.rect_filled(corner, 0.0, c);
                    }
                }

                // === CONTEÚDO DO PAINEL ESQUERDO ===
                // Hierarquia: logo (display) + título + descrição + meta
                ui.allocate_ui_with_layout(
                    rect.size(),
                    egui::Layout::top_down(egui::Align::Center),
                    |ui| {
                        ui.add_space(SP_XXL * 2.5);

                        // === LOGOTIPO ===
                        // Logo GAR original, ampliado, com glow cyan sutil atrás.
                        if let Some(tex) = logo {
                            let logo_size = egui::vec2(320.0, 181.0);
                            let logo_rect = egui::Rect::from_center_size(
                                rect.center() + egui::vec2(0.0, -rect.height() * 0.18),
                                logo_size,
                            );
                            // Halo cyan (8% alpha) atrás do logo
                            let glow = cores_tema::CYAN_GLOW.linear_multiply(0.08);
                            painter.add(egui::Shape::circle_filled(logo_rect.center(), 180.0, glow));
                            ui.add(
                                egui::Image::new(tex)
                                    .fit_to_exact_size(logo_size)
                                    .maintain_aspect_ratio(true),
                            );
                        }
                        ui.add_space(SP_XL);

                        // === MENSAGEM PRINCIPAL (display) ===
                        ui.label(
                            egui::RichText::new("Gestão inteligente.")
                                .color(cores_tema::TEXT_PRIMARY)
                                .size(20.0)
                                .strong(),
                        );
                        ui.label(
                            egui::RichText::new("Operação integrada.")
                                .color(cores_tema::TEXT_PRIMARY)
                                .size(20.0)
                                .strong(),
                        );
                        ui.add_space(SP_MD);
                        // === DESCRIÇÃO (body) ===
                        ui.label(
                            egui::RichText::new("Uma plataforma para organizar clientes,")
                                .color(cores_tema::TEXT_SECONDARY)
                                .size(tipografia::FONT_BODY),
                        );
                        ui.label(
                            egui::RichText::new("serviços técnicos e operações em um só lugar.")
                                .color(cores_tema::TEXT_SECONDARY)
                                .size(tipografia::FONT_BODY),
                        );
                    },
                );

                // === BADGE DE LICENCIAMENTO (rodapé do painel esquerdo) ===
                ui.with_layout(
                    egui::Layout::bottom_up(egui::Align::Center),
                    |ui| {
                        ui.add_space(SP_XXL + SP_LG);
                        // Badge: ● Licenciado (com glow verde sutil)
                        ui.horizontal(|ui| {
                            let badge_pos = ui.next_widget_position();
                            // Glow verde
                            let badge_glow = cores_tema::SUCCESS.linear_multiply(0.15);
                            painter.add(egui::Shape::circle_filled(
                                badge_pos + egui::vec2(6.0, 6.0),
                                16.0,
                                badge_glow,
                            ));
                            // Ponto verde
                            painter.add(egui::Shape::circle_filled(
                                badge_pos + egui::vec2(6.0, 6.0),
                                5.0,
                                cores_tema::SUCCESS,
                            ));
                            ui.add_space(20.0);
                            ui.label(
                                egui::RichText::new("Licenciado")
                                    .color(cores_tema::SUCCESS)
                                    .size(tipografia::FONT_LABEL)
                                    .strong(),
                            );
                        });
                        ui.add_space(SP_SM);
                        ui.label(
                            egui::RichText::new("CNPJ 12.345.678/0001-90")
                                .color(cores_tema::TEXT_MUTED)
                                .size(tipografia::FONT_LABEL),
                        );
                    },
                );
            });
        let _ = side_response; // silence unused

        // === PAINEL DIREITO: formulário ===
        egui::CentralPanel::default()
            .frame(
                egui::Frame::default()
                    .fill(cores_tema::BG)
                    .inner_margin(egui::Margin {
                        left: SP_XXL as i8,
                        right: SP_XXL as i8,
                        top: SP_XXL as i8,
                        bottom: 80, // reserva pro footer (60px + respiro)
                    }),
            )
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
                ui.add_space(SP_SM);

                // Container do form (centrado verticalmente)
                ui.vertical_centered(|ui| {
                    ui.set_max_width(360.0);
                    // Centraliza vertical: usa min_height igual à área disponível
                    // e adiciona o form no centro
                    let total_h = ui.available_height() - 60.0; // reserva pro footer
                    let top_pad = total_h * 0.20;
                    ui.add_space(top_pad);

                    ui.label(
                        egui::RichText::new("Bem-vindo de volta")
                            .color(cores_tema::TEXT_PRIMARY)
                            .size(tipografia::FONT_DISPLAY)
                            .strong(),
                    );
                    ui.add_space(SP_XS);
                    ui.label(
                        egui::RichText::new("Acesse sua conta para continuar")
                            .color(cores_tema::TEXT_SECONDARY)
                            .size(tipografia::FONT_BODY),
                    );
                    ui.add_space(SP_XXL + SP_LG);

                    // === DIVIDER SUTIL ===
                    ui.add(egui::Separator::default().spacing(SP_SM));
                    ui.add_space(SP_XL);

                    // === CAMPOS ===
                    ui.add_enabled_ui(!form_disabled, |ui| {
                        // Empresa (multi-tenant: seleção de CNPJ)
                        use crate::telas::login_empresas::{EMPRESAS_DEMO, EmpresaOpcao};
                        ui.label(
                            egui::RichText::new("Empresa")
                                .color(cores_tema::TEXT_SECONDARY)
                                .size(tipografia::FONT_LABEL)
                                .strong(),
                        );
                        ui.add_space(SP_XS);
                        // ComboBox com a lista de empresas demo.
                        // Em produção, isso vira `egui::ComboBox::from_label` populado
                        // por uma chamada ao backend (GET /tenants/active).
                        let empresa_frame = egui::Frame::default()
                            .fill(cores_tema::SURFACE_ELEV)
                            .stroke(egui::Stroke::new(1.0, cores_tema::BORDER))
                            .corner_radius(egui::CornerRadius::same(8))
                            .inner_margin(egui::Margin::symmetric(SP_MD as i8, SP_XS as i8));
                        empresa_frame.show(ui, |ui| {
                            // Encontra o índice atual baseado no slug armazenado
                            let selected_label: String = if self.empresa.is_empty() {
                                "Selecione a empresa...".to_string()
                            } else {
                                EMPRESAS_DEMO
                                    .iter()
                                    .find(|e| e.slug == self.empresa)
                                    .map(|e| format!("{} ({})", e.razao_social, e.cnpj))
                                    .unwrap_or_else(|| self.empresa.clone())
                            };
                            egui::ComboBox::from_id_salt("empresa_select")
                                .selected_text(selected_label)
                                .height(SP_XL * 2.0)
                                .show_ui(ui, |ui| {
                                    for emp in EMPRESAS_DEMO.iter() {
                                        let label = format!("{} — {}", emp.razao_social, emp.cnpj);
                                        let response = ui.selectable_label(
                                            self.empresa == emp.slug,
                                            label,
                                        );
                                        if response.clicked() {
                                            self.empresa = emp.slug.to_string();
                                        }
                                    }
                                });
                        });

                        ui.add_space(SP_LG);

                        // Usuário
                        ui.label(
                            egui::RichText::new("Usuário")
                                .color(cores_tema::TEXT_SECONDARY)
                                .size(tipografia::FONT_LABEL)
                                .strong(),
                        );
                        ui.add_space(SP_XS);
                        let user_frame = egui::Frame::default()
                            .fill(cores_tema::SURFACE_ELEV)
                            .stroke(egui::Stroke::new(1.0, cores_tema::BORDER))
                            .corner_radius(egui::CornerRadius::same(8))
                            .inner_margin(egui::Margin::symmetric(SP_MD as i8, SP_LG as i8));
                        let user_resp = user_frame.show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut self.nome_usuario)
                                    .hint_text(
                                        egui::RichText::new("seu usuário")
                                            .color(cores_tema::TEXT_MUTED)
                                            .size(tipografia::FONT_BODY),
                                    )
                                    .desired_width(f32::INFINITY)
                                    .frame(false)
                                    .text_color(cores_tema::TEXT_PRIMARY),
                            )
                        }).inner;

                        ui.add_space(SP_LG);

                        // Senha
                        ui.label(
                            egui::RichText::new("Senha")
                                .color(cores_tema::TEXT_SECONDARY)
                                .size(tipografia::FONT_LABEL)
                                .strong(),
                        );
                        ui.add_space(SP_XS);
                        let pass_frame = egui::Frame::default()
                            .fill(cores_tema::SURFACE_ELEV)
                            .stroke(egui::Stroke::new(1.0, cores_tema::BORDER))
                            .corner_radius(egui::CornerRadius::same(8))
                            .inner_margin(egui::Margin::symmetric(SP_MD as i8, SP_LG as i8));
                        let pass_resp = pass_frame.show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut self.senha)
                                    .password(true)
                                    .hint_text(
                                        egui::RichText::new("••••••••")
                                            .color(cores_tema::TEXT_MUTED)
                                            .size(tipografia::FONT_BODY),
                                    )
                                    .desired_width(f32::INFINITY)
                                    .frame(false)
                                    .text_color(cores_tema::TEXT_PRIMARY),
                            )
                        }).inner;

                        ui.add_space(SP_MD);

                        // Lembrar de mim (alinhado à esquerda, mesmo X dos campos)
                        let resp = ui.checkbox(
                            &mut self.lembrar_usuario,
                            egui::RichText::new("  Lembrar de mim")
                                .color(cores_tema::TEXT_SECONDARY)
                                .size(tipografia::FONT_BODY),
                        );
                        if resp.changed() {
                            if let Some(storage) = frame.storage_mut() {
                                self.salvar_estado_login(storage);
                                self.last_lembrar = self.lembrar_usuario;
                            }
                        }

                        // Tab nav (empresa → usuário via Enter ou Tab natural)
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

                        // === BOTÃO ENTRAR (primário, full width, 56px) ===
                        let btn = egui::Button::new(
                            egui::RichText::new("Entrar  →")
                                .color(Color32::WHITE)
                                .size(tipografia::FONT_SUBHEAD)
                                .strong(),
                        )
                        .fill(cores_tema::PRIMARY)
                        .min_size(egui::vec2(0.0, 56.0))
                        .corner_radius(egui::CornerRadius::same(10));
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
                    ui.add_space(SP_XXL + SP_LG);
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
                    ui.add_space(SP_XS);
                    // Práticas egui (Emil): warning visível em dev + link pro código + Powered by
                    egui::warn_if_debug_build(ui);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(SP_SM, 0.0);
                        powered_by_egui_and_eframe(ui);
                        ui.add(egui::Hyperlink::from_label_and_url(
                            "source",
                            "https://github.com/RAGton/GARSystem",
                        ));
                    });
                });
            });
    }
}

/// Interpolação linear entre dois u8 (0-255). `t` em [0.0, 1.0].
fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 * (1.0 - t) + b as f32 * t).round() as u8
}

/// Prática egui (Emil Ernerfeldt): "Powered by egui + eframe" no rodapé
/// Reconhece o framework + dá crédito. Idiomático em apps egui.
fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" + ");
        ui.hyperlink_to("eframe", "https://github.com/emilk/egui/tree/master/crates/eframe");
    });
}

impl TelaLogin {
    fn iniciar_processo_login(&mut self, ctx: &egui::Context) {
        if self.empresa.is_empty() || self.nome_usuario.is_empty() || self.senha.is_empty() {
            *self.estado.lock().unwrap() = EstadoLogin::Falha {
                titulo: "Campos Inválidos".to_string(),
                mensagem: "Empresa, usuário e senha não podem estar vazios.".to_string(),
            };
            return;
        }

        *self.estado.lock().unwrap() = EstadoLogin::EmProgresso;

        let estado_clone = self.estado.clone();
        let empresa_clone = self.empresa.clone();
        let nome_usuario_clone = self.nome_usuario.clone();
        let senha_clone = self.senha.clone();
        let ctx_clone = ctx.clone();
        let endereco_servidor = self.endereco_servidor.lock().unwrap().clone();
        let token_jwt_clone = self.token_jwt.clone();

        // Use o executor compartilhado para evitar spawn ilimitado de threads.
        crate::executor::spawn(move || {
            let url = format!("{}/login", endereco_servidor);
            let body = serde_json::json!({
                "empresa": empresa_clone,
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
