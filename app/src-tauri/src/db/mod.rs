mod hospital_arcangel;
mod postafeta;

use postgresql_embedded::{PostgreSQL, Settings};
use serde_json::Value;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};

/// Empresa activa: cada una vive en su propia base de datos dentro del mismo
/// servidor Postgres embebido (Etapa 11-G: el esquema cambia por completo al
/// cambiar de empresa; el progreso de rango/perks vive fuera de este módulo).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Company {
    HospitalArcangel,
    Postafeta,
}

impl Company {
    fn db_name(self) -> &'static str {
        match self {
            Company::HospitalArcangel => hospital_arcangel::DB_NAME,
            Company::Postafeta => postafeta::DB_NAME,
        }
    }

    fn schema_sql(self) -> &'static str {
        match self {
            Company::HospitalArcangel => hospital_arcangel::SCHEMA_SQL,
            Company::Postafeta => postafeta::SCHEMA_SQL,
        }
    }

    fn seed_sql(self) -> &'static str {
        match self {
            Company::HospitalArcangel => hospital_arcangel::SEED_SQL,
            Company::Postafeta => postafeta::SEED_SQL,
        }
    }
}

#[derive(serde::Serialize)]
pub struct QueryResult {
    pub rows: Vec<Value>,
}

/// Arranca Postgres embebido (descarga en compile-time vía el feature `bundled`,
/// cero red en runtime). Devuelve el manejador del servidor — hay que mantenerlo
/// vivo mientras la app corra.
pub async fn init_embedded_postgres() -> anyhow::Result<PostgreSQL> {
    let settings = Settings::new();
    let mut pg = PostgreSQL::new(settings);
    pg.setup().await?;
    pg.start().await?;
    Ok(pg)
}

/// Crea (si hace falta) la base de datos de `company`, carga su esquema + seed
/// la primera vez, y devuelve el pool de conexión ya listo para usarse.
pub async fn load_company(pg: &PostgreSQL, company: Company) -> anyhow::Result<PgPool> {
    let db_name = company.db_name();
    let ya_existia = pg.database_exists(db_name).await?;
    if !ya_existia {
        pg.create_database(db_name).await?;
    }

    let url = pg.settings().url(db_name);
    let pool = PgPoolOptions::new().max_connections(5).connect(&url).await?;

    if !ya_existia {
        sqlx::raw_sql(company.schema_sql()).execute(&pool).await?;
        sqlx::raw_sql(company.seed_sql()).execute(&pool).await?;
    }

    Ok(pool)
}

/// Ejecuta SQL arbitrario escrito por el jugador. Alcance actual (Etapa 14):
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn ambas_empresas_conviven_en_el_mismo_servidor() {
        let pg = init_embedded_postgres().await.expect("Postgres embebido debe arrancar");

        let hospital = load_company(&pg, Company::HospitalArcangel)
            .await
            .expect("Hospital Arcángel debe cargar");
        let postafeta = load_company(&pg, Company::Postafeta)
            .await
            .expect("Postafeta debe cargar");

        let pacientes = run_query(&hospital, "SELECT * FROM pacientes").await.unwrap();
        assert_eq!(pacientes.rows.len(), 20);

        let paquetes = run_query(&postafeta, "SELECT * FROM paquetes").await.unwrap();
        assert_eq!(paquetes.rows.len(), 30);

        hospital.close().await;
        postafeta.close().await;
        pg.stop().await.expect("Postgres debe detenerse limpiamente");
    }
}
