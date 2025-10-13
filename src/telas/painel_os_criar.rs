// src/telas/painel_os_criar.rs

use crate::aplicacao::AppEvent;
use crate::servicos::{InfoUsuario, OrdemServico, Peca, PecaOS, SituacaoOS, StatusOS};
use eframe::egui;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
enum EstadoListaPecas {
    Carregando,
    Pronto(Vec<Peca>),
    Erro(String),
}

#[derive(Clone)]
enum EstadoListaTecnicos {
    Ocioso,
    Carregando,
    Pronto(Vec<InfoUsuario>),
    Erro(String),
}

pub struct TelaCriarOs {
    endereco_servidor: Arc<Mutex<String>>,
    os_data: OrdemServico,
    usuario_logado: Option<String>,
    estado_pecas: Arc<Mutex<EstadoListaPecas>>,
    filtro_peca: String,
    peca_selecionada_id: Option<u32>,
    quantidade_peca: u32,
    salvando: Arc<Mutex<bool>>,
    resultado_salvar: Arc<Mutex<Option<Result<u32, String>>>>,
    erro_ultimo: Option<String>,
    // novos campos para modais e busca
    mostrar_modal_tecnicos: bool,
    estado_tecnicos: Arc<Mutex<EstadoListaTecnicos>>,
    filtro_tecnico: String,
    mostrar_modal_pecas: bool,
    filtro_peca_modal: String,
    tipo_adicao: u8, // 0 = Requisição, 1 = Orçamento
}

impl TelaCriarOs {
    pub fn new(endereco_servidor: Arc<Mutex<String>>, usuario_logado: Option<String>) -> Self {
        let mut os_inicial = OrdemServico {
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
            servicos: Vec::new(),
            total_servicos: 0.0,
        };
        if let Some(nome) = &usuario_logado {
            os_inicial.atendente = nome.clone();
        }

        let instancia = Self {
            endereco_servidor,
            os_data: os_inicial,
            usuario_logado,
            estado_pecas: Arc::new(Mutex::new(EstadoListaPecas::Carregando)),
            filtro_peca: String::new(),
            peca_selecionada_id: None,
            quantidade_peca: 1,
            salvando: Arc::new(Mutex::new(false)),
            resultado_salvar: Arc::new(Mutex::new(None)),
            erro_ultimo: None,
            mostrar_modal_tecnicos: false,
            estado_tecnicos: Arc::new(Mutex::new(EstadoListaTecnicos::Ocioso)),
            filtro_tecnico: String::new(),
            mostrar_modal_pecas: false,
            filtro_peca_modal: String::new(),
            tipo_adicao: 0,
        };
        instancia.disparar_carregamento_pecas(None);
        instancia
    }

    fn disparar_carregamento_pecas(&self, ctx: Option<egui::Context>) {
        let endereco = self.endereco_servidor.lock().unwrap().clone();
        {
            let mut estado = self.estado_pecas.lock().unwrap();
            *estado = EstadoListaPecas::Carregando;
        }

        let estado_clone = Arc::clone(&self.estado_pecas);
        crate::executor::spawn(move || {
            let url = format!("{}/estoque/pecas", endereco);
            let client = crate::http_client::get_client();
            let resultado = client.get(&url).send();
            let mut estado = estado_clone.lock().unwrap();
            *estado = match resultado {
                Ok(resp) => match resp.json::<Vec<Peca>>() {
                    Ok(lista) => EstadoListaPecas::Pronto(lista),
                    Err(e) => {
                        EstadoListaPecas::Erro(format!("Erro ao interpretar lista de peças: {}", e))
                    }
                },
                Err(e) => EstadoListaPecas::Erro(format!("Erro ao consultar {}: {}", url, e)),
            };
            if let Some(ctx) = ctx {
                ctx.request_repaint();
            }
        });
    }

