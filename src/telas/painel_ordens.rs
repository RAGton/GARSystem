// src/telas/painel_ordens.rs

use crate::aplicacao::AppEvent;
use crate::servicos::{OrdemServico, PapelUsuario};
use eframe::egui;
use std::sync::{Arc, Mutex};

pub struct TelaOrdens {
    ordens: Arc<Mutex<Vec<OrdemServico>>>,
    filtro_busca: String,
    carregando: Arc<Mutex<bool>>,
    papel: PapelUsuario,
}

impl TelaOrdens {
    pub fn new(papel: PapelUsuario, _endereco_servidor: Arc<Mutex<String>>) -> Self {
        Self {
            ordens: Arc::new(Mutex::new(Vec::new())),
            filtro_busca: String::new(),
            carregando: Arc::new(Mutex::new(false)),
            papel,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Option<AppEvent> {
        let mut evento_emitido = None;

        // Carrega ordens em background se necessário
        let precisa_carregar = {
            let ord_guard = self.ordens.lock().unwrap();
            let carreg = *self.carregando.lock().unwrap();
            ord_guard.is_empty() && !carreg
        };
        if precisa_carregar {
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

        egui::TopBottomPanel::top("top_ordens").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("⬅ Voltar ao Dashboard").clicked() {
                    evento_emitido = Some(AppEvent::VoltarParaDashboard);
                }
                ui.separator();
                ui.heading("Ordens de Serviço");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Buscar:");
                ui.text_edit_singleline(&mut self.filtro_busca);
                if ui.button("Limpar").clicked() {
                    self.filtro_busca.clear();
                }
                // Botão criar dentro da área de busca (visível para Comercial/Administrador)
                use crate::servicos::PapelUsuario::*;
                if matches!(self.papel, Comercial | Administrador) {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("➕ Criar OS").clicked() {
                            evento_emitido =
                                Some(AppEvent::NavegarPara(crate::aplicacao::TelaAtiva::CriarOs));
                        }
                    });
                }
            });
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("grid_os_list")
                    .num_columns(6)
                    .striped(true)
                    .spacing([12.0, 8.0])
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("ID").strong());
                        ui.label(egui::RichText::new("Cliente").strong());
                        ui.label(egui::RichText::new("Equipamento").strong());
                        ui.label(egui::RichText::new("Status").strong());
                        ui.label(egui::RichText::new("Técnico").strong());
                        ui.label("");
                        ui.end_row();

                        let ords = { self.ordens.lock().unwrap().clone() };
                        for os in &ords {
                            if self.filtro_busca.is_empty()
                                || os
                                    .cliente
                                    .to_lowercase()
                                    .contains(&self.filtro_busca.to_lowercase())
                                || os
                                    .equipamento
                                    .to_lowercase()
                                    .contains(&self.filtro_busca.to_lowercase())
                            {
                                ui.label(os.id.to_string());
                                ui.label(&os.cliente);
                                ui.label(&os.equipamento);
                                ui.label(format!("{:?}", os.status));
                                ui.label(&os.nome_tecnico_responsavel);

                                // Ações por papel
                                match self.papel {
                                    PapelUsuario::Tecnico => {
                                        if ui.button("Gerenciar").clicked() {
                                            evento_emitido = Some(AppEvent::AbrirEditorOS(os.id));
                                        }
                                    }
                                    PapelUsuario::Comercial | PapelUsuario::Administrador => {
                                        ui.horizontal(|ui| {
                                            if ui.button("Abrir").clicked() {
                                                evento_emitido =
                                                    Some(AppEvent::AbrirEditorOS(os.id));
                                            }
                                            if ui.button("✚ Criar").clicked() {
                                                // abre a tela de criação
                                                evento_emitido = Some(AppEvent::NavegarPara(
                                                    crate::aplicacao::TelaAtiva::CriarOs,
                                                ));
                                            }
                                        });
                                    }
                                    _ => {
                                        if ui.button("Ver").clicked() {
                                            evento_emitido = Some(AppEvent::AbrirEditorOS(os.id));
                                        }
                                    }
                                }

                                ui.end_row();
                            }
                        }
                    });
            });
        });

        evento_emitido
    }
}
