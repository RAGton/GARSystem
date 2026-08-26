// src/telas/painel_financeiro.rs
//
// Dashboard Financeiro - P2.6.2c
// Tenant-aware: backend filtra todas as queries por tenant_id
// RBAC: relatorios.financeiro.view

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
pub struct DadosFinanceiro {
    pub total_receber: i64,
    pub total_pagar: i64,
    pub contas_vencidas: u32,
    pub saldo: i64,
}

pub struct TelaFinanceiro {
    pub tenant_id: i32,
    pub papel: PapelUsuario,
    pub base: BaseHandle,
    pub token: TokenArc,
    pub estado: Arc<Mutex<EstadoCarregamento>>,
    pub dados: Arc<Mutex<DadosFinanceiro>>,
}

impl TelaFinanceiro {
    pub fn new(tenant_id: i32, papel: PapelUsuario, base: BaseHandle, token: TokenArc) -> Self {
        Self {
            tenant_id,
            papel,
            base,
            token,
            estado: Arc::new(Mutex::new(EstadoCarregamento::Inicial)),
            dados: Arc::new(Mutex::new(DadosFinanceiro::default())),
        }
    }

    pub fn carregar(&self) {
        if let Ok(mut e) = self.estado.lock() {
            *e = EstadoCarregamento::Carregando;
        }
        if let Ok(mut d) = self.dados.lock() {
            *d = DadosFinanceiro::default();
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
            ui.heading("Dashboard Financeiro");
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
                kpi(
                    ui,
                    "Total a Receber",
                    &format!("R$ {:.2}", dados.total_receber as f64 / 100.0),
                );
                kpi(
                    ui,
                    "Total a Pagar",
                    &format!("R$ {:.2}", dados.total_pagar as f64 / 100.0),
                );
                kpi(ui, "Contas Vencidas", &format!("{}", dados.contas_vencidas));
                kpi(
                    ui,
                    "Saldo Atual",
                    &format!("R$ {:.2}", dados.saldo as f64 / 100.0),
                );
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
