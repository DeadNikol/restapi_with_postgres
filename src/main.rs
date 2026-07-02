use axum::{Router, extract::State, routing::get};
use serde::Serialize;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::{env};

/// Состояние приложения, которое хранит пул подключений к БД
#[derive(Clone)]
struct AppState {
    db: PgPool,
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
    // let database_url = "postgres://user:password@my_postgres:5432/app_db";

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Не удалось подключиться к базе данных");

    let app_state = AppState { db: pool };

    let app: Router = Router::new()
        .route("/health", get(get_health))
        .route("/get_all_table", get(get_all_table))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Сервер запущен на порту :3000");
    axum::serve(listener, app).await.unwrap();
}
