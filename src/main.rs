use axum::{Router, extract::{State, Path}, routing::get};
use serde::{Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::{env};
use redis::{Client as RedisClient, AsyncCommands};

/// Состояние приложения, которое хранит пул подключений к БД
#[derive(Clone)]
struct AppState {
    db: PgPool,
    redis: RedisClient,
}

/// Структура для получения ответа из таблицы slow_data
#[derive(Debug, Serialize, sqlx::FromRow)]
struct SlowDataResponse {
    id: i32,
    name: String,
    description: String,
    created_at: String,
}

#[derive(Serialize, sqlx::FromRow)]
struct TableResponse {
    id: i32,
    value: String,
    b_id: i32,
    b_name: String,
    a_created_at: String,
    b_created_at: String,
}

async fn get_slow_data_by_id(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> String {
    let cache_key = format!("slow_data:{}", id);

    // 1. Пытаемся получить данные из Redis
    let mut conn = match state.redis.get_multiplexed_async_connection().await {
        Ok(c) => c,
        Err(e) => return format!("Ошибка подключения к Redis: {}", e),
    };

    // Проверяем, есть ли данные в кеше
    if let Ok(Some(json)) = conn.get::<_, Option<String>>(&cache_key).await {
        // Данные найдены в кеше — возвращаем их
        return json;
    }

    // 2. Данных в кеше нет — идём в PostgreSQL
    let result = sqlx::query_as::<_, SlowDataResponse>(
        r#"
        SELECT 
            id,
            name,
            description,
            created_at::text as created_at
        FROM slow_data
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await;

    match result {
        Ok(Some(record)) => {
            // Сериализуем данные в JSON
            let json_data = serde_json::to_string(&record).unwrap_or_else(|_| "{}".to_string());
            
            // Сохраняем в Redis на 10 секунд
            let _: () = conn.set_ex(&cache_key, &json_data, 10).await.unwrap_or_else(|_| ());
            
            json_data
        }
        Ok(None) => format!("Запись с id {} не найдена", id),
        Err(e) => format!("Ошибка базы данных: {}", e),
    }
}
async fn get_all_table(State(state): State<AppState>) -> String {
    // Используем query_as (без !), запрос проверяется во время выполнения
    let result = sqlx::query_as::<_, TableResponse>(
        r#"
        SELECT 
            a.id,
            a.value,
            a.b_id,
            b.name as b_name,
            a.created_at::text as a_created_at,
            b.created_at::text as b_created_at
        FROM table_a a
        JOIN table_b b ON a.b_id = b.id
        ORDER BY a.id
        "#,
    )
    .fetch_all(&state.db)
    .await;

    match result {
        Ok(records) => serde_json::to_string(&records).unwrap_or_else(|_| "[]".to_string()),
        Err(e) => format!("Ошибка базы данных: {}", e),
    }
}

/// Обработчик endpoint'а `/health`
async fn get_health(State(state): State<AppState>) -> String {
    match sqlx::query("SELECT 1").fetch_one(&state.db).await {
        Ok(_) => "Сервер и база данных работают".to_string(),
        Err(e) => format!("Ошибка базы данных: {}", e),
    }
}

#[tokio::main]
async fn main() {
    let database_url = env::var("DATABASE_URL").unwrap();
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://redis:6379".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Не удалось подключиться к базе данных");

    let redis_client = RedisClient::open(redis_url).expect("Не удалось подключиться к Redis");
    let mut conn = redis_client.get_multiplexed_async_connection().await.unwrap();

    // let _: () = conn.set("my_key", "value").await.unwrap();
    // let value: String = conn.get("my_key").await.expect("Результата с таким ключом нет");
    // println!("{}", value);


    let app_state = AppState { db: pool , redis: redis_client};

    let app: Router = Router::new()
        .route("/health", get(get_health))
        .route("/get_all_table", get(get_all_table))
        .route("/get_slow_data/{id}", get(get_slow_data_by_id))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Сервер запущен на порту :3000");
    axum::serve(listener, app).await.unwrap();
}
