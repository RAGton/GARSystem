// src/telas/painel_adm.rs

use crate::aplicacao::AppEvent;
use crate::servicos::{InfoUsuario, PapelUsuario};
use eframe::egui;
use std::sync::{Arc, Mutex};
use std::thread;

// [NOVO] Enum para controlar o estado de carregamento da lista de usuários.
#[derive(Clone)]
enum EstadoCarregamento {
    Carregando,
    Sucesso(Vec<InfoUsuario>),
    Falha(String),
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
    // [NOVO] O estado da tela agora controla o carregamento assíncrono.
    estado_carregamento: Arc<Mutex<EstadoCarregamento>>,

    formulario: FormularioNovoUsuario,
    mostrar_janela_confirmacao: bool,
    usuario_para_remover: Option<String>,
    mostrar_janela_alterar_senha: bool,
    usuario_para_alterar_senha: Option<InfoUsuario>,
    nova_senha: String,
    confirmar_nova_senha: String,
    mensagem_alterar_senha: String,
}

impl TelaAdmin {
    pub fn new() -> Self {
        let estado_carregamento = Arc::new(Mutex::new(EstadoCarregamento::Carregando));
        let estado_clone = estado_carregamento.clone();
        let ctx_clone = eframe::egui::Context::default(); // Precisamos de um contexto para redesenhar

        // [NOVO] Inicia uma thread para buscar os usuários da API assim que a tela é criada.
        thread::spawn(move || {
            let client = reqwest::blocking::Client::new();
            let response = client.get("http://localhost:3000/usuarios").send();

            let mut estado = estado_clone.lock().unwrap();
            match response {
                Ok(res) => match res.json::<Vec<InfoUsuario>>() {
                    Ok(usuarios) => *estado = EstadoCarregamento::Sucesso(usuarios),
                    Err(e) => {
                        *estado =
                            EstadoCarregamento::Falha(format!("Erro ao processar usuários: {}", e))
                    }
                },
                Err(e) => *estado = EstadoCarregamento::Falha(format!("Erro de conexão: {}", e)),
            }
            ctx_clone.request_repaint(); // Pede para a UI redesenhar com os novos dados
        });

        Self {
            estado_carregamento,
            formulario: FormularioNovoUsuario::default(),
            mostrar_janela_confirmacao: false,
            usuario_para_remover: None,
            mostrar_janela_alterar_senha: false,
            usuario_para_alterar_senha: None,
            nova_senha: String::new(),
            confirmar_nova_senha: String::new(),
            mensagem_alterar_senha: String::new(),
        }
    }

    // As outras funções permanecem as mesmas, mas precisarão ser adaptadas para usar HTTP no futuro.
    fn recarregar_usuarios(&mut self) { /* ... */
    }
    fn tentar_criar_usuario(&mut self) { /* ... */
    }
    fn mostrar_modal_confirmacao(&mut self, ctx: &egui::Context) { /* ... */
    }
    fn mostrar_modal_alterar_senha(&mut self, ctx: &egui::Context) { /* ... */
    }

    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Option<AppEvent> {
        let mut evento_emitido = None;

        egui::TopBottomPanel::top("painel_superior_adm").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("⬅ Voltar ao Dashboard").clicked() {
                    evento_emitido = Some(AppEvent::VoltarParaDashboard);
                }
                ui.separator();
                ui.heading("Painel de Administração (Usuários)");
            });
        });

        let modal_aberto = self.mostrar_janela_confirmacao || self.mostrar_janela_alterar_senha;
        let estado_atual = self.estado_carregamento.lock().unwrap().clone();

        ctx.request_repaint(); // Pede para a UI redesenhar continuamente enquanto carrega

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_enabled_ui(!modal_aberto, |ui| {
                // ... (UI de criar novo usuário permanece a mesma) ...

                ui.separator();
                ui.heading("Usuários Existentes");

                // [NOVO] Mostra feedback de carregamento ou a lista de usuários.
                match estado_atual {
                    EstadoCarregamento::Carregando => {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Carregando usuários...");
                        });
                    }
                    EstadoCarregamento::Falha(erro) => {
                        ui.label(egui::RichText::new(erro).color(egui::Color32::RED));
                    }
                    EstadoCarregamento::Sucesso(usuarios) => {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            let mut usuario_a_remover = None;
                            let mut usuario_a_alterar_senha = None;

                            for usuario in &usuarios {
                                ui.horizontal(|ui| {
                                    ui.label(format!(
                                        "ID: {:<3} - Usuário: {}",
                                        usuario.id, usuario.nome_usuario
                                    ));

                                    if usuario.nome_usuario != "admin" {
                                        if ui.button("Alterar Senha").clicked() {
                                            usuario_a_alterar_senha = Some(usuario.clone());
                                        }
                                        if ui.button("Remover").clicked() {
                                            usuario_a_remover = Some(usuario.nome_usuario.clone());
                                        }
                                    }
                                });
                                ui.separator();
                            }

                            if let Some(nome) = usuario_a_remover {
                                self.usuario_para_remover = Some(nome);
                                self.mostrar_janela_confirmacao = true;
                            }
                            if let Some(usuario) = usuario_a_alterar_senha {
                                self.usuario_para_alterar_senha = Some(usuario);
                                self.mostrar_janela_alterar_senha = true;
                                self.nova_senha.clear();
                                self.confirmar_nova_senha.clear();
                                self.mensagem_alterar_senha.clear();
                            }
                        });
                    }
                }
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
}
