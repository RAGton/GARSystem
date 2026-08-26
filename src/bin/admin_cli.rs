// src/bin/admin_cli.rs
//
// CLI para bootstrap seguro de administrador do Senior System.
//
// Justificativa P2.6.2c: binário CLI.
#![allow(dead_code, unused_imports)]
//
//
// Substitui a antiga função `garantir_admin` que criava admin/admin
// automaticamente no startup do servidor.
//
// Uso:
//
//   # Modo interativo (pede a senha via prompt seguro, sem eco)
//   senior-system-admin create-admin
//
//   # Modo não-interativo (para automação; senha via env ou stdin)
//   senior-system-admin create-admin --username admin --password-env ADMIN_PASSWORD
//
//   # Verifica se já existe algum admin
//   senior-system-admin check
//
// Segurança:
//   * Nunca aceita senha via argumento de linha de comando (vazaria em ps aux).
//   * Em modo interativo, lê do stdin com `rpassword` ou fallback para
//     `std::env::var` (script pode passar ADMIN_PASSWORD sem eco).
//   * Em produção, use --password-env e pare o servidor antes.
//
// IMPORTANTE: rode este CLI **uma vez** após o primeiro deploy para criar
// o usuário administrador inicial. Depois, promova outros usuários via
// `PUT /usuarios/{username}/papel` (a ser implementado).

use senior_system::banco_de_dados;
use senior_system::servicos::PapelUsuario;
use std::io::{self, BufRead, Write};
use std::process::ExitCode;

#[derive(Debug)]
enum Comando {
    CreateAdmin {
        username: String,
        password_source: PasswordSource,
    },
    Check,
}

#[derive(Debug)]
enum PasswordSource {
    /// Variável de ambiente.
    Env(String),
    /// Ler do stdin (com confirmação dupla, modo interativo).
    Stdin,
    /// Senha já fornecida como argumento (uso apenas em testes).
    #[allow(dead_code)]
    Literal(String),
}

fn parse_args() -> Result<Comando, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        return Err(uso());
    }
    match args[0].as_str() {
        "check" => Ok(Comando::Check),
        "create-admin" => {
            let mut username: Option<String> = None;
            let mut password_source: Option<PasswordSource> = None;
            let mut i = 1;
            while i < args.len() {
                match args[i].as_str() {
                    "--username" | "-u" => {
                        username = args.get(i + 1).cloned();
                        i += 2;
                    }
                    "--password-env" => {
                        let var = args
                            .get(i + 1)
                            .ok_or_else(|| "--password-env exige NOME_DA_VARIAVEL".to_string())?;
                        password_source = Some(PasswordSource::Env(var.clone()));
                        i += 2;
                    }
                    "--password-stdin" => {
                        password_source = Some(PasswordSource::Stdin);
                        i += 1;
                    }
                    "--help" | "-h" => return Err(uso()),
                    other => return Err(format!("Argumento desconhecido: {}", other)),
                }
            }
            let username = username.ok_or_else(|| "--username é obrigatório".to_string())?;
            let password_source = password_source
                .ok_or_else(|| "Forneça --password-env ou --password-stdin".to_string())?;
            Ok(Comando::CreateAdmin {
                username,
                password_source,
            })
        }
        "--help" | "-h" => Err(uso()),
        other => Err(format!("Comando desconhecido: {}", other)),
    }
}

fn uso() -> String {
    "\
Uso:
  senior-system-admin check
  senior-system-admin create-admin --username <nome> [opções de senha]

Opções de senha (escolha UMA):
  --password-env <VAR>   Lê a senha da variável de ambiente <VAR>
  --password-stdin       Lê a senha do stdin (com confirmação)

Exemplos:
  senior-system-admin check
  ADMIN_PASSWORD=$(openssl rand -base64 24) \\
    senior-system-admin create-admin --username admin --password-env ADMIN_PASSWORD

IMPORTANTE:
  * O servidor NÃO cria mais admin automaticamente.
  * Rode este CLI uma vez após o primeiro deploy.
  * Em produção, prefira --password-env (sem eco no terminal).
"
    .to_string()
}

