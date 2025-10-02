use crate::servicos::{self, InfoUsuario, PapelUsuario};
use eframe::egui;

pub enum AcaoAdmin {
    Nenhuma,
    Voltar,
}

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

    fn recarregar_usuarios(&mut self) {
        self.usuarios = servicos::listar_usuarios();
    }

    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> AcaoAdmin {
        let mut acao = AcaoAdmin::Nenhuma;

        egui::TopBottomPanel::top("painel_superior_adm").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("⬅ Voltar ao Dashboard").clicked() {
                    acao = AcaoAdmin::Voltar;
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
                        // CORRIGIDO: Aviso de depreciação. `from_id_source` renomeado para `from_id_salt`.
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

        acao
    }

    // CORRIGIDO: A lógica foi reestruturada para evitar o erro de empréstimo (borrowing error).
    fn tentar_criar_usuario(&mut self) {
        if self.formulario.nome_usuario.trim().is_empty() || self.formulario.senha.is_empty() {
            self.formulario.mensagem = "Usuário e senha não podem estar em branco.".to_string();
            self.formulario.e_erro = true;
            return;
        }

        // 1. Copiamos os dados necessários para variáveis locais.
        // Isso libera `self.formulario` de qualquer empréstimo.
        let nome_usuario = self.formulario.nome_usuario.clone();
        let senha = self.formulario.senha.clone();
        let papel = self.formulario.papel_selecionado;

        // 2. Chamamos o serviço com os dados copiados.
        match servicos::criar_usuario(&nome_usuario, &senha, papel) {
            Ok(_) => {
                // 3. Como `self` está livre, agora podemos chamá-lo de forma mutável.
                self.formulario.mensagem =
                    format!("Usuário '{}' criado com sucesso!", nome_usuario);
                self.formulario.e_erro = false;
                self.recarregar_usuarios(); // Este &mut self agora é válido.
                self.formulario = FormularioNovoUsuario::default(); // E este também.
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
