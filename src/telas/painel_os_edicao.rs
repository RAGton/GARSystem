// src/telas/painel_os_edicao.rs

use crate::aplicacao::{AppEvent, TelaAtiva};
use crate::gui_services::{self, BaseHandle, ErroServico, TokenArc};
use crate::servicos::{
    InfoUsuario, OrdemServico, Peca, PecaOS, Servico, ServicoOS, SituacaoOS, StatusOS,
};
use eframe::egui::{self, RichText, Ui};
use egui_extras::{Column, TableBuilder};
use serde::Serialize;
use std::sync::{Arc, Mutex};

// Abas para organizar a UI
#[derive(PartialEq, Eq, Clone, Copy)]
enum AbaDetalhesOS {
    Produtos,
    Servicos,
    Historico,
}

// Estado para gerenciar carregamento de dados da rede
#[derive(Clone)]
enum EstadoCarregamento<T> {
    Ocioso,
    Carregando,
    Pronto(Vec<T>),
    Erro(String),
}

#[allow(clippy::derivable_impls)]
impl<T> Default for EstadoCarregamento<T> {
    fn default() -> Self {
        Self::Ocioso
    }
}

// Payload para a requisição de atualização da OS
#[derive(Serialize)]
struct AtualizarOrdemPayload {
    os: OrdemServico,
    usuario: String,
}

pub struct TelaOsEdicao {
    os_id: u32,
    usuario_logado: String,
    base: BaseHandle,
    token: TokenArc,
    tx_evento: std::sync::mpsc::Sender<AppEvent>,

    estado_os: Arc<Mutex<Option<Result<OrdemServico, String>>>>,
    os_editada: OrdemServico,
    salvando: bool,
    resultado_salvar: Arc<Mutex<Option<Result<(), String>>>>,
    notificacao: Option<(String, egui::Color32)>,

    aba_ativa: AbaDetalhesOS,
    lista_pecas_estoque: Arc<Mutex<EstadoCarregamento<Peca>>>,
    lista_servicos_catalogo: Arc<Mutex<EstadoCarregamento<Servico>>>,
    lista_tecnicos: Arc<Mutex<EstadoCarregamento<InfoUsuario>>>,

    filtro_peca: String,
    filtro_servico: String,
    filtro_tecnico: String,
    mostrar_modal_tecnico: bool,
}

impl TelaOsEdicao {
    pub fn new(
        os_id: u32,
        base: BaseHandle,
        token: TokenArc,
        usuario_logado: String,
        tx_evento: std::sync::mpsc::Sender<AppEvent>,
    ) -> Self {
        let mut tela = Self {
            os_id,
            usuario_logado,
            base,
            token,
            tx_evento,
            estado_os: Arc::new(Mutex::new(None)),
            os_editada: OrdemServico::placeholder(os_id),
            salvando: false,
            resultado_salvar: Arc::new(Mutex::new(None)),
            notificacao: None,
            aba_ativa: AbaDetalhesOS::Produtos,
            lista_pecas_estoque: Arc::new(Mutex::new(EstadoCarregamento::Ocioso)),
            lista_servicos_catalogo: Arc::new(Mutex::new(EstadoCarregamento::Ocioso)),
            lista_tecnicos: Arc::new(Mutex::new(EstadoCarregamento::Ocioso)),
            filtro_peca: String::new(),
            filtro_servico: String::new(),
            filtro_tecnico: String::new(),
            mostrar_modal_tecnico: false,
        };

        tela.carregar_dados_iniciais();
        tela
    }

