// src/telas/painel_os_edicao.rs

use crate::aplicacao::{AppEvent, TelaAtiva};
use crate::servicos::{OrdemServico, Peca, PecaOS};
use eframe::egui;
use std::sync::{Arc, Mutex};

// A struct que guarda o estado da tela de edição.
pub struct TelaOsEdicao {
    os_data: OrdemServico,
    usuario_logado: String,
    // Estado para o seletor de peças
    lista_pecas_estoque: Vec<Peca>,
    filtro_peca_selecao: String,
    peca_selecionada_id: Option<u32>,
    quantidade_peca_adicionar: u32,
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
        }
    }

    // O update desta tela também retorna um evento para a aplicação principal.
    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Option<AppEvent> {
        let mut evento_emitido = None;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(format!("Gestão da Ordem de Serviço #{}", self.os_data.id));
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                self.ui_detalhes_edicao(ui);
                ui.separator();
                self.ui_pecas_servicos(ui); // Nova seção de peças
            });

            ui.separator();
            ui.heading("Histórico de Alterações");
            self.ui_tabela_historico(ui);
            ui.add_space(20.0);

            ui.horizontal(|ui| {
                if ui.button("Salvar Alterações").clicked() {
                    match crate::servicos::atualizar_os(&self.os_data, &self.usuario_logado) {
                        Ok(_) => println!("OS {} salva com sucesso!", self.os_data.id),
                        Err(e) => eprintln!("Erro ao salvar OS {}: {}", self.os_data.id, e),
                    }
                    // Idealmente, aqui você emitiria um evento para mostrar uma notificação de sucesso/erro.
                    // evento_emitido = Some(AppEvent::MostrarNotificacao("OS salva!".to_string()));
                }
                if ui.button("Voltar ao Painel Técnico").clicked() {
                    evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Tecnico));
                }
            });
        });

        evento_emitido
    }

    fn ui_detalhes_edicao(&mut self, ui: &mut egui::Ui) {
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
                        self.peca_selecionada_id = None; // Resetar seleção
                        self.quantidade_peca_adicionar = 1;
                    }
                }
            }
        });

        egui::Grid::new("grid_pecas_os")
            .num_columns(4)
            .show(ui, |ui| {
                ui.strong("Descrição da Peça");
                ui.strong("Qtd.");
                ui.strong("Vlr. Unit.");
                ui.strong("Vlr. Total");
                ui.end_row();

                for peca in &self.os_data.pecas {
                    ui.label(&peca.descricao);
                    ui.label(peca.quantidade.to_string());
                    ui.label(format!("R$ {:.2}", peca.preco_venda_unitario));
                    ui.label(format!("R$ {:.2}", peca.preco_total));
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

    fn recalcular_totais(&mut self) {
        self.os_data.total_pecas = self.os_data.pecas.iter().map(|p| p.preco_total).sum();
    }
}
