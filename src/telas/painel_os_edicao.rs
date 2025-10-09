// src/telas/painel_os_edicao.rs

use crate::aplicacao::{AppEvent, TelaAtiva};
use crate::servicos::{OrdemServico, Peca, PecaOS};
use eframe::egui;
use std::sync::{Arc, Mutex};

// Abas internas da tela de edição de OS
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum AbaOs {
    Geral,
    Produtos,
    Servicos,
    Devolucoes,
    Despesas,
    Outras,
}

// A struct que guarda o estado da tela de edição.
pub struct TelaOsEdicao {
    os_data: OrdemServico,
    usuario_logado: String,
    // Estado para o seletor de peças
    lista_pecas_estoque: Vec<Peca>,
    filtro_peca_selecao: String,
    peca_selecionada_id: Option<u32>,
    quantidade_peca_adicionar: u32,
    // UI state
    aba_ativa: AbaOs,
    valor_servico: f64,
    // Serviços
    lista_servicos: Vec<crate::servicos::Servico>,
    servico_selecionado_id: Option<u32>,
    quantidade_servico: u32,
    // Busca / modais
    abrir_busca_cliente: bool,
    filtro_busca_cliente: String,
    abrir_busca_tecnico: bool,
    filtro_busca_tecnico: String,
    abrir_busca_tipo: bool,
    filtro_busca_tipo: String,
}

impl TelaOsEdicao {
    // O construtor recebe o ID da OS que precisa ser editada.
    pub fn new(os_id: u32, _endereco_servidor: Arc<Mutex<String>>) -> Self {
        let os_data = crate::servicos::buscar_os_por_id(os_id)
            .expect("Falha ao carregar dados da Ordem de Serviço");

        Self {
            os_data,
            usuario_logado: "admin".to_string(), // Placeholder
            lista_pecas_estoque: crate::banco_de_dados::estoque::listar_pecas().unwrap_or_default(),
            filtro_peca_selecao: String::new(),
            peca_selecionada_id: None,
            quantidade_peca_adicionar: 1,
            aba_ativa: AbaOs::Geral,
            valor_servico: 0.0,
            lista_servicos: crate::servicos::listar_servicos(),
            servico_selecionado_id: None,
            quantidade_servico: 1,
            abrir_busca_cliente: false,
            filtro_busca_cliente: String::new(),
            abrir_busca_tecnico: false,
            filtro_busca_tecnico: String::new(),
            abrir_busca_tipo: false,
            filtro_busca_tipo: String::new(),
        }
    }