    fn carregar_dados_iniciais(&mut self) {
        self.disparar_carregamento_os();
        self.disparar_carregamento_pecas();
        self.disparar_carregamento_servicos();
        self.disparar_carregamento_tecnicos();
    }

    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Option<AppEvent> {
        self.processar_resultados_carregamento(ctx);

        let mut app_event = None;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading(format!("Detalhes da Ordem de Serviço #{}", self.os_id));
            });
            ui.add_space(10.0);

            self.desenhar_toolbar(ui, &mut app_event);
            ui.separator();

            if let Some((msg, cor)) = &self.notificacao {
                ui.label(RichText::new(msg).color(*cor));
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                self.desenhar_dados_gerais(ui);
                ui.add_space(15.0);
                self.desenhar_seletor_abas(ui);
                ui.add_space(10.0);

                match self.aba_ativa {
                    AbaDetalhesOS::Produtos => self.desenhar_aba_produtos(ui),
                    AbaDetalhesOS::Servicos => self.desenhar_aba_servicos(ui),
                    AbaDetalhesOS::Historico => self.desenhar_aba_historico(ui),
                }
            });
        });

        self.render_modal_tecnico(ctx);
        app_event
    }

    fn desenhar_toolbar(&mut self, ui: &mut Ui, app_event: &mut Option<AppEvent>) {
        ui.horizontal(|ui| {
            if ui
                .add_enabled(!self.salvando, egui::Button::new("💾 Salvar Alterações"))
                .clicked()
            {
                self.disparar_salvamento();
            }
            if self.salvando {
                ui.spinner();
                ui.label("Salvando...");
            }

            if ui.button("↩ Voltar para Lista").clicked() {
                *app_event = Some(AppEvent::NavegarPara(TelaAtiva::Ordens));
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let total_geral = self.os_editada.total_pecas + self.os_editada.total_servicos;
                ui.label(
                    RichText::new(format!("Total Geral: R$ {:.2}", total_geral))
                        .strong()
                        .size(16.0),
                );
                ui.separator();
                ui.label(format!(
                    "Serviços: R$ {:.2}",
                    self.os_editada.total_servicos
                ));
                ui.separator();
                ui.label(format!("Peças: R$ {:.2}", self.os_editada.total_pecas));
            });
        });
    }

    fn desenhar_dados_gerais(&mut self, ui: &mut Ui) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.heading("Informações Gerais");
            ui.separator();

            egui::Grid::new("grid_dados_gerais")
                .num_columns(2)
                .spacing([20.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Cliente:");
                    ui.text_edit_singleline(&mut self.os_editada.cliente);
                    ui.end_row();

                    ui.label("Equipamento:");
                    ui.text_edit_singleline(&mut self.os_editada.equipamento);
                    ui.end_row();

                    ui.label("Nº de Série:");
                    ui.text_edit_singleline(&mut self.os_editada.numero_serie_equipamento);
                    ui.end_row();

                    ui.label("Status:");
                    egui::ComboBox::from_id_salt("status_os")
                        .selected_text(format!("{:?}", self.os_editada.status))
                        .show_ui(ui, |ui| {
                            for status in StatusOS::iter() {
                                ui.selectable_value(
                                    &mut self.os_editada.status,
                                    status,
                                    format!("{:?}", status),
                                );
                            }
                        });
                    ui.end_row();

                    ui.label("Situação:");
                    egui::ComboBox::from_id_salt("situacao_os")
                        .selected_text(format!("{:?}", self.os_editada.situacao))
                        .show_ui(ui, |ui| {
                            for situacao in SituacaoOS::iter() {
                                ui.selectable_value(
                                    &mut self.os_editada.situacao,
                                    situacao,
                                    format!("{:?}", situacao),
                                );
                            }
                        });
                    ui.end_row();

                    ui.label("Técnico Responsável:");
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.os_editada.nome_tecnico_responsavel);
                        if ui.button("👤 Buscar").clicked() {
                            self.mostrar_modal_tecnico = true;
                        }
                    });
                    ui.end_row();
                });

            ui.add_space(10.0);
            ui.label("Defeito Relatado:");
            ui.text_edit_multiline(&mut self.os_editada.defeito_relatado);

            ui.label("Parecer Técnico:");
            ui.text_edit_multiline(&mut self.os_editada.parecer_tecnico);

            ui.label("Observações Internas:");
            ui.text_edit_multiline(&mut self.os_editada.observacoes);
        });
    }

    fn desenhar_seletor_abas(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.style_mut().visuals.selection.bg_fill = egui::Color32::from_rgb(50, 100, 150);
            ui.selectable_value(
                &mut self.aba_ativa,
                AbaDetalhesOS::Produtos,
                "Produtos / Peças",
            );
            ui.selectable_value(
                &mut self.aba_ativa,
                AbaDetalhesOS::Servicos,
                "Serviços / Mão de Obra",
            );
            ui.selectable_value(
                &mut self.aba_ativa,
                AbaDetalhesOS::Historico,
                "Histórico de Alterações",
            );
        });
        ui.separator();
    }

    fn desenhar_aba_produtos(&mut self, ui: &mut Ui) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.heading("Adicionar Peça do Estoque");
            ui.horizontal(|ui| {
                ui.label("Buscar Peça:");
                ui.text_edit_singleline(&mut self.filtro_peca);
            });

            let pecas_guard = self.lista_pecas_estoque.lock().unwrap();
            match &*pecas_guard {
                EstadoCarregamento::Carregando => {
                    ui.spinner();
                }
                EstadoCarregamento::Erro(e) => {
                    ui.colored_label(egui::Color32::RED, e);
                }
                EstadoCarregamento::Pronto(pecas) => {
                    let pecas_clone = pecas.clone();
                    drop(pecas_guard);

                    let filtro = self.filtro_peca.to_lowercase();
                    let pecas_filtradas: Vec<_> = pecas_clone
                        .into_iter()
                        .filter(|p| {
                            self.filtro_peca.is_empty()
                                || p.descricao.to_lowercase().contains(&filtro)
                                || p.codigo_interno.to_lowercase().contains(&filtro)
                        })
                        .collect();

                    egui::ScrollArea::vertical()
                        .max_height(150.0)
                        .show(ui, |ui| {
                            for peca in pecas_filtradas.iter().take(10) {
                                ui.horizontal(|ui| {
                                    ui.label(format!(
                                        "{} - {}",
                                        peca.codigo_interno, peca.descricao
                                    ));
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui.button("➕").clicked() {
                                                self.adicionar_peca(peca);
                                                self.filtro_peca.clear();
                                            }
                                        },
                                    );
                                });
                                ui.separator();
                            }
                        });
                    return;
                }
                _ => {}
            }
        });

        ui.add_space(10.0);
        ui.heading("Peças na Ordem de Serviço");
        let table = TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto())
            .column(Column::initial(80.0))
            .column(Column::initial(100.0))
            .column(Column::initial(100.0))
            .column(Column::initial(30.0));

        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Descrição");
                });
                header.col(|ui| {
                    ui.strong("Qtd.");
                });
                header.col(|ui| {
                    ui.strong("Vlr. Unit.");
                });
                header.col(|ui| {
                    ui.strong("Vlr. Total");
                });
                header.col(|ui| {
                    ui.strong("Ação");
                });
            })
            .body(|mut body| {
                let mut peca_a_remover: Option<usize> = None;
                let mut houve_alteracao = false;
                let len = self.os_editada.pecas.len();
                for i in 0..len {
                    // get mutable reference to the current item without holding other borrows
                    let (_left, right) = self.os_editada.pecas.split_at_mut(i);
                    let peca_os = &mut right[0];
                    body.row(30.0, |mut row| {
                        row.col(|ui| {
                            ui.label(&peca_os.descricao);
                        });
                        row.col(|ui| {
                            if ui
                                .add(
                                    egui::DragValue::new(&mut peca_os.quantidade)
                                        .speed(1)
                                        .range(1..=999),
                                )
                                .changed()
                            {
                                houve_alteracao = true;
                            }
                        });
                        row.col(|ui| {
                            ui.label(format!("R$ {:.2}", peca_os.preco_venda_unitario));
                        });
                        row.col(|ui| {
                            ui.label(format!(
                                "R$ {:.2}",
                                peca_os.quantidade as f64 * peca_os.preco_venda_unitario
                            ));
                        });
                        row.col(|ui| {
                            if ui.button("❌").on_hover_text("Remover peça").clicked() {
                                peca_a_remover = Some(i);
                            }
                        });
                    });
                }
                if let Some(index) = peca_a_remover {
                    self.os_editada.pecas.remove(index);
                    houve_alteracao = true;
                }
                if houve_alteracao {
                    self.recalcular_totais();
                }
            });
    }

    fn desenhar_aba_servicos(&mut self, ui: &mut Ui) {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.heading("Adicionar Serviço / Mão de Obra");
            ui.horizontal(|ui| {
                ui.label("Buscar Serviço:");
                ui.text_edit_singleline(&mut self.filtro_servico);
            });

            let servicos_guard = self.lista_servicos_catalogo.lock().unwrap();
            match &*servicos_guard {
                EstadoCarregamento::Carregando => {
                    ui.spinner();
                }
                EstadoCarregamento::Erro(e) => {
                    ui.colored_label(egui::Color32::RED, e);
                }
                EstadoCarregamento::Pronto(servicos) => {
                    let servicos_clone = servicos.clone();
                    drop(servicos_guard);

                    let filtro = self.filtro_servico.to_lowercase();
                    let servicos_filtrados: Vec<_> = servicos_clone
                        .into_iter()
                        .filter(|s| {
                            self.filtro_servico.is_empty()
                                || s.nome.to_lowercase().contains(&filtro)
                        })
                        .collect();

                    egui::ScrollArea::vertical()
                        .max_height(150.0)
                        .show(ui, |ui| {
                            for servico in servicos_filtrados.iter().take(10) {
                                ui.horizontal(|ui| {
                                    ui.label(&servico.nome);
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            if ui.button("➕").clicked() {
                                                self.adicionar_servico(servico);
                                                self.filtro_servico.clear();
                                            }
                                        },
                                    );
                                });
                                ui.separator();
                            }
                        });
                    return;
                }
                _ => {}
            }
        });

        ui.add_space(10.0);
        ui.heading("Serviços na Ordem de Serviço");
        let table = TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto())
            .column(Column::initial(80.0))
            .column(Column::initial(100.0))
            .column(Column::initial(100.0))
            .column(Column::initial(30.0));

        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong("Descrição");
                });
                header.col(|ui| {
                    ui.strong("Qtd.");
                });
                header.col(|ui| {
                    ui.strong("Vlr. Unit.");
                });
                header.col(|ui| {
                    ui.strong("Vlr. Total");
                });
                header.col(|ui| {
                    ui.strong("Ação");
                });
            })
            .body(|mut body| {
                let mut servico_a_remover: Option<usize> = None;
                let mut houve_alteracao = false;
                let len = self.os_editada.servicos.len();
                for i in 0..len {
                    let (_left, right) = self.os_editada.servicos.split_at_mut(i);
                    let servico_os = &mut right[0];
                    body.row(30.0, |mut row| {
                        row.col(|ui| {
                            ui.label(&servico_os.nome);
                        });
                        row.col(|ui| {
                            if ui
                                .add(
                                    egui::DragValue::new(&mut servico_os.quantidade)
                                        .speed(1)
                                        .range(1..=999),
                                )
                                .changed()
                            {
                                houve_alteracao = true;
                            }
                        });
                        row.col(|ui| {
                            ui.label(format!("R$ {:.2}", servico_os.preco_unitario));
                        });
                        row.col(|ui| {
                            ui.label(format!(
                                "R$ {:.2}",
                                servico_os.quantidade as f64 * servico_os.preco_unitario
                            ));
                        });
                        row.col(|ui| {
                            if ui.button("❌").on_hover_text("Remover serviço").clicked() {
                                servico_a_remover = Some(i);
                            }
                        });
                    });
                }
                if let Some(index) = servico_a_remover {
                    self.os_editada.servicos.remove(index);
                    houve_alteracao = true;
                }
                if houve_alteracao {
                    self.recalcular_totais();
                }
            });
    }

    fn desenhar_aba_historico(&mut self, ui: &mut Ui) {
        ui.heading("Histórico de Alterações");
        egui::ScrollArea::vertical().show(ui, |ui| {
            if self.os_editada.historico_edicoes.is_empty() {
                ui.label("Nenhum histórico de alteração para esta OS.");
            } else {
                for entrada in &self.os_editada.historico_edicoes {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&entrada.campo_alterado).strong());
                            ui.label(format!("por {}", entrada.usuario));
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(&entrada.data_hora);
                                },
                            );
                        });
                        ui.label(format!("De: '{}'", entrada.valor_antigo));
                        ui.label(format!("Para: '{}'", entrada.valor_novo));
                    });
                }
            }
        });
    }

    fn render_modal_tecnico(&mut self, ctx: &egui::Context) {
        if !self.mostrar_modal_tecnico {
            return;
        }

        // prepare snapshot of technicians and avoid holding locks while showing UI
        let tecnicos_snapshot = {
            let guard = self.lista_tecnicos.lock().unwrap();
            match &*guard {
                EstadoCarregamento::Pronto(v) => Some(v.clone()),
                _ => None,
            }
        };

        let mut selected: Option<String> = None;
        egui::Window::new("Selecionar Técnico")
            .collapsible(false)
            .resizable(true)
            .show(ctx, |ui| {
                ui.text_edit_singleline(&mut self.filtro_tecnico);
                ui.separator();

                if let Some(tecnicos) = &tecnicos_snapshot {
                    let filtro = self.filtro_tecnico.to_lowercase();
                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            for tecnico in tecnicos
                                .iter()
                                .filter(|t| t.nome_usuario.to_lowercase().contains(&filtro))
                            {
                                if ui.selectable_label(false, &tecnico.nome_usuario).clicked() {
                                    selected = Some(tecnico.nome_usuario.clone());
                                }
                            }
                        });
                } else {
                    ui.label("Carregando técnicos...");
                }
            });

        if let Some(nome) = selected {
            self.os_editada.nome_tecnico_responsavel = nome;
            self.mostrar_modal_tecnico = false;
        }
    }

    fn adicionar_peca(&mut self, peca_estoque: &Peca) {
        if let Some(existente) = self
            .os_editada
            .pecas
            .iter_mut()
            .find(|p| p.id_peca == peca_estoque.id)
        {
            existente.quantidade += 1;
        } else {
            self.os_editada.pecas.push(PecaOS {
                id_peca: peca_estoque.id,
                codigo_interno: peca_estoque.codigo_interno.clone(),
                descricao: peca_estoque.descricao.clone(),
                quantidade: 1,
                preco_venda_unitario: peca_estoque.preco_venda,
                preco_total: 0.0,
            });
        }
        self.recalcular_totais();
    }

    fn adicionar_servico(&mut self, servico_catalogo: &Servico) {
        if let Some(existente) = self
            .os_editada
            .servicos
            .iter_mut()
            .find(|s| s.id_servico == servico_catalogo.id)
        {
            existente.quantidade += 1;
        } else {
            self.os_editada.servicos.push(ServicoOS {
                id_servico: servico_catalogo.id,
                nome: servico_catalogo.nome.clone(),
                descricao: servico_catalogo.descricao.clone(),
                quantidade: 1,
                preco_unitario: servico_catalogo.preco,
                preco_total: 0.0,
            });
        }
        self.recalcular_totais();
    }

    fn recalcular_totais(&mut self) {
        self.os_editada.total_pecas = self
            .os_editada
            .pecas
            .iter_mut()
            .map(|p| {
                p.preco_total = p.quantidade as f64 * p.preco_venda_unitario;
                p.preco_total
            })
            .sum();
        self.os_editada.total_servicos = self
            .os_editada
            .servicos
            .iter_mut()
            .map(|s| {
                s.preco_total = s.quantidade as f64 * s.preco_unitario;
                s.preco_total
            })
            .sum();
    }

    // --- Funções de Comunicação com o Servidor ---

    fn get_server_address(&self) -> String {
        // Mantido por compat — preferir self.base.url() nos novos códigos.
        self.base.url("")
    }

    fn disparar_carregamento_os(&self) {
        let os_id = self.os_id;
        let base = self.base.clone();
        let token = self.token.clone();
        let tx = self.tx_evento.clone();
        let estado_os = self.estado_os.clone();

        crate::executor::spawn(move || {
            let res: Result<OrdemServico, ErroServico> =
                gui_services::obter_ordem(&base, &token, os_id);
            *estado_os.lock().unwrap() = Some(res.map_err(|e| e.mensagem));
            let _ = tx.send(AppEvent::Repaint);
        });
    }

    fn disparar_carregamento_pecas(&self) {
        let base = self.base.clone();
        let token = self.token.clone();
        let tx = self.tx_evento.clone();
        let estado_pecas = self.lista_pecas_estoque.clone();
        *estado_pecas.lock().unwrap() = EstadoCarregamento::Carregando;

        crate::executor::spawn(move || {
            let res: Result<Vec<Peca>, ErroServico> = gui_services::listar_pecas(&base, &token);
            *estado_pecas.lock().unwrap() = match res {
                Ok(pecas) => EstadoCarregamento::Pronto(pecas),
                Err(e) => EstadoCarregamento::Erro(e.mensagem),
            };
            let _ = tx.send(AppEvent::Repaint);
        });
    }

    fn disparar_carregamento_servicos(&self) {
        let base = self.base.clone();
        let token = self.token.clone();
        let tx = self.tx_evento.clone();
        let estado_servicos = self.lista_servicos_catalogo.clone();
        *estado_servicos.lock().unwrap() = EstadoCarregamento::Carregando;

        crate::executor::spawn(move || {
            let res: Result<Vec<Servico>, ErroServico> =
                gui_services::listar_servicos(&base, &token);
            *estado_servicos.lock().unwrap() = match res {
                Ok(servicos) => EstadoCarregamento::Pronto(servicos),
                Err(e) => EstadoCarregamento::Erro(e.mensagem),
            };
            let _ = tx.send(AppEvent::Repaint);
        });
    }

    fn disparar_carregamento_tecnicos(&self) {
        let base = self.base.clone();
        let token = self.token.clone();
        let tx = self.tx_evento.clone();
        let estado_tecnicos = self.lista_tecnicos.clone();
        *estado_tecnicos.lock().unwrap() = EstadoCarregamento::Carregando;

        crate::executor::spawn(move || {
            let res: Result<Vec<InfoUsuario>, ErroServico> =
                gui_services::listar_usuarios(&base, &token);
            *estado_tecnicos.lock().unwrap() = match res {
                Ok(tecnicos) => EstadoCarregamento::Pronto(tecnicos),
                Err(e) => EstadoCarregamento::Erro(e.mensagem),
            };
            let _ = tx.send(AppEvent::Repaint);
        });
    }

    fn disparar_salvamento(&mut self) {
        self.salvando = true;
        self.notificacao = None;

        let payload = AtualizarOrdemPayload {
            os: self.os_editada.clone(),
            usuario: self.usuario_logado.clone(),
        };
        let base = self.base.clone();
        let token = self.token.clone();
        let os_id = self.os_id;
        let tx = self.tx_evento.clone();
        let resultado_salvar = self.resultado_salvar.clone();

        crate::executor::spawn(move || {
            let res: Result<(), ErroServico> =
                gui_services::atualizar_ordem(&base, &token, os_id, &payload);
            *resultado_salvar.lock().unwrap() = Some(res.map_err(|e| e.mensagem));
            let _ = tx.send(AppEvent::Repaint);
        });
    }

    fn processar_resultados_carregamento(&mut self, _ctx: &egui::Context) {
        // Take the value out of the mutex in a short scope so the guard is dropped
        let maybe_resultado = {
            let mut guard = self.estado_os.lock().unwrap();
            guard.take()
        };

        if let Some(resultado) = maybe_resultado {
            match resultado {
                Ok(os) => {
                    self.os_editada = os;
                    self.recalcular_totais();
                }
                Err(e) => {
                    self.notificacao =
                        Some((format!("Erro ao carregar OS: {}", e), egui::Color32::RED))
                }
            }
        }

        if let Some(resultado) = self.resultado_salvar.lock().unwrap().take() {
            self.salvando = false;
            match resultado {
                Ok(_) => {
                    self.notificacao = Some((
                        "Ordem de Serviço salva com sucesso!".to_string(),
                        egui::Color32::GREEN,
                    ));
                    self.disparar_carregamento_os();
                }
                Err(e) => {
                    self.notificacao = Some((format!("Erro ao salvar: {}", e), egui::Color32::RED))
                }
            }
        }
    }
}
