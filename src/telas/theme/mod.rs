// src/telas/theme/mod.rs
// MVP — apenas dark, sem persistência. Spec grande expande pra dark/light + persistência.
//
// Uso:
//   use crate::telas::theme;
//   theme::aplicar(&ctx, theme::Tema::Dark);
//
// Práticas egui (Emil Ernerfeldt): module documentado, tokens semânticos
// centralizados, fn `aplicar` idempotente.

pub mod cores;
pub mod espacamento;
pub mod tipografia;

pub use cores::*;
pub use espacamento::*;
pub use tipografia::*;

use eframe::egui;

/// Tema visual ativo.
///
/// MVP: apenas `Dark`. Spec grande
/// (`docs/superpowers/specs/2026-10-09-elevacao-ui-gar-system.md`)
/// adiciona `Light` + persistência via `~/.config/gar-system/theme.toml`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tema {
    /// Tema escuro — único suportado no MVP. Background `#0A0A0F` (ink),
    /// surface `#15161D` (ink-soft), primary `#0096FF` (blue-500).
    Dark,
}

/// Aplica o tema ao contexto do egui.
///
/// Chame uma vez no startup (em `main.rs`, antes do primeiro frame).
/// Idempotente — chamar de novo só reaplica os mesmos tokens.
///
/// # Panics
/// Não panica. Se `ctx.style()` falhar (improvável), o `clone()` é infallible.
///
/// # Example
/// ```no_run
/// use gar_system::telas::theme::{aplicar, Tema};
/// let ctx: &egui::Context = unimplemented!();
/// aplicar(ctx, Tema::Dark);
/// ```
pub fn aplicar(ctx: &egui::Context, tema: Tema) {
    let mut style = (*ctx.style()).clone();

    // === Cores principais ===
    style.visuals.override_text_color = Some(TEXT_PRIMARY);
    style.visuals.window_fill = SURFACE;
    style.visuals.panel_fill = SURFACE;
    style.visuals.faint_bg_color = SURFACE_ELEV;
    style.visuals.extreme_bg_color = BG;

    // Borders
    style.visuals.widgets.noninteractive.bg_stroke.color = BORDER;
    style.visuals.widgets.inactive.bg_stroke.color = BORDER;
    style.visuals.widgets.hovered.bg_stroke.color = BORDER_FOCUS;
    style.visuals.widgets.active.bg_stroke.color = BORDER_FOCUS;

    // Backgrounds de widgets
    style.visuals.widgets.noninteractive.bg_fill = BG;
    style.visuals.widgets.inactive.bg_fill = SURFACE_ELEV;
    style.visuals.widgets.hovered.bg_fill = SURFACE_ELEV;
    style.visuals.widgets.active.bg_fill = PRIMARY;

    // Selection
    style.visuals.selection.bg_fill = PRIMARY.gamma_multiply(0.4);
    style.visuals.selection.stroke.color = BORDER_FOCUS;

    // Hyperlinks
    style.visuals.hyperlink_color = PRIMARY;

    // === Espaçamento e raio ===
    style.spacing.item_spacing = egui::vec2(SP_SM, SP_SM);
    style.spacing.button_padding = egui::vec2(SP_LG, SP_MD);
    style.spacing.window_margin = egui::Margin::same(SP_XL as i8);
    // egui 0.33: window_rounding foi removido; usa corner_radius dos widgets.
    let radius_8 = egui::CornerRadius::same(8);
    let radius_12 = egui::CornerRadius::same(12);
    style.visuals.widgets.noninteractive.corner_radius = radius_8;
    style.visuals.widgets.inactive.corner_radius = radius_8;
    style.visuals.widgets.hovered.corner_radius = radius_8;
    style.visuals.widgets.active.corner_radius = radius_8;
    // Window rounding via Window helper
    style.visuals.window_corner_radius = radius_12;

    // === Tipografia ===
    // Mantém as famílias padrão (Proportional/Monospace) já configuradas pelo main.rs
    // com JetBrains Mono. Spec grande embedda Inter.
    let _ = tema; // suprimir warning unused

    ctx.set_style(style);
}
