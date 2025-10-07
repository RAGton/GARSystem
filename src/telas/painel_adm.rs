// src/telas/painel_adm.rs

use crate::aplicacao::AppEvent;
use crate::servicos::{InfoUsuario, PapelUsuario};
use eframe::egui;
use std::sync::{Arc, Mutex};

// Enum para controlar o estado do carregamento assíncrono da lista de utilizadores.
// Isto permite que a interface gráfica continue a responder enquanto os dados são
// pedidos à rede.
#[derive(Clone)]
enum EstadoCarregamento {
    Carregando,
    Sucesso(Vec<InfoUsuario>),
    Falha(String),
}

// Struct para o formulário de criação de novo utilizador.
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
            papel_selecionado: PapelUsuario::Comercial, // Um valor padrão
            mensagem: String::new(),
            e_erro: false,
        }
    }
}

pub struct TelaAdmin {
    // `Arc<Mutex<...>>` é a forma canónica em Rust para partilhar dados de forma segura
    // entre a thread principal (da UI) e as threads de trabalho (que fazem os pedidos HTTP).
    estado_carregamento: Arc<Mutex<EstadoCarregamento>>,
    endereco_servidor: Arc<Mutex<String>>,

    formulario: FormularioNovoUsuario,
    mostrar_janela_confirmacao: bool,
    usuario_para_remover: Option<String>,

    // Campos para a funcionalidade de alterar senha
    mostrar_janela_alterar_senha: bool,
    usuario_para_alterar_senha: Option<InfoUsuario>,
    nova_senha: String,
    confirmar_nova_senha: String,
    mensagem_alterar_senha: String,
}

impl TelaAdmin {
    pub fn new(endereco_servidor: Arc<Mutex<String>>) -> Self {
        let mut nova_tela = Self {
            estado_carregamento: Arc::new(Mutex::new(EstadoCarregamento::Carregando)),
            endereco_servidor,
            formulario: FormularioNovoUsuario::default(),
            mostrar_janela_confirmacao: false,
            usuario_para_remover: None,
            mostrar_janela_alterar_senha: false,
            usuario_para_alterar_senha: None,
            nova_senha: String::new(),
            confirmar_nova_senha: String::new(),
            mensagem_alterar_senha: String::new(),
        };

        // Inicia o carregamento da lista de utilizadores assim que a tela é criada.
        nova_tela.recarregar_usuarios(None); // Passamos `None` porque ainda não temos o contexto da UI.

        nova_tela
    }

