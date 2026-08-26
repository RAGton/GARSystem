// src/banco_de_dados/pagination.rs
//
// Helpers para paginação cursor-based e offset-based.
//
// ## Estratégia
//
// Para listas grandes (clientes, OS, etc.) evitamos `OFFSET N LIMIT M`
// puro (que degrada com N alto) usando uma combinação de filtros
// estáveis e `LIMIT M` com um cap máximo.
//
// Para telas típicas de gestão (até ~10k registros), offset+limit
// é suficiente. Quando passar de 100k, refatorar para cursor-based.

use serde::{Deserialize, Serialize};

/// Parâmetros de paginação vindos do cliente.
/// Defaults: page=1, limit=50, max_limit=200.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Paginacao {
    /// 1-based: page=1 é a primeira.
    pub page: u32,
    /// Quantos itens por página.
    pub limit: u32,
}

impl Default for Paginacao {
    fn default() -> Self {
        Self { page: 1, limit: 50 }
    }
}

impl Paginacao {
    /// Limite máximo permitido por request (defesa contra DoS).
    pub const MAX_LIMIT: u32 = 200;

    /// Normaliza a paginação: page>=1, limit em [1, MAX_LIMIT].
    pub fn sanitizar(mut self) -> Self {
        if self.page == 0 {
            self.page = 1;
        }
        if self.limit == 0 {
            self.limit = 50;
        }
        if self.limit > Self::MAX_LIMIT {
            self.limit = Self::MAX_LIMIT;
        }
        self
    }

    /// `OFFSET` derivado de page.
    pub fn offset(&self) -> u32 {
        (self.page - 1) * self.limit
    }
}

/// Resposta padrão de listagem paginada.
#[derive(Debug, Clone, Serialize)]
pub struct Pagina<T> {
    pub items: Vec<T>,
    pub page: u32,
    pub limit: u32,
    pub total: u64,
    pub total_paginas: u32,
}

impl<T> Pagina<T> {
    pub fn from_items(items: Vec<T>, page: u32, limit: u32, total: u64) -> Self {
        let total_paginas = if limit == 0 {
            0
        } else {
            ((total as f64) / (limit as f64)).ceil() as u32
        };
        Self {
            items,
            page,
            limit,
            total,
            total_paginas,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paginacao_default_sanitizada() {
        let p = Paginacao::default().sanitizar();
        assert_eq!(p.page, 1);
        assert_eq!(p.limit, 50);
        assert_eq!(p.offset(), 0);
    }

    #[test]
    fn paginacao_clamp_max_limit() {
        let p = Paginacao {
            page: 1,
            limit: 9999,
        }
        .sanitizar();
        assert_eq!(p.limit, Paginacao::MAX_LIMIT);
    }

    #[test]
    fn paginacao_zero_vira_default() {
        let p = Paginacao { page: 0, limit: 0 }.sanitizar();
        assert_eq!(p.page, 1);
        assert_eq!(p.limit, 50);
    }

    #[test]
    fn paginacao_offset_page_2() {
        let p = Paginacao { page: 2, limit: 20 }.sanitizar();
        assert_eq!(p.offset(), 20);
    }

    #[test]
    fn pagina_total_paginas() {
        let p: Pagina<i32> = Pagina::from_items(vec![], 1, 10, 95);
        assert_eq!(p.total_paginas, 10); // ceil(95/10)
    }

    #[test]
    fn pagina_exata() {
        let p: Pagina<i32> = Pagina::from_items(vec![], 1, 10, 100);
        assert_eq!(p.total_paginas, 10);
    }

    #[test]
    fn pagina_uma_pagina() {
        let p: Pagina<i32> = Pagina::from_items(vec![], 1, 10, 5);
        assert_eq!(p.total_paginas, 1);
    }
}
