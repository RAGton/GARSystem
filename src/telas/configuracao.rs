// src/telas/configuracao.rs

use eframe::egui;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const DISCOVERY_PORT: u16 = 3001;
const DISCOVERY_MESSAGE: &str = "SENIOR_SYSTEM_DISCOVERY_REQUEST";

#[derive(Clone, PartialEq, Debug)]
pub enum EstadoScanner {
    Ocioso,
    Escaneando,
    Sucesso(Vec<String>),
    Falha(String),
}

/// A tela de configuração do servidor.
pub struct TelaConfiguracao {
    /// O endereço do servidor que o usuário pode editar.
    pub endereco_servidor: String,
    /// O estado atual do scanner de rede.
    estado_scanner: Arc<Mutex<EstadoScanner>>,
    /// Indica se o usuário salvou a configuração.
    salvo: bool,
    /// Indica se o usuário quer voltar para a tela de login.
    voltar: bool,
}

impl TelaConfiguracao {
    pub fn new(endereco_atual: &str) -> Self {
        Self {
            endereco_servidor: endereco_atual.to_string(),
            estado_scanner: Arc::new(Mutex::new(EstadoScanner::Ocioso)),
            salvo: false,
            voltar: false,
        }
    }

    /// Retorna `true` se o usuário clicou em Salvar e reseta o estado.
    pub fn foi_salvo(&mut self) -> bool {
        if self.salvo {
            self.salvo = false; // Reseta para evitar loop
            true
        } else {
            false
        }
    }

    /// Retorna `true` se o usuário quer voltar para a tela de login.
    pub fn deve_voltar(&self) -> bool {
        self.voltar
    }

    pub fn update(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.add_space(ui.available_height() * 0.15);
                ui.heading("Configuração do Servidor");
                ui.add_space(30.0);

                // --- CORREÇÃO: `Frame::new()` e margens com inteiros ---
                let frame = egui::Frame::new().outer_margin(egui::Margin::symmetric(20, 0));

                frame.show(ui, |ui| {
                    ui.set_max_width(400.0);

                    // --- Seção de Configuração Manual ---
                    ui.label("Endereço do Servidor:");
                    ui.text_edit_singleline(&mut self.endereco_servidor);
                    ui.add_space(5.0);
                    ui.label(
                        egui::RichText::new("Ex: http://192.168.0.10:3000")
                            .small()
                            .color(egui::Color32::GRAY),
                    );
                    ui.add_space(20.0);

                    // --- Seção do Scanner de Rede ---
                    ui.separator();
                    ui.add_space(20.0);
                    ui.heading("Descoberta Automática na Rede");
                    ui.add_space(10.0);

                    let mut estado_scanner_guard = self.estado_scanner.lock().unwrap();
                    let escaneando = matches!(*estado_scanner_guard, EstadoScanner::Escaneando);

                    if ui
                        .add_enabled(!escaneando, egui::Button::new("Procurar Servidor na Rede"))
                        .clicked()
                    {
                        *estado_scanner_guard = EstadoScanner::Escaneando;
                        self.escanear_rede(ctx.clone());
                    }

                    ui.add_space(20.0);

                    // Mostra o resultado do scanner
                    match &*estado_scanner_guard {
                        EstadoScanner::Ocioso => {
                            ui.label("Clique no botão acima para encontrar servidores.");
                        }
                        EstadoScanner::Escaneando => {
                            ui.horizontal(|ui| {
                                ui.spinner();
                                ui.label("Procurando por servidores na rede local...");
                            });
                        }
                        EstadoScanner::Sucesso(servidores) => {
                            if servidores.is_empty() {
                                ui.label(
                                    egui::RichText::new("Nenhum servidor foi encontrado.")
                                        .color(egui::Color32::YELLOW),
                                );
                            } else {
                                ui.label("Servidores encontrados (clique para selecionar):");
                                egui::Frame::group(ui.style()).show(ui, |ui| {
                                    for servidor in servidores {
                                        let endereco_completo = format!("http://{}:3000", servidor);
                                        if ui
                                            .selectable_label(
                                                self.endereco_servidor == endereco_completo,
                                                &endereco_completo,
                                            )
                                            .clicked()
                                        {
                                            self.endereco_servidor = endereco_completo;
                                        }
                                    }
                                });
                            }
                        }
                        EstadoScanner::Falha(erro) => {
                            ui.label(
                                egui::RichText::new(format!("Erro ao escanear: {}", erro))
                                    .color(egui::Color32::RED),
                            );
                        }
                    }
                });

                ui.add_space(30.0);
                if ui.button("   Salvar e Voltar   ").clicked() {
                    self.salvo = true;
                    self.voltar = true;
                }
            });
        });
    }

    /// Inicia uma thread para escanear a rede por servidores.
    fn escanear_rede(&self, ctx: egui::Context) {
        let estado_clone = self.estado_scanner.clone();
        thread::spawn(move || {
            // --- CORREÇÃO: Lógica de `match` para capturar o resultado corretamente ---
            let resultado = match UdpSocket::bind("0.0.0.0:0") {
                Ok(socket) => {
                    socket
                        .set_broadcast(true)
                        .unwrap_or_else(|e| eprintln!("Falha ao setar broadcast: {}", e));
                    socket
                        .set_read_timeout(Some(Duration::from_secs(3)))
                        .unwrap_or_else(|e| eprintln!("Falha ao setar timeout: {}", e));

                    let broadcast_addr = format!("255.255.255.255:{}", DISCOVERY_PORT);
                    if let Err(e) = socket.send_to(DISCOVERY_MESSAGE.as_bytes(), &broadcast_addr) {
                        Err(format!("Falha ao enviar mensagem de descoberta: {}", e))
                    } else {
                        let mut buf = [0; 1024];
                        let mut servidores_encontrados = Vec::new();

                        while let Ok((_, addr)) = socket.recv_from(&mut buf) {
                            servidores_encontrados.push(addr.ip().to_string());
                        }

                        servidores_encontrados.sort();
                        servidores_encontrados.dedup();
                        Ok(servidores_encontrados)
                    }
                }
                Err(e) => Err(e.to_string()),
            };

            *estado_clone.lock().unwrap() = match resultado {
                Ok(servidores) => EstadoScanner::Sucesso(servidores),
                Err(e) => EstadoScanner::Falha(e),
            };
            ctx.request_repaint();
        });
    }
}
