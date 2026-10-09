// src/telas/painel_estoque.rs

use crate::aplicacao::AppEvent;
use crate::gui_services::{self, BaseHandle, ErroServico, TokenArc};
use crate::servicos::{Fornecedor, Peca};
use eframe::egui;

/// Tokens de cor (mesmos valores de `telas/theme/cores.rs` da branch de rebrand;
/// locais aqui porque esta base ainda não possui o módulo `theme`).
mod theme {
    use eframe::egui::Color32;
    pub const PRIMARY: Color32 = Color32::from_rgb(0, 150, 255);
    pub const PRIMARY_PRESSED: Color32 = Color32::from_rgb(0, 71, 171);
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(160, 165, 180);
    pub const ERROR: Color32 = Color32::from_rgb(229, 57, 53);
    pub const SUCCESS: Color32 = Color32::from_rgb(34, 197, 94);
    pub const WARNING: Color32 = Color32::from_rgb(245, 158, 11);
}

#[derive(PartialEq, Debug)]
enum AbaEstoque {
    Pecas,
    Fornecedores,
    EntradaNF,
    Movimentacoes,
}

pub struct TelaEstoque {
    base: BaseHandle,
    token: TokenArc,
    aba_ativa: AbaEstoque,
    lista_pecas: std::sync::Arc<std::sync::Mutex<Vec<Peca>>>,
    lista_fornecedores: std::sync::Arc<std::sync::Mutex<Vec<Fornecedor>>>,
    form_peca: Peca,
    form_fornecedor: Fornecedor,
    filtro_peca: String,
    peca_selecionada_edicao: Option<Peca>,
    mensagem: Option<(String, bool)>,
    mensagem_edicao: Option<(String, bool)>,
    mostrar_form_nova_peca: bool,
}

