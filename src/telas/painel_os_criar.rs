// src/telas/painel_os_criar.rs

use crate::aplicacao::AppEvent;
use crate::servicos::{Fornecedor, InfoUsuario, OrdemServico, Peca, PecaOS, SituacaoOS, StatusOS};
use eframe::egui;
use std::sync::{Arc, Mutex};

pub struct TelaCriarOs {
    endereco_servidor: Arc<Mutex<String>>,
    os_data: OrdemServico,
    lista_pecas_estoque: Vec<Peca>,
    filtro_peca: String,
    peca_selecionada_id: Option<u32>,
    quantidade_peca: u32,
    salvando: Arc<Mutex<bool>>,
    resultado_salvar: Arc<Mutex<Option<Result<u32, String>>>>,
    erro_ultimo: Option<String>,
    // novos campos para modais e busca
    mostrar_modal_tecnicos: bool,
    lista_tecnicos: Vec<InfoUsuario>,
    filtro_tecnico: String,
    mostrar_modal_pecas: bool,
    filtro_peca_modal: String,
    tipo_adicao: u8, // 0 = Requisição, 1 = Orçamento
}

impl TelaCriarOs {
    pub fn new(endereco_servidor: Arc<Mutex<String>>) -> Self {
        let lista_pecas = crate::banco_de_dados::estoque::listar_pecas().unwrap_or_default();
        Self {
            endereco_servidor,
            os_data: OrdemServico {
                id: 0,
                cliente: String::new(),
                equipamento: String::new(),
                defeito_relatado: String::new(),
                status: StatusOS::Aberta,
                parecer_tecnico: String::new(),
                situacao: SituacaoOS::Orcamento,
                numero_serie_equipamento: String::new(),
                observacoes: String::new(),
                nome_tecnico_responsavel: String::new(),
                atendente: String::new(),
                horario_abertura: String::new(),
                telefone_cliente: String::new(),
                data_chegada: String::new(),
                prazo_entrega: String::new(),
                historico_edicoes: Vec::new(),
                pecas: Vec::new(),
                total_pecas: 0.0,
            },
            lista_pecas_estoque: lista_pecas,
            filtro_peca: String::new(),
            peca_selecionada_id: None,
            quantidade_peca: 1,
            salvando: Arc::new(Mutex::new(false)),
            resultado_salvar: Arc::new(Mutex::new(None)),
            erro_ultimo: None,
            mostrar_modal_tecnicos: false,
            lista_tecnicos: Vec::new(),
            filtro_tecnico: String::new(),
            mostrar_modal_pecas: false,
            filtro_peca_modal: String::new(),
            tipo_adicao: 0,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Option<AppEvent> {
        // Mantemos compatibilidade: update atual ainda mostra no CentralPanel
        egui::CentralPanel::default().show(ctx, |ui| {
            // Reusar a função de render
            let ev = self.render_content(ui, ctx);
            if ev.is_some() {
                // repaint request se necessário
            }
        });

        None
    }

    // Renderiza o conteúdo principal dentro de um `Ui`. Retorna possível AppEvent.
    fn render_content(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) -> Option<AppEvent> {
        let mut evento_emitido: Option<AppEvent> = None;

        ui.heading("Criar Nova Ordem de Serviço");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Cliente:");
            ui.text_edit_singleline(&mut self.os_data.cliente);
            ui.label("Telefone:");
            ui.text_edit_singleline(&mut self.os_data.telefone_cliente);
        });

        ui.horizontal(|ui| {
            ui.label("Equipamento:");
            ui.text_edit_singleline(&mut self.os_data.equipamento);
            ui.label("Nº Série:");
            ui.text_edit_singleline(&mut self.os_data.numero_serie_equipamento);
        });

        ui.label("Defeito relatado:");
        ui.text_edit_multiline(&mut self.os_data.defeito_relatado);

        ui.horizontal(|ui| {
            ui.label("Atendente:");
            ui.text_edit_singleline(&mut self.os_data.atendente);
            ui.label("Técnico Responsável:");
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.os_data.nome_tecnico_responsavel);
                if ui.small_button("🔎").clicked() {
                    self.mostrar_modal_tecnicos = true;
                }
            });
        });

        ui.horizontal(|ui| {
            ui.label("Prazo Entrega (YYYY-MM-DD):");
            ui.text_edit_singleline(&mut self.os_data.prazo_entrega);
            ui.label("Observações:");
            ui.text_edit_singleline(&mut self.os_data.observacoes);
        });

        ui.separator();
        ui.heading("Peças (opcional)");
        ui.horizontal(|ui| {
            ui.label("Buscar peça:");
            ui.text_edit_singleline(&mut self.filtro_peca);
            if ui.small_button("🔎").clicked() {
                self.mostrar_modal_pecas = true;
            }
            ui.add(egui::DragValue::new(&mut self.quantidade_peca).range(1..=999));
            if ui.button("Adicionar peça").clicked() {
                if let Some(id) = self.peca_selecionada_id {
                    if let Some(p) = self.lista_pecas_estoque.iter().find(|x| x.id == id) {
                        let peca = PecaOS {
                            id_peca: p.id,
                            codigo_interno: p.codigo_interno.clone(),
                            descricao: p.descricao.clone(),
                            quantidade: self.quantidade_peca,
                            preco_venda_unitario: p.preco_venda,
                            preco_total: p.preco_venda * self.quantidade_peca as f64,
                        };
                        self.os_data.pecas.push(peca);
                        self.os_data.total_pecas =
                            self.os_data.pecas.iter().map(|p| p.preco_total).sum();
                    }
                }
            }
        });

        egui::ScrollArea::vertical()
            .max_height(160.0)
            .show(ui, |ui| {
                let filtro = self.filtro_peca.to_lowercase();
                for p in self.lista_pecas_estoque.iter().filter(|p| {
                    p.descricao.to_lowercase().contains(&filtro)
                        || p.codigo_interno.to_lowercase().contains(&filtro)
                }) {
                    ui.horizontal(|ui| {
                        ui.label(&p.descricao);
                        if ui.button("Selecionar").clicked() {
                            self.peca_selecionada_id = Some(p.id);
                        }
                    });
                }
            });

        // Modais (reaproveitamos o código existente)
        if self.mostrar_modal_tecnicos {
            if self.lista_tecnicos.is_empty() {
                self.lista_tecnicos = crate::servicos::listar_usuarios();
            }
            egui::Window::new("Pesquisar Técnicos")
                .collapsible(false)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Buscar técnico:");
                        ui.text_edit_singleline(&mut self.filtro_tecnico);
                        if ui.button("Fechar").clicked() {
                            self.mostrar_modal_tecnicos = false;
                        }
                    });

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let filtro = self.filtro_tecnico.to_lowercase();
                        for t in &self.lista_tecnicos {
                            if filtro.is_empty() || t.nome_usuario.to_lowercase().contains(&filtro)
                            {
                                ui.horizontal(|ui| {
                                    ui.label(&t.nome_usuario);
                                    if ui.button("Selecionar").clicked() {
                                        self.os_data.nome_tecnico_responsavel =
                                            t.nome_usuario.clone();
                                        self.mostrar_modal_tecnicos = false;
                                    }
                                });
                            }
                        }
                    });
                });
        }

        if self.mostrar_modal_pecas {
            egui::Window::new("Pesquisar Produtos")
                .collapsible(false)
                .resizable(true)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Buscar produto:");
                        ui.text_edit_singleline(&mut self.filtro_peca_modal);
                        ui.label("Tipo:");
                        ui.radio_value(&mut self.tipo_adicao, 0, "Requisição");
                        ui.radio_value(&mut self.tipo_adicao, 1, "Orçamento");
                        if ui.button("Fechar").clicked() {
                            self.mostrar_modal_pecas = false;
                        }
                    });

                    let filtro = self.filtro_peca_modal.to_lowercase();
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for p in self.lista_pecas_estoque.iter().filter(|p| {
                            filtro.is_empty()
                                || p.descricao.to_lowercase().contains(&filtro)
                                || p.codigo_interno.to_lowercase().contains(&filtro)
                        }) {
                            ui.group(|ui| {
                                ui.label(egui::RichText::new(&p.descricao).strong());
                                ui.label(format!(
                                    "Código: {} | Fabricante: {}",
                                    p.codigo_interno, p.fabricante
                                ));
                                ui.label(format!(
                                    "Estoque: {} | Preço: R$ {:.2}",
                                    p.estoque_atual, p.preco_venda
                                ));
                                ui.horizontal(|ui| {
                                    let mut qtd = self.quantidade_peca;
                                    ui.add(egui::DragValue::new(&mut qtd).range(1..=999));
                                    if ui.button("Adicionar a OS").clicked() {
                                        let peca = PecaOS {
                                            id_peca: p.id,
                                            codigo_interno: p.codigo_interno.clone(),
                                            descricao: p.descricao.clone(),
                                            quantidade: qtd,
                                            preco_venda_unitario: p.preco_venda,
                                            preco_total: p.preco_venda * qtd as f64,
                                        };
                                        self.os_data.pecas.push(peca);
                                        self.os_data.total_pecas =
                                            self.os_data.pecas.iter().map(|p| p.preco_total).sum();
                                    }
                                    self.quantidade_peca = qtd;
                                });
                            });
                            ui.separator();
                        }
                    });
                });
        }

        ui.separator();
        ui.horizontal(|ui| {
            let esta_salvando = *self.salvando.lock().unwrap();
            if ui
                .add_enabled(!esta_salvando, egui::Button::new("Criar Ordem de Serviço"))
                .clicked()
            {
                *self.salvando.lock().unwrap() = true;
                self.erro_ultimo = None;
                let os_para_enviar = self.os_data.clone();
                let salvando_flag = Arc::clone(&self.salvando);
                let resultado = Arc::clone(&self.resultado_salvar);
                let ctx_clone = ctx.clone();
                crate::executor::spawn(move || {
                    let mut os_mut = os_para_enviar;
                    let res = match crate::servicos::criar_ordem_servico(&mut os_mut) {
                        Ok(id) => Ok(id),
                        Err(e) => Err(format!("{}", e)),
                    };
                    *resultado.lock().unwrap() = Some(res);
                    *salvando_flag.lock().unwrap() = false;
                    ctx_clone.request_repaint();
                });
            }

            if ui.button("Cancelar").clicked() {
                evento_emitido = Some(AppEvent::VoltarParaDashboard);
            }
        });

        if let Some(res) = self.resultado_salvar.lock().unwrap().take() {
            match res {
                Ok(id) => {
                    evento_emitido = Some(AppEvent::AbrirEditorOS(id));
                }
                Err(msg) => {
                    self.erro_ultimo = Some(msg);
                }
            }
        }

        if let Some(err) = &self.erro_ultimo {
            ui.colored_label(egui::Color32::RED, format!("Erro ao criar OS: {}", err));
        }

        evento_emitido
    }

    // Mostra esta tela como uma janela flutuante grande sem fechar a tela subjacente.
    pub fn show_as_window(
        &mut self,
        ctx: &egui::Context,
        frame: &mut eframe::Frame,
    ) -> Option<AppEvent> {
        let mut evento = None;
        egui::Window::new("Criar Nova Ordem de Serviço")
            .collapsible(false)
            .resizable(true)
            .default_width(900.0)
            .default_height(700.0)
            .show(ctx, |ui| {
                evento = self.render_content(ui, ctx);
            });
        evento
    }
}
