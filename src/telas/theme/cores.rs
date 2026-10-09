// src/telas/theme/cores.rs
// Tokens de cor do GAR System. Apenas dark no MVP.
// Spec grande adiciona paleta Light com token switching.

use eframe::egui::Color32;

// === Superfícies ===
pub const BG: Color32 = Color32::from_rgb(10, 10, 15);          // #0A0A0F ink
pub const SURFACE: Color32 = Color32::from_rgb(21, 22, 29);      // #15161D ink-soft
pub const SURFACE_ELEV: Color32 = Color32::from_rgb(28, 30, 40); // cards, panels elevados

// === Bordas ===
pub const BORDER: Color32 = Color32::from_rgb(40, 42, 55);
pub const BORDER_FOCUS: Color32 = Color32::from_rgb(0, 229, 255); // #00E5FF cyan-glow

// === Texto ===
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(240, 240, 245);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(160, 165, 180);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(110, 115, 130);

// === Ação ===
pub const PRIMARY: Color32 = Color32::from_rgb(0, 150, 255);      // #0096FF blue-500
pub const PRIMARY_HOVER: Color32 = Color32::from_rgb(0, 170, 255);
pub const PRIMARY_PRESSED: Color32 = Color32::from_rgb(0, 71, 171); // #0047AB blue-700

// === Estados ===
pub const ERROR: Color32 = Color32::from_rgb(229, 57, 53);
pub const SUCCESS: Color32 = Color32::from_rgb(34, 197, 94);
pub const WARNING: Color32 = Color32::from_rgb(245, 158, 11);

// === Marca ===
pub const CYAN_GLOW: Color32 = Color32::from_rgb(0, 229, 255);
pub const SILVER: Color32 = Color32::from_rgb(192, 192, 192); // #C0C0C0