impl TelaEstoque {
    pub fn new(base: BaseHandle, token: TokenArc) -> Self {
        let tela = Self {
            base,
            token,
            aba_ativa: AbaEstoque::Pecas,
            lista_pecas: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            lista_fornecedores: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            form_peca: Peca::default(),
            form_fornecedor: Fornecedor::default(),
            filtro_peca: String::new(),
            peca_selecionada_edicao: None,
            mensagem: None,
            mensagem_edicao: None,
            mostrar_form_nova_peca: false,
        };

        // Carrega a lista de peças via API autenticada (PAGINADA).
        // Limita a primeira página com 50 itens para não carregar 10k+ registros.
        let pecas_clone = tela.lista_pecas.clone();
        let base = tela.base.clone();
        let token = tela.token.clone();
        crate::executor::spawn(move || {
            let res = gui_services::listar_pecas_paginado(&base, &token, 1, 50);
            match res {
                Ok(pagina) => {
                    // Decodifica items como Vec<Peca>
                    let lista: Vec<Peca> =
                        serde_json::from_value(serde_json::Value::Array(pagina.items))
                            .unwrap_or_default();
                    tracing::info!(
                        "✅ listar_pecas_paginado: página {}/{}, {} total",
                        pagina.page,
                        pagina.total_paginas,
                        pagina.total
                    );
                    *pecas_clone.lock().unwrap() = lista;
                }
                Err(e) => tracing::error!("❌ listar_pecas_paginado: {}", e),
            }
        });

        tela
    }

    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Option<AppEvent> {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.heading(egui::RichText::new("Gestão de Estoque").size(25.0).strong());
                    ui.label(egui::RichText::new("Peças, fornecedores e movimentações").color(ui.visuals().weak_text_color()));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let total = self.lista_pecas.lock().map(|p| p.len()).unwrap_or(0);
                    ui.label(egui::RichText::new(format!("{} itens carregados", total)).color(theme::TEXT_SECONDARY));
                });
            });
            ui.add_space(12.0);
            ui.separator();
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                for (aba, titulo) in [
                    (AbaEstoque::Pecas, "Peças"),
                    (AbaEstoque::Fornecedores, "Fornecedores"),
                    (AbaEstoque::EntradaNF, "Entrada (NF)"),
                    (AbaEstoque::Movimentacoes, "Movimentações"),
                ] {
                    let selecionada = self.aba_ativa == aba;
                    let texto = if selecionada { egui::RichText::new(titulo).strong().color(egui::Color32::from_rgb(230, 246, 255)) } else { egui::RichText::new(titulo).color(ui.visuals().text_color()) };
                    if ui.add_sized([ui.available_width().min(170.0).max(110.0), 34.0], egui::Button::new(texto).selected(selecionada)).clicked() { self.aba_ativa = aba; }
                }
            });
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(8.0);
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| match self.aba_ativa {
                AbaEstoque::Pecas => self.ui_pecas(ui),
                AbaEstoque::Fornecedores => self.ui_fornecedores(ui),
                AbaEstoque::EntradaNF => { ui.add_space(28.0); ui.vertical_centered(|ui| { ui.heading("Entrada de Nota Fiscal"); ui.add_space(8.0); ui.label("Este fluxo ainda não está implementado."); ui.label("Nenhum dado fictício é exibido."); }); }
                AbaEstoque::Movimentacoes => { ui.add_space(28.0); ui.vertical_centered(|ui| { ui.heading("Movimentações de estoque"); ui.add_space(8.0); ui.label("O histórico de movimentações ainda não está disponível nesta interface."); ui.label("Nenhum dado fictício é exibido."); }); }
            });
        });
        None
    }

    fn ui_pecas(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading("Peças cadastradas");
                ui.label(
                    egui::RichText::new("Consulte e mantenha os itens do estoque.")
                        .color(ui.visuals().weak_text_color()),
                );
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("＋ Nova peça")
                                .strong()
                                .color(egui::Color32::WHITE),
                        )
                        .fill(theme::PRIMARY_PRESSED)
                        .min_size(egui::vec2(130.0, 36.0)),
                    )
                    .clicked()
                {
                    self.form_peca = Peca::default();
                    self.mensagem = None;
                    self.mostrar_form_nova_peca = !self.mostrar_form_nova_peca;
                }
            });
        });
        if self.mostrar_form_nova_peca {
            egui::Frame::group(ui.style())
                .inner_margin(egui::Margin::same(14))
                .show(ui, |ui| {
                    ui.heading("Cadastrar peça");
                    ui.add_space(8.0);
                    self.form_nova_peca(ui);
                });
        }
        if let Some((mensagem, sucesso)) = &self.mensagem {
            let cor = if *sucesso {
                theme::SUCCESS
            } else {
                theme::ERROR
            };
            ui.add_space(8.0);
            ui.label(egui::RichText::new(mensagem).color(cor));
        }
        ui.add_space(12.0);
        egui::Frame::group(ui.style())
            .inner_margin(egui::Margin::same(12))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Buscar").strong());
                    ui.add_sized(
                        [ui.available_width().min(440.0), 34.0],
                        egui::TextEdit::singleline(&mut self.filtro_peca)
                            .hint_text("Nome, descrição ou código interno…"),
                    );
                    if !self.filtro_peca.is_empty() && ui.button("Limpar").clicked() {
                        self.filtro_peca.clear();
                    }
                });
                ui.add_space(8.0);
                self.tabela_pecas(ui);
            });

        if let Some(mut peca_edicao) = self.peca_selecionada_edicao.clone() {
            let mut open = true;
            let mut salvar = false;
            egui::Window::new("Editar Peça")
                .open(&mut open)
                .show(ui.ctx(), |ui| {
                    self.form_edicao_peca(ui, &mut peca_edicao);
                    if let Some((msg, sucesso)) = &self.mensagem_edicao {
                        let cor = if *sucesso {
                            theme::SUCCESS
                        } else {
                            theme::ERROR
                        };
                        ui.label(egui::RichText::new(msg).color(cor));
                    }
                    if ui.button("Salvar Alterações").clicked() {
                        salvar = true;
                    }
                });
            if salvar {
                match validar_peca(&peca_edicao) {
                    Err(msg) => self.mensagem_edicao = Some((msg, false)),
                    Ok(()) => {
                        let t = crate::servicos::tenant_padrao();
                        match crate::banco_de_dados::estoque::atualizar_peca(t, &peca_edicao) {
                            Ok(()) => {
                                self.peca_selecionada_edicao = None;
                                self.mensagem_edicao = None;
                                self.mensagem =
                                    Some(("Peça atualizada com sucesso.".to_string(), true));
                                self.recarregar_listas();
                                return;
                            }
                            Err(e) => {
                                self.mensagem_edicao = Some((
                                    format!("Não foi possível atualizar a peça: {}", e),
                                    false,
                                ));
                            }
                        }
                    }
                }
            }
            if !open {
                self.peca_selecionada_edicao = None;
                self.mensagem_edicao = None;
            } else {
                // Mantém o que foi digitado entre quadros.
                self.peca_selecionada_edicao = Some(peca_edicao);
            }
        }
    }

    fn tabela_pecas(&mut self, ui: &mut egui::Ui) {
        let filtro = self.filtro_peca.trim().to_lowercase();
        let lista = self
            .lista_pecas
            .lock()
            .map(|p| p.clone())
            .unwrap_or_default();
        let filtradas: Vec<Peca> = lista
            .into_iter()
            .filter(|p| {
                p.nome.to_lowercase().contains(&filtro)
                    || p.descricao.to_lowercase().contains(&filtro)
                    || p.codigo_interno.to_lowercase().contains(&filtro)
            })
            .collect();
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Resultados").strong());
            ui.label(
                egui::RichText::new(format!("{}", filtradas.len()))
                    .color(ui.visuals().weak_text_color()),
            );
        });
        ui.add_space(6.0);
        if filtradas.is_empty() {
            ui.add_space(18.0);
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("▦").size(34.0).color(theme::PRIMARY));
                ui.add_space(6.0);
                ui.heading(if filtro.is_empty() {
                    "Seu estoque ainda não tem peças"
                } else {
                    "Nenhuma peça encontrada"
                });
                ui.label(if filtro.is_empty() {
                    "Cadastre a primeira peça para começar a controlar o estoque."
                } else {
                    "Tente outro nome, descrição ou código interno."
                });
                if filtro.is_empty()
                    && ui
                        .add(
                            egui::Button::new("＋ Cadastrar primeira peça")
                                .fill(theme::PRIMARY_PRESSED),
                        )
                        .clicked()
                {
                    self.mostrar_form_nova_peca = true;
                }
            });
            return;
        }
        egui::ScrollArea::horizontal().show(ui, |ui| {
            egui::Grid::new("grid_pecas")
                .num_columns(6)
                .striped(true)
                .min_col_width(90.0)
                .spacing([16.0, 10.0])
                .show(ui, |ui| {
                    ui.strong("Código interno");
                    ui.strong("Estoque");
                    ui.strong("Peça / descrição");
                    ui.strong("Preço de venda");
                    ui.strong("Localização");
                    ui.strong("Ações");
                    ui.end_row();
                    for peca in &filtradas {
                        ui.label(egui::RichText::new(&peca.codigo_interno).monospace());
                        ui.label(
                            egui::RichText::new(peca.estoque_atual.to_string())
                                .strong()
                                .color(if peca.estoque_atual <= 0 {
                                    theme::ERROR
                                } else if peca.estoque_atual <= peca.estoque_minimo {
                                    theme::WARNING
                                } else {
                                    theme::SUCCESS
                                }),
                        );
                        ui.vertical(|ui| {
                            let (titulo, detalhe) = if peca.nome.trim().is_empty() {
                                (peca.descricao.as_str(), "")
                            } else {
                                (peca.nome.as_str(), peca.descricao.as_str())
                            };
                            ui.label(egui::RichText::new(titulo).strong());
                            if !detalhe.is_empty() {
                                ui.label(
                                    egui::RichText::new(detalhe)
                                        .small()
                                        .color(ui.visuals().weak_text_color()),
                                );
                            }
                        });
                        ui.label(formatar_brl(peca.preco_venda));
                        ui.label(if peca.localizacao.trim().is_empty() {
                            "—"
                        } else {
                            &peca.localizacao
                        });
                        if ui.button("Editar").clicked() {
                            self.peca_selecionada_edicao = Some(peca.clone());
                        }
                        ui.end_row();
                    }
                });
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

            ui.label("Estoque inicial:");
            ui.add(egui::DragValue::new(&mut self.form_peca.estoque_atual).range(0..=i32::MAX));
            ui.end_row();

            ui.label("Estoque mínimo:");
            ui.add(egui::DragValue::new(&mut self.form_peca.estoque_minimo).range(0..=i32::MAX));
            ui.end_row();
        });

        ui.horizontal(|ui| {
            if ui.button("Salvar peça").clicked() {
                if let Err(msg) = validar_peca(&self.form_peca) {
                    self.mensagem = Some((msg, false));
                } else {
                    let t = crate::servicos::tenant_padrao();
                    match crate::banco_de_dados::estoque::criar_peca(t, &self.form_peca) {
                        Ok(()) => {
                            self.form_peca = Peca::default();
                            self.mensagem =
                                Some(("Peça cadastrada com sucesso.".to_string(), true));
                            self.mostrar_form_nova_peca = false;
                            self.recarregar_listas();
                        }
                        Err(e) => {
                            self.mensagem =
                                Some((format!("Não foi possível cadastrar a peça: {}", e), false));
                        }
                    }
                }
            }
            if ui.button("Cancelar").clicked() {
                self.mostrar_form_nova_peca = false;
                self.mensagem = None;
            }
        });
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
                let t = crate::servicos::tenant_padrao();
                if let Err(e) =
                    crate::banco_de_dados::estoque::criar_fornecedor(t, &self.form_fornecedor)
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
        let base = self.base.clone();
        let token = self.token.clone();
        crate::executor::spawn(move || {
            match gui_services::listar_pecas_paginado(&base, &token, 1, 50) {
                Ok(pagina) => {
                    let lista: Vec<Peca> =
                        serde_json::from_value(serde_json::Value::Array(pagina.items))
                            .unwrap_or_default();
                    *pecas.lock().unwrap() = lista;
                }
                Err(e) => {
                    let e: ErroServico = e;
                    tracing::error!("❌ recarregar peças: {}", e);
                }
            }
        });
    }
}

