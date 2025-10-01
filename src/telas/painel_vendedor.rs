// src/telas/painel_vendedor.rs
use eframe::egui;

pub enum AcaoVendedor { Nenhuma }

pub struct TelaVendedor {}

impl TelaVendedor {
    pub fn new() -> Self { Self {} }

    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> AcaoVendedor {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Painel do Vendedor");
            ui.label("Conteúdo da tela do vendedor aqui.");
        });
        AcaoVendedor::Nenhuma
    }
}