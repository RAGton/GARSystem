# MVP — Tela de Login + Tema GAR System

**Data:** 2026-10-09
**Status:** APROVADO (decisão humana)
**Escopo:** Tema base + 1 tela (login) + logo sidebar + assets
**Não-escopo:** outras 18 telas, modais completos, componentes avançados, dark/light toggle (deixa pra spec grande)

---

## 1. Objetivo

Resolver "tá horrível" **HOJE** com:
- Fundo de login limpo (sem logo borrada)
- Inputs visíveis (contraste alto)
- Logo GAR no header da sidebar (cyan glow, sem fundo branco)
- Tipografia consistente (Inter)
- Paleta GAR aplicada de verdade
- Footer com ano correto (2026)

**Resultado esperado:** 1 tela de login que pareça produto, não protótipo.

## 2. Entregas

| Item | Arquivo | LOC estimadas |
|---|---|---|
| Módulo de tema | `src/telas/theme/{mod,cores,tipografia,espacamento}.rs` | ~150 |
| Componente logo | `src/telas/componentes/logo.rs` | ~50 |
| Refactor login | `src/telas/login.rs` | ~200 (de 392) |
| Logo sidebar | `assets/logo-sidebar.png` | gerado |
| Background login | `assets/backgrounds/login-bg.png` | gerado |

## 3. Tema (locking)

```rust
// Cores (dark only por agora)
pub const BG: Color32 = Color32::from_rgb(10, 10, 15);          // #0A0A0F ink
pub const SURFACE: Color32 = Color32::from_rgb(21, 22, 29);      // #15161D ink-soft
pub const SURFACE_ELEV: Color32 = Color32::from_rgb(28, 30, 40);
pub const BORDER: Color32 = Color32::from_rgb(40, 42, 55);
pub const BORDER_FOCUS: Color32 = Color32::from_rgb(0, 229, 255); // #00E5FF cyan-glow
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(240, 240, 245);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(160, 165, 180);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(110, 115, 130);
pub const PRIMARY: Color32 = Color32::from_rgb(0, 150, 255);     // #0096FF blue-500
pub const PRIMARY_HOVER: Color32 = Color32::from_rgb(0, 170, 255);
pub const ERROR: Color32 = Color32::from_rgb(229, 57, 53);
pub const SUCCESS: Color32 = Color32::from_rgb(34, 197, 94);

// Espaçamento
pub const SP_XS: f32 = 4.0;
pub const SP_SM: f32 = 8.0;
pub const SP_MD: f32 = 12.0;
pub const SP_LG: f32 = 16.0;
pub const SP_XL: f32 = 24.0;
pub const SP_XXL: f32 = 32.0;

// Tipografia (Inter via system font até embedar)
pub const FONT_BODY: f32 = 14.0;
pub const FONT_LABEL: f32 = 12.0;
pub const FONT_HEAD: f32 = 20.0;
pub const FONT_DISPLAY: f32 = 28.0;
```

## 4. Login refactor (especificação visual)

**Layout split 50/50** (mantém o dual-pane, mas resolve os problemas):

```
┌─────────────────────────┬──────────────────────────┐
│                         │                          │
│   [LOGO GAR grande]     │   Bem-vindo de volta     │
│                         │   ──────────────         │
│   Sistema de gestão     │                          │
│   integrado             │   Usuário                │
│   para assistência      │   ┌──────────────────┐  │
│   técnica               │   │ admin            │  │
│                         │   └──────────────────┘  │
│                         │                          │
│   v 1.10.0              │   Senha                  │
│   © 2026 RAGton         │   ┌──────────────────┐  │
│                         │   │ ••••••••         │  │
│                         │   └──────────────────┘  │
│                         │                          │
│                         │   ☐ Lembrar de mim       │
│                         │                          │
│                         │   ┌──────────────────┐  │
│                         │   │    ENTRAR        │  │
│                         │   └──────────────────┘  │
│                         │                          │
│                         │   v 1.10.0 · © 2026      │
└─────────────────────────┴──────────────────────────┘
```

