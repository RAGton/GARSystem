// src/telas/painel_tecnico.rs

use crate::aplicacao::AppEvent;
use crate::servicos::{OrdemServico, StatusOS};
use eframe::egui;
use std::sync::{Arc, Mutex};

// O enum `AcaoTecnico` foi removido.

pub struct TelaTecnico {
    ordens: Vec<OrdemServico>,
    ordem_selecionada: Option<u32>,
    filtro_busca: String,
}

impl TelaTecnico {
    pub fn new(_endereco_servidor: Arc<Mutex<String>>) -> Self {
        // Dados de exemplo para popular a tela.
        let ordens_mock = vec![
            OrdemServico {
                id: 101,
                cliente: "Ana Silva".into(),
                equipamento: "Notebook Dell Vostro".into(),
                defeito_relatado: "Não liga, sem sinal de LED.".into(),
                status: StatusOS::Aberta,
            },
            OrdemServico {
                id: 102,
                cliente: "Bruno Costa".into(),
                equipamento: "PC Gamer".into(),
                defeito_relatado: "Tela azul ao iniciar jogos pesados.".into(),
                status: StatusOS::EmAndamento,
            },
            OrdemServico {
                id: 103,
                cliente: "Carla Dias".into(),
                equipamento: "Impressora HP".into(),
                defeito_relatado: "Atolando papel constantemente.".into(),
                status: StatusOS::AguardandoPeca,
            },
            OrdemServico {
                id: 104,
                cliente: "Daniel Farias".into(),
                equipamento: "Macbook Pro 2019".into(),
                defeito_relatado: "Teclado com falha em algumas teclas.".into(),
                status: StatusOS::Finalizada,
            },
            OrdemServico {
                id: 105,
                cliente: "Empresa XYZ".into(),
                equipamento: "Servidor Rack".into(),
                defeito_relatado: "Fonte redundante queimou.".into(),
                status: StatusOS::Cancelada,
            },
        ];

        Self {
            ordens: ordens_mock,
            ordem_selecionada: None,
            filtro_busca: String::new(),
        }
    }

    // [CORREÇÃO] A função `update` agora retorna `Option<AppEvent>`.
    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Option<AppEvent> {
        let mut evento_emitido = None;

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

                        for os in &self.ordens {
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
                                // [CORREÇÃO] O botão agora emite o evento para abrir o editor.
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
