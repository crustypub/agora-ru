# Agora RU

Монорепозиторий с фронтендом (Nuxt.js) и бэкендом (Rust / Zino).

## Структура проекта

```
agora-ru/
├── frontend/          # Nuxt.js 3 — фронтенд
│   ├── app.vue
│   ├── pages/
│   ├── nuxt.config.ts
│   ├── Dockerfile
│   └── package.json
├── backend/           # Rust / Zino — бэкенд
│   ├── src/main.rs
│   ├── config/config.toml
│   ├── Cargo.toml
│   └── Dockerfile
├── docker-compose.yml
└── README.md
```

## Запуск

### Через Docker Compose (рекомендуется)

```bash
docker compose up --build
```

Сервисы будут доступны:

| Сервис    | URL                    |
| --------- | ---------------------- |
| Фронтенд  | http://localhost:3000  |
| Бэкенд    | http://localhost:6080  |

### Локально

**Фронтенд:**

```bash
cd frontend
npm install
npm run dev
```

**Бэкенд:**

```bash
cd backend
cargo run
```

## Технологии

- **Фронтенд:** [Nuxt.js 3](https://nuxt.com/) (Vue 3, TypeScript)
- **Бэкенд:** [Zino](https://github.com/zino-rs/zino) (Rust, Axum)
- **Оркестрация:** Docker Compose
