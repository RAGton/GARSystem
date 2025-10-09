// src/telas/painel_orcamentos.rs
use crate::aplicacao::AppEvent;
use crate::http_client::get_client;
use eframe::egui;
use serde::Serialize;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Clone, Debug)]
pub struct ItemOrcamento {
    pub descricao: String,
    pub quantidade: u32,
    pub preco_unitario: f64,
}

#[derive(Serialize)]
struct OrcamentoPayloadItem {
    descricao: String,
    quantidade: u32,
    preco_unitario: f64,
    preco_total: f64,
}

#[derive(Serialize)]
struct OrcamentoPayload {
    id: u32,
    cliente_id: u32,
    items: Vec<OrcamentoPayloadItem>,
    total: f64,
}

pub struct TelaOrcamentos {
    cliente: String,
    itens: Vec<ItemOrcamento>,
    novo_item_descricao: String,
    novo_item_qtd: u32,
    novo_item_preco: f64,
    endereco_servidor: Arc<Mutex<String>>,
}

impl TelaOrcamentos {
    pub fn new(endereco_servidor: Arc<Mutex<String>>) -> Self {
        Self {
            cliente: String::new(),
            itens: Vec::new(),
            novo_item_descricao: String::new(),
            novo_item_qtd: 1,
            novo_item_preco: 0.0,
            endereco_servidor,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Option<AppEvent> {
        let mut evento = None;
        // Toolbar + responsive two-column layout
        egui::CentralPanel::default().show(ctx, |ui| {
            // Top toolbar
            ui.horizontal(|ui| {
                ui.add_enabled_ui(true, |ui| {
                    if ui
                        .add_sized([130.0, 30.0], egui::Button::new("💾 Salvar Orçamento"))
                        .clicked()
                    {
                        // salvar sem travar UI -> enviar POST /orcamentos
                        let _cliente = self.cliente.clone();
                        let itens = self.itens.clone();
                        let endereco = Arc::clone(&self.endereco_servidor);
                        thread::spawn(move || {
                            let endereco_str = endereco.lock().unwrap().clone();
                            let url = format!("{}/orcamentos", endereco_str);
                            let total: f64 = itens
                                .iter()
                                .map(|it| it.preco_unitario * it.quantidade as f64)
                                .sum();
                            let payload = OrcamentoPayload {
                                id: 0,
                                cliente_id: 0, // por enquanto nome do cliente livre; integração futura por id
                                items: itens
                                    .into_iter()
                                    .map(|it| OrcamentoPayloadItem {
                                        descricao: it.descricao,
                                        quantidade: it.quantidade,
                                        preco_unitario: it.preco_unitario,
                                        preco_total: it.preco_unitario * it.quantidade as f64,
                                    })
                                    .collect(),
                                total,
                            };

                            let client = get_client();
                            match client.post(&url).json(&payload).send() {
                                Ok(resp) => {
                                    if resp.status().is_success() {
                                        println!("Orçamento salvo com sucesso (HTTP).");
                                    } else {
                                        eprintln!(
                                            "Falha ao salvar orcamento: status {}",
                                            resp.status()
                                        );
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Erro HTTP ao salvar orcamento: {:?}", e);
                                }
                            }
                        });
                    }
                    if ui
                        .add_sized([100.0, 30.0], egui::Button::new("✖ Cancelar"))
                        .clicked()
                    {
                        evento = Some(AppEvent::VoltarParaDashboard);
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.heading("Orçamentos");
                });
            });

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Left column: items
                    ui.vertical(|ui| {
                        ui.add_space(4.0);
                        ui.group(|ui| {
                            ui.heading("Itens (Produtos / Serviços)");
                            ui.add_space(6.0);

                            ui.horizontal(|ui| {
                                ui.label("Descrição:");
                                ui.text_edit_singleline(&mut self.novo_item_descricao);
                            });
                            ui.horizontal(|ui| {
                                ui.label("Qtd:");
                                ui.add(
                                    egui::DragValue::new(&mut self.novo_item_qtd)
                                        .speed(1.0)
                                        .range(1..=999),
                                );
                                ui.label("Preço Unit.: R$");
                                ui.add(egui::DragValue::new(&mut self.novo_item_preco).speed(0.1));
                                if ui.button("➕ Adicionar").clicked() {
                                    if !self.novo_item_descricao.trim().is_empty() {
                                        self.itens.push(ItemOrcamento {
                                            descricao: self.novo_item_descricao.clone(),
                                            quantidade: self.novo_item_qtd,
                                            preco_unitario: self.novo_item_preco,
                                        });
                                        self.novo_item_descricao.clear();
                                        self.novo_item_qtd = 1;
                                        self.novo_item_preco = 0.0;
                                    }
                                }
                            });

                            ui.add_space(8.0);
                            egui::Grid::new("grid_itens_orc")
                                .num_columns(4)
                                .min_col_width(80.0)
                                .show(ui, |ui| {
                                    ui.strong("Descrição");
                                    ui.strong("Qtd");
                                    ui.strong("Vlr Unit.");
                                    ui.strong("Vlr Total");
                                    ui.end_row();
                                    let mut remover_idx: Option<usize> = None;
                                    for (idx, item) in self.itens.iter().enumerate() {
                                        ui.label(&item.descricao);
                                        ui.label(item.quantidade.to_string());
                                        ui.label(format!("R$ {:.2}", item.preco_unitario));
                                        ui.label(format!(
                                            "R$ {:.2}",
                                            item.preco_unitario * item.quantidade as f64
                                        ));
                                        if ui.button("Remover").clicked() {
                                            remover_idx = Some(idx);
                                        }
                                        ui.end_row();
                                    }
                                    if let Some(i) = remover_idx {
                                        self.itens.remove(i);
                                    }
                                });
                        });
                    });

                    // Right column: cliente e resumo
                    ui.vertical(|ui| {
                        ui.add_space(4.0);
                        ui.group(|ui| {
                            ui.heading("Cliente");
                            ui.add_space(4.0);
                            ui.label("Nome/Contato:");
                            ui.text_edit_singleline(&mut self.cliente);
                        });

                        ui.add_space(8.0);
                        ui.group(|ui| {
                            ui.heading("Resumo");
                            let total: f64 = self
                                .itens
                                .iter()
                                .map(|it| it.preco_unitario * it.quantidade as f64)
                                .sum();
                            ui.add_space(6.0);
                            ui.horizontal(|ui| {
                                ui.label("Total:");
                                ui.strong(format!("R$ {:.2}", total));
                            });
                            ui.add_space(6.0);
                            if ui.button("Gerar PDF (Em breve)").clicked() {
                                // placeholder
                            }
                        });
                    });
                });
            });
        });

        evento
    }
}

impl From<TelaOrcamentos> for crate::aplicacao::EstadoTela {
    fn from(t: TelaOrcamentos) -> Self {
        crate::aplicacao::EstadoTela::Orcamentos(t)
    }
}
