// src/telas/painel_adm.rs

use crate::aplicacao::AppEvent;
use crate::servicos::{self, InfoUsuario, PapelUsuario};
use eframe::egui;

// Este enum não é mais necessário na nova arquitetura de eventos.
// pub enum AcaoAdmin {
//     Nenhuma,
//     Voltar,
// }

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
            papel_selecionado: PapelUsuario::Comercial,
            mensagem: String::new(),
            e_erro: false,
        }
    }
}

// [CORREÇÃO] A struct agora é pública (`pub`), tornando-a visível para o `aplicacao.rs`.
pub struct TelaAdmin {
    usuarios: Vec<InfoUsuario>,
    formulario: FormularioNovoUsuario,
    mostrar_janela_confirmacao: bool,
    usuario_para_remover: Option<String>,
}

impl TelaAdmin {
    pub fn new() -> Self {
        // NOTA: Na arquitetura cliente-servidor, esta chamada direta será substituída
        // por uma requisição HTTP para buscar os usuários quando a tela for aberta.
        Self {
            usuarios: servicos::listar_usuarios(),
            formulario: FormularioNovoUsuario::default(),
            mostrar_janela_confirmacao: false,
            usuario_para_remover: None,
        }
    }

    fn recarregar_usuarios(&mut self) {
        // Esta função também será substituída por uma requisição HTTP.
        self.usuarios = servicos::listar_usuarios();
    }

    // [CORREÇÃO] A função agora retorna `Option<AppEvent>` para se integrar ao sistema de eventos.
    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Option<AppEvent> {
        let mut evento_emitido = None;

        egui::TopBottomPanel::top("painel_superior_adm").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("⬅ Voltar ao Dashboard").clicked() {
                    // Emite o evento para voltar, em vez de retornar um enum local.
                    evento_emitido = Some(AppEvent::VoltarParaDashboard);
                }
                ui.separator();
                ui.heading("Painel de Administração");
            });
        });

        let modal_aberto = self.mostrar_janela_confirmacao;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_enabled_ui(!modal_aberto, |ui| {
                ui.group(|ui| {
                    ui.heading("Criar Novo Usuário");
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.label("Usuário:");
                        ui.text_edit_singleline(&mut self.formulario.nome_usuario);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Senha:  ");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.formulario.senha).password(true),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Papel:  ");
                        // [CORREÇÃO] `from_id` trocado pela API correta `from_id_salt`.
                        egui::ComboBox::from_id_salt("combo_papel")
                            .selected_text(format!("{:?}", self.formulario.papel_selecionado))
                            .show_ui(ui, |ui| {
                                for papel in PapelUsuario::iter() {
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
                        self.tentar_criar_usuario();
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
                    let mut usuario_a_remover = None;
                    for usuario in &self.usuarios {
                        ui.horizontal(|ui| {
                            ui.label(format!(
                                "ID: {:<3} - Usuário: {}",
                                usuario.id, usuario.nome_usuario
                            ));
                            if usuario.nome_usuario != "admin" && ui.button("Remover").clicked() {
                                usuario_a_remover = Some(usuario.nome_usuario.clone());
                            }
                        });
                        ui.separator();
                    }
                    if let Some(nome) = usuario_a_remover {
                        self.usuario_para_remover = Some(nome);
                        self.mostrar_janela_confirmacao = true;
                    }
                });
            });
        });

        if self.mostrar_janela_confirmacao {
            self.mostrar_modal_confirmacao(ctx);
        }

        evento_emitido
    }

    // NOTA: Esta função precisará ser reescrita para usar `reqwest` e chamar a API do backend.
    fn tentar_criar_usuario(&mut self) {
        if self.formulario.nome_usuario.trim().is_empty() || self.formulario.senha.is_empty() {
            self.formulario.mensagem = "Usuário e senha não podem estar em branco.".to_string();
            self.formulario.e_erro = true;
            return;
        }

        let nome_usuario = self.formulario.nome_usuario.clone();
        let senha = self.formulario.senha.clone();
        let papel = self.formulario.papel_selecionado;

        // Lógica temporária. No futuro, isto será uma chamada HTTP.
        match servicos::criar_usuario(&nome_usuario, &senha, papel) {
            Ok(_) => {
                self.formulario.mensagem =
                    format!("Usuário '{}' criado com sucesso!", nome_usuario);
                self.formulario.e_erro = false;
                self.recarregar_usuarios();
                self.formulario = FormularioNovoUsuario::default();
            }
            Err(e) => {
                self.formulario.mensagem = e.to_string();
                self.formulario.e_erro = true;
            }
        }
    }

    fn mostrar_modal_confirmacao(&mut self, ctx: &egui::Context) {
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
                            // NOTA: Esta lógica também será uma chamada HTTP no futuro.
                            match servicos::remover_usuario(&usuario_clone) {
                                Ok(_) => {
                                    self.formulario.mensagem =
                                        format!("Usuário '{}' removido.", usuario_clone);
                                    self.formulario.e_erro = false;
                                    self.recarregar_usuarios();
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
}
