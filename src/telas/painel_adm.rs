// src/telas/painel_adm.rs

use crate::servicos::{self, InfoUsuario, PapelUsuario};
use eframe::egui;

pub enum AcaoAdmin {
    Nenhuma,
   // Deslogar,
    Voltar,
}

// ... (struct FormularioNovoUsuario e impl Default não mudam) ...
struct FormularioNovoUsuario {
    nome_usuario: String,
    senha: String,
    papel_selecionado: PapelUsuario,
    mensagem: String,
    e_erro: bool,
}

impl Default for FormularioNovoUsuario {
    fn default() -> Self {
        Self {
            nome_usuario: String::new(),
            senha: String::new(),
            papel_selecionado: PapelUsuario::Vendedor,
            mensagem: String::new(),
            e_erro: false,
        }
    }
}

pub struct TelaAdmin {
    usuarios: Vec<InfoUsuario>,
    formulario: FormularioNovoUsuario,
    mostrar_janela_confirmacao: bool,
    usuario_para_remover: Option<String>,
}

impl TelaAdmin {
    pub fn new() -> Self {
        Self {
            usuarios: servicos::listar_usuarios(),
            formulario: FormularioNovoUsuario::default(),
            mostrar_janela_confirmacao: false,
            usuario_para_remover: None,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> AcaoAdmin {
        let mut acao = AcaoAdmin::Nenhuma;

        egui::TopBottomPanel::top("painel_superior_adm").show(ctx, |ui| {
            ui.horizontal(|ui| {
                // O botão de voltar na barra de topo global já faz isso, mas podemos manter por redundância.
                if ui.button("⬅ Voltar").clicked() {
                    acao = AcaoAdmin::Voltar;
                }
                ui.separator();
                ui.heading("Painel de Administração");
            });
        });

        let modal_aberto = self.mostrar_janela_confirmacao;
        egui::CentralPanel::default().show(ctx, |ui| {
            // Nova API para habilitar/desabilitar
            ui.add_enabled_ui(!modal_aberto, |ui| {
                ui.group(|ui| {
                    ui.heading("Criar Novo Usuário");
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Usuário:").strong());
                        ui.text_edit_singleline(&mut self.formulario.nome_usuario);
                    });
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Senha:   ").strong());
                        ui.add(
                            egui::TextEdit::singleline(&mut self.formulario.senha).password(true),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Papel:   ").strong());
                        egui::ComboBox::from_label("")
                            .selected_text(format!("{:?}", self.formulario.papel_selecionado))
                            .show_ui(ui, |ui| {
                                for papel in PapelUsuario::todos() {
                                    ui.selectable_value(
                                        &mut self.formulario.papel_selecionado,
                                        *papel,
                                        format!("{:?}", papel),
                                    );
                                }
                            });
                    });
                    ui.add_space(10.0);
                    if ui.button("Criar Usuário").clicked() {
                        let form = &mut self.formulario;
                        if !form.nome_usuario.is_empty() && !form.senha.is_empty() {
                            match servicos::criar_usuario(
                                &form.nome_usuario,
                                &form.senha,
                                form.papel_selecionado,
                            ) {
                                Ok(_) => {
                                    form.mensagem =
                                        format!("Usuário '{}' criado!", form.nome_usuario);
                                    form.e_erro = false;
                                    self.usuarios = servicos::listar_usuarios();
                                    *form = FormularioNovoUsuario::default();
                                }
                                Err(e) => {
                                    form.mensagem = e.to_string();
                                    form.e_erro = true;
                                }
                            }
                        } else {
                            form.mensagem = "Usuário e senha são obrigatórios.".to_string();
                            form.e_erro = true;
                        }
                    }
                    if !self.formulario.mensagem.is_empty() {
                        let cor = if self.formulario.e_erro {
                            egui::Color32::RED
                        } else {
                            egui::Color32::GREEN
                        };
                        ui.label(egui::RichText::new(&self.formulario.mensagem).color(cor));
                    }
                });

                ui.separator();
                ui.heading("Usuários Existentes");

                egui::ScrollArea::vertical().show(ui, |ui| {
                    let mut usuario_marcado_para_remocao = None;
                    for usuario in &self.usuarios {
                        ui.horizontal(|ui| {
                            ui.label(format!(
                                "ID: {} - Usuário: {}",
                                usuario.id, usuario.nome_usuario
                            ));
                            if usuario.nome_usuario != "admin" {
                                if ui.button("Remover").clicked() {
                                    usuario_marcado_para_remocao =
                                        Some(usuario.nome_usuario.clone());
                                }
                            }
                        });
                        ui.separator();
                    }
                    if let Some(nome_usuario) = usuario_marcado_para_remocao {
                        self.usuario_para_remover = Some(nome_usuario);
                        self.mostrar_janela_confirmacao = true;
                    }
                });
            });
        });

        if self.mostrar_janela_confirmacao {
            if let Some(usuario_clone) = self.usuario_para_remover.clone() {
                egui::Window::new("Confirmar Remoção")
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                    .show(ctx, |ui| {
                        ui.label(format!(
                            "Tem certeza que deseja remover o usuário '{}'?",
                            usuario_clone
                        ));
                        ui.add_space(20.0);
                        ui.horizontal(|ui| {
                            if ui.button("Sim, remover").clicked() {
                                match servicos::remover_usuario(&usuario_clone) {
                                    Ok(_) => {
                                        self.formulario.mensagem =
                                            format!("Usuário '{}' removido.", usuario_clone);
                                        self.formulario.e_erro = false;
                                        self.usuarios = servicos::listar_usuarios();
                                    }
                                    Err(e) => {
                                        self.formulario.mensagem = e.to_string();
                                        self.formulario.e_erro = true;
                                    }
                                }
                                self.mostrar_janela_confirmacao = false;
                                self.usuario_para_remover = None;
                            }
                            if ui.button("Cancelar").clicked() {
                                self.mostrar_janela_confirmacao = false;
                                self.usuario_para_remover = None;
                            }
                        });
                    });
            } else {
                self.mostrar_janela_confirmacao = false;
            }
        }
        acao
    }
}
