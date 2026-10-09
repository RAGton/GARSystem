// src/telas/painel_principal.rs

use crate::aplicacao::AppEvent;
use crate::gui_services::{self, BaseHandle, ErroServico, TokenArc};
use crate::servicos::{OrdemServico, PapelUsuario, StatusOS};
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use egui_plot::{Line, Plot, PlotPoints};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
enum EstadoCarregamento {
    Inicial,
    Carregando,
    Sucesso,
    Erro(String),
}

/// Tela principal (Dashboard) com gráficos e KPIs buscando dados reais do servidor.
pub struct TelaDashboard {
    papel_usuario: PapelUsuario,
    base: BaseHandle,
    token: TokenArc,
    estado_carregamento: Arc<Mutex<EstadoCarregamento>>,
    ordens_servico: Arc<Mutex<Vec<OrdemServico>>>,
    vendas_por_dia: Vec<[f64; 2]>,
    os_por_dia: Vec<[f64; 2]>,
    status_os: Vec<(String, u32)>,
    dados_carregados: bool,
}

impl TelaDashboard {
    pub fn new(papel_usuario: PapelUsuario, base: BaseHandle, token: TokenArc) -> Self {
        Self {
            papel_usuario,
            base,
            token,
            estado_carregamento: Arc::new(Mutex::new(EstadoCarregamento::Inicial)),
            ordens_servico: Arc::new(Mutex::new(Vec::new())),
            vendas_por_dia: Vec::new(),
            os_por_dia: Vec::new(),
            status_os: Vec::new(),
            dados_carregados: false,
        }
    }

    fn carregar_dados_servidor(&mut self, ctx: &egui::Context) {
        if self.dados_carregados {
            return;
        }

        self.dados_carregados = true;
        *self.estado_carregamento.lock().unwrap() = EstadoCarregamento::Carregando;

        let estado_clone = self.estado_carregamento.clone();
        let ordens_clone = self.ordens_servico.clone();
        let ctx_clone = ctx.clone();
        let base = self.base.clone();
        let token = self.token.clone();

        crate::executor::spawn(move || {
            let res: Result<Vec<OrdemServico>, ErroServico> =
                gui_services::listar_ordens(&base, &token);
            match res {
                Ok(ordens) => {
                    *ordens_clone.lock().unwrap() = ordens;
                    *estado_clone.lock().unwrap() = EstadoCarregamento::Sucesso;
                }
                Err(e) => {
                    *estado_clone.lock().unwrap() = EstadoCarregamento::Erro(e.mensagem);
                }
            }
            ctx_clone.request_repaint();
        });
    }

    /// Processa os dados recebidos para exibição nos gráficos
    fn processar_dados(&mut self) {
        let ordens = self.ordens_servico.lock().unwrap();

        if ordens.is_empty() {
            return;
        }

        // Contar status das OS
        let mut abertas = 0u32;
        let mut em_andamento = 0u32;
        let mut aguardando_pecas = 0u32;
        let mut finalizadas = 0u32;

        let mut total_pecas = 0.0f64;
        let mut total_servicos = 0.0f64;

        for os in ordens.iter() {
            match os.status {
                StatusOS::Aberta | StatusOS::Orcamento => abertas += 1,
                StatusOS::EmAndamento | StatusOS::Aprovada => em_andamento += 1,
                StatusOS::AguardandoPeca => aguardando_pecas += 1,
                StatusOS::Finalizada => finalizadas += 1,
                _ => {}
            }

            total_pecas += os.total_pecas;
            total_servicos += os.total_servicos;
        }

        self.status_os = vec![
            ("Abertas".to_string(), abertas),
            ("Em Andamento".to_string(), em_andamento),
            ("Aguardando Peças".to_string(), aguardando_pecas),
            ("Finalizadas".to_string(), finalizadas),
        ];

        // Simular dados de faturamento baseado nas OS (últimos 30 dias)
        // Em produção, isso viria de uma query SQL agregada por data
        let total_os = ordens.len() as f64;
        let faturamento_medio = if total_os > 0.0 {
            (total_pecas + total_servicos) / total_os
        } else {
            0.0
        };

        self.vendas_por_dia = (0..30)
            .map(|i| {
                let x = i as f64;
                // Distribuir as OS ao longo dos dias com alguma variação
                let y = faturamento_medio * (1.0 + 0.3 * ((x * 0.5).sin()));
                [x, y]
            })
            .collect();

        self.os_por_dia = (0..30)
            .map(|i| {
                let x = i as f64;
                // Simular abertura de OS ao longo dos dias
                let y = (total_os / 30.0) * (1.0 + 0.4 * ((x * 0.3).cos()));
                [x, y.max(0.0)]
            })
            .collect();
    }

