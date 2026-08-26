// src/telas/painel_crm.rs
//
// Dashboard CRM - P2.6.2c
// Tenant-aware: backend filtra clientes e ordens por tenant_id
// RBAC: crm.dashboard.view

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
pub struct DadosCrm {
    pub clientes_total: u32,
    pub clientes_ativos: u32,
    pub novos_30d: u32,
    pub ticket_medio: f64,
}

pub struct TelaDashboardCrm {
    pub tenant_id: i32,
    pub papel: PapelUsuario,
    pub base: BaseHandle,
    pub token: TokenArc,
    pub estado: Arc<Mutex<EstadoCarregamento>>,
    pub dados: Arc<Mutex<DadosCrm>>,
}

impl TelaDashboardCrm {
    pub fn new(tenant_id: i32, papel: PapelUsuario, base: BaseHandle, token: TokenArc) -> Self {
        Self {
            tenant_id,
            papel,
            base,
            token,
            estado: Arc::new(Mutex::new(EstadoCarregamento::Inicial)),
            dados: Arc::new(Mutex::new(DadosCrm::default())),
        }
    }

    pub fn carregar(&self) {
        if let Ok(mut e) = self.estado.lock() {
            *e = EstadoCarregamento::Carregando;
        }
        if let Ok(mut d) = self.dados.lock() {
            *d = DadosCrm::default();
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
            ui.heading("Dashboard CRM");
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
                kpi(ui, "Clientes Total", &format!("{}", dados.clientes_total));
                kpi(ui, "Clientes Ativos", &format!("{}", dados.clientes_ativos));
                kpi(ui, "Novos (30d)", &format!("{}", dados.novos_30d));
                kpi(ui, "Ticket Médio", &format!("R$ {:.2}", dados.ticket_medio));
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