**Mudanças vs atual:**
- ❌ Remove fundo de logo borrada
- ❌ Remove itálico cursivo "Bem-vindo! Faça o login para continuar"
- ❌ Remove logo GAR com fundo branco
- ✅ Fundo `ink` (#0A0A0F) em ambos os lados
- ✅ Inputs com `bg = surface-elev`, `border = BORDER`, foco `BORDER_FOCUS` (cyan glow)
- ✅ Botão "Entrar" com `bg = PRIMARY`, texto branco, hover `PRIMARY_HOVER`, full-width
- ✅ Label "Usuário" / "Senha" com `TEXT_SECONDARY` (não preto)
- ✅ Esquerda: logo GAR grande (200x200), tipografia prata, tagline
- ✅ Direita: form centrado vertical
- ✅ Footer: ano 2026 (não 2025), versão pequena, `TEXT_MUTED`

## 5. Logo sidebar (asset novo)

- `assets/logo-sidebar.png`
- 256×256 px, RGBA, fundo **transparente**
- Estilo: a **garra isolada** do logo principal, com leve glow cyan interno
- A mesma que existe hoje em `assets/logo-mark.png` (1290×1219) mas:
  - Versão **otimizada pra header** (padding interno generoso, glow sutil, sem texto)
  - **Reduzida e limpa** pra caber em 64×64 no sidebar sem distorcer
- Procedimento: pega `assets/logo-mark.png` (que já existe do rebranding), aplica padding, gera nova versão

## 6. Background login (asset novo)

- `assets/backgrounds/login-bg.png`
- 1920×1080 px
- Gradiente diagonal cyan→ink, com silhueta da garra em alpha 5% no canto inferior direito
- Aplica **só no lado esquerdo** do split (50% da tela)
- Lado direito fica com `ink` sólido + form centrado

## 7. Comportamento de interação

- Foco em input: borda muda pra `BORDER_FOCUS` (cyan glow) com transição de 150ms
- Hover no botão: muda pra `PRIMARY_HOVER`
- Botão durante submit: mostra "Entrando..." com spinner, fica desabilitado
- Erro de credencial: toast vermelho acima do form + shake animation sutil
- Tab navega entre os campos na ordem certa

## 8. Critérios de aceitação

- [ ] `cargo build --bin gar-system-gui` exit 0
- [ ] `cargo test --lib` ≥ 92 passed
- [ ] Login abre, fundo é sólido `#0A0A0F` (não mais a logo borrada)
- [ ] Inputs de usuário/senha são visíveis (não mais pretos sobre preto)
- [ ] Logo GAR no header da sidebar aparece SEM fundo branco
- [ ] Botão "Entrar" tem cor primária GAR (azul), não cinza genérico
- [ ] Footer mostra "© 2026", não "© 2025"
- [ ] "Bem-vindo!" não está mais em itálico cursivo
- [ ] Tab entre campos funciona, foco visualmente claro
- [ ] Login com `admin` / `gar_dev_2026` funciona

## 9. Não-objetivos (explícito)

- ❌ Dark/light toggle (deixa pra spec grande)
- ❌ Modais (deixa pra spec grande)
- ❌ Migrar outras 18 telas (deixa pra fase 2)
- ❌ Inter embedded (usa system font por agora; embed vem na fase 2)
- ❌ Phosphor/Lucide icons (deixa pra fase 2; sidebar usa a logo-mark só)
- ❌ Animações complexas (só transições simples de 150ms)

## 10. Riscos

| Risco | Mitigação |
|---|---|
| `egui::Context::set_style()` não aplica tudo que preciso | Setar via `Style` clone + override field-by-field |
| Logo com fundo branco persiste em outros lugares | Troca única em `sidebar.rs` (header) e `login.rs` (esquerda) |
| Inter não instalado no sistema | Fallback: usar `egui::FontFamily::Proportional` default (sans do sistema) |
| Usar system font causa look "genérico" | Aceitável pra MVP, embed real vem na fase 2 |
| Regressão visual em outras telas | Não toco nas outras telas; login é isolado |

## 11. Sequência de execução

1. **Spec aprovado** ← estamos aqui
2. Gerar 2 assets (logo-sidebar.png, login-bg.png) via `image_generate` skill
3. Criar `src/telas/theme/` (4 arquivos)
4. Criar `src/telas/componentes/logo.rs` (1 arquivo, simples)
5. Refactor `src/telas/login.rs` (1 arquivo, mantém API pública)
6. Atualizar `src/telas/mod.rs` pra incluir `theme` e `componentes::logo`
7. `cargo build --bin gar-system-gui` + `cargo test --lib`
8. Screenshot pra tu ver
9. Commit único: `feat(ui): MVP tema GAR + login redesign`

**FIM DO SPEC MVP**
