// src/rbac/cache.rs
//
// Permission cache in-memory com TTL. Thread-safe via RwLock.
//
// ## Estratégia
//
// - HashMap<user_id, (empresa_id, Vec<permission>, timestamp)>
// - TTL: 5 minutos (configurável por env `SENIOR_PERMISSION_CACHE_TTL_SECS`)
// - Invalidation:
//   - Login (não cacheia antes de autenticar)
//   - Logout (invalidar imediatamente)
//   - Mudança de role (admin atribuiu/removeu role → invalidar)
//
// ## Preparação para Redis
//
// A interface `PermissionCache` é abstrata o suficiente para trocar a
// implementação in-memory por Redis sem mudar os callers. Quando P3+
//
// trouxer Redis, basta implementar `PermissionCache` com driver redis-rs.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

const DEFAULT_TTL_SECS: u64 = 300; // 5 minutos

/// Entry no cache: tenant + permissions + timestamp.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub empresa_id: i32,
    pub permissions: Vec<String>,
    pub cached_at: Instant,
}

/// Trait abstrato (preparação para Redis).
pub trait PermissionCache: Send + Sync {
    fn get(&self, user_id: i32) -> Option<CacheEntry>;
    fn put(&self, user_id: i32, entry: CacheEntry);
    fn invalidate(&self, user_id: i32);
    fn invalidate_all(&self);
}

/// Implementação in-memory com TTL.
pub struct InMemoryPermissionCache {
    inner: Arc<RwLock<HashMap<i32, CacheEntry>>>,
    ttl: Duration,
}

impl InMemoryPermissionCache {
    pub fn new() -> Self {
        let ttl_secs = std::env::var("SENIOR_PERMISSION_CACHE_TTL_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_TTL_SECS);
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            ttl: Duration::from_secs(ttl_secs),
        }
    }

    pub fn with_ttl(ttl: Duration) -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }

    fn is_expired(&self, entry: &CacheEntry) -> bool {
        entry.cached_at.elapsed() > self.ttl
    }
}

impl Default for InMemoryPermissionCache {
    fn default() -> Self {
        Self::new()
    }
}

impl PermissionCache for InMemoryPermissionCache {
    fn get(&self, user_id: i32) -> Option<CacheEntry> {
        let map = self.inner.read().ok()?;
        let entry = map.get(&user_id)?;
        if self.is_expired(entry) {
            drop(map);
            self.invalidate(user_id);
            return None;
        }
        Some(entry.clone())
    }

    fn put(&self, user_id: i32, entry: CacheEntry) {
        if let Ok(mut map) = self.inner.write() {
            map.insert(user_id, entry);
        }
    }

    fn invalidate(&self, user_id: i32) {
        if let Ok(mut map) = self.inner.write() {
            map.remove(&user_id);
        }
    }

    fn invalidate_all(&self) {
        if let Ok(mut map) = self.inner.write() {
            map.clear();
        }
    }
}

/// Singleton do cache (in-memory).
///
/// P2.6.1: in-memory. P3+ (Redis): trocar a implementação por trás da trait.
pub fn permission_cache() -> Arc<InMemoryPermissionCache> {
    use once_cell::sync::Lazy;
    static CACHE: Lazy<Arc<InMemoryPermissionCache>> =
        Lazy::new(|| Arc::new(InMemoryPermissionCache::new()));
    CACHE.clone()
}

/// Helper para invalidar o cache de um usuário.
pub fn invalidar_cache(user_id: i32) {
    permission_cache().invalidate(user_id);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(perms: Vec<&str>) -> CacheEntry {
        CacheEntry {
            empresa_id: 1,
            permissions: perms.into_iter().map(String::from).collect(),
            cached_at: Instant::now(),
        }
    }

    #[test]
    fn put_get_roundtrip() {
        let c = InMemoryPermissionCache::new();
        c.put(1, entry(vec!["crm.cliente.create", "os.view"]));
        let got = c.get(1).unwrap();
        assert_eq!(got.empresa_id, 1);
        assert_eq!(got.permissions.len(), 2);
    }

    #[test]
    fn invalidate_remove() {
        let c = InMemoryPermissionCache::new();
        c.put(1, entry(vec!["x"]));
        c.invalidate(1);
        assert!(c.get(1).is_none());
    }

    #[test]
    fn ttl_expirado_retorna_none() {
        let c = InMemoryPermissionCache::with_ttl(Duration::from_millis(10));
        c.put(1, entry(vec!["x"]));
        std::thread::sleep(Duration::from_millis(20));
        assert!(c.get(1).is_none());
    }

    #[test]
    fn invalidate_all_limpa_tudo() {
        let c = InMemoryPermissionCache::new();
        c.put(1, entry(vec!["a"]));
        c.put(2, entry(vec!["b"]));
        c.invalidate_all();
        assert!(c.get(1).is_none());
        assert!(c.get(2).is_none());
    }
}
