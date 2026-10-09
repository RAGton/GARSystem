use crate::aplicacao::AppEvent;
use crate::servicos::Servico;
use eframe::egui;
use std::sync::{Arc, Mutex};

pub struct TelaServicos {
    lista: Arc<Mutex<Option<Vec<Servico>>>>,
    nome: String,
    descricao: String,
    preco: f64,
}

impl TelaServicos {
    pub fn new() -> Self {
        let lista = Arc::new(Mutex::new(None));
        let lista_bg = Arc::clone(&lista);
        crate::executor::spawn(move || {
            let t = crate::servicos::tenant_padrao();
            let l = crate::servicos::listar_servicos(t);
            if let Ok(mut g) = lista_bg.lock() {
                *g = Some(l);
            }
        });

        Self {
            lista,
            nome: String::new(),
            descricao: String::new(),
            preco: 0.0,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Option<AppEvent> {
        egui::CentralPanel::default().show(ctx, |ui| {
            crate::telas::componentes::ui_kit::cabecalho(
                ui,
                "Serviços",
                "Gerenciamento de serviços e mão de obra oferecida.",
                None,
            );

            ui.columns(2, |cols| {
                // Form
                cols[0].group(|ui| {
                    ui.heading("Novo Serviço");
                    ui.add_space(8.0);
                    
                    egui::Grid::new("grid_novo_servico").num_columns(2).spacing([12.0, 12.0]).show(ui, |ui| {
                        ui.label(egui::RichText::new("Nome").strong());
                        ui.text_edit_singleline(&mut self.nome);
                        ui.end_row();

                        ui.label(egui::RichText::new("Preço (R$)").strong());
                        ui.add(egui::DragValue::new(&mut self.preco).speed(0.5).prefix("R$ ").min_decimals(2).max_decimals(2));
                        ui.end_row();

                        ui.label(egui::RichText::new("Descrição").strong());
                        ui.add(egui::TextEdit::multiline(&mut self.descricao).desired_rows(3));
                        ui.end_row();
                    });
                    
                    ui.add_space(16.0);
                    
                    if ui.add_enabled(
                        !self.nome.is_empty() && self.preco > 0.0,
                        egui::Button::new("＋ Criar Serviço").fill(crate::telas::theme::PRIMARY_PRESSED)
                    ).clicked() {
                        let s = Servico {
                            id: 0,
                            nome: self.nome.clone(),
                            descricao: self.descricao.clone(),
                            preco: self.preco,
                        };
                        let lista_for_update = Arc::clone(&self.lista);
                        crate::executor::spawn(move || {
                            let t = crate::servicos::tenant_padrao();
                            let _ = crate::servicos::criar_servico(t, &s);
                            let l = crate::servicos::listar_servicos(t);
                            if let Ok(mut g) = lista_for_update.lock() {
                                *g = Some(l);
                            }
                        });
                        self.nome.clear();
                        self.descricao.clear();
                        self.preco = 0.0;
                    }
                });

                // List
                cols[1].group(|ui| {
                    ui.heading("Serviços Cadastrados");
                    ui.add_space(8.0);
                    
                    if let Ok(g) = self.lista.lock() {
                        if let Some(list) = &*g {
                            if list.is_empty() {
                                crate::telas::componentes::ui_kit::estado_vazio(
                                    ui,
                                    "🛠",
                                    "Nenhum serviço",
                                    "Cadastre serviços para usá-los nas ordens e orçamentos.",
                                    None,
                                );
                            } else {
                                egui::ScrollArea::vertical().id_salt("lista_servicos").show(ui, |ui| {
                                    for s in list.iter() {
                                        egui::Frame::NONE
                                            .inner_margin(egui::Margin::same(8))
                                            .fill(ui.visuals().faint_bg_color)
                                            .corner_radius(4)
                                            .show(ui, |ui| {
                                                ui.set_width(ui.available_width());
                                                ui.horizontal(|ui| {
                                                    ui.vertical(|ui| {
                                                        ui.label(egui::RichText::new(&s.nome).strong());
                                                        if !s.descricao.is_empty() {
                                                            ui.label(egui::RichText::new(&s.descricao).color(ui.visuals().weak_text_color()));
                                                        }
                                                    });
                                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                        ui.label(egui::RichText::new(format!("R$ {:.2}", s.preco)).strong().color(crate::telas::theme::SUCCESS));
                                                    });
                                                });
                                            });
                                        ui.add_space(4.0);
                                    }
                                });
                            }
                        } else {
                            ui.spinner();
                            ui.label("Carregando serviços...");
                        }
                    } else {
                        ui.spinner();
                        ui.label("Carregando serviços...");
                    }
                });
            });
        });
        None
    }
}
