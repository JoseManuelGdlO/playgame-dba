mod hospital_arcangel;

pub use hospital_arcangel::TICKET_ENUNCIADO;

use postgresql_embedded::{PostgreSQL, Settings};
use serde_json::Value;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};

#[derive(serde::Serialize)]
pub struct QueryResult {
    pub rows: Vec<Value>,
}

/// Arranca Postgres embebido (descarga en compile-time vía el feature `bundled`,
/// cero red en runtime), crea la base de Hospital Arcángel y carga schema+seed.
/// Devuelve el manejador del servidor (hay que mantenerlo vivo mientras la app
/// corra) y el pool de conexión.
pub async fn init_embedded_postgres() -> anyhow::Result<(PostgreSQL, PgPool)> {
    let settings = Settings::new();
    let mut pg = PostgreSQL::new(settings);
    pg.setup().await?;
    pg.start().await?;

    let db_name = hospital_arcangel::DB_NAME;
    if !pg.database_exists(db_name).await? {
        pg.create_database(db_name).await?;
    }

    let url = pg.settings().url(db_name);
    let pool = PgPoolOptions::new().max_connections(5).connect(&url).await?;

    sqlx::raw_sql(hospital_arcangel::SCHEMA_SQL).execute(&pool).await?;
    sqlx::raw_sql(hospital_arcangel::SEED_SQL).execute(&pool).await?;

    Ok((pg, pool))
}

/// Ejecuta SQL arbitrario escrito por el jugador. Alcance del spike (Etapa 14):
/// solo lectura — SELECT/CTE (incl. recursivo) y EXPLAIN.
///
/// El texto viene del jugador, así que sqlx exige envolverlo en `AssertSqlSafe`
/// para reconocer explícitamente que no hay bind params posibles aquí: ejecutar
/// SQL libre del jugador es el propósito del juego, no una vulnerabilidad.
pub async fn run_query(pool: &PgPool, sql: &str) -> anyhow::Result<QueryResult> {
    let trimmed = sql.trim().trim_end_matches(';');
    if trimmed.is_empty() {
        anyhow::bail!("La query está vacía.");
    }

    let rows: Vec<Value> = if trimmed[..7.min(trimmed.len())].eq_ignore_ascii_case("explain") {
        let db_rows = sqlx::query(sqlx::AssertSqlSafe(trimmed.to_string()))
            .fetch_all(pool)
            .await?;
        db_rows
            .into_iter()
            .map(|row| {
                let plan_line: String = row.try_get(0).unwrap_or_default();
                serde_json::json!({ "QUERY PLAN": plan_line })
            })
            .collect()
    } else {
        let wrapped = format!(
            "SELECT coalesce(json_agg(row_to_json(query_result_row)), '[]'::json) AS result FROM ({trimmed}) AS query_result_row"
        );
        let row = sqlx::query(sqlx::AssertSqlSafe(wrapped))
            .fetch_one(pool)
            .await?;
        let value: Value = row.try_get(0)?;
        match value {
            Value::Array(items) => items,
            other => vec![other],
        }
    };

    Ok(QueryResult { rows })
}

pub async fn run_ticket_solution(pool: &PgPool) -> anyhow::Result<QueryResult> {
    run_query(pool, hospital_arcangel::TICKET_SOLUCION).await
}
