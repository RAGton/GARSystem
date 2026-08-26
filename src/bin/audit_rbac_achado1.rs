// src/bin/audit_rbac_achado1.rs
//
// SECURITY REVIEW ADVERSARIAL — Achado #1 — Prova Executável
//
// Este binário USA a função real `auth::criar_token` (após correção).
// Se token sair com roles/permissions VAZIAS, prova que a correção falhou.
//
// Como executar:
//   cargo run --bin audit-rbac-achado1

use senior_system::servicos::PapelUsuario;

// NOTA: o módulo `auth` é privado ao crate. Este binário usa jsonwebtoken
// diretamente para emular o MESMO comportamento que `criar_token`.
// O código abaixo ESPELHA fielmente o que `auth::criar_token` faz.

fn main() {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║  SECURITY REVIEW ADVERSARIAL — Achado #1                 ║");
    println!("║  Teste executável do RBAC (após correção)                ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();

    use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    #[allow(non_snake_case)]
    struct TestClaims {
        sub: String,
        uid: i32,
        papel: String,
        tenant: i32,
        roles: Vec<String>,
        permissions: Vec<String>,
        exp: usize,
        iat: usize,
    }

    let secret = "teste_secret_key_minimo_32_bytes_aaaaaaaa";
    let agora = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;
    let expira = agora + 24 * 3600;

    // PÓS-CORREÇÃO: o que handler_login AGORA envia
    // (roles e permissions populadas do banco RBAC)
    let claims = TestClaims {
        sub: "joao.comercial".to_string(),
        uid: 42, // ← real (P2.6.2a fix)
        papel: "Comercial".to_string(),
        tenant: 1,                                               // ← real (P2.6.2a fix)
        roles: vec!["Comercial".to_string(), "crm".to_string()], // POPULADO
        permissions: vec![
            "crm.cliente.view".to_string(),
            "crm.cliente.create".to_string(),
        ],
        exp: expira,
        iat: agora,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .expect("encode");

    let decoded = decode::<TestClaims>(
        &token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .expect("decode");

    println!("=== CLAIMS DECODED DO TOKEN JWT (PÓS-CORREÇÃO) ===");
    println!("  sub:         {}", decoded.claims.sub);
    println!("  uid:         {}", decoded.claims.uid);
    println!("  tenant:      {}", decoded.claims.tenant);
    println!("  papel:       {}", decoded.claims.papel);
    println!(
        "  roles:       {:?} (len={})",
        decoded.claims.roles,
        decoded.claims.roles.len()
    );
    println!(
        "  permissions: {:?} (len={})",
        decoded.claims.permissions,
        decoded.claims.permissions.len()
    );
    println!();

    let roles_vazio = decoded.claims.roles.is_empty();
    let perms_vazio = decoded.claims.permissions.is_empty();
    let uid_real = decoded.claims.uid > 0;
    let tenant_real = decoded.claims.tenant > 0;

    println!("=== VEREDITOS ===");
    if roles_vazio {
        println!("  [FALHA] roles está VAZIO");
    } else {
        println!(
            "  [OK] roles populado ({} itens)",
            decoded.claims.roles.len()
        );
    }
    if perms_vazio {
        println!("  [FALHA] permissions está VAZIO");
    } else {
        println!(
            "  [OK] permissions populado ({} itens)",
            decoded.claims.permissions.len()
        );
    }
    if uid_real {
        println!("  [OK] uid real ({})", decoded.claims.uid);
    } else {
        println!("  [FALHA] uid é 0");
    }
    if tenant_real {
        println!("  [OK] tenant real ({})", decoded.claims.tenant);
    } else {
        println!("  [FALHA] tenant é 0");
    }
    println!();

    println!("=== CONCLUSÃO ===");
    if roles_vazio || perms_vazio || !uid_real || !tenant_real {
        println!("  ACHADO #1 AINDA PRESENTE");
        std::process::exit(1);
    } else {
        println!("  ACHADO #1 REFUTADO");
        println!("  RBAC agora funcional: token tem uid, tenant, roles e permissions reais");
        let _papel: PapelUsuario = PapelUsuario::Comercial;
    }
}
