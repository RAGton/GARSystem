// src/telas/login.rs

use crate::servicos::{ErroAplicacao, PapelUsuario};
use eframe::egui::{self, ColorImage, TextureHandle};
use std::sync::mpsc::{Receiver, Sender};

fn carregar_imagem_de_arquivo(caminho: &std::path::Path) -> Result<ColorImage, image::ImageError> {
    let imagem = image::io::Reader::open(caminho)?.decode()?;
    let tamanho = [imagem.width() as _, imagem.height() as _];
    let buffer_imagem = imagem.to_rgba8();
    let pixels = buffer_imagem.as_flat_samples();
    Ok(ColorImage::from_rgba_unmultiplied(
        tamanho,
        pixels.as_slice(),
    ))
}

#[derive(Debug, PartialEq)]
pub(crate) enum EstadoLogin {
    Ocioso,
    EmProgresso,
    Sucesso,
    Falha(String),
}

pub struct TelaLogin {
    nome_usuario: String,
    senha: String,
    estado: EstadoLogin,
    envio_db: Sender<(String, String)>,
    resultado_login: Option<Result<PapelUsuario, ErroAplicacao>>,
    logo: Option<TextureHandle>,
}

impl TelaLogin {
    pub fn new(envio_db: Sender<(String, String)>) -> Self {
        Self {
            nome_usuario: String::new(),
            senha: String::new(),
            estado: EstadoLogin::Ocioso,
            envio_db,
            resultado_login: None,
            logo: None,
        }
    }

    pub fn obter_resultado_login(&mut self) -> Option<Result<PapelUsuario, ErroAplicacao>> {
        self.resultado_login.take()
    }

    pub fn update(
        &mut self,
        ctx: &egui::Context,
        _frame: &mut eframe::Frame,
        recebimento_db: &Receiver<Result<PapelUsuario, ErroAplicacao>>,
    ) {
        if self.logo.is_none() {
            if let Ok(imagem) =
                carregar_imagem_de_arquivo(std::path::Path::new("./assets/logo.png"))
            {
                self.logo = Some(ctx.load_texture("logo-empresa", imagem, Default::default()));
            }
        }

        if let Ok(resultado) = recebimento_db.try_recv() {
            match &resultado {
                Ok(_) => self.estado = EstadoLogin::Sucesso,
                Err(erro) => self.estado = EstadoLogin::Falha(erro.to_string()),
            }
            self.resultado_login = Some(resultado);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.add_space(ui.available_height() * 0.1);
                if let Some(logo) = &self.logo {
                    ui.add(egui::Image::new(logo).max_size(egui::vec2(280.0, 158.0)));
                }
                ui.add_space(20.0);
                ui.label(
                    egui::RichText::new("Bem-vindo! Faça o login para continuar.")
                        .italics()
                        .size(16.0),
                );
                ui.add_space(30.0);

                ui.add_enabled_ui(!matches!(self.estado, EstadoLogin::EmProgresso), |ui| {
                    egui::Frame::new().show(ui, |ui| {
                        ui.set_max_width(300.0);
                        ui.label("Usuário:");
                        ui.text_edit_singleline(&mut self.nome_usuario);
                        ui.add_space(10.0);
                        ui.label("Senha:");
                        ui.add(egui::TextEdit::singleline(&mut self.senha).password(true));
                        ui.add_space(20.0);

                        ui.horizontal(|ui| {
                            let texto_botao = match self.estado {
                                EstadoLogin::EmProgresso => "Entrando...",
                                _ => "   Entrar   ",
                            };
                            let frame_botao_entrar = egui::Frame::new()
                                .inner_margin(egui::vec2(12.0, 6.0))
                                .fill(ui.style().visuals.selection.bg_fill)
                                .corner_radius(5.0);
                            if frame_botao_entrar
                                .show(ui, |ui| {
                                    ui.add(
                                        egui::Button::new(
                                            egui::RichText::new(texto_botao)
                                                .color(ui.style().visuals.selection.stroke.color)
                                                .strong(),
                                        )
                                        .frame(false),
                                    )
                                })
                                .inner
                                .clicked()
                            {
                                self.estado = EstadoLogin::EmProgresso;
                                let _ = self
                                    .envio_db
                                    .send((self.nome_usuario.clone(), self.senha.clone()));
                            }
                            if ui.button("Cancelar").clicked() {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        });
                    });
                });

                if let EstadoLogin::Falha(msg_erro) = &self.estado {
                    ui.add_space(10.0);
                    ui.label(egui::RichText::new(msg_erro).color(egui::Color32::RED));
                }
                ui.add_space(ui.available_height() - 40.0);
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("© RAG - 2025")
                            .color(egui::Color32::GRAY)
                            .size(20.0),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("Versão: {}", env!("CARGO_PKG_VERSION")))
                                .color(egui::Color32::GRAY)
                                .size(20.0),
                        );
                    });
                });
            });
        });
        if self.estado == EstadoLogin::EmProgresso {
            ctx.request_repaint();
        }
    }
}
