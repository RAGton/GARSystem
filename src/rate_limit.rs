// src/rate_limit.rs
//
// Rate limiter simples, in-memory, por chave (IP ou username).
// Suficiente para um único processo. Em deploy multi-pod, substituir por
// Redis ou similar (TODO: Sprint 1.5).

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Janela deslizante mínima. Default: 60 segundos.
const JANELA: Duration = Duration::from_secs(60);

/// Estrutura para contar tentativas dentro da janela.
#[derive(Debug)]
struct Contador {
    tentativas: Vec<Instant>,
}

impl Contador {
    fn nova() -> Self {
        Self {
            tentativas: Vec::new(),
        }
    }

    /// Limpa entradas antigas (fora da janela).
    fn limpar_antigas(&mut self) {
        let limite = Instant::now() - JANELA;
        self.tentativas.retain(|t| *t > limite);
    }

    /// Conta quantas tentativas dentro da janela.
    fn contar(&mut self) -> usize {
        self.limpar_antigas();
        self.tentativas.len()
    }

    /// Registra uma tentativa.
    fn registrar(&mut self) {
        self.limpar_antigas();
        self.tentativas.push(Instant::now());
    }

    /// Espera até a próxima tentativa caber na janela.
    /// Retorna a duração a esperar. Zero se já pode tentar.
    fn tempo_ate_reset(&mut self, limite: usize) -> Duration {
        self.limpar_antigas();
        if self.tentativas.len() < limite {
            return Duration::ZERO;
        }
        // A tentativa mais antiga determina quando libera a vaga.
        match self.tentativas.first() {
            Some(t) => {
                let expiracao = *t + JANELA;
                let agora = Instant::now();
                if expiracao > agora {
                    expiracao - agora
                } else {
                    Duration::ZERO
                }
            }
            None => Duration::ZERO,
        }
    }
}

/// Rate limiter global do processo. Thread-safe via Mutex.
pub struct RateLimiter {
    /// chave (ex: "login:127.0.0.1") → Contador
    contadores: Mutex<HashMap<String, Contador>>,
    /// Limite padrão (tentativas por janela).
    limite_padrao: usize,
}

impl RateLimiter {
    pub fn novo(limite_padrao: usize) -> Self {
        Self {
            contadores: Mutex::new(HashMap::new()),
            limite_padrao,
        }
    }

    /// Verifica se a chave está dentro do limite. Se sim, registra e retorna
    /// `Ok`. Se não, retorna `Err` com a duração a esperar antes de tentar
    /// de novo.
    pub fn tentar(&self, chave: &str) -> Result<(), Duration> {
        self.tentar_com_limite(chave, self.limite_padrao)
    }

    pub fn tentar_com_limite(&self, chave: &str, limite: usize) -> Result<(), Duration> {
        let mut map = self.contadores.lock().expect("rate_limit mutex poisoned");
        let entry = map.entry(chave.to_string()).or_insert_with(Contador::nova);
        if entry.contar() < limite {
            entry.registrar();
            Ok(())
        } else {
            Err(entry.tempo_ate_reset(limite))
        }
    }

    /// Reseta a chave (ex: após login bem-sucedido, libera o IP).
    pub fn resetar(&self, chave: &str) {
        if let Ok(mut map) = self.contadores.lock() {
            map.remove(chave);
        }
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        // Lê do env, default 5/min.
        let limite = std::env::var("LOGIN_RATE_LIMIT_PER_MINUTE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(5);
        Self::novo(limite)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permite_ate_o_limite() {
        let rl = RateLimiter::novo(3);
        assert!(rl.tentar("k").is_ok());
        assert!(rl.tentar("k").is_ok());
        assert!(rl.tentar("k").is_ok());
        assert!(rl.tentar("k").is_err());
    }

    #[test]
    fn chaves_independentes() {
        let rl = RateLimiter::novo(1);
        assert!(rl.tentar("a").is_ok());
        assert!(rl.tentar("a").is_err());
        assert!(rl.tentar("b").is_ok());
    }

    #[test]
    fn resetar_limpa() {
        let rl = RateLimiter::novo(1);
        assert!(rl.tentar("a").is_ok());
        assert!(rl.tentar("a").is_err());
        rl.resetar("a");
        assert!(rl.tentar("a").is_ok());
    }
}
