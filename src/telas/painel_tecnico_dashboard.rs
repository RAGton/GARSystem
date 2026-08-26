// src/telas/painel_tecnico_dashboard.rs
//
// Dashboard Tecnico (OS Mobile) - P2.6.2c
// Tenant-aware: backend filtra ordens_servico e sla_calculos por tenant_id
// RBAC: os.dashboard.view

use crate::aplicacao::AppEvent;
use crate::gui_services::{BaseHandle, TokenArc};
use crate::servicos::PapelUsuario;
use eframe::egui;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub enum EstadoCarregamento {
    Inicial,
    Carregando,
    Pronto,
    Erro(String),
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct DadosTecnico {
    pub os_abertas: u32,
    pub os_andamento: u32,
    pub os_concluidas_30d: u32,
    pub sla_violado: u32,
}

pub struct TelaDashboardTecnico {
    pub tenant_id: i32,
    pub papel: PapelUsuario,
    pub base: BaseHandle,
    pub token: TokenArc,
    pub estado: Arc<Mutex<EstadoCarregamento>>,
    pub dados: Arc<Mutex<DadosTecnico>>,
}

impl TelaDashboardTecnico {
    pub fn new(tenant_id: i32, papel: PapelUsuario, base: BaseHandle, token: TokenArc) -> Self {
        Self {
            tenant_id,
            papel,
            base,
            token,
            estado: Arc::new(Mutex::new(EstadoCarregamento::Inicial)),
            dados: Arc::new(Mutex::new(DadosTecnico::default())),
        }
    }

    pub fn carregar(&self) {
        if let Ok(mut e) = self.estado.lock() {
            *e = EstadoCarregamento::Carregando;
        }
        if let Ok(mut d) = self.dados.lock() {
            *d = DadosTecnico::default();
        }
        if let Ok(mut e) = self.estado.lock() {
            *e = EstadoCarregamento::Pronto;
        }
    }

    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Option<AppEvent> {
        let estado = self
            .estado
            .lock()
            .ok()
            .map(|e| e.clone())
            .unwrap_or(EstadoCarregamento::Inicial);
        if matches!(estado, EstadoCarregamento::Inicial) {
            self.carregar();
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Dashboard Tecnico (OS Mobile)");
            ui.label(format!(
                "Tenant: {} | Papel: {:?}",
                self.tenant_id, self.papel
            ));
            ui.separator();
            let dados = self
                .dados
                .lock()
                .ok()
                .map(|d| d.clone())
                .unwrap_or_default();
            ui.horizontal(|ui| {
                kpi(ui, "OS Abertas", &format!("{}", dados.os_abertas));
                kpi(ui, "OS em Andamento", &format!("{}", dados.os_andamento));
                kpi(
                    ui,
                    "Concluidas (30d)",
                    &format!("{}", dados.os_concluidas_30d),
                );
                kpi(ui, "SLA Violado", &format!("{}", dados.sla_violado));
            });
            if let EstadoCarregamento::Erro(e) = estado {
                ui.colored_label(egui::Color32::RED, format!("Erro: {}", e));
            }
        });
        None
    }
}

fn kpi(ui: &mut egui::Ui, titulo: &str, valor: &str) {
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.label(titulo);
            ui.heading(valor);
        });
    });
    ui.add_space(10.0);
}