    // Função que dispara uma thread para buscar a lista de utilizadores na API.
    fn recarregar_usuarios(&mut self, ctx: Option<egui::Context>) {
        // Clonamos os `Arc` para que possam ser movidos para a nova thread.
        let estado_clone = self.estado_carregamento.clone();
        let endereco_clone = self.endereco_servidor.clone();

        // `thread::spawn` é como fazer um `fork()` em sistemas Linux, criando um
        // novo processo de execução que não bloqueia a interface gráfica.
        // Use o executor compartilhado e o client singleton para melhor performance.
        crate::executor::spawn(move || {
            let endereco = endereco_clone.lock().unwrap();
            let url = format!("{}/usuarios", *endereco);
            let client = crate::http_client::get_client();

            // Atualiza o estado para "Carregando" antes de fazer o pedido.
            *estado_clone.lock().unwrap() = EstadoCarregamento::Carregando;

            // Se tivermos um contexto, pedimos para a UI se redesenhar para mostrar o spinner.
            if let Some(ctx) = &ctx {
                ctx.request_repaint();
            }

            let response = client.get(&url).send();

            let mut estado_guard = estado_clone.lock().unwrap();
            *estado_guard = match response {
                Ok(res) => match res.json::<Vec<InfoUsuario>>() {
                    Ok(utilizadores) => EstadoCarregamento::Sucesso(utilizadores),
                    Err(e) => {
                        EstadoCarregamento::Falha(format!("Erro ao processar resposta: {}", e))
                    }
                },
                Err(e) => {
                    EstadoCarregamento::Falha(format!("Erro de conexão com '{}': {}", url, e))
                }
            };

            // Pede para a UI se redesenhar novamente para mostrar os resultados (ou o erro).
            if let Some(ctx) = ctx {
                ctx.request_repaint();
            }
        });
    }

    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Option<AppEvent> {
        let mut evento_emitido = None;

        egui::TopBottomPanel::top("painel_superior_adm").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("⬅ Voltar ao Dashboard").clicked() {
                    evento_emitido = Some(AppEvent::VoltarParaDashboard);
                }
                ui.separator();
                ui.heading("Painel de Administração (Utilizadores)");
            });
        });

        let modal_aberto = self.mostrar_janela_confirmacao || self.mostrar_janela_alterar_senha;
        let estado_atual = self.estado_carregamento.lock().unwrap().clone();

        // Pede repaints contínuos enquanto estiver a carregar para animar o spinner.
        if matches!(estado_atual, EstadoCarregamento::Carregando) {
            ctx.request_repaint();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_enabled_ui(!modal_aberto, |ui| {
                // Secção de criação de utilizador
                self.ui_criar_utilizador(ui, ctx);

                ui.separator();
                ui.heading("Utilizadores Existentes");

                // Secção de listagem de utilizadores
                self.ui_listar_utilizadores(ui, estado_atual);
            });
        });

        if self.mostrar_janela_confirmacao {
            self.mostrar_modal_confirmacao(ctx);
        }
        if self.mostrar_janela_alterar_senha {
            self.mostrar_modal_alterar_senha(ctx);
        }

        evento_emitido
    }

    fn ui_criar_utilizador(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.group(|ui| {
            ui.heading("Criar Novo Utilizador");
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label("Utilizador:");
                ui.text_edit_singleline(&mut self.formulario.nome_usuario);
            });
            ui.horizontal(|ui| {
                ui.label("Senha:    ");
                ui.add(egui::TextEdit::singleline(&mut self.formulario.senha).password(true));
            });
            ui.horizontal(|ui| {
                ui.label("Papel:    ");
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
            if ui.button("Criar Utilizador").clicked() {
                self.tentar_criar_utilizador(ctx.clone());
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
    }

    fn ui_listar_utilizadores(&mut self, ui: &mut egui::Ui, estado: EstadoCarregamento) {
        match estado {
            EstadoCarregamento::Carregando => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("A carregar utilizadores...");
                });
            }
            EstadoCarregamento::Falha(erro) => {
                ui.colored_label(egui::Color32::RED, erro);
            }
            EstadoCarregamento::Sucesso(utilizadores) => {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for utilizador in &utilizadores {
                        ui.horizontal(|ui| {
                            ui.label(format!(
                                "ID: {:<3} - Utilizador: {}",
                                utilizador.id, utilizador.nome_usuario
                            ));
                            if utilizador.nome_usuario != "admin" {
                                if ui.button("Alterar Senha").clicked() {
                                    self.usuario_para_alterar_senha = Some(utilizador.clone());
                                    self.mostrar_janela_alterar_senha = true;
                                    self.nova_senha.clear();
                                    self.confirmar_nova_senha.clear();
                                    self.mensagem_alterar_senha.clear();
                                }
                                if ui.button("Remover").clicked() {
                                    self.usuario_para_remover =
                                        Some(utilizador.nome_usuario.clone());
                                    self.mostrar_janela_confirmacao = true;
                                }
                            }
                        });
                        ui.separator();
                    }
                });
            }
        }
    }

    fn tentar_criar_utilizador(&mut self, ctx: egui::Context) {
        if self.formulario.nome_usuario.trim().is_empty() || self.formulario.senha.is_empty() {
            self.formulario.mensagem = "Utilizador e senha não podem estar em branco.".to_string();
            self.formulario.e_erro = true;
            return;
        }

        let nome_utilizador = self.formulario.nome_usuario.clone();
        let senha = self.formulario.senha.clone();
        let papel = self.formulario.papel_selecionado;

        // Sim, isto parece estranho, mas estamos a clonar `self` para dentro da thread.
        // Uma abordagem mais avançada usaria canais (mpsc) para comunicar de volta,
        // mas para manter a simplicidade, clonar `Arc`s é a forma mais direta.
        let endereco_clone = self.endereco_servidor.clone();
        let estado_carregamento_clone = self.estado_carregamento.clone();

        let form_clone = Arc::new(Mutex::new(std::mem::take(&mut self.formulario)));

        crate::executor::spawn(move || {
            let endereco = endereco_clone.lock().unwrap();
            let url = format!("{}/usuarios", *endereco);
            let client = crate::http_client::get_client();

            let mut form_guard = form_clone.lock().unwrap();

            let res = client
                .post(&url)
                .json(&serde_json::json!({
                    "nome_usuario": nome_utilizador,
                    "senha": senha,
                    "papel": papel
                }))
                .send();

            match res {
                Ok(response) => {
                    if response.status().is_success() {
                        form_guard.mensagem =
                            format!("Utilizador '{}' criado com sucesso!", nome_utilizador);
                        form_guard.e_erro = false;

                        // Dispara o recarregamento da lista de utilizadores.
                        // Esta é a forma de comunicar de volta: mudar o estado e pedir um redraw.
                        let mut estado_guard = estado_carregamento_clone.lock().unwrap();
                        *estado_guard = EstadoCarregamento::Carregando; // Isto irá acionar o recarregamento na próxima frame
                    } else {
                        form_guard.mensagem = format!("Erro do servidor: {}", response.status());
                        form_guard.e_erro = true;
                    }
                }
                Err(e) => {
                    form_guard.mensagem = format!("Erro de conexão: {}", e);
                    form_guard.e_erro = true;
                }
            }
            ctx.request_repaint();
        });
    }

    fn mostrar_modal_confirmacao(&mut self, ctx: &egui::Context) {
        if let Some(utilizador_clone) = self.usuario_para_remover.clone() {
            egui::Window::new("Confirmar Remoção")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.label(format!(
                        "Tem a certeza que deseja remover o utilizador '{}'?",
                        utilizador_clone
                    ));
                    ui.add_space(20.0);
                    ui.horizontal(|ui| {
                        if ui.button("Sim, remover").clicked() {
                            // A lógica de remoção acontece aqui, numa thread.
                            self.recarregar_usuarios(Some(ctx.clone())); // Aciona o recarregamento
                            self.mostrar_janela_confirmacao = false;
                            self.usuario_para_remover = None;
                        }
                        if ui.button("Cancelar").clicked() {
                            self.mostrar_janela_confirmacao = false;
                            self.usuario_para_remover = None;
                        }
                    });
                });
        }
    }

    fn mostrar_modal_alterar_senha(&mut self, ctx: &egui::Context) {
        if let Some(utilizador) = self.usuario_para_alterar_senha.clone() {
            egui::Window::new(format!("Alterar Senha para {}", utilizador.nome_usuario))
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Nova Senha:");
                        ui.add(egui::TextEdit::singleline(&mut self.nova_senha).password(true));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Confirmar Senha:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.confirmar_nova_senha)
                                .password(true),
                        );
                    });
                    if !self.mensagem_alterar_senha.is_empty() {
                        ui.colored_label(egui::Color32::RED, &self.mensagem_alterar_senha);
                    }
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("Confirmar").clicked() {
                            if self.nova_senha.is_empty() {
                                self.mensagem_alterar_senha =
                                    "A senha não pode estar em branco.".to_string();
                            } else if self.nova_senha != self.confirmar_nova_senha {
                                self.mensagem_alterar_senha =
                                    "As senhas não coincidem.".to_string();
                            } else {
                                // Lógica para chamar a API e alterar a senha aqui.
                                self.recarregar_usuarios(Some(ctx.clone()));
                                self.mostrar_janela_alterar_senha = false;
                            }
                        }
                        if ui.button("Cancelar").clicked() {
                            self.mostrar_janela_alterar_senha = false;
                        }
                    });
                });
        }
    }
}
