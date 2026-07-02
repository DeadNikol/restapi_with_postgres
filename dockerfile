# Используем образ Rust
FROM rust:latest AS builder

# Создаём директорию
WORKDIR /app

# Копируем файлы проекта для сборки зависимостей
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch

# Копируем исходный код и собираем приложение
COPY src ./src
RUN cargo build --release


# Финальный образ
FROM debian:bookworm-slim

# Устанавливаем библиотеки, необходимые для запуска (включая SSL)
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Копируем скомпилированный бинарный файл из первого этапа
COPY --from=builder /app/target/release/TrainRust /app/TrainRust

# Открываем порт, который будет слушать наше приложение
EXPOSE 3000

# Команда для запуска
CMD ["./TrainRust"]