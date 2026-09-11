// tests/regra11_audit.rs
//
// REGRA 11 — MULTI-TENANT
// A única métrica aceita é auditoria do SQL executável.
//
// Justificativa P2.6.2c: testes de auditoria.
#![allow(unused_imports, unused_variables, dead_code, unused_mut)]
//
// Este teste executa o script de auditoria e valida que
// 100% das queries em tabelas tenant-aware têm tenant_id.

use std::process::Command;

#[test]
#[ignore = "executa script Python; requer Python 3 disponível"]
fn regra11_metric_100_por_cento_tenant_id() {
    // Executa o script de auditoria
    let output = Command::new("python3")
        .arg("-c")
        .arg(include_str!("../scripts/audit_tenant_id.py"))
        .output()
        .expect("Falha ao executar script de auditoria");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    eprintln!("=== AUDIT OUTPUT ===");
    eprintln!("{}", stdout);
    eprintln!("{}", stderr);

    // Deve ter "100.00%" e "ZERO vazamentos"
    assert!(
        stdout.contains("100.00%"),
        "Métrica deve ser 100.00%. Saída:\n{}",
        stdout
    );
    assert!(
        stdout.contains("ZERO vazamentos"),
        "Deve ter ZERO vazamentos. Saída:\n{}",
        stdout
    );
}

#[test]
fn regra11_assinatura_tem_parametros_tenant_id() {
    // Verifica que TODAS as funções pub em repository.rs têm tenant_id
    // (defense in depth no signature)
    //
    // NOTA: esta é uma métrica INDIRETA, mas a métrica DIRETA
    // (SQL executável) é testada em regra11_metric_100_por_cento_tenant_id
    use std::fs;

    let mut total = 0;
    let mut com_tenant_id = 0;

    fn walk_dir(path: &str, cb: &mut dyn FnMut(&str)) {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    walk_dir(p.to_str().unwrap(), cb);
                } else if p.extension().and_then(|s| s.to_str()) == Some("rs")
                    && p.to_str().unwrap().ends_with("repository.rs")
                {
                    if let Ok(content) = std::fs::read_to_string(&p) {
                        cb(&content);
                    }
                }
            }
        }
    }

    let mut count_fns = |content: &str| {
        // Pegar cada "pub fn X(" e seu bloco de assinatura (até ")")
        let mut chars = content.chars().peekable();
        let mut current = String::new();
        let mut in_fn = false;
        let mut depth = 0;
        let mut sig = String::new();
        let mut after_sig_collecting = false;
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();
            if line.starts_with("pub fn ") && line.contains("(") {
                // Coletar assinatura completa (multi-linha)
                let mut sig_full = line.to_string();
                let mut paren_count =
                    line.matches('(').count() as i32 - line.matches(')').count() as i32;
                while paren_count > 0 && i + 1 < lines.len() {
                    i += 1;
                    sig_full.push(' ');
                    sig_full.push_str(lines[i].trim());
                    paren_count +=
                        lines[i].matches('(').count() as i32 - lines[i].matches(')').count() as i32;
                }
                total += 1;
                if sig_full.contains("tenant_id") || sig_full.contains("empresa_id") {
                    com_tenant_id += 1;
                }
            }
            i += 1;
        }
    };

    // Apenas módulos tenant-aware (NÃO contar empresa/ e rbac/ que são SISTEMA)
    // Listar arquivos .rs diretamente sem walk recursivo
    let modulos_tenant = [
        "src/banco_de_dados/cliente.rs",
        "src/banco_de_dados/estoque.rs",
        "src/banco_de_dados/ordem_servico.rs",
        "src/banco_de_dados/orcamento.rs",
        "src/banco_de_dados/servico.rs",
        "src/crm/repository.rs",
        "src/arquivos/repository.rs",
        "src/cotacao_orcamento/repository.rs",
        "src/operations/repository.rs",
        "src/os_mobile/repository.rs",
        "src/financial/repository.rs",
    ];
    for path in modulos_tenant {
        if let Ok(content) = std::fs::read_to_string(path) {
            count_fns(&content);
        }
    }

    eprintln!(
        "Repository functions: {} total, {} com tenant_id",
        total, com_tenant_id
    );
    assert!(
        com_tenant_id == total,
        "Todas as {} funções de repository devem ter tenant_id, mas apenas {} têm",
        total,
        com_tenant_id
    );
}