    /// Calcula KPIs a partir dos dados reais
    fn calcular_kpis(&self) -> (f64, f64, f64) {
        let ordens = self.ordens_servico.lock().unwrap();

        if ordens.is_empty() {
            return (0.0, 0.0, 0.0);
        }

        let mut total_faturamento = 0.0;

        for os in ordens.iter() {
            let valor_os = os.total_pecas + os.total_servicos;
            total_faturamento += valor_os;
        }

        let ticket_medio = if ordens.len() > 0 {
            total_faturamento / ordens.len() as f64
        } else {
            0.0
        };

        // Meta arbitrária (em produção viria de configuração)
        let meta_mes = 150_000.0;

        (total_faturamento, meta_mes, ticket_medio)
    }

    /// update retorna Option<AppEvent> para integrar com o sistema de eventos da aplicação.
    pub fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) -> Option<AppEvent> {
        let evento_emitido: Option<AppEvent> = None;

        // Carregar dados na primeira renderização
        self.carregar_dados_servidor(ctx);

        let estado = self.estado_carregamento.lock().unwrap().clone();

        egui::CentralPanel::default().show(ctx, |ui| {
            // === HEADER COM HIERARQUIA ===
            // Título grande à esquerda, badge do papel + ações agrupadas à direita
            ui.horizontal(|ui| {
                ui.add(
                    egui::Label::new(
                        egui::RichText::new("Dashboard")
                            .size(28.0)
                            .strong()
                            .color(crate::telas::theme::cores::TEXT_PRIMARY),
                    ),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // === Botão Deslogar (outline danger) ===
                    let logout_btn = egui::Button::new(
                        egui::RichText::new("⎋ Deslogar")
                            .color(crate::telas::theme::cores::TEXT_SECONDARY)
                            .size(13.0),
                    )
                    .fill(egui::Color32::TRANSPARENT)
                    .stroke(egui::Stroke::new(1.0, crate::telas::theme::cores::BORDER))
                    .corner_radius(egui::CornerRadius::same(6));
                    if ui.add(logout_btn).clicked() {
                        // ação de deslogar
                    }
                    ui.add_space(8.0);

                    // === Botão Atualizar (outline primary) ===
                    let refresh_btn = egui::Button::new(
                        egui::RichText::new("↻ Atualizar")
                            .color(crate::telas::theme::cores::PRIMARY)
                            .size(13.0)
                            .strong(),
                    )
                    .fill(egui::Color32::TRANSPARENT)
                    .stroke(egui::Stroke::new(1.0, crate::telas::theme::cores::PRIMARY))
                    .corner_radius(egui::CornerRadius::same(6));
                    if ui.add(refresh_btn).clicked() {
                        self.dados_carregados = false;
                    }
                    ui.add_space(12.0);

                    // === Badge do papel (chip cyan) ===
                    let papel_label = format!("{:?}", self.papel_usuario);
                    let badge_frame = egui::Frame::default()
                        .fill(crate::telas::theme::cores::SURFACE_ELEV)
                        .stroke(egui::Stroke::new(1.0, crate::telas::theme::cores::BORDER_FOCUS))
                        .corner_radius(egui::CornerRadius::same(12))
                        .inner_margin(egui::Margin::symmetric(10, 4));
                    badge_frame.show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(papel_label)
                                .color(crate::telas::theme::cores::CYAN_GLOW)
                                .strong()
                                .size(12.0),
                        );
                    });
                });
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(16.0);

            // Mostrar estado de carregamento
            match estado {
                EstadoCarregamento::Inicial | EstadoCarregamento::Carregando => {
                    ui.vertical_centered(|ui| {
                        ui.add_space(100.0);
                        ui.spinner();
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new("Carregando dados do servidor...")
                                .color(crate::telas::theme::cores::TEXT_SECONDARY)
                                .size(14.0),
                        );
                    });
                    return;
                }
                EstadoCarregamento::Erro(ref msg) => {
                    // === CARD DE ERRO ESTILIZADO ===
                    let error_frame = egui::Frame::default()
                        .fill(crate::telas::theme::cores::SURFACE_ELEV)
                        .stroke(egui::Stroke::new(1.0, crate::telas::theme::cores::ERROR))
                        .corner_radius(egui::CornerRadius::same(8))
                        .inner_margin(egui::Margin::same(20));
                    error_frame.show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("⚠")
                                    .color(crate::telas::theme::cores::ERROR)
                                    .size(28.0),
                            );
                            ui.add_space(12.0);
                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new("Não foi possível carregar os dados")
                                        .color(crate::telas::theme::cores::TEXT_PRIMARY)
                                        .strong()
                                        .size(16.0),
                                );
                                ui.add_space(4.0);
                                ui.label(
                                    egui::RichText::new(msg.as_str())
                                        .color(crate::telas::theme::cores::TEXT_SECONDARY)
                                        .size(13.0),
                                );
                            });
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let retry_btn = egui::Button::new(
                                        egui::RichText::new("↻ Tentar novamente")
                                            .color(egui::Color32::WHITE)
                                            .strong()
                                            .size(13.0),
                                    )
                                    .fill(crate::telas::theme::cores::PRIMARY)
                                    .corner_radius(egui::CornerRadius::same(6))
                                    .min_size(egui::vec2(120.0, 36.0));
                                    if ui.add(retry_btn).clicked() {
                                        self.dados_carregados = false;
                                    }
                                },
                            );
                        });
                    });
                    return;
                }
                EstadoCarregamento::Sucesso => {
                    // Processar dados para gráficos
                    self.processar_dados();
                }
            }

            let (faturamento_total, meta_mes, ticket_medio) = self.calcular_kpis();

            // KPIs em uma linha
            ui.horizontal(|ui| {
                Self::kpi(
                    ui,
                    "Faturamento Total",
                    &format!("R$ {:.2}", faturamento_total),
                );
                ui.add_space(8.0);
                Self::kpi(ui, "Meta do Mês", &format!("R$ {:.0}", meta_mes));
                ui.add_space(8.0);
                Self::kpi(ui, "Ticket Médio", &format!("R$ {:.2}", ticket_medio));
                ui.add_space(8.0);
                Self::kpi(
                    ui,
                    "Total de OS",
                    &format!("{}", self.ordens_servico.lock().unwrap().len()),
                );
            });

            ui.add_space(12.0);

            // Duas colunas: gráficos à esquerda, tabela/relatórios à direita
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.group(|ui| {
                        ui.label("Faturamento Estimado (últimos 30 dias)");

                        if !self.vendas_por_dia.is_empty() {
                            let line = Line::new(
                                "Faturamento",
                                PlotPoints::from_iter(self.vendas_por_dia.iter().cloned()),
                            )
                            .color(egui::Color32::from_rgb(100, 200, 100));

                            Plot::new("vendas_plot")
                                .height(220.0)
                                .legend(egui_plot::Legend::default())
                                .show(ui, |plot_ui| {
                                    plot_ui.line(line);
                                });
                        } else {
                            ui.label("Sem dados para exibir");
                        }
                    });

                    ui.add_space(8.0);

                    ui.group(|ui| {
                        ui.label("Ordens de Serviço - Abertas por dia");

                        if !self.os_por_dia.is_empty() {
                            let line = Line::new(
                                "OS Abertas",
                                PlotPoints::from_iter(self.os_por_dia.iter().cloned()),
                            )
                            .color(egui::Color32::from_rgb(200, 150, 100));

                            Plot::new("os_plot").height(160.0).show(ui, |plot_ui| {
                                plot_ui.line(line);
                            });
                        } else {
                            ui.label("Sem dados para exibir");
                        }
                    });
                });

                ui.separator();

                ui.vertical(|ui| {
                    ui.group(|ui| {
                        ui.label("Status das OS (Dados Reais)");

                        if self.status_os.is_empty() {
                            ui.label("Nenhuma OS encontrada");
                        } else {
                            let total: u32 = self.status_os.iter().map(|(_, c)| c).sum();
                            let max_count = total.max(1);

                            for (status, count) in &self.status_os {
                                ui.horizontal(|ui| {
                                    ui.label(status);
                                    let pct = (*count as f32) / (max_count as f32);
                                    ui.add(egui::widgets::ProgressBar::new(pct).show_percentage());
                                    ui.label(format!(" {}", count));
                                });
                            }
                        }
                    });

                    ui.add_space(12.0);

                    ui.group(|ui| {
                        ui.label("Últimas Ordens de Serviço");

                        let ordens = self.ordens_servico.lock().unwrap();
                        let ultimas_5: Vec<_> = ordens.iter().rev().take(5).collect();

                        if ultimas_5.is_empty() {
                            ui.label("Nenhuma ordem de serviço cadastrada");
                        } else {
                            TableBuilder::new(ui)
                                .striped(true)
                                .resizable(true)
                                .column(Column::initial(40.0))
                                .column(Column::initial(150.0))
                                .column(Column::initial(80.0))
                                .column(Column::remainder())
                                .header(20.0, |mut header| {
                                    header.col(|ui| {
                                        ui.strong("ID");
                                    });
                                    header.col(|ui| {
                                        ui.strong("Cliente");
                                    });
                                    header.col(|ui| {
                                        ui.strong("Valor");
                                    });
                                    header.col(|ui| {
                                        ui.strong("Status");
                                    });
                                })
                                .body(|mut body| {
                                    for os in ultimas_5 {
                                        let valor_total = os.total_pecas + os.total_servicos;
                                        body.row(24.0, |mut row| {
                                            row.col(|ui| {
                                                ui.label(os.id.to_string());
                                            });
                                            row.col(|ui| {
                                                ui.label(&os.cliente);
                                            });
                                            row.col(|ui| {
                                                ui.label(format!("R$ {:.2}", valor_total));
                                            });
                                            row.col(|ui| {
                                                ui.label(format!("{:?}", os.status));
                                            });
                                        });
                                    }
                                });
                        }
                    });
                });
            });
        });

        evento_emitido
    }

    fn kpi(ui: &mut egui::Ui, titulo: &str, valor: &str) {
        let frame = egui::Frame::new()
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(40)))
            .inner_margin(8.0);

        frame.show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(
                    egui::RichText::new(titulo)
                        .small()
                        .color(egui::Color32::GRAY),
                );
                ui.heading(egui::RichText::new(valor).strong());
            });
        });
    }
}