/// Valida os campos obrigatórios de uma peça (o banco não possui coluna `nome`).
fn validar_peca(p: &Peca) -> Result<(), String> {
    if p.codigo_interno.trim().is_empty() {
        return Err("Informe o código interno da peça.".to_string());
    }
    if p.descricao.trim().is_empty() {
        return Err("Informe a descrição da peça.".to_string());
    }
    if !p.preco_venda.is_finite() || p.preco_venda < 0.0 {
        return Err("O preço de venda não pode ser negativo.".to_string());
    }
    if p.estoque_atual < 0 || p.estoque_minimo < 0 {
        return Err("As quantidades de estoque não podem ser negativas.".to_string());
    }
    Ok(())
}

/// Formata valor em reais no padrão brasileiro: `R$ 1.234,56`.
fn formatar_brl(valor: f64) -> String {
    let centavos = (valor.abs() * 100.0).round() as u64;
    let (inteiro, cent) = (centavos / 100, centavos % 100);
    let digitos = inteiro.to_string();
    let mut milhar = String::new();
    for (i, c) in digitos.chars().enumerate() {
        if i > 0 && (digitos.len() - i) % 3 == 0 {
            milhar.push('.');
        }
        milhar.push(c);
    }
    let sinal = if valor < 0.0 && centavos > 0 { "-" } else { "" };
    format!("{}R$ {},{:02}", sinal, milhar, cent)
}

