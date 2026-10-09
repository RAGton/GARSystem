// src/telas/theme/tipografia.rs
// Tamanhos e pesos tipográficos. Spec grande embedda Inter como proporcional.
//
// MVP: usa as famílias que o main.rs já registra (JetBrains Mono).
// Pra textos longos/cabeçalhos: depende da família "Proportional" do egui,
// que no Linux geralmente resolve pra uma sans do sistema (DejaVu Sans,
// Cantarell, ou similar). Aceitável pro MVP.

pub const FONT_BODY: f32 = 14.0;
pub const FONT_LABEL: f32 = 12.0;
pub const FONT_SUBHEAD: f32 = 15.0;
pub const FONT_HEAD: f32 = 20.0;
pub const FONT_DISPLAY: f32 = 28.0;
