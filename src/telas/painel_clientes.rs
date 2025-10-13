use crate::aplicacao::AppEvent;
use crate::servicos::Cliente;
use eframe::egui;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(serde::Deserialize)]
struct ResumoClienteResponse {
    gastos_totais: f64,
    credito_disponivel: f64,
}

#[derive(Clone)]
enum EstadoListaClientes {
    Carregando,
    Sucesso(Vec<Cliente>),
    Erro(String),
}

#[derive(Clone)]
enum EstadoResumoCliente {
    Ocioso,
    Carregando,
    Sucesso { gastos: f64, credito: f64 },
    Erro(String),
}

pub struct TelaClientes {
    endereco_servidor: Arc<Mutex<String>>,
    clientes: Arc<Mutex<EstadoListaClientes>>,
    selecionado: Option<u32>,
    resumos: Arc<Mutex<HashMap<u32, EstadoResumoCliente>>>,
}

impl TelaClientes {
    pub fn new(endereco_servidor: Arc<Mutex<String>>) -> Self {
        let instancia = Self {
            endereco_servidor,
            clientes: Arc::new(Mutex::new(EstadoListaClientes::Carregando)),
            selecionado: None,
            resumos: Arc::new(Mutex::new(HashMap::new())),
        };
        instancia.disparar_carregamento_clientes(None);
        instancia
    }

    fn disparar_carregamento_clientes(&self, ctx: Option<egui::Context>) {
        let endereco = self.endereco_servidor.lock().unwrap().clone();
        {
            let mut guard = self.clientes.lock().unwrap();
            *guard = EstadoListaClientes::Carregando;
        }

        let clientes_clone = Arc::clone(&self.clientes);
        crate::executor::spawn(move || {
            let url = format!("{}/clientes", endereco);
            let client = crate::http_client::get_client();
            let response = client.get(&url).send();

            let mut guard = clientes_clone.lock().unwrap();
            *guard = match response {
                Ok(resp) => match resp.json::<Vec<Cliente>>() {
                    Ok(lista) => EstadoListaClientes::Sucesso(lista),
                    Err(e) => EstadoListaClientes::Erro(format!(
                        "Falha ao processar resposta /clientes: {}",
                        e
                    )),
                },
                Err(e) => EstadoListaClientes::Erro(format!("Falha ao consultar {}: {}", url, e)),
            };

            if let Some(ctx) = ctx {
                ctx.request_repaint();
            }
        });
    }

    fn disparar_resumo_cliente(&self, id: u32, ctx: egui::Context) {
        let endereco = self.endereco_servidor.lock().unwrap().clone();
        {
            let mut guard = self.resumos.lock().unwrap();
            guard.insert(id, EstadoResumoCliente::Carregando);
        }

        let resumos_clone = Arc::clone(&self.resumos);
        crate::executor::spawn(move || {
            let url = format!("{}/clientes/{}/resumo", endereco, id);
            let client = crate::http_client::get_client();
            let resposta = client.get(&url).send();

            let mut guard = resumos_clone.lock().unwrap();
            guard.insert(
                id,
                match resposta {
                    Ok(resp) => match resp.json::<ResumoClienteResponse>() {
                        Ok(r) => EstadoResumoCliente::Sucesso {
                            gastos: r.gastos_totais,
                            credito: r.credito_disponivel,
                        },
                        Err(e) => {
                            EstadoResumoCliente::Erro(format!("Erro ao interpretar resumo: {}", e))
                        }
                    },
                    Err(e) => {
                        EstadoResumoCliente::Erro(format!("Erro de conexão com {}: {}", url, e))
                    }
                },
            );
            ctx.request_repaint();
        });
    }

    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Option<AppEvent> {
        let estado_clientes = { self.clientes.lock().unwrap().clone() };
        if matches!(estado_clientes, EstadoListaClientes::Carregando) {
            ctx.request_repaint();
        }

        egui::CentralPanel::default()
            .show(ctx, |ui| {
                ui.heading("Clientes");
                ui.horizontal(|ui| {
                    if ui.button("Novo Cliente").clicked() {
                        self.selecionado = None;
                    }
                    if ui.button("Atualizar lista").clicked() {
                        self.disparar_carregamento_clientes(Some(ctx.clone()));
                    }
                });

                ui.separator();

                match estado_clientes {
                    EstadoListaClientes::Carregando => {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Carregando clientes...");
                        });
                        return None;
                    }
                    EstadoListaClientes::Erro(msg) => {
                        ui.colored_label(egui::Color32::RED, msg);
                        if ui.button("Tentar novamente").clicked() {
                            self.disparar_carregamento_clientes(Some(ctx.clone()));
                        }
                        return None;
                    }
                    EstadoListaClientes::Sucesso(lista) => {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label("Lista de clientes:");
                                for c in &lista {
                                    if ui
                                        .selectable_label(self.selecionado == Some(c.id), &c.nome)
                                        .clicked()
                                    {
                                        self.selecionado = Some(c.id);
                                    }
                                }
                            });

                            ui.vertical(|ui| {
                                if let Some(id) = self.selecionado {
                                    if let Some(c) = lista.iter().find(|x| x.id == id) {
                                        ui.group(|ui| {
                                            ui.label(format!("Nome: {}", c.nome));
                                            ui.label(format!("Email: {}", c.email));
                                            ui.label(format!("Telefone: {}", c.telefone));

                                            let resumo_estado = {
                                                let mut map = self.resumos.lock().unwrap();
                                                map.entry(c.id)
                                                    .or_insert(EstadoResumoCliente::Ocioso)
                                                    .clone()
                                            };

                                            match resumo_estado {
                                                EstadoResumoCliente::Ocioso => {
                                                    self.disparar_resumo_cliente(c.id, ctx.clone());
                                                    ui.horizontal(|ui| {
                                                        ui.spinner();
                                                        ui.label("Calculando resumo...");
                                                    });
                                                }
                                                EstadoResumoCliente::Carregando => {
                                                    ui.horizontal(|ui| {
                                                        ui.spinner();
                                                        ui.label("Calculando resumo...");
                                                    });
                                                }
                                                EstadoResumoCliente::Sucesso {
                                                    gastos,
                                                    credito,
                                                } => {
                                                    ui.label(format!(
                                                        "Gastos totais: R$ {:.2}",
                                                        gastos
                                                    ));
                                                    ui.label(format!(
                                                        "Crédito disponível: R$ {:.2}",
                                                        credito
                                                    ));
                                                }
                                                EstadoResumoCliente::Erro(msg) => {
                                                    ui.colored_label(egui::Color32::RED, msg);
                                                    if ui.button("Tentar novamente").clicked() {
                                                        self.disparar_resumo_cliente(
                                                            c.id,
                                                            ctx.clone(),
                                                        );
                                                    }
                                                }
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
                    }
                }

                None
            })
            .inner
    }
}
