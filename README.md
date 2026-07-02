# REST API на Rust

REST API, написанное на **Rust** с использованием **Axum** и **Tokio**. Для хранения данных используется **PostgreSQL**, а приложение и база данных запускаются в контейнерах через **Docker Compose**.

## Используемые технологии

* Rust
* Axum
* Tokio
* PostgreSQL
* Docker
* Docker Compose

## Запуск проекта

1. Скопируйте файл с переменными окружения:

```bash
cp .env.example .env
```

2. Соберите и запустите контейнеры:

```bash
docker-compose up --build
```

После запуска приложение и база данных будут автоматически доступны в Docker.
