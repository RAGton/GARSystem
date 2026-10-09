# Dockerfile

# ---- Estágio 1: Builder ----
# [CORRIGIDO] Usamos a imagem `latest` para garantir uma versão recente do Cargo.
FROM rust:latest AS builder

# Cria um diretório de trabalho dentro do contêiner
WORKDIR /usr/src/app

# Copia os arquivos do projeto para o contêiner
COPY . .

# Instala a dependência de build correta (libmariadb-dev)
RUN apt-get update && apt-get install -y libmariadb-dev

# Compila o servidor em modo de release para performance
RUN cargo build --release --bin gar-system-server

# ---- Estágio 2: Runner ----
# Usamos uma imagem base Debian slim, que é muito menor que a imagem do Rust
FROM debian:12-slim

# Instala apenas a dependência de tempo de execução correta
RUN apt-get update && apt-get install -y libmariadb-dev && rm -rf /var/lib/apt/lists/*

# Copia APENAS o binário compilado do estágio de build para a imagem final
COPY --from=builder /usr/src/app/target/release/gar-system-server /usr/local/bin/gar-system-server

# Expõe a porta 3000, onde nosso servidor Axum está rodando
EXPOSE 3000

# O comando que será executado quando o contêiner iniciar
CMD ["gar-system-server"]