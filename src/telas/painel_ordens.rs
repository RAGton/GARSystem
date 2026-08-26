// src/telas/painel_ordens.rs

use crate::aplicacao::AppEvent;
use crate::gui_services::{self, BaseHandle, ErroServico, TokenArc};
use crate::servicos::{OrdemServico, PapelUsuario};
use eframe::egui;
use genpdf::{elements, Document};
use std::sync::{Arc, Mutex};

fn gerar_texto_os(os: &crate::servicos::OrdemServico) -> String {
    let mut s = String::new();
    s.push_str(&format!("Ordem de Serviço Nº {}\n", os.id));
    s.push_str(&format!("Cliente: {}\n", os.cliente));
    s.push_str(&format!("Equipamento: {}\n", os.equipamento));
    s.push_str(&format!("Técnico: {}\n", os.nome_tecnico_responsavel));
    s.push_str(&format!("Status: {:?}\n\n", os.status));

    s.push_str("Peças:\n");
    for p in &os.pecas {
        s.push_str(&format!(
            "- {} x{} -> R$ {:.2}\n",
            p.descricao, p.quantidade, p.preco_total
        ));
    }
    s.push_str(&format!("Total Peças: R$ {:.2}\n\n", os.total_pecas));

    s.push_str("Serviços:\n");
    for svc in &os.servicos {
        s.push_str(&format!(
            "- {} x{} -> R$ {:.2}\n",
            svc.nome, svc.quantidade, svc.preco_total
        ));
    }
    s.push_str(&format!("Total Serviços: R$ {:.2}\n\n", os.total_servicos));

    s.push_str(&format!("Observações:\n{}\n", os.observacoes));
    s
}

pub struct TelaOrdens {
    base: BaseHandle,
    token: TokenArc,
    ordens: Arc<Mutex<Vec<OrdemServico>>>,
    filtro_busca: String,
    carregando: Arc<Mutex<bool>>,
    papel: PapelUsuario,
    modal_visualizar_aberto: bool,
    os_atual: Option<OrdemServico>,
    mensagem: Option<(String, bool)>,
}

fn exportar_pdf_os(os: &crate::servicos::OrdemServico, caminho: &str) -> Result<(), String> {
    let font_family = genpdf::fonts::from_files("./assets/fonts", "JetBrainsMono-Regular", None)
        .map_err(|e| e.to_string())?;
    let mut doc = Document::new(font_family);
    doc.set_minimal_conformance();
    let decorator = genpdf::SimplePageDecorator::new();
    doc.set_page_decorator(decorator);

    let texto = gerar_texto_os(os);
    let para = elements::Paragraph::new(texto);
    doc.push(para);

    doc.render_to_file(caminho).map_err(|e| e.to_string())
}

