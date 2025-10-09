use crate::aplicacao::AppEvent;
use crate::servicos::{Cliente, listar_clientes, obter_gastos_e_credito, criar_ou_atualizar_cliente};
use eframe::egui;
use std::sync::{Arc, Mutex};

pub struct TelaClientes {
    clientes: Arc<Mutex<Vec<Cliente>>>,
    selecionado: Option<u32>,
}

impl TelaClientes {
    pub fn new(_endereco_servidor: Arc<Mutex<String>>) -> Self {
        let list = listar_clientes().unwrap_or_default();
        Self { clientes: Arc::new(Mutex::new(list)), selecionado: None }
    }

    pub fn atualizar_lista(&mut self) {
        if let Ok(list) = listar_clientes() {
            *self.clientes.lock().unwrap() = list;
        }
    }

    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Option<AppEvent> {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Clientes");
            ui.horizontal(|ui| {
                if ui.button("Novo Cliente").clicked() {
                    // abre formulário simples — aqui apenas atualizamos a lista
                    self.selecionado = None;
                }
                if ui.button("Atualizar lista").clicked() {
                    self.atualizar_lista();
                }
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("Lista de clientes:");
                    for c in self.clientes.lock().unwrap().iter() {
                        if ui.selectable_label(self.selecionado == Some(c.id), &c.nome).clicked() {
                            self.selecionado = Some(c.id);
                        }
                    }
                });

                ui.vertical(|ui| {
                    if let Some(id) = self.selecionado {
                        if let Some(c) = self.clientes.lock().unwrap().iter().find(|x| x.id == id).cloned() {
                            ui.group(|ui| {
                                ui.label(format!("Nome: {}", c.nome));
                                ui.label(format!("Email: {}", c.email));
                                ui.label(format!("Telefone: {}", c.telefone));
                                if let Ok((gastos, credito)) = obter_gastos_e_credito(c.id) {
                                    ui.label(format!("Gastos totais: R$ {:.2}", gastos));
                                    ui.label(format!("Crédito disponível: R$ {:.2}", credito));
                                }
                                if ui.button("Editar (Em breve)").clicked() {
                                    // abrir modal de edição futuramente
                                }
                            });
                        }
                    } else {
                        ui.label("Nenhum cliente selecionado");
                    }
                });
            });

            None
        }).inner
    }
}
