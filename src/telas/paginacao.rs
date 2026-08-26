// src/telas/paginacao.rs
//
// Componente de UI reutilizável para paginação.
//
// Sprint P1.5 (Fase 3 - Paginação GUI).
//
// Renderiza:
//   ← [Anterior] | Página 2 de 5 (123 registros) | [Próxima] → 🔄 ⟳ (erro)
//
// Onde:
//   - `← [Anterior]` fica desabilitado se `page == 1` ou `carregando`
//   - `Página X de Y (Z registros)` é o label central
//   - `[Próxima] →` fica desabilitado se `page == total_paginas` ou `carregando`
//   - `🔄` é um spinner durante carregamento
//   - `⟳` mostra mensagem de erro se houver
//
// ## Como usar
//
// ```ignore
// use crate::telas::paginacao::{EstadoPaginacao, ui_controles_paginacao};
//
// // No estado da tela:
// pub struct TelaClientes {
//     estado: EstadoPaginacao<Cliente>,
//     ...
// }
//
// // No método ui():
// ui_controles_paginacao(ui, &mut self.estado);
// if let Some(p) = &self.estado.pagina {
//     for c in &p.items { /* render */ }
// }
// ```

use eframe::egui;

#[derive(Debug, Clone)]
pub struct EstadoPaginacao<T: Clone> {
    pub pagina: Option<crate::gui_services::Pagina<T>>,
    pub carregando: bool,
    pub erro: Option<String>,
    /// Callback chamado quando o usuário clica em Anterior/Próxima.
    /// Recebe o número da página desejada (1-based).
    /// Como não temos dyn-clone, callbacks são fn pointers.
    pub on_navigate: Option<fn(u32)>,
}

impl<T: Clone> Default for EstadoPaginacao<T> {
    fn default() -> Self {
        Self {
            pagina: None,
            carregando: false,
            erro: None,
            on_navigate: None,
        }
    }
}

impl<T: Clone> EstadoPaginacao<T> {
    pub fn novo() -> Self {
        Self::default()
    }

    pub fn pagina_atual(&self) -> u32 {
        self.pagina.as_ref().map(|p| p.page).unwrap_or(1)
    }

    pub fn total_paginas(&self) -> u32 {
        self.pagina.as_ref().map(|p| p.total_paginas).unwrap_or(0)
    }

    pub fn total_registros(&self) -> u64 {
        self.pagina.as_ref().map(|p| p.total).unwrap_or(0)
    }
}

/// Renderiza os controles de paginação. Retorna `Some(nova_pagina)` se
/// o usuário clicou em Anterior/Próxima, senão `None`.
pub fn ui_controles_paginacao<T: Clone>(
    ui: &mut egui::Ui,
    estado: &mut EstadoPaginacao<T>,
) -> Option<u32> {
    let page = estado.pagina_atual();
    let total_paginas = estado.total_paginas();
    let total_registros = estado.total_registros();
    let mut clicked: Option<u32> = None;

    ui.horizontal(|ui| {
        ui.add_enabled_ui(!estado.carregando && page > 1, |ui| {
            if ui.button("← Anterior").clicked() {
                clicked = Some(page.saturating_sub(1));
            }
        });

        ui.label(format!(
            "Página {} de {} ({} registros)",
            page, total_paginas, total_registros
        ));

        ui.add_enabled_ui(!estado.carregando && page < total_paginas, |ui| {
            if ui.button("Próxima →").clicked() {
                clicked = Some(page + 1);
            }
        });

        if estado.carregando {
            ui.spinner();
        }
    });

    if let Some(err) = &estado.erro {
        ui.colored_label(egui::Color32::from_rgb(200, 50, 50), format!("⚠ {}", err));
    }

    clicked
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui_services::Pagina;

    #[test]
    fn estado_paginacao_default_vazio() {
        let e: EstadoPaginacao<String> = EstadoPaginacao::default();
        assert_eq!(e.pagina_atual(), 1);
        assert_eq!(e.total_paginas(), 0);
        assert_eq!(e.total_registros(), 0);
        assert!(!e.carregando);
        assert!(e.erro.is_none());
    }

    #[test]
    fn estado_paginacao_com_pagina() {
        let e: EstadoPaginacao<String> = EstadoPaginacao {
            pagina: Some(Pagina {
                items: vec!["a".to_string(), "b".to_string()],
                page: 3,
                limit: 50,
                total: 150,
                total_paginas: 3,
            }),
            carregando: false,
            erro: None,
            on_navigate: None,
        };
        assert_eq!(e.pagina_atual(), 3);
        assert_eq!(e.total_paginas(), 3);
        assert_eq!(e.total_registros(), 150);
    }
}