impl TelaOrdens {
    pub fn new(papel: PapelUsuario, base: BaseHandle, token: TokenArc) -> Self {
        Self {
            base,
            token,
            ordens: Arc::new(Mutex::new(Vec::new())),
            filtro_busca: String::new(),
            carregando: Arc::new(Mutex::new(false)),
            papel,
            modal_visualizar_aberto: false,
            os_atual: None,
            mensagem: None,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Option<AppEvent> {
        let mut evento_emitido = None;

        let precisa_carregar = {
            let ord_guard = self.ordens.lock().unwrap();
            let carreg = *self.carregando.lock().unwrap();
            ord_guard.is_empty() && !carreg
        };
        if precisa_carregar {
            *self.carregando.lock().unwrap() = true;
            let ord_clone = Arc::clone(&self.ordens);
            let carreg_clone = Arc::clone(&self.carregando);
            let ctx_clone = ctx.clone();
            let base = self.base.clone();
            let token = self.token.clone();
            crate::executor::spawn(move || {
                // Versão paginada: limita a 50 ordens (primeira página).
                // Telas que precisarem de mais podem iterar page=2,3,...
                let res = gui_services::listar_ordens_paginado(&base, &token, 1, 50);
                match res {
                    Ok(pagina) => {
                        let lista: Vec<OrdemServico> =
                            serde_json::from_value(serde_json::Value::Array(pagina.items))
                                .unwrap_or_default();
                        tracing::info!(
                            "✅ listar_ordens_paginado: página {}/{}, {} total",
                            pagina.page,
                            pagina.total_paginas,
                            pagina.total
                        );
                        *ord_clone.lock().unwrap() = lista;
                    }
                    Err(e) => tracing::error!("❌ listar_ordens_paginado: {}", e),
                }
                *carreg_clone.lock().unwrap() = false;
                ctx_clone.request_repaint();
            });
        }

        egui::TopBottomPanel::top("top_ordens").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("⬅ Voltar ao Dashboard").clicked() {
                    evento_emitido = Some(AppEvent::VoltarParaDashboard);
                }
                ui.separator();
                ui.heading("Ordens de Serviço");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Buscar:");
                ui.text_edit_singleline(&mut self.filtro_busca);
                if ui.button("Limpar").clicked() {
                    self.filtro_busca.clear();
                }
                // Botão criar dentro da área de busca (visível para Comercial/Administrador)
                use crate::servicos::PapelUsuario::*;
                if matches!(self.papel, Comercial | Administrador) {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("➕ Criar OS").clicked() {
                            evento_emitido =
                                Some(AppEvent::NavegarPara(crate::aplicacao::TelaAtiva::CriarOs));
                        }
                    });
                }
            });
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("grid_os_list")
                    .num_columns(6)
                    .striped(true)
                    .spacing([12.0, 8.0])
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("ID").strong());
                        ui.label(egui::RichText::new("Cliente").strong());
                        ui.label(egui::RichText::new("Equipamento").strong());
                        ui.label(egui::RichText::new("Status").strong());
                        ui.label(egui::RichText::new("Técnico").strong());
                        ui.label("");
                        ui.end_row();

                        let ords = { self.ordens.lock().unwrap().clone() };
                        for os in &ords {
                            if self.filtro_busca.is_empty()
                                || os
                                    .cliente
                                    .to_lowercase()
                                    .contains(&self.filtro_busca.to_lowercase())
                                || os
                                    .equipamento
                                    .to_lowercase()
                                    .contains(&self.filtro_busca.to_lowercase())
                            {
                                ui.label(os.id.to_string());
                                ui.label(&os.cliente);
                                ui.label(&os.equipamento);
                                ui.label(format!("{:?}", os.status));
                                ui.label(&os.nome_tecnico_responsavel);

                                // Ações por papel
                                match self.papel {
                                    PapelUsuario::Tecnico => {
                                        if ui.button("Gerenciar").clicked() {
                                            evento_emitido = Some(AppEvent::AbrirEditorOS(os.id));
                                        }
                                    }
                                    PapelUsuario::Comercial | PapelUsuario::Administrador => {
                                        ui.horizontal(|ui| {
                                            if ui.button("Abrir").clicked() {
                                                evento_emitido =
                                                    Some(AppEvent::AbrirEditorOS(os.id));
                                            }
                                            if ui.button("✚ Criar").clicked() {
                                                // abre a tela de criação
                                                evento_emitido = Some(AppEvent::NavegarPara(
                                                    crate::aplicacao::TelaAtiva::CriarOs,
                                                ));
                                            }
                                            if ui.button("📄 Ver/Imprimir").clicked() {
                                                // busca a OS por id e abre modal
                                                let t = crate::servicos::tenant_padrao();
                                                match crate::servicos::buscar_os_por_id(t, os.id) {
                                                    Ok(os_full) => {
                                                        self.os_atual = Some(os_full);
                                                        self.modal_visualizar_aberto = true;
                                                    }
                                                    Err(e) => eprintln!(
                                                        "Falha ao carregar OS {}: {}",
                                                        os.id, e
                                                    ),
                                                }
                                            }
                                        });
                                    }
                                    _ => {
                                        if ui.button("Ver").clicked() {
                                            evento_emitido = Some(AppEvent::AbrirEditorOS(os.id));
                                        }
                                    }
                                }

                                ui.end_row();
                            }
                        }
                    });
            });
        });

        // Modal de visualização / impressão de OS
        if self.modal_visualizar_aberto {
            if let Some(os) = self.os_atual.clone() {
                // clonamos a OS para evitar empréstimos simultâneos de `self` dentro do closure
                let os_clone = os.clone();
                egui::Window::new(format!("Visualizar OS {}", os_clone.id))
                    .resizable(true)
                    .collapsible(false)
                    .show(ctx, |ui| {
                        ui.vertical(|ui| {
                            ui.label(format!("Ordem de Serviço Nº {}", os_clone.id));
                            ui.separator();
                            ui.label(format!("Cliente: {}", os_clone.cliente));
                            ui.label(format!("Equipamento: {}", os_clone.equipamento));
                            ui.label(format!("Técnico: {}", os_clone.nome_tecnico_responsavel));
                            ui.label(format!("Status: {:?}", os_clone.status));
                            ui.add_space(6.0);
                            ui.heading("Peças");
                            for p in &os_clone.pecas {
                                ui.label(format!(
                                    "- {} x{} -> R$ {:.2}",
                                    p.descricao, p.quantidade, p.preco_total
                                ));
                            }
                            ui.add_space(6.0);
                            ui.heading("Serviços");
                            for s in &os_clone.servicos {
                                ui.label(format!(
                                    "- {} x{} -> R$ {:.2}",
                                    s.nome, s.quantidade, s.preco_total
                                ));
                            }
                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                if ui.button("Exportar TXT").clicked() {
                                    // Gera texto e salva em arquivo
                                    let texto = gerar_texto_os(&os_clone);
                                    let nome_arquivo = format!("os_{}.txt", os_clone.id);
                                    match std::fs::write(&nome_arquivo, texto) {
                                        Ok(_) => println!("Arquivo salvo: {}", nome_arquivo),
                                        Err(e) => eprintln!("Falha ao salvar arquivo: {}", e),
                                    }
                                }
                                if ui.button("Exportar TXT / Imprimir").clicked() {
                                    let nome_txt = format!("os_{}.txt", os_clone.id);
                                    let texto = gerar_texto_os(&os_clone);
                                    match std::fs::write(&nome_txt, texto) {
                                        Ok(_) => {
                                            println!("Arquivo salvo: {}", nome_txt);
                                            if let Err(e) = open::that(&nome_txt) {
                                                eprintln!("Falha ao abrir arquivo: {}", e);
                                            }
                                        }
                                        Err(e) => eprintln!("Falha ao salvar arquivo: {}", e),
                                    }
                                }
                                if ui.button("Exportar PDF").clicked() {
                                    let nome_pdf = format!("os_{}.pdf", os_clone.id);
                                    if let Err(e) = exportar_pdf_os(&os_clone, &nome_pdf) {
                                        eprintln!("Erro ao gerar PDF: {}", e);
                                    } else {
                                        println!("PDF salvo: {}", nome_pdf);
                                    }
                                }
                                if ui.button("Fechar").clicked() {
                                    self.modal_visualizar_aberto = false;
                                    self.os_atual = None;
                                }
                            });
                        });
                    });
            } else {
                // Caso inconsistente, fecha modal
                self.modal_visualizar_aberto = false;
            }
        }

        evento_emitido
    }
}