#[cfg(test)]
mod testes {
    use super::*;

    fn peca_valida() -> Peca {
        Peca {
            codigo_interno: "P-001".into(),
            descricao: "Filtro de ar".into(),
            ..Peca::default()
        }
    }

    #[test]
    fn validacao_aceita_peca_valida() {
        assert!(validar_peca(&peca_valida()).is_ok());
    }

    #[test]
    fn validacao_exige_codigo_e_descricao() {
        let mut p = peca_valida();
        p.codigo_interno = "  ".into();
        assert!(validar_peca(&p).is_err());
        let mut p = peca_valida();
        p.descricao.clear();
        assert!(validar_peca(&p).is_err());
    }

    #[test]
    fn validacao_rejeita_valores_negativos() {
        let mut p = peca_valida();
        p.preco_venda = -1.0;
        assert!(validar_peca(&p).is_err());
        let mut p = peca_valida();
        p.estoque_minimo = -1;
        assert!(validar_peca(&p).is_err());
    }

    #[test]
    fn formata_reais() {
        assert_eq!(formatar_brl(0.0), "R$ 0,00");
        assert_eq!(formatar_brl(12.5), "R$ 12,50");
        assert_eq!(formatar_brl(1234.567), "R$ 1.234,57");
        assert_eq!(formatar_brl(1234567.0), "R$ 1.234.567,00");
    }
}
