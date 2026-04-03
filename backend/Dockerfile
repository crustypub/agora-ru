# --- Этап разработки (Development) ---
FROM rust:1.88-bookworm AS dev

# Устанавливаем инструменты для быстрой разработки
RUN apt-get update && apt-get install -y clang mold && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-watch

WORKDIR /app

# Флаги для ускорения линковки (mold)
ENV RUSTFLAGS="-C link-arg=-fuse-ld=mold"

# Копируем только файлы зависимостей для кэширования
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() { println!("stub"); }' > src/main.rs
RUN cargo build && rm -rf src # Предсборка зависимостей

# Копируем весь остальной код
COPY . .

# По умолчанию в деве просто ждем команду из docker-compose
CMD ["cargo", "watch", "-x", "run"]


# --- Этап сборки (Builder для Production) ---
FROM dev AS builder
RUN cargo build --release


# --- Финальный образ (Production) ---
FROM debian:bookworm-slim AS release

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/agora-ru-backend /app/agora-ru-backend
COPY --from=builder /app/config /app/config

EXPOSE 6080
CMD ["./agora-ru-backend"]