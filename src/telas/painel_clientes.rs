use crate::aplicacao::AppEvent;
use crate::gui_services::{self, BaseHandle, ErroServico, TokenArc};
use crate::servicos::Cliente;
use eframe::egui;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(serde::Deserialize)]
struct ResumoClienteResponse {
    gastos_totais: f64,
    credito_disponivel: f64,
}

/// Estado da lista paginada de clientes.
#[derive(Clone)]
enum EstadoListaClientes {
    Carregando,
    /// Sucesso — contém a página atual + lista de clientes.
    Sucesso {
        lista: Vec<Cliente>,
        pagina_atual: u32,
        total_paginas: u32,
        total_registros: u64,
    },
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
    base: BaseHandle,
    token: TokenArc,
    clientes: Arc<Mutex<EstadoListaClientes>>,
    /// Página atual (1-based). Persiste entre atualizações.
    pagina_atual: u32,
    /// Itens por página.
    page_size: u32,
    selecionado: Option<u32>,
    resumos: Arc<Mutex<HashMap<u32, EstadoResumoCliente>>>,
}

const DEFAULT_PAGE_SIZE: u32 = 50;

impl TelaClientes {
    pub fn new(base: BaseHandle, token: TokenArc) -> Self {
        let instancia = Self {
            base,
            token,
            clientes: Arc::new(Mutex::new(EstadoListaClientes::Carregando)),
            pagina_atual: 1,
            page_size: DEFAULT_PAGE_SIZE,
            selecionado: None,
            resumos: Arc::new(Mutex::new(HashMap::new())),
        };
        instancia.disparar_carregamento_clientes(None);
        instancia
    }

    fn disparar_carregamento_clientes(&self, ctx: Option<egui::Context>) {
        *self.clientes.lock().unwrap() = EstadoListaClientes::Carregando;
        let base = self.base.clone();
        let token = self.token.clone();
        let clientes_clone = Arc::clone(&self.clientes);
        let page = self.pagina_atual;
        let limit = self.page_size;

        crate::executor::spawn(move || {
            let res = gui_services::listar_clientes_paginado(&base, &token, page, limit);
            *clientes_clone.lock().unwrap() = match res {
                Ok(pagina) => {
                    // Decodifica items como Vec<Cliente>
                    let lista: Vec<Cliente> =
                        serde_json::from_value(serde_json::Value::Array(pagina.items))
                            .unwrap_or_default();
                    EstadoListaClientes::Sucesso {
                        lista,
                        pagina_atual: pagina.page,
                        total_paginas: pagina.total_paginas,
                        total_registros: pagina.total,
                    }
                }
                Err(e) => EstadoListaClientes::Erro(e.mensagem),
            };
            if let Some(ctx) = ctx {
                ctx.request_repaint();
            }
        });
    }

    fn ir_para_pagina(&mut self, page: u32, ctx: egui::Context) {
        if page < 1 {
            return;
        }
        self.pagina_atual = page;
        self.disparar_carregamento_clientes(Some(ctx));
    }

    fn disparar_resumo_cliente(&self, id: u32, ctx: egui::Context) {
        self.resumos
            .lock()
            .unwrap()
            .insert(id, EstadoResumoCliente::Carregando);
        let base = self.base.clone();
        let token = self.token.clone();
        let resumos_clone = Arc::clone(&self.resumos);

        crate::executor::spawn(move || {
            let res: Result<ResumoClienteResponse, ErroServico> =
                gui_services::resumo_cliente(&base, &token, id);
            resumos_clone.lock().unwrap().insert(
                id,
                match res {
                    Ok(r) => EstadoResumoCliente::Sucesso {
                        gastos: r.gastos_totais,
                        credito: r.credito_disponivel,
                    },
                    Err(e) => EstadoResumoCliente::Erro(e.mensagem),
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
                if crate::telas::componentes::ui_kit::cabecalho(
                    ui,
                    "Clientes",
                    "Gerencie o cadastro e limite de crédito dos seus clientes.",
                    Some("＋ Novo Cliente"),
                ) {
                    self.selecionado = None;
                    // TODO: mostrar modal/form de novo cliente
                }
                
                ui.horizontal(|ui| {
                    if ui.button("↻ Atualizar lista").clicked() {
                        self.disparar_carregamento_clientes(Some(ctx.clone()));
                    }
                });
                ui.add_space(8.0);

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
                    EstadoListaClientes::Sucesso {
                        lista,
                        pagina_atual,
                        total_paginas,
                        total_registros,
                    } => {
                        // Controles de paginação
                        ui.horizontal(|ui| {
                            ui.add_enabled_ui(pagina_atual > 1, |ui| {
                                if ui.button("← Anterior").clicked() {
                                    self.ir_para_pagina(pagina_atual - 1, ctx.clone());
                                }
                            });
                            ui.label(format!(
                                "Página {} de {} ({} clientes no total)",
                                pagina_atual, total_paginas, total_registros
                            ));
                            ui.add_enabled_ui(pagina_atual < total_paginas, |ui| {
                                if ui.button("Próxima →").clicked() {
                                    self.ir_para_pagina(pagina_atual + 1, ctx.clone());
                                }
                            });
                        });
                        ui.separator();

                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(format!("Lista de clientes (página {}):", pagina_atual));
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
