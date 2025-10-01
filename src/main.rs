// src/main.rs

mod aplicacao;
mod banco_de_dados;
mod servicos;
mod telas;

use aplicacao::AplicativoPrincipal;
use servicos::{ErroAplicacao, PapelUsuario};
use std::sync::mpsc;
use std::thread;

fn main() {
    servicos::inicializar();

    let (envio_ui, recebimento_db) = mpsc::channel::<(String, String)>();
    let (envio_db, recebimento_ui) = mpsc::channel::<Result<PapelUsuario, ErroAplicacao>>();

    thread::spawn(move || {
        for (usuario, senha) in recebimento_db {
            let resultado = servicos::verificar_login(&usuario, &senha);
            let _ = envio_db.send(resultado);
        }
    });

    let opcoes_janela = eframe::NativeOptions {
        initial_window_size: Some(eframe::egui::vec2(1024.0, 768.0)),
        min_window_size: Some(eframe::egui::vec2(800.0, 600.0)),
        ..Default::default()
    };

    println!("Iniciando a interface gráfica...");
    eframe::run_native(
        "Senior System - NasthyCloud",
        opcoes_janela,
        Box::new(|_cc| Box::new(AplicativoPrincipal::new(envio_ui, recebimento_ui))),
    );
}