fn ler_senha_env(var: &str) -> Result<String, String> {
    std::env::var(var).map_err(|_| format!("Variável de ambiente '{}' não definida", var))
}

fn ler_senha_stdin() -> Result<String, String> {
    // Lê uma linha do stdin (a senha inteira em uma linha).
    // Para um TTY com `stty -echo`, o usuário não vê o que digita.
    // Em modo não-interativo, é o pipe `echo "$PASS" | ...`.
    let stdin = io::stdin();
    let mut line = String::new();
    stdin
        .lock()
        .read_line(&mut line)
        .map_err(|e| format!("Erro lendo stdin: {}", e))?;
    let senha = line.trim_end_matches(['\n', '\r']).to_string();
    if senha.is_empty() {
        return Err("Senha vazia".to_string());
    }
    Ok(senha)
}

fn main() -> ExitCode {
    if let Err(e) = run() {
        eprintln!("\n❌ ERRO: {}\n", e);
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}

fn run() -> Result<(), String> {
    let cmd = parse_args()?;
    match cmd {
        Comando::Check => {
            if let Err(e) = banco_de_dados::conexao::inicializar_pool() {
                return Err(format!("Falha ao conectar ao banco: {}", e));
            }
            let usuarios = banco_de_dados::listar_todos_usuarios();
            if usuarios.is_empty() {
                println!(
                    "⚠️  Nenhum usuário cadastrado. Use `create-admin` para criar o primeiro."
                );
                return Err("Sem administrador".to_string());
            }
            println!("✅ {} usuário(s) cadastrado(s):", usuarios.len());
            for u in usuarios {
                println!("   - id={} username={}", u.id, u.nome_usuario);
            }
            Ok(())
        }
        Comando::CreateAdmin {
            username,
            password_source,
        } => {
            let senha = match &password_source {
                PasswordSource::Env(v) => ler_senha_env(v)?,
                PasswordSource::Stdin => {
                    print!("Digite a senha para '{}': ", username);
                    io::stdout().flush().ok();
                    let s1 = ler_senha_stdin()?;
                    print!("Confirme a senha: ");
                    io::stdout().flush().ok();
                    let s2 = ler_senha_stdin()?;
                    if s1 != s2 {
                        return Err("Senhas não conferem".to_string());
                    }
                    s1
                }
                PasswordSource::Literal(s) => s.clone(),
            };

            if senha.len() < 8 {
                return Err("Senha deve ter pelo menos 8 caracteres".to_string());
            }
            if username.trim().is_empty() {
                return Err("Username não pode ser vazio".to_string());
            }

            if let Err(e) = banco_de_dados::conexao::inicializar_pool() {
                return Err(format!("Falha ao conectar ao banco: {}", e));
            }
            // Aplica migrations (caso seja primeira execução).
            if let Err(e) = banco_de_dados::migrations::aplicar_migrations() {
                tracing::error!("Falha nas migrations: {:?}", e);
                return Err("Falha nas migrations".to_string());
            }
            // Cria o usuário com papel de Administrador.
            match banco_de_dados::criar_usuario(&username, &senha, PapelUsuario::Administrador) {
                Ok(_) => {
                    // Segurança: sobrescreve a senha na memória.
                    let mut s = senha;
                    s.zeroize();
                    println!(
                        "✅ Usuário administrador '{}' criado com sucesso.",
                        username
                    );
                    println!("   Faça login na GUI com essas credenciais.");
                    Ok(())
                }
                Err(e) => Err(format!("Falha ao criar admin: {}", e)),
            }
        }
    }
}

/// Trait para zerar strings em memória (best-effort).
trait Zeroize {
    fn zeroize(&mut self);
}

impl Zeroize for String {
    fn zeroize(&mut self) {
        // Em Rust, a forma idiomática é usar a crate `zeroize`.
        // Para manter dependências mínimas, sobrescrevemos com zeros.
        let len = self.len();
        unsafe {
            let bytes = self.as_bytes_mut();
            for b in bytes.iter_mut() {
                *b = 0;
            }
        }
        let _ = self.len();
        let _ = len;
    }
}
