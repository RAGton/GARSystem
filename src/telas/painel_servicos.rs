use crate::aplicacao::AppEvent;
use crate::servicos::Servico;
use eframe::egui;

pub struct TelaServicos {
    lista: Vec<Servico>,
    nome: String,
    descricao: String,
    preco: f64,
}

impl TelaServicos {
    pub fn new() -> Self {
        Self {
            lista: crate::servicos::listar_servicos(),
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
                let _id = crate::servicos::criar_servico(&s);
                self.lista = crate::servicos::listar_servicos();
                self.nome.clear();
                self.descricao.clear();
                self.preco = 0.0;
            }
            ui.separator();
            for s in &self.lista {
                ui.horizontal(|ui| {
                    ui.label(format!("[{}] {} - R$ {:.2}", s.id, s.nome, s.preco));
                });
            }
        });
        None
    }
}
