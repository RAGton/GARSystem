# Especificação — Elevação de UI/UX do GAR System

**Data:** 2026-10-09
**Autor:** AURA-PLANNER (esboço) → revisão do humano → KORA-ARCHITECT (execução)
**Status:** RASCUNHO para revisão
**Escopo:** redesign visual e arquitetural da camada de apresentação GUI (egui) do GAR System
**Não-escopo:** backend (Axum), banco (MySQL), regras de negócio, RBAC, OaaS

---

## 1. Contexto e motivação

O GAR System herdou do `senior_system` um frontend eframe/egui com:
- Tema nunca aplicado (`egui::Context::set_style()` não é chamado em lugar nenhum)
- Cores `Color32::from_rgb(...)` hardcoded em 6+ arquivos de tela
- 1 componente reaproveitável (`componentes/sidebar.rs`); 18 telas escrevem seus próprios botões/modais
- Login poluído visualmente (`login.rs`, 392 linhas, todo desenhado à mão)
- Densidade inconsistente — algumas telas "espremidas", outras "respiro demais"

Resultado: a GUI parece "engessada" e a paleta da marca (cyan/blue/silver/ink) **não aparece** em quase nada.

**Decisão do humano (2026-10-09):**
- "Quero algo único, vendável, com UI mais moderna, telas mais desenhadas, inspirado nas melhores do mercado mas melhor"
- "Criar algo único" — usaremos a paleta GAR (que já é única no mercado ERP brasileiro) como assinatura visual
- Referência: playlist do **Data 7** no YouTube (sistema "vendável e funcional") + estado da arte (Linear, Stripe, Vercel)

---

## 2. Objetivos mensuráveis

| Objetivo | Métrica | Meta |
|---|---|---|
| Consistência visual | Telas com cor hardcoded fora do tema | 0 |
| Reaproveitamento | Telas com botões/modais próprios reinventados | 0 (todos usam `componentes::*`) |
| Personalização | Tema trocado em runtime (dark/light) sem rebuild | Funcional + persiste em `~/.config/gar-system/theme.toml` |
| Densidade | Listas/tabelas: 8px entre linhas; formulários: 16-24px | Aplicado em todas as 19 telas |
| Modais (Q4 prioritário) | Modais próprios (não `egui::Window` cru) | 100% das telas migradas |
| Feedback de erro (Q5) | Toast + inline juntos em falhas | Padrão aplicado em todas as ações mutantes |
| Build | `cargo build` | 0 erros, 0 novos warnings |
| Testes | `cargo test --lib` | mantém ≥ 92 passed |

**Não-objetivos (explícitos):**
- Não migrar pra Tauri/Next.js (questão considerada e descartada — o framework eframe é adequado)
- Não trocar a paleta de cores da marca
- Não mexer em regras de negócio, fluxos ou modelo de dados
- Não refatorar lógica de `aplicacao.rs` (apenas call sites visuais)
- Não criar animações pesadas (egui é immediate-mode — animar é caro, manter sutileza)

---

## 3. Decisões de design (locking decisions)

### 3.1 Paleta de cores (já vigente via skill `gar-system-brand`)

```text
--gar-cyan-glow:  #00E5FF   /* destaque, glow em foco */
--gar-blue-500:   #0096FF   /* primária */
--gar-blue-700:   #0047AB   /* secundária */
--gar-blue-900:   #001A4D   /* profundidade, headers */
--gar-silver:     #C0C0C0   /* metálico, "System" */
--gar-silver-hi:  #E8E8E8   /* highlight prata */
--gar-ink:        #0A0A0F   /* background dark */
--gar-ink-soft:   #15161D   /* surface dark (cards, panels) */
--gar-paper:      #FFFFFF   /* background light */
--gar-paper-soft: #F4F5F8   /* surface light */
```

**Regras de uso:**
- Primária (botões, links, foco) → `--gar-blue-500`
- Acento (glow, badges de sucesso) → `--gar-cyan-glow`
- Header / brand mark → `--gar-blue-900` com tipografia prata
- Erro destrutivo → vermelho `#E53935` (não está na paleta, é a única exceção — convenção universal de erro)
- Sucesso → `#22C55E` (verde, fora da paleta, convenção)
- Aviso → `#F59E0B` (âmbar, fora da paleta, convenção)

### 3.2 Tipografia

- **Sans:** **Inter** (Open Font License, weights 400/500/600/700)
- **Mono:** **JetBrains Mono** (campos técnicos, IDs, valores)
- Tamanhos: `12px` (caption), `13px` (body), `15px` (subhead), `18px` (head), `24px` (display)
- Line-height: 1.45 (body), 1.2 (head/display)
- Tracking: `-0.01em` em heads (mais apertado), `0` em body, `+0.02em` em captions

