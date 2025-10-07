use once_cell::sync::Lazy;
use std::sync::Mutex;
use threadpool::ThreadPool;

// Tamanho do pool baseado em número de CPUs para operações de bloqueio
static POOL: Lazy<Mutex<ThreadPool>> = Lazy::new(|| {
    let n = std::cmp::max(2, num_cpus::get());
    Mutex::new(ThreadPool::new(n))
});

pub fn spawn<F>(job: F)
where
    F: FnOnce() + Send + 'static,
{
    let guard = POOL.lock().unwrap();
    guard.execute(job);
}