    fn disparar_carregamento_tecnicos(&self, ctx: egui::Context) {
        let endereco = self.endereco_servidor.lock().unwrap().clone();
        {
            let mut estado = self.estado_tecnicos.lock().unwrap();
            *estado = EstadoListaTecnicos::Carregando;
        }

        let estado_clone = Arc::clone(&self.estado_tecnicos);
        crate::executor::spawn(move || {
            let url = format!("{}/usuarios", endereco);
            let client = crate::http_client::get_client();
            let resultado = client.get(&url).send();
            let mut estado = estado_clone.lock().unwrap();
            *estado = match resultado {
                Ok(resp) => match resp.json::<Vec<InfoUsuario>>() {
                    Ok(lista) => EstadoListaTecnicos::Pronto(lista),
                    Err(e) => EstadoListaTecnicos::Erro(format!(
                        "Erro ao interpretar lista de usuários: {}",
                        e
                    )),
                },
                Err(e) => EstadoListaTecnicos::Erro(format!("Erro ao consultar {}: {}", url, e)),
            };
            ctx.request_repaint();
        });
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

        let estado_pecas_atual = { self.estado_pecas.lock().unwrap().clone() };
        if matches!(estado_pecas_atual, EstadoListaPecas::Carregando) {
            ctx.request_repaint();
        }

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
            let adicionar_habilitado = matches!(estado_pecas_atual, EstadoListaPecas::Pronto(_));
            if ui
                .add_enabled(adicionar_habilitado, egui::Button::new("Adicionar peça"))
                .clicked()
            {
                if let (Some(id), EstadoListaPecas::Pronto(lista)) =
                    (self.peca_selecionada_id, &estado_pecas_atual)
                {
                    if let Some(p) = lista.iter().find(|x| x.id == id) {
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

        match &estado_pecas_atual {
            EstadoListaPecas::Carregando => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Carregando peças do estoque...");
                });
            }
            EstadoListaPecas::Erro(msg) => {
                ui.colored_label(egui::Color32::RED, msg);
                if ui.button("Tentar novamente").clicked() {
                    self.disparar_carregamento_pecas(Some(ctx.clone()));
                }
            }
            EstadoListaPecas::Pronto(lista) => {
                egui::ScrollArea::vertical()
                    .max_height(160.0)
                    .show(ui, |ui| {
                        let filtro = self.filtro_peca.to_lowercase();
                        for p in lista.iter().filter(|p| {
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
            }
        }

        // Modais (reaproveitamos o código existente)
        if self.mostrar_modal_tecnicos {
            let estado_tecnicos = { self.estado_tecnicos.lock().unwrap().clone() };
            if matches!(estado_tecnicos, EstadoListaTecnicos::Carregando) {
                ctx.request_repaint();
            }

            if matches!(estado_tecnicos, EstadoListaTecnicos::Ocioso) {
                self.disparar_carregamento_tecnicos(ctx.clone());
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

                    match estado_tecnicos {
                        EstadoListaTecnicos::Carregando | EstadoListaTecnicos::Ocioso => {
                            ui.horizontal(|ui| {
                                ui.spinner();
                                ui.label("Carregando técnicos...");
                            });
                        }
                        EstadoListaTecnicos::Pronto(lista) => {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                let filtro = self.filtro_tecnico.to_lowercase();
                                for t in lista.iter().filter(|t| {
                                    filtro.is_empty()
                                        || t.nome_usuario.to_lowercase().contains(&filtro)
                                }) {
                                    ui.horizontal(|ui| {
                                        ui.label(&t.nome_usuario);
                                        if ui.button("Selecionar").clicked() {
                                            self.os_data.nome_tecnico_responsavel =
                                                t.nome_usuario.clone();
                                            self.mostrar_modal_tecnicos = false;
                                        }
                                    });
                                }
                            });
                        }
                        EstadoListaTecnicos::Erro(msg) => {
                            ui.colored_label(egui::Color32::RED, msg);
                            if ui.button("Tentar novamente").clicked() {
                                self.disparar_carregamento_tecnicos(ctx.clone());
                            }
                        }
                    }
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

                    match &estado_pecas_atual {
                        EstadoListaPecas::Carregando => {
                            ui.horizontal(|ui| {
                                ui.spinner();
                                ui.label("Carregando peças...");
                            });
                        }
                        EstadoListaPecas::Erro(msg) => {
                            ui.colored_label(egui::Color32::RED, msg);
                            if ui.button("Tentar novamente").clicked() {
                                self.disparar_carregamento_pecas(Some(ctx.clone()));
                            }
                        }
                        EstadoListaPecas::Pronto(lista) => {
                            let filtro = self.filtro_peca_modal.to_lowercase();
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                for p in lista.iter().filter(|p| {
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
                                                self.os_data.total_pecas = self
                                                    .os_data
                                                    .pecas
                                                    .iter()
                                                    .map(|p| p.preco_total)
                                                    .sum();
                                            }
                                            self.quantidade_peca = qtd;
                                        });
                                    });
                                    ui.separator();
                                }
                            });
                        }
                    }
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
                let usuario = self.usuario_logado.clone();
                let endereco = self.endereco_servidor.lock().unwrap().clone();
                let salvando_flag = Arc::clone(&self.salvando);
                let resultado = Arc::clone(&self.resultado_salvar);
                let ctx_clone = ctx.clone();
                crate::executor::spawn(move || {
                    let client = crate::http_client::get_client();
                    let url = format!("{}/ordens", endereco);
                    let mut payload = os_para_enviar;
                    if payload.atendente.is_empty() {
                        if let Some(nome) = usuario {
                            payload.atendente = nome;
                        }
                    }
                    let res = client.post(&url).json(&payload).send();

                    let resultado_final = match res {
                        Ok(resp) => match resp.json::<OrdemServico>() {
                            Ok(os_criada) => Ok(os_criada.id),
                            Err(e) => Err(format!(
                                "Erro ao interpretar resposta da criação da OS: {}",
                                e
                            )),
                        },
                        Err(e) => Err(format!("Falha ao criar OS: {}", e)),
                    };

                    *resultado.lock().unwrap() = Some(resultado_final);
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
        _frame: &mut eframe::Frame,
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
