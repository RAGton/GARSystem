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
            let l = crate::servicos::listar_servicos();
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
            ui.heading("Serviços (Admin)");
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Nome:");
                ui.text_edit_singleline(&mut self.nome);
                ui.label("Preço:");
                ui.add(egui::DragValue::new(&mut self.preco).speed(0.5));
            });
            ui.label("Descrição:");
            ui.text_edit_singleline(&mut self.descricao);
            if ui.button("Criar Serviço").clicked() {
                let s = Servico {
                    id: 0,
                    nome: self.nome.clone(),
                    descricao: self.descricao.clone(),
                    preco: self.preco,
                };
                // criar e recarregar em background
                let lista_for_update = Arc::clone(&self.lista);
                crate::executor::spawn(move || {
                    let _ = crate::servicos::criar_servico(&s);
                    let l = crate::servicos::listar_servicos();
                    if let Ok(mut g) = lista_for_update.lock() {
                        *g = Some(l);
                    }
                });
                self.nome.clear();
                self.descricao.clear();
                self.preco = 0.0;
            }
            ui.separator();

            if let Ok(g) = self.lista.lock() {
                if let Some(list) = &*g {
                    for s in list.iter() {
                        ui.horizontal(|ui| {
                            ui.label(format!("[{}] {} - R$ {:.2}", s.id, s.nome, s.preco));
                        });
                    }
                } else {
                    ui.label("Carregando serviços...");
                }
            } else {
                ui.label("Carregando serviços...");
            }
        });
        None
    }
}
