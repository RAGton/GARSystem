// src/telas/painel_gerente.rs
use eframe::egui;

pub enum AcaoGerente { Nenhuma }

pub struct TelaGerente {}

impl TelaGerente {
    pub fn new() -> Self { Self {} }

    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> AcaoGerente {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Painel do Gerente");
            ui.label("Conteúdo da tela do gerente aqui.");
        });
        AcaoGerente::Nenhuma
    }
}