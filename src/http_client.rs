use once_cell::sync::Lazy;
use reqwest::blocking::Client;
use std::time::Duration;

/// Singleton reqwest blocking client reutilizável por toda a aplicação GUI.
/// Reusar o mesmo Client reduz criação de sockets e handshakes TLS repetidos.
static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(8))
        // permitir reuso de conexões, keep-alive já é padrão
        .build()
        .expect("Falha ao criar Client HTTP global")
});

pub fn get_client() -> &'static Client {
    &HTTP_CLIENT
}
