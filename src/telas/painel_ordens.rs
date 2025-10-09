// src/telas/painel_ordens.rs

use crate::aplicacao::AppEvent;
use crate::servicos::{OrdemServico, PapelUsuario};
use eframe::egui;
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

#[allow(dead_code)]
fn exportar_pdf_os(os: &crate::servicos::OrdemServico, caminho: &str) -> Result<(), String> {
    use printpdf::{Mm, PdfDocument};
    use std::fs::File;

    // Monta o texto simples
    let mut texto = String::new();
    texto.push_str(&format!("Ordem de Serviço Nº {}\n\n", os.id));
    texto.push_str(&format!("Cliente: {}\n", os.cliente));
    texto.push_str(&format!("Equipamento: {}\n", os.equipamento));
    texto.push_str(&format!("Técnico: {}\n", os.nome_tecnico_responsavel));
    texto.push_str(&format!("Status: {:?}\n\n", os.status));
    texto.push_str("Peças:\n");
    for p in &os.pecas {
        texto.push_str(&format!(
            "- {} x{} -> R$ {:.2}\n",
            p.descricao, p.quantidade, p.preco_total
        ));
    }
    texto.push_str(&format!("Total Peças: R$ {:.2}\n\n", os.total_pecas));
    texto.push_str("Serviços:\n");
    for svc in &os.servicos {
        texto.push_str(&format!(
            "- {} x{} -> R$ {:.2}\n",
            svc.nome, svc.quantidade, svc.preco_total
        ));
    }
    texto.push_str(&format!("Total Serviços: R$ {:.2}\n\n", os.total_servicos));
    texto.push_str(&format!("Observações:\n{}\n", os.observacoes));

    // Cria documento PDF
    let (doc, page1, layer1) =
        PdfDocument::new(&format!("OS {}", os.id), Mm(210.0), Mm(297.0), "Layer 1");

    // Tenta carregar fonte local (assets/fonts/JetBrainsMono-Regular.ttf)
    let font_path = "assets/fonts/JetBrainsMono-Regular.ttf";
    let font_file = File::open(font_path).map_err(|e| format!("Falha ao abrir fonte: {}", e))?;
    let font = doc
        .add_external_font(font_file)
        .map_err(|e| format!("Falha ao adicionar fonte: {}", e))?;

    let current_layer = doc.get_page(page1).get_layer(layer1);

    // Escrever o texto linha por linha
    let mut y = Mm(280.0);
    let lines: Vec<&str> = texto.lines().collect();
    for line in lines {
        current_layer.use_text(line, 12.0, Mm(10.0), y, &font);
        y -= Mm(6.0);
        if y.0 < 10.0 {
            // sem paginação por simplicidade
            break;
        }
    }

    use std::io::BufWriter;
    let file = File::create(caminho).map_err(|e| format!("Falha ao criar arquivo: {}", e))?;
    let mut writer = BufWriter::new(file);
    doc.save(&mut writer)
        .map_err(|e| format!("Falha ao salvar PDF: {}", e))?;

    Ok(())
}

pub struct TelaOrdens {
    ordens: Arc<Mutex<Vec<OrdemServico>>>,
    filtro_busca: String,
    carregando: Arc<Mutex<bool>>,
    papel: PapelUsuario,
    // Estado para modal de visualização/impressão
    modal_visualizar_aberto: bool,
    os_atual: Option<OrdemServico>,
}

impl TelaOrdens {
    pub fn new(papel: PapelUsuario, _endereco_servidor: Arc<Mutex<String>>) -> Self {
        Self {
            ordens: Arc::new(Mutex::new(Vec::new())),
            filtro_busca: String::new(),
            carregando: Arc::new(Mutex::new(false)),
            papel,
            modal_visualizar_aberto: false,
            os_atual: None,
        }
    }

    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Option<AppEvent> {
        let mut evento_emitido = None;

        // Carrega ordens em background se necessário
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
            crate::executor::spawn(move || {
                match crate::servicos::listar_ordens_servico() {
                    Ok(res) => {
                        let mut guard = ord_clone.lock().unwrap();
                        *guard = res;
                    }
                    Err(e) => {
                        eprintln!("Erro ao carregar ordens de serviço: {}", e);
                    }
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
                                                match crate::servicos::buscar_os_por_id(os.id) {
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
                                if ui.button("Exportar PDF / Imprimir").clicked() {
                                    let nome_pdf = format!("os_{}.pdf", os_clone.id);
                                    match exportar_pdf_os(&os_clone, &nome_pdf) {
                                        Ok(_) => {
                                            println!("PDF gerado: {}", nome_pdf);
                                            // Tenta abrir com aplicativo padrão
                                            if let Err(e) = open::that(&nome_pdf) {
                                                eprintln!("Falha ao abrir PDF: {}", e);
                                            }
                                        }
                                        Err(err) => eprintln!("Erro ao gerar PDF: {}", err),
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
