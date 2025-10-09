// src/telas/painel_tecnico.rs

use crate::aplicacao::AppEvent;
use crate::servicos::OrdemServico;
use eframe::egui;
use std::sync::{Arc, Mutex};

// O enum `AcaoTecnico` foi removido.

pub struct TelaTecnico {
    ordens: Arc<Mutex<Vec<OrdemServico>>>,
    ordem_selecionada: Option<u32>,
    filtro_busca: String,
    carregando: Arc<Mutex<bool>>,
}

impl TelaTecnico {
    pub fn new(_endereco_servidor: Arc<Mutex<String>>) -> Self {
        Self {
            ordens: Arc::new(Mutex::new(Vec::new())),
            ordem_selecionada: None,
            filtro_busca: String::new(),
            carregando: Arc::new(Mutex::new(false)),
        }
    }

    // [CORREÇÃO] A função `update` agora retorna `Option<AppEvent>`.
    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Option<AppEvent> {
        let mut evento_emitido = None;

        // Se ainda não carregamos as ordens, disparamos o carregamento em background.
        let precisa_carregar = {
            let ord_guard = self.ordens.lock().unwrap();
            let carreg = *self.carregando.lock().unwrap();
            ord_guard.is_empty() && !carreg
        };
        if precisa_carregar {
            // marca que estamos carregando
            *self.carregando.lock().unwrap() = true;
            let ord_clone = Arc::clone(&self.ordens);
            let carreg_clone = Arc::clone(&self.carregando);
            let ctx_clone = ctx.clone();
            crate::executor::spawn(move || {
                match crate::servicos::listar_ordens_servico() {
                    Ok(res) => {
                        let mut guard = ord_clone.lock().unwrap();
                        *guard = res;
                    }
                    Err(e) => {
                        eprintln!("Erro ao carregar ordens de serviço: {}", e);
                    }
                }
                *carreg_clone.lock().unwrap() = false;
                ctx_clone.request_repaint();
            });
        }

        egui::TopBottomPanel::top("painel_superior_tecnico").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("⬅ Voltar ao Dashboard").clicked() {
                    // [CORREÇÃO] Emite o evento para voltar.
                    evento_emitido = Some(AppEvent::VoltarParaDashboard);
                }
                ui.separator();
                ui.heading("Painel Técnico - Ordens de Serviço");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Buscar OS:");
                ui.text_edit_singleline(&mut self.filtro_busca);
                if ui.button("Limpar").clicked() {
                    self.filtro_busca.clear();
                }
            });
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("grid_os")
                    .num_columns(5)
                    .striped(true)
                    .spacing([20.0, 8.0])
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("ID").strong());
                        ui.label(egui::RichText::new("Cliente").strong());
                        ui.label(egui::RichText::new("Equipamento").strong());
                        ui.label(egui::RichText::new("Status").strong());
                        ui.label(""); // Coluna para o botão de ação
                        ui.end_row();

                        let ords = { self.ordens.lock().unwrap().clone() };
                        for os in &ords {
                            if self.filtro_busca.is_empty()
                                || os
                                    .cliente
                                    .to_lowercase()
                                    .contains(&self.filtro_busca.to_lowercase())
                            {
                                ui.label(os.id.to_string());
                                ui.label(&os.cliente);
                                ui.label(&os.equipamento);
                                ui.label(format!("{:?}", os.status));
                                if ui.button("Gerenciar OS ⚙️").clicked() {
                                    evento_emitido = Some(AppEvent::AbrirEditorOS(os.id));
                                }
                                ui.end_row();
                            }
                        }
                    });
            });
        });

        // O modal de detalhes pode ser removido ou mantido, mas a navegação principal
        // agora é feita através do evento `AbrirEditorOS`.

        evento_emitido
    }
}
