// src/telas/painel_estoque.rs

use crate::aplicacao::AppEvent;
use crate::servicos::{Fornecedor, Peca};
use eframe::egui;

#[derive(PartialEq, Debug)]
enum AbaEstoque {
    Pecas,
    Fornecedores,
    EntradaNF,
    Movimentacoes,
}

pub struct TelaEstoque {
    aba_ativa: AbaEstoque,
    lista_pecas: std::sync::Arc<std::sync::Mutex<Vec<Peca>>>,
    lista_fornecedores: std::sync::Arc<std::sync::Mutex<Vec<Fornecedor>>>,
    // Formulários
    form_peca: Peca,
    form_fornecedor: Fornecedor,
    // Filtros
    filtro_peca: String,
    // Estado da UI
    peca_selecionada_edicao: Option<Peca>,
}

impl TelaEstoque {
    pub fn new() -> Self {
        let tela = Self {
            aba_ativa: AbaEstoque::Pecas,
            lista_pecas: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            lista_fornecedores: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            form_peca: Peca::default(),
            form_fornecedor: Fornecedor::default(),
            filtro_peca: String::new(),
            peca_selecionada_edicao: None,
        };

        // Carrega as listas em background
        let pecas_clone = tela.lista_pecas.clone();
        let fornec_clone = tela.lista_fornecedores.clone();
        crate::executor::spawn(move || {
            let p = crate::banco_de_dados::estoque::listar_pecas().unwrap_or_default();
            let f = crate::banco_de_dados::estoque::listar_fornecedores().unwrap_or_default();
            *pecas_clone.lock().unwrap() = p;
            *fornec_clone.lock().unwrap() = f;
        });

        tela
    }

    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Option<AppEvent> {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Gestão de Estoque");
            ui.separator();

            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.aba_ativa, AbaEstoque::Pecas, "📦 Peças");
                ui.selectable_value(
                    &mut self.aba_ativa,
                    AbaEstoque::Fornecedores,
                    "🚚 Fornecedores",
                );
                ui.selectable_value(
                    &mut self.aba_ativa,
                    AbaEstoque::EntradaNF,
                    "📥 Entrada (NF)",
                );
                ui.selectable_value(
                    &mut self.aba_ativa,
                    AbaEstoque::Movimentacoes,
                    "📋 Movimentações",
                );
            });

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| match self.aba_ativa {
                AbaEstoque::Pecas => self.ui_pecas(ui),
                AbaEstoque::Fornecedores => self.ui_fornecedores(ui),
                AbaEstoque::EntradaNF => {
                    ui.label("WIP: Tela de Entrada de Nota Fiscal");
                }
                AbaEstoque::Movimentacoes => {
                    ui.label("WIP: Tela de Movimentações de Estoque");
                }
            });
        });
        None
    }

    fn ui_pecas(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("📝 Cadastrar Nova Peça", |ui| {
            self.form_nova_peca(ui);
        });
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("🔎 Buscar Peça:");
            ui.text_edit_singleline(&mut self.filtro_peca);
        });

        self.tabela_pecas(ui);

        if let Some(mut peca_edicao) = self.peca_selecionada_edicao.clone() {
            let mut open = true;
            egui::Window::new("Editar Peça")
                .open(&mut open)
                .show(ui.ctx(), |ui| {
                    self.form_edicao_peca(ui, &mut peca_edicao);
                    if ui.button("Salvar Alterações").clicked() {
                        if let Err(e) = crate::banco_de_dados::estoque::atualizar_peca(&peca_edicao)
                        {
                            eprintln!("Erro ao atualizar peça: {}", e);
                        } else {
                            self.peca_selecionada_edicao = None;
                            self.recarregar_listas();
                        }
                    }
                });
            if !open {
                self.peca_selecionada_edicao = None;
            }
        }
    }

    fn tabela_pecas(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("grid_pecas")
            .num_columns(6)
            .spacing([10.0, 4.0])
            .show(ui, |ui| {
                ui.strong("Cód. Interno");
                ui.strong("Estoque");
                ui.strong("Descrição");
                ui.strong("Preço Venda");
                ui.strong("Localização");
                ui.strong("Ações");
                ui.end_row();

                let filtro = self.filtro_peca.to_lowercase();
                let lista = { self.lista_pecas.lock().unwrap().clone() };
                for peca in lista.iter().filter(|p| {
                    p.descricao.to_lowercase().contains(&filtro)
                        || p.codigo_interno.to_lowercase().contains(&filtro)
                }) {
                    ui.label(&peca.codigo_interno);
                    ui.label(peca.estoque_atual.to_string());
                    ui.label(&peca.descricao);
                    ui.label(format!("R$ {:.2}", peca.preco_venda));
                    ui.label(&peca.localizacao);
                    if ui.button("✏️").clicked() {
                        self.peca_selecionada_edicao = Some(peca.clone());
                    }
                    ui.end_row();
                }
            });
    }

    fn form_nova_peca(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("form_peca").num_columns(2).show(ui, |ui| {
            ui.label("Código Interno:");
            ui.text_edit_singleline(&mut self.form_peca.codigo_interno);
            ui.end_row();

            ui.label("Descrição Completa:");
            ui.text_edit_singleline(&mut self.form_peca.descricao);
            ui.end_row();

            ui.label("Fabricante:");
            ui.text_edit_singleline(&mut self.form_peca.fabricante);
            ui.end_row();

            ui.label("Localização (Prateleira):");
            ui.text_edit_singleline(&mut self.form_peca.localizacao);
            ui.end_row();

            ui.label("Preço de Venda:");
            ui.add(
                egui::DragValue::new(&mut self.form_peca.preco_venda)
                    .speed(0.1)
                    .prefix("R$ "),
            );
            ui.end_row();
        });

        if ui.button("Salvar Nova Peça").clicked() {
            if let Err(e) = crate::banco_de_dados::estoque::criar_peca(&self.form_peca) {
                eprintln!("Erro ao criar peça: {}", e);
            } else {
                self.form_peca = Peca::default();
                self.recarregar_listas();
            }
        }
    }

    fn form_edicao_peca(&mut self, ui: &mut egui::Ui, peca: &mut Peca) {
        egui::Grid::new("form_peca_edicao")
            .num_columns(2)
            .show(ui, |ui| {
                ui.label("Código Interno:");
                ui.text_edit_singleline(&mut peca.codigo_interno);
                ui.end_row();

                ui.label("Descrição Completa:");
                ui.text_edit_singleline(&mut peca.descricao);
                ui.end_row();

                ui.label("Preço de Venda:");
                ui.add(
                    egui::DragValue::new(&mut peca.preco_venda)
                        .speed(0.1)
                        .prefix("R$ "),
                );
                ui.end_row();
            });
    }

    fn ui_fornecedores(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("📝 Cadastrar Novo Fornecedor", |ui| {
            egui::Grid::new("form_fornecedor")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label("Nome:");
                    ui.text_edit_singleline(&mut self.form_fornecedor.nome);
                    ui.end_row();
                    ui.label("CNPJ:");
                    ui.text_edit_singleline(&mut self.form_fornecedor.cnpj);
                    ui.end_row();
                });
            if ui.button("Salvar Fornecedor").clicked() {
                if let Err(e) =
                    crate::banco_de_dados::estoque::criar_fornecedor(&self.form_fornecedor)
                {
                    eprintln!("Erro ao criar fornecedor: {}", e);
                } else {
                    self.form_fornecedor = Fornecedor::default();
                    self.recarregar_listas();
                }
            }
        });
        ui.separator();

        egui::Grid::new("grid_fornecedores")
            .num_columns(2)
            .show(ui, |ui| {
                ui.strong("Nome");
                ui.strong("CNPJ");
                ui.end_row();

                let lista = { self.lista_fornecedores.lock().unwrap().clone() };
                for f in &lista {
                    ui.label(&f.nome);
                    ui.label(&f.cnpj);
                    ui.end_row();
                }
            });
    }

    fn recarregar_listas(&mut self) {
        let pecas = self.lista_pecas.clone();
        let fornec = self.lista_fornecedores.clone();
        crate::executor::spawn(move || {
            let p = crate::banco_de_dados::estoque::listar_pecas().unwrap_or_default();
            let f = crate::banco_de_dados::estoque::listar_fornecedores().unwrap_or_default();
            *pecas.lock().unwrap() = p;
            *fornec.lock().unwrap() = f;
        });
    }
}
