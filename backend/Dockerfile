FROM rust:1.88-bookworm AS builder

WORKDIR /app

# Копируем файлы для кэширования зависимостей
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() { println!("stub"); }' > src/main.rs
RUN cargo build --release || true

# Удаляем stub-артефакты, чтобы cargo пересобрал с реальным кодом
RUN rm -rf src target/release/deps/agora_ru_backend* target/release/agora-ru-backend

# Копируем исходный код и конфиги
COPY . .

# Пересобираем с реальным кодом
RUN cargo build --release

# --- Финальный образ ---
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/agora-ru-backend /app/agora-ru-backend
COPY --from=builder /app/config /app/config

ENV CARGO_PKG_NAME="agora-ru-backend"
ENV CARGO_PKG_VERSION="0.1.0"

EXPOSE 6080

CMD ["./agora-ru-backend"]
