# Dockerfile

# ---- Estágio 1: Builder ----
# [CORRIGIDO] Usamos a imagem `latest` para garantir uma versão recente do Cargo.
FROM docker.io/library/rust:latest AS builder

# Cria um diretório de trabalho dentro do contêiner
WORKDIR /usr/src/app

# Copia os arquivos do projeto para o contêiner
COPY . .

USER root
RUN mkdir -p /var/lib/apt/lists/partial && apt-get update && apt-get install -y libmariadb-dev

# Compila o servidor em modo de release para performance
RUN cargo build --release --bin senior-system-server

# ---- Estágio 2: Runner ----
# Usamos uma imagem base Debian slim, que é muito menor que a imagem do Rust
FROM docker.io/library/debian:12-slim

# Instala apenas a dependência de tempo de execução correta
RUN apt-get update && apt-get install -y libmariadb-dev && rm -rf /var/lib/apt/lists/*

# Copia APENAS o binário compilado do estágio de build para a imagem final
COPY --from=builder /usr/src/app/target/release/senior-system-server /usr/local/bin/senior-system-server

# Expõe a porta 3000, onde nosso servidor Axum está rodando
EXPOSE 3000

# O comando que será executado quando o contêiner iniciar
CMD ["senior-system-server"]