**Empacotamento:** Inter e JetBrains Mono já disponíveis em `assets/fonts/` (verificar). Se não, baixar uma vez e commitar. TTF/OTF embedded no binário via `include_bytes!`.

### 3.3 Tema dark/light

- **Default:** dark
- **Switch:** toggle no header da sidebar (`componentes/sidebar.rs`) — ícone lua/sol
- **Persistência:** `~/.config/gar-system/theme.toml` (escrito quando muda, lido no startup)
- **Escopo:** tokens visuais (cores, sombras), não conteúdo. Tabelas/formulários se adaptam.
- **Quem decide se dark/light no primeiro start:** dark (decisão consciente — combina com a paleta)

### 3.4 Densidade (Q2c: híbrido)

Tokens de espaçamento (mapeados para `TokenEspacamento` da §4.2):

| Token | Valor (px) | Uso |
|---|---|---|
| `Xs`  | 4  | gap entre ícone e texto inline |
| `Sm`  | 8  | entre linhas de tabela, padding de botão pequeno |
| `Md`  | 12 | padding de campo de formulário, item de lista |
| `Lg`  | 16 | gap entre seções, padding de cartão |
| `Xl`  | 24 | padding de modal, separação de blocos |
| `Xxl` | 32 | entre páginas/seções principais |

Densidade aplicada por contexto:

| Contexto | Padding interno | Altura de linha | Tamanho de fonte |
|---|---|---|---|
| Tabela/lista | `Sm` | 28px | 13px |
| Formulário (campo) | `Md` vertical | 36px altura | 14px |
| Formulário (label) | — | — | 13px muted |
| Cartão/KPI | `Lg` | — | 18px valor / 13px label |
| Modal | `Xl` | 40px área | 15px |
| Header de página | `Xl` | — | 24px |
| Sidebar item | `Md` vertical | 40px altura | 14px |

### 3.5 Componentes prioritários (Q4 = modais)

**Esta spec trata APENAS do sistema de modais.** Componentes adicionais (botões, tabelas, formulários, toasts) serão especificados em specs separadas após este redesign validar.

Modais que devem existir:
- `ModalConfirmacao` — "Tem certeza?" com botão primário destrutivo ou neutro
- `ModalFormulario` — form completo dentro de overlay
- `ModalInfo` — texto + botão OK
- `ModalAlerta` — ícone + texto + ação
- `ModalCarregando` — spinner + texto (não-dismissable)

**Visual:**
- Overlay: `ink` com 60% opacity
- Container: cantos 12px, sombra `0 8px 24px rgba(0,0,0,0.4)` (dark) / `0 8px 24px rgba(0,0,0,0.12)` (light)
- Borda interna: `cyan-glow` 1px com `alpha = 0.3` (assinatura GAR)
- Header: tipografia prata, fundo `ink-soft` / `paper-soft`
- Botão fechar (X) top-right
- Animação de entrada: 120ms fade + scale de 0.96 → 1.0

### 3.6 Feedback de erro (Q5d: mix inteligente)

Regra aplicada em **toda ação mutante** (POST, PUT, DELETE):

1. **Validação client-side (egui):** marca campo com borda vermelha + helper text inline
2. **Submissão:** botão vira spinner, fica desabilitado, label muda pra "Salvando..."
3. **Sucesso:** toast verde no canto inferior direito (3s) + fecha modal OU atualiza lista
4. **Erro HTTP:**
   - **4xx (400/401/403/404/409/422):** toast âmbar + inline no campo se for erro de campo
   - **5xx:** toast vermelho + inline genérico "Tente novamente em alguns instantes"
5. **Network error (sem resposta):** toast vermelho + botão "Tentar novamente"
6. Toast tem: ícone, mensagem curta, botão "×" pra fechar

Componentes necessários (futuras specs): `Toast`, `NotificacaoCentral`.

---

## 4. Arquitetura proposta

### 4.1 Estrutura de arquivos

```
src/
├── telas/
│   ├── theme/
│   │   ├── mod.rs
│   │   ├── cores.rs            // constantes Color32, paletas dark/light
│   │   ├── tipografia.rs       // FontFamily, FontSize, inter/jetbrains
│   │   ├── espacamento.rs      // constantes de padding/spacing
│   │   └── estilo.rs           // fn aplicar_tema(ctx, modo: Tema)
│   ├── componentes/
│   │   ├── mod.rs
│   │   ├── sidebar.rs          // (existente, refatorar p/ usar tema)
│   │   ├── modal.rs            // NOVO — Q4 prioritário
│   │   ├── botao.rs            // NOVO (spec futura)
│   │   ├── cartao.rs           // NOVO (spec futura)
│   │   └── toast.rs            // NOVO (spec futura)
│   ├── login.rs                // refatorado p/ usar tema
│   ├── painel_os_edicao.rs     // refatorado (maior tela)
│   └── ... (outras 17 telas)   // refatoração opportunística
```

