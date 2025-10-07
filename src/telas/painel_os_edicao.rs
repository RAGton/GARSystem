// src/telas/painel_os_edicao.rs

use crate::aplicacao::{AppEvent, TelaAtiva};
use eframe::egui;
use std::sync::{Arc, Mutex};

// A struct que guarda o estado da tela de edição.
pub struct TelaOsEdicao {
    os_id: u32,
    // Futuramente, você carregará os dados da OS aqui.
    // Ex: os_data: Option<OrdemServico>
    parecer_tecnico: String,
}

impl TelaOsEdicao {
    // O construtor recebe o ID da OS que precisa ser editada.
    pub fn new(os_id: u32, _endereco_servidor: Arc<Mutex<String>>) -> Self {
        // Aqui seria o local ideal para iniciar uma carga assíncrona dos dados da OS do banco.
        // Por enquanto, apenas exibimos o ID.
        println!("Iniciando tela de edição para a OS de ID: {}", os_id);
        Self {
            os_id,
            parecer_tecnico: String::new(),
        }
    }

    // O update desta tela também retorna um evento para a aplicação principal.
    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Option<AppEvent> {
        let mut evento_emitido = None;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(format!("Gestão da Ordem de Serviço #{}", self.os_id));
            ui.separator();

            // Aqui você implementaria a UI detalhada da edição da OS
            // (parecer, peças, serviços, etc.)
            ui.label("Parecer Técnico:");
            ui.text_edit_multiline(&mut self.parecer_tecnico);
            ui.add_space(20.0);

            ui.horizontal(|ui| {
                if ui.button("Salvar Alterações").clicked() {
                    // 1. Lógica para enviar os dados para o backend (via canal mpsc) para salvar.
                    println!("Salvando alterações para a OS {}...", self.os_id);
                    // 2. Emite um evento para notificar o sucesso (opcional).
                    // evento_emitido = Some(AppEvent::MostrarNotificacao("OS salva com sucesso!".to_string()));
                }

                if ui.button("Voltar ao Painel Técnico").clicked() {
                    // Emite um evento para navegar de volta.
                    evento_emitido = Some(AppEvent::NavegarPara(TelaAtiva::Tecnico));
                }
            });
        });

        evento_emitido
    }
}
