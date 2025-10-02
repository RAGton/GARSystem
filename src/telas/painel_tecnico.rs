// src/telas/painel_tecnico.rs

use crate::servicos::{OrdemServico, StatusOS};
use eframe::egui;

pub enum AcaoTecnico {
    Nenhuma,
    Voltar,
}

pub struct TelaTecnico {
    ordens: Vec<OrdemServico>,
    ordem_selecionada: Option<u32>,
    filtro_busca: String,
}

impl TelaTecnico {
    pub fn new() -> Self {
        // Dados de exemplo para popular a tela.
        // No futuro, isso virá do banco de dados.
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

    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> AcaoTecnico {
        let mut acao = AcaoTecnico::Nenhuma;

        egui::TopBottomPanel::top("painel_superior_tecnico").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("⬅ Voltar ao Dashboard").clicked() {
                    acao = AcaoTecnico::Voltar;
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

            // Área de rolagem para a lista de OS
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("grid_os")
                    .num_columns(5)
                    .striped(true)
                    .spacing([20.0, 8.0])
                    .show(ui, |ui| {
                        // Cabeçalho da tabela
                        ui.label(egui::RichText::new("ID").strong());
                        ui.label(egui::RichText::new("Cliente").strong());
                        ui.label(egui::RichText::new("Equipamento").strong());
                        ui.label(egui::RichText::new("Status").strong());
                        ui.label(""); // Coluna para o botão de ação
                        ui.end_row();

                        // Linhas da tabela
                        for os in &self.ordens {
                            // Aplica o filtro (case-insensitive)
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
                                if ui.button("Detalhes").clicked() {
                                    self.ordem_selecionada = Some(os.id);
                                }
                                ui.end_row();
                            }
                        }
                    });
            });
        });

        // Janela modal para mostrar detalhes da OS selecionada
        if let Some(id_selecionado) = self.ordem_selecionada {
            let os_selecionada = self
                .ordens
                .iter()
                .find(|os| os.id == id_selecionado)
                .cloned();

            if let Some(os) = os_selecionada {
                egui::Window::new(format!("Detalhes da OS #{}", os.id))
                    .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                    .resizable(false)
                    .collapsible(false)
                    .show(ctx, |ui| {
                        ui.heading(&os.cliente);
                        ui.label(format!("Equipamento: {}", os.equipamento));
                        ui.add_space(10.0);
                        ui.label("Defeito Relatado:");
                        ui.group(|ui| {
                            ui.label(&os.defeito_relatado);
                        });
                        ui.add_space(10.0);

                        if ui.button("Fechar").clicked() {
                            self.ordem_selecionada = None;
                        }
                    });
            } else {
                self.ordem_selecionada = None; // Fecha o modal se a OS não for encontrada
            }
        }

        acao
    }
}