### 4.2 API do módulo `theme`

```rust
// src/telas/theme/mod.rs
pub enum Tema { Dark, Light }

/// Tokens semânticos de cor — toda tela consome isso, nunca literal.
pub enum TokenCor {
    Fundo,            // fundo da janela
    Superficie,       // cards, panels
    SuperficieElevada,// modais, popovers
    Borda,
    BordaFoco,        // glow cyan
    TextoPrimario,
    TextoSecundario,
    TextoMudo,
    Primario,         // ação principal
    PrimarioHover,
    Sucesso,
    Aviso,
    Erro,
    CyanGlow,         // destaque heróico (logo, foco raro)
}

pub enum TokenEspacamento {
    Xs,  // 4
    Sm,  // 8
    Md,  // 12
    Lg,  // 16
    Xl,  // 24
    Xxl, // 32
}

pub enum PesoFonte { Regular, Medium, SemiBold, Bold }

pub fn carregar() -> Tema { /* lê ~/.config/gar-system/theme.toml */ }

pub fn aplicar(ctx: &egui::Context, tema: Tema) {
    // chama ctx.set_style() com a paleta escolhida
    // registra fonts customizadas
    // configura shadows, corner radius, espaçamentos
}

pub fn cor(ctx: &egui::Context, token: TokenCor) -> Color32 { /* lookup */ }
pub fn fonte(ctx: &egui::Context, peso: PesoFonte) -> FontId { /* lookup */ }
pub fn espacamento(token: TokenEspacamento) -> f32 { /* lookup */ }
```

### 4.3 API do módulo `componentes/modal`

```rust
// src/telas/componentes/modal.rs

/// Modal de confirmação. Retorna Some(true) se confirmou, Some(false) se cancelou, None se ainda aberto.
pub fn confirmacao(
    ctx: &egui::Context,
    aberto: &mut bool,
    titulo: &str,
    mensagem: &str,
    texto_confirmar: &str,
    destrutivo: bool,
) -> Option<bool>;

/// Modal de info simples (botão OK).
pub fn info(ctx: &egui::Context, aberto: &mut bool, titulo: &str, mensagem: &str);

/// Modal de alerta com ação. Retorna true se a ação foi clicada.
pub fn alerta(
    ctx: &egui::Context,
    aberto: &mut bool,
    titulo: &str,
    mensagem: &str,
    texto_acao: &str,
) -> bool;

/// Modal genérico com conteúdo customizado (uso para formulários inteiros).
pub fn custom(
    ctx: &egui::Context,
    aberto: &mut bool,
    titulo: &str,
    conteudo: impl FnOnce(&mut Ui),
);
```

### 4.4 Migração das telas (ordem)

| Fase | Telas | Justificativa |
|---|---|---|
| **0 — Tema** | nenhuma ainda | `theme/` precisa existir antes |
| **1 — Login** | `login.rs` | Porta de entrada — primeiro impacto visual |
| **2 — Modal POC** | `painel_principal.rs` | Tela simples pra validar `componentes/modal.rs` |
| **3 — OS** | `painel_os_edicao.rs`, `painel_os_criar.rs` | Maior tela (812 linhas) — vitrine |
| **4 — Cotação** | `painel_orcamentos.rs` | Fluxo de produto |
| **5 — Demais** | 14 telas restantes | Refatoração opportunística, em ondas de 3-4 telas/semana |

### 4.5 Persistência do tema

Arquivo `~/.config/gar-system/theme.toml`:

```toml
# Tema: "dark" | "light"
tema = "dark"
```

Schema completo (parse com `toml` crate):

| Campo | Tipo | Default | Valores aceitos |
|---|---|---|---|
| `tema` | string | `"dark"` | `"dark"` \| `"light"` |

Comportamento:
- **Startup:** `theme::carregar()` lê o arquivo. Se não existir, cria com `tema = "dark"`.
- **Runtime:** quando o usuário troca (sidebar), escreve no arquivo e aplica imediatamente.
- **Erro de parse:** log warning, cai no default (`dark`). Nunca crasha o app.

**Diretório de config:** `~/.config/gar-system/` — criado se não existir (via `std::fs::create_dir_all`).

---

## 5. Riscos e mitigações

