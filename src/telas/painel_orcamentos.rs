// src/telas/painel_orcamentos.rs
use crate::aplicacao::AppEvent;
use crate::gui_services::{self, BaseHandle, ErroServico, TokenArc};
use eframe::egui;
use serde::Serialize;

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
    base: BaseHandle,
    token: TokenArc,
    mensagem: Option<(String, bool)>, // (mensagem, é_erro)
}

impl TelaOrcamentos {
    pub fn new(base: BaseHandle, token: TokenArc) -> Self {
        Self {
            cliente: String::new(),
            itens: Vec::new(),
            novo_item_descricao: String::new(),
            novo_item_qtd: 1,
            novo_item_preco: 0.0,
            base,
            token,
            mensagem: None,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Option<AppEvent> {
        let mut evento = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            crate::telas::componentes::ui_kit::cabecalho(
                ui,
                "Novo Orçamento",
                "Gere orçamentos rápidos para clientes com produtos e serviços.",
                None,
            );

            ui.horizontal(|ui| {
                if ui.button("← Voltar").clicked() {
                    evento = Some(AppEvent::VoltarParaDashboard);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(egui::Button::new(egui::RichText::new("💾 Salvar Orçamento").strong().color(egui::Color32::WHITE)).fill(crate::telas::theme::SUCCESS)).clicked() {
                        let itens = self.itens.clone();
                        let total: f64 = itens.iter().map(|it| it.preco_unitario * it.quantidade as f64).sum();
                        let payload = OrcamentoPayload {
                            id: 0,
                            cliente_id: 0,
                            items: itens.into_iter().map(|it| OrcamentoPayloadItem {
                                descricao: it.descricao,
                                quantidade: it.quantidade,
                                preco_unitario: it.preco_unitario,
                                preco_total: it.preco_unitario * it.quantidade as f64,
                            }).collect(),
                            total,
                        };
                        let base = self.base.clone();
                        let token = self.token.clone();
                        crate::executor::spawn(move || {
                            let res: Result<serde_json::Value, ErroServico> = gui_services::criar_orcamento(&base, &token, &payload);
                            match res {
                                Ok(_) => tracing::info!("✅ Orçamento salvo"),
                                Err(e) => tracing::error!("❌ Falha ao salvar orçamento: {}", e),
                            }
                        });
                        evento = Some(AppEvent::VoltarParaDashboard);
                    }
                });
            });

            ui.add_space(16.0);

            egui::ScrollArea::vertical().id_salt("scroll_orcamento").show(ui, |ui| {
                ui.columns(2, |cols| {
                    // Left column: Client & Summary
                    cols[0].group(|ui| {
                        ui.heading("Dados do Cliente");
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("Nome / Contato").strong());
                        ui.text_edit_singleline(&mut self.cliente);
                        
                        ui.add_space(16.0);
                        
                        ui.heading("Resumo");
                        let total: f64 = self.itens.iter().map(|it| it.preco_unitario * it.quantidade as f64).sum();
                        ui.add_space(8.0);
                        egui::Frame::NONE
                            .fill(ui.visuals().faint_bg_color)
                            .corner_radius(4)
                            .inner_margin(egui::Margin::same(12))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("Total do Orçamento:").strong().size(16.0));
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(egui::RichText::new(format!("R$ {:.2}", total)).strong().size(18.0).color(crate::telas::theme::SUCCESS));
                                    });
                                });
                            });
                        
                        ui.add_space(12.0);
                        if ui.button("📄 Gerar PDF (Em breve)").clicked() {}
                    });

                    // Right column: Items
                    cols[1].group(|ui| {
                        ui.heading("Itens (Produtos / Serviços)");
                        ui.add_space(8.0);

                        egui::Grid::new("add_item_orc").num_columns(2).spacing([12.0, 8.0]).show(ui, |ui| {
                            ui.label(egui::RichText::new("Descrição").strong());
                            ui.text_edit_singleline(&mut self.novo_item_descricao);
                            ui.end_row();

                            ui.label(egui::RichText::new("Qtd / Valor").strong());
                            ui.horizontal(|ui| {
                                ui.add(egui::DragValue::new(&mut self.novo_item_qtd).speed(1.0).range(1..=999));
                                ui.label(" x ");
                                ui.add(egui::DragValue::new(&mut self.novo_item_preco).speed(0.5).prefix("R$ ").min_decimals(2));
                            });
                            ui.end_row();
                        });
                        
                        ui.add_space(8.0);
                        if ui.add_enabled(!self.novo_item_descricao.trim().is_empty(), egui::Button::new("➕ Adicionar Item")).clicked() {
                            self.itens.push(ItemOrcamento {
                                descricao: self.novo_item_descricao.clone(),
                                quantidade: self.novo_item_qtd,
                                preco_unitario: self.novo_item_preco,
                            });
                            self.novo_item_descricao.clear();
                            self.novo_item_qtd = 1;
                            self.novo_item_preco = 0.0;
                        }

                        ui.add_space(16.0);
                        ui.separator();
                        ui.add_space(8.0);

                        if self.itens.is_empty() {
                            ui.label(egui::RichText::new("Nenhum item adicionado.").color(ui.visuals().weak_text_color()));
                        } else {
                            egui::Grid::new("grid_itens_orc").num_columns(4).spacing([16.0, 8.0]).striped(true).show(ui, |ui| {
                                ui.label(egui::RichText::new("Descrição").strong());
                                ui.label(egui::RichText::new("Qtd").strong());
                                ui.label(egui::RichText::new("Total").strong());
                                ui.label("");
                                ui.end_row();
                                
                                let mut remover_idx: Option<usize> = None;
                                for (idx, item) in self.itens.iter().enumerate() {
                                    ui.label(&item.descricao);
                                    ui.label(item.quantidade.to_string());
                                    ui.label(egui::RichText::new(format!("R$ {:.2}", item.preco_unitario * item.quantidade as f64)).strong());
                                    if ui.button("🗑").clicked() {
                                        remover_idx = Some(idx);
                                    }
                                    ui.end_row();
                                }
                                if let Some(i) = remover_idx {
                                    self.itens.remove(i);
                                }
                            });
                        }
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