    // O update desta tela também retorna um evento para a aplicação principal.
    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Option<AppEvent> {
        let mut evento_emitido = None;

        egui::CentralPanel::default().show(ctx, |ui| {
            // Top toolbar (ícones - OK/Cancelar)
            ui.horizontal(|ui| {
                if ui.button("🖶 Salvar").clicked() {
                    match crate::servicos::atualizar_os(&self.os_data, &self.usuario_logado) {
                        Ok(_) => println!("OS {} salva com sucesso!", self.os_data.id),
                        Err(e) => eprintln!("Erro ao salvar OS {}: {}", self.os_data.id, e),
                    }
                }
                if ui.button("✖ Cancelar").clicked() {
                    evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Tecnico));
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("Nº OS: {}", self.os_data.id));
                });
            });

            ui.separator();

            // Conteúdo principal com duas colunas
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Coluna esquerda: Geral
                    ui.vertical(|ui| {
                        ui.group(|ui| {
                            ui.heading("Geral");
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label("Solicitante / Atendente");
                                    ui.text_edit_singleline(&mut self.os_data.atendente);
                                    ui.label("Prev. Entrega");
                                    ui.text_edit_singleline(&mut self.os_data.prazo_entrega);
                                    ui.label("Hora Entrega");
                                    ui.text_edit_singleline(&mut self.os_data.horario_abertura);
                                });
                                ui.vertical(|ui| {
                                    ui.label("Solicitação");
                                    ui.text_edit_multiline(&mut self.os_data.observacoes);
                                });
                            });
                        });

                        ui.add_space(8.0);

                        // Abas simuladas
                        ui.horizontal(|ui| {
                            ui.selectable_value(&mut self.aba_ativa, AbaOs::Produtos, "Produtos");
                            ui.selectable_value(
                                &mut self.aba_ativa,
                                AbaOs::Servicos,
                                "Serviços/Mão de Obra",
                            );
                            ui.selectable_value(
                                &mut self.aba_ativa,
                                AbaOs::Devolucoes,
                                "Devoluções",
                            );
                            ui.selectable_value(
                                &mut self.aba_ativa,
                                AbaOs::Despesas,
                                "Despesas/Serv.Terceiros",
                            );
                            ui.selectable_value(
                                &mut self.aba_ativa,
                                AbaOs::Outras,
                                "Outras Informações",
                            );
                        });

                        ui.add_space(6.0);

                        // Conteúdo das abas (usar blocos para garantir retorno `()` em todos os braços)
                        match self.aba_ativa {
                            AbaOs::Produtos => {
                                self.ui_pecas_servicos(ui);
                            }
                            AbaOs::Servicos => {
                                self.ui_servicos(ui);
                            }
                            AbaOs::Devolucoes => {
                                ui.label("Nenhuma devolução registrada.");
                            }
                            AbaOs::Despesas => {
                                ui.label("Nenhuma despesa registrada.");
                            }
                            AbaOs::Outras => {
                                ui.label("Informações adicionais...");
                            }
                            AbaOs::Geral => {
                                // nada a fazer
                            }
                        }
                    });

                    // Coluna direita: informações do cliente, técnico e valores
                    ui.vertical(|ui| {
                        ui.group(|ui| {
                            ui.heading("Informações");

                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label("Cliente:");
                                    ui.text_edit_singleline(&mut self.os_data.cliente);
                                });
                                // lupa conectada ao campo
                                if ui
                                    .add_sized([28.0, 24.0], egui::Button::new("🔍"))
                                    .clicked()
                                {
                                    self.abrir_busca_cliente = true;
                                }
                            });

                            ui.label(format!("Telefone: {}", self.os_data.telefone_cliente));

                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label("Técnico:");
                                    ui.text_edit_singleline(
                                        &mut self.os_data.nome_tecnico_responsavel,
                                    );
                                });
                                if ui
                                    .add_sized([28.0, 24.0], egui::Button::new("🔍"))
                                    .clicked()
                                {
                                    self.abrir_busca_tecnico = true;
                                }
                            });

                            ui.separator();

                            ui.label("Condição de Pagamento / Tipo OS");
                            ui.vertical(|ui| {
                                ui.text_edit_singleline(&mut self.os_data.observacoes);
                                // botão de busca embaixo do campo tipo (conectado)
                                if ui
                                    .add_sized([80.0, 22.0], egui::Button::new("🔍 Buscar Tipo"))
                                    .clicked()
                                {
                                    self.abrir_busca_tipo = true;
                                }
                            });
                        });

                        ui.add_space(8.0);

                        ui.group(|ui| {
                            ui.heading("Valores");
                            ui.horizontal(|ui| {
                                ui.label("Valor Peças:");
                                ui.strong(format!("R$ {:.2}", self.os_data.total_pecas));
                            });
                            ui.horizontal(|ui| {
                                ui.label("Valor Serviço:");
                                ui.add(egui::DragValue::new(&mut self.valor_servico).speed(0.5));
                            });
                            ui.separator();
                            ui.horizontal(|ui| {
                                ui.label("Valor Total:");
                                ui.strong(format!(
                                    "R$ {:.2}",
                                    self.os_data.total_pecas + self.valor_servico
                                ));
                            });
                        });
                    });
                });
            });

            ui.separator();
            ui.heading("Histórico de Alterações");
            self.ui_tabela_historico(ui);
            ui.add_space(12.0);

            ui.horizontal(|ui| {
                if ui.button("Salvar Alterações").clicked() {
                    match crate::servicos::atualizar_os(&self.os_data, &self.usuario_logado) {
                        Ok(_) => println!("OS {} salva com sucesso!", self.os_data.id),
                        Err(e) => eprintln!("Erro ao salvar OS {}: {}", self.os_data.id, e),
                    }
                }
                if ui.button("Voltar ao Painel Técnico").clicked() {
                    evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Tecnico));
                }
            });
        });

        // Modal de busca de cliente
        if self.abrir_busca_cliente {
            egui::Window::new("Buscar Cliente")
                .resizable(true)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label("Filtrar:");
                    ui.text_edit_singleline(&mut self.filtro_busca_cliente);
                    ui.separator();
                    if let Ok(list) = crate::servicos::listar_clientes() {
                        for c in list.into_iter().filter(|c| {
                            c.nome
                                .to_lowercase()
                                .contains(&self.filtro_busca_cliente.to_lowercase())
                        }) {
                            if ui.button(format!("{} - {}", c.id, c.nome)).clicked() {
                                self.os_data.cliente = c.nome.clone();
                                self.abrir_busca_cliente = false;
                                break;
                            }
                        }
                    } else {
                        ui.label("Falha ao obter clientes");
                    }
                    if ui.button("Fechar").clicked() {
                        self.abrir_busca_cliente = false;
                    }
                });
        }

        // Modal de busca de técnico (lista de usuários com papel Técnico)
        if self.abrir_busca_tecnico {
            egui::Window::new("Buscar Técnico")
                .resizable(true)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label("Filtrar:");
                    ui.text_edit_singleline(&mut self.filtro_busca_tecnico);
                    ui.separator();
                    let usuarios = crate::servicos::listar_usuarios();
                    for u in usuarios.into_iter().filter(|u| {
                        u.nome_usuario
                            .to_lowercase()
                            .contains(&self.filtro_busca_tecnico.to_lowercase())
                    }) {
                        // nota: coluna role não está no InfoUsuario; estamos simplificando
                        if ui
                            .button(format!("{} - {}", u.id, u.nome_usuario))
                            .clicked()
                        {
                            self.os_data.nome_tecnico_responsavel = u.nome_usuario.clone();
                            self.abrir_busca_tecnico = false;
                            break;
                        }
                    }
                    if ui.button("Fechar").clicked() {
                        self.abrir_busca_tecnico = false;
                    }
                });
        }

        // Modal de busca de tipo
        if self.abrir_busca_tipo {
            egui::Window::new("Buscar Tipo de OS")
                .resizable(true)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label("Filtrar:");
                    ui.text_edit_singleline(&mut self.filtro_busca_tipo);
                    ui.separator();
                    let tipos = vec!["Ordem de Serviço", "Orçamento", "Manutenção", "Instalação"];
                    for t in tipos.into_iter().filter(|t| {
                        t.to_lowercase()
                            .contains(&self.filtro_busca_tipo.to_lowercase())
                    }) {
                        if ui.button(t).clicked() {
                            // colocar o tipo no campo observacoes por enquanto
                            self.os_data.observacoes = format!("Tipo: {}", t);
                            self.abrir_busca_tipo = false;
                            break;
                        }
                    }
                    if ui.button("Fechar").clicked() {
                        self.abrir_busca_tipo = false;
                    }
                });
        }

        evento_emitido
    }

    fn ui_detalhes_edicao(&mut self, ui: &mut egui::Ui) {
        // Este método foi reduzido — a maior parte dos campos foi movida para o novo layout
        ui.label("Parecer Técnico:");
        ui.text_edit_multiline(&mut self.os_data.parecer_tecnico);
    }

    fn ui_pecas_servicos(&mut self, ui: &mut egui::Ui) {
        ui.heading("Peças e Serviços");
        ui.horizontal(|ui| {
            ui.label("Adicionar Peça:");
            let nome_peca_selecionada = self
                .peca_selecionada_id
                .and_then(|id| self.lista_pecas_estoque.iter().find(|p| p.id == id))
                .map_or_else(
                    || "Selecione uma peça...".to_string(),
                    |p| p.descricao.clone(),
                );

            egui::ComboBox::from_id_salt("seletor_peca")
                .selected_text(nome_peca_selecionada)
                .show_ui(ui, |ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.filtro_peca_selecao)
                            .hint_text("🔎 Buscar..."),
                    );
                    ui.separator();
                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            let filtro = self.filtro_peca_selecao.to_lowercase();
                            for peca in self
                                .lista_pecas_estoque
                                .iter()
                                .filter(|p| p.descricao.to_lowercase().contains(&filtro))
                            {
                                ui.selectable_value(
                                    &mut self.peca_selecionada_id,
                                    Some(peca.id),
                                    &peca.descricao,
                                );
                            }
                        });
                });

            ui.add(
                egui::DragValue::new(&mut self.quantidade_peca_adicionar)
                    .speed(1.0)
                    .range(1..=99),
            );

            if ui.button("➕ Adicionar").clicked() {
                if let Some(id) = self.peca_selecionada_id {
                    if let Some(peca_estoque) = self.lista_pecas_estoque.iter().find(|p| p.id == id)
                    {
                        let peca_os = PecaOS {
                            id_peca: peca_estoque.id,
                            codigo_interno: peca_estoque.codigo_interno.clone(),
                            descricao: peca_estoque.descricao.clone(),
                            quantidade: self.quantidade_peca_adicionar,
                            preco_venda_unitario: peca_estoque.preco_venda,
                            preco_total: peca_estoque.preco_venda
                                * self.quantidade_peca_adicionar as f64,
                        };
                        self.os_data.pecas.push(peca_os);
                        self.recalcular_totais();
                        self.peca_selecionada_id = None;
                        self.quantidade_peca_adicionar = 1;
                    }
                }
            }
        });

        egui::Grid::new("grid_pecas_os")
            .num_columns(5)
            .show(ui, |ui| {
                ui.strong("Descrição");
                ui.strong("Qtd.");
                ui.strong("Vlr. Unit.");
                ui.strong("Vlr. Total");
                ui.strong("Ações");
                ui.end_row();

                for (idx, peca) in self.os_data.pecas.iter_mut().enumerate() {
                    ui.label(&peca.descricao);
                    ui.add(
                        egui::DragValue::new(&mut peca.quantidade)
                            .speed(1.0)
                            .range(1..=999),
                    );
                    ui.label(format!("R$ {:.2}", peca.preco_venda_unitario));
                    ui.label(format!("R$ {:.2}", peca.preco_total));
                    if ui.button("Remover").clicked() {
                        self.os_data.pecas.remove(idx);
                        self.recalcular_totais();
                        break;
                    }
                    ui.end_row();
                }
            });

        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Total em Peças:");
            ui.strong(format!("R$ {:.2}", self.os_data.total_pecas));
        });
    }

    fn ui_tabela_historico(&mut self, ui: &mut egui::Ui) {
        // Implementação simples para evitar warning de variável não usada.
        ui.label("Sem histórico disponível");
    }

    // Placeholder para a aba de serviços (Mão de obra). Implementação mínima
    // por enquanto para compilar e permitir futura expansão.
    fn ui_servicos(&mut self, ui: &mut egui::Ui) {
        ui.heading("Serviços / Mão de Obra");
        ui.label("Adicionar descrição do serviço e valor.");
        ui.horizontal(|ui| {
            ui.label("Descrição:");
            ui.text_edit_singleline(&mut self.os_data.observacoes);
        });
        ui.horizontal(|ui| {
            ui.label("Valor:");
            // Como valor de serviço separado usamos self.valor_servico
            ui.add(egui::DragValue::new(&mut self.valor_servico).speed(0.5));
        });
        ui.horizontal(|ui| {
            ui.label("Adicionar serviço:");
            let nome = self
                .servico_selecionado_id
                .and_then(|id| self.lista_servicos.iter().find(|s| s.id == id))
                .map_or_else(|| "Selecione...".to_string(), |s| s.nome.clone());
            egui::ComboBox::from_id_salt("seletor_servico")
                .selected_text(nome)
                .show_ui(ui, |ui| {
                    for s in &self.lista_servicos {
                        ui.selectable_value(&mut self.servico_selecionado_id, Some(s.id), &s.nome);
                    }
                });
            ui.add(egui::DragValue::new(&mut self.quantidade_servico).range(1..=99));
            if ui.button("Adicionar serviço").clicked() {
                if let Some(id) = self.servico_selecionado_id {
                    if let Some(s) = self.lista_servicos.iter().find(|x| x.id == id) {
                        let svc = crate::servicos::ServicoOS {
                            id_servico: s.id,
                            nome: s.nome.clone(),
                            descricao: s.descricao.clone(),
                            quantidade: self.quantidade_servico,
                            preco_unitario: s.preco,
                            preco_total: s.preco * self.quantidade_servico as f64,
                        };
                        self.os_data.servicos.push(svc);
                        self.recalcular_totais();
                    }
                }
            }
        });

        egui::Grid::new("grid_servicos_os")
            .num_columns(5)
            .show(ui, |ui| {
                ui.strong("Serviço");
                ui.strong("Qtd.");
                ui.strong("Vlr. Unit.");
                ui.strong("Vlr. Total");
                ui.strong("Ações");
                ui.end_row();

                for (idx, s) in self.os_data.servicos.iter_mut().enumerate() {
                    ui.label(&s.nome);
                    ui.add(
                        egui::DragValue::new(&mut s.quantidade)
                            .speed(1.0)
                            .range(1..=999),
                    );
                    ui.label(format!("R$ {:.2}", s.preco_unitario));
                    ui.label(format!("R$ {:.2}", s.preco_total));
                    if ui.button("Remover").clicked() {
                        self.os_data.servicos.remove(idx);
                        self.recalcular_totais();
                        break;
                    }
                    ui.end_row();
                }
            });
    }

    fn recalcular_totais(&mut self) {
        self.os_data.total_pecas = self.os_data.pecas.iter().map(|p| p.preco_total).sum();
        self.os_data.total_servicos = self.os_data.servicos.iter().map(|s| s.preco_total).sum();
    }
}