| Risco | Probabilidade | Mitigação |
|---|---|---|
| Inter/JetBrains Mono não disponíveis em `assets/fonts/` | Média | Verificar antes; se faltar, baixar de fonts.google.com (licença OFL) e commitar |
| egui não suporta bem múltiplas famílias de fonte | Baixa | egui 0.33 suporta; testar com POC antes |
| Tema dark deixa alguma tela ilegível | Média | Migrar todas as telas com tokens, não com cor literal |
| Migração de 19 telas causa regressão visual | Média | Migrar uma tela por vez, comparar antes/depois com screenshot |
| Tempo estimado (1-2 meses) estoura | Alta | Spec 1 cobre só tema + modais. Demais componentes (botões, tabelas, toasts) ganham specs próprias depois |
| Tema switch não persiste entre sessões | Baixa | Arquivo TOML é texto simples, baixa chance de erro |
| GPU rendering quebra com fontes customizadas | Baixa | egui usa wgpu por padrão, fontes são texturas, funciona |

---

## 6. Critérios de aceitação (verificáveis)

- [ ] `src/telas/theme/{cores,tipografia,espacamento,estilo}.rs` existem
- [ ] `src/telas/componentes/modal.rs` existe com 4 funções públicas
- [ ] `cargo build` exit 0, 0 novos warnings
- [ ] `cargo test --lib` ≥ 92 passed (cuidado: alterações em componentes não devem quebrar testes existentes; componentes visuais são opacos ao teste de unidade, mas dependências de modelos/dto não podem regredir)
- [ ] `grep -r "Color32::from_rgb" src/telas/` retorna apenas o módulo `theme/cores.rs`
- [ ] `grep -r "egui::Window" src/telas/` retorna 0 (todos modais migrados)
- [ ] Login renderiza com tema dark aplicado (verificar manualmente)
- [ ] Toggle dark/light na sidebar alterna visualmente
- [ ] `~/.config/gar-system/theme.toml` é criado ao alternar, lido no próximo startup
- [ ] `painel_principal.rs` usa `componentes::modal::info()` em vez de `egui::Window`
- [ ] `painel_os_edicao.rs` usa `componentes::modal::confirmacao()` em ações destrutivas

---

## 7. Fora de escopo (explícito)

- ❌ Componentes: botões, tabelas, formulários, toasts, cartões (specs futuras)
- ❌ Migração de tema dark/light nas 14 telas restantes (só login + principal + OS + cotação nesta spec)
- ❌ Mudança de idioma (i18n)
- ❌ Acessibilidade avançada (leitor de tela, navegação por teclado completa)
- ❌ Animações complexas (parallax, transições de página)
- ❌ Testes visuais automatizados (egui não tem suporte maduro)

---

## 8. Próximos passos (após aprovação)

1. Aprovação humana deste spec
2. Self-review do spec (placeholder scan, contradições, ambiguidade)
3. Commit do spec em `docs/superpowers/specs/`
4. Invocar skill `superpowers:writing-plans` para gerar plano de execução
5. Plano vai pro KORA-ARCHITECT (implementação) — eu AURA-PLANNER não escrevo código
6. PR por fase, com screenshots de cada tela migrada

---

## Anexo A — Inspirações visuais declaradas

- **Data 7** (playlist YouTube do humano) — referência de "vendável, funcional"
- **Linear** (linear.app) — densidade, animações sutis, foco em produtividade
- **Stripe** (stripe.com) — uso de espaço, tipografia hierárquica
- **Vercel** (vercel.com) — dark mode premium, sombras, glow
- **Bling/Omie/Conta Azul** — referência de ERP brasileiro moderno

**Não-cópia:** a paleta cyan/blue/silver/ink do GAR não é encontrada em nenhum dos concorrentes diretos. Essa é a assinatura única.

---

## Anexo B — Esboço visual do modal (ASCII)

```
┌──────────────────────────────────────────┐
│  ──── Título do Modal ───────────────  ×  │  ← header: ink-soft, prata
├──────────────────────────────────────────┤
│                                          │
│   Mensagem do modal aqui. Corpo do       │  ← body: paper/ink, 15px
│   texto com respiração generosa.         │
│                                          │
│   ┌─ info secundária ────────────────┐  │
│   │ Detalhe adicional                │  │  ← inline opcional
│   └──────────────────────────────────┘  │
│                                          │
├──────────────────────────────────────────┤
│         [ Cancelar ]    [ Confirmar ]    │  ← footer: paper-soft, 40px
└──────────────────────────────────────────┘
   ↑ container: 12px corner radius, sombra, borda cyan-glow sutil (1px, 30% alpha)
   ↑ overlay: ink 60% opacity
```

---

**FIM DO SPEC (RASCUNHO)**
