// src/main.rs

mod aplicacao;
mod banco_de_dados;
mod servicos;
mod telas;

use aplicacao::AplicativoPrincipal;
use eframe::egui;
use servicos::{ErroAplicacao, PapelUsuario};
use std::sync::mpsc;
use std::thread;

fn main() -> Result<(), eframe::Error> {
    servicos::inicializar();

    let (envio_ui, recebimento_db) = mpsc::channel::<(String, String)>();
    let (envio_db, recebimento_ui) = mpsc::channel::<Result<PapelUsuario, ErroAplicacao>>();

    thread::spawn(move || {
        for (usuario, senha) in recebimento_db {
            let resultado = servicos::verificar_login(&usuario, &senha);
            if envio_db.send(resultado).is_err() {
                break;
            }
        }
    });

    let opcoes_janela = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([624.0, 468.0])
            .with_resizable(false), // A janela já começa não redimensionável
        centered: true, // A centralização agora é um campo separado
        ..Default::default()
    };

    println!("Iniciando a interface gráfica...");
    eframe::run_native(
        "Senior System - NasthyCloud",
        opcoes_janela,
        // A closure de criação agora precisa retornar um `Ok`
        Box::new(|_cc| Ok(Box::new(AplicativoPrincipal::new(envio_ui, recebimento_ui)))),
    )
}
