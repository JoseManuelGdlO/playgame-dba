use sqlx::{PgPool, Row};

/// Costo estimado (Etapa 17-B) del plan de ejecución de `sql`, extraído de
/// `EXPLAIN (FORMAT JSON)`. Determinista y reproducible: depende solo de las
/// estadísticas del planificador sobre el dataset congelado (Etapa 16), nunca
/// del reloj real ni del hardware.
pub async fn costo_del_plan(pool: &PgPool, sql: &str) -> anyhow::Result<f64> {
    let trimmed = sql.trim().trim_end_matches(';');
    let explain_sql = format!("EXPLAIN (FORMAT JSON) {trimmed}");
    let fila = sqlx::query(sqlx::AssertSqlSafe(explain_sql))
        .fetch_one(pool)
        .await?;
    let plan: serde_json::Value = fila.try_get(0)?;
    plan[0]["Plan"]["Total Cost"]
        .as_f64()
        .ok_or_else(|| anyhow::anyhow!("no se pudo leer el costo total del plan de EXPLAIN"))
}

/// Puntaje 0-100 (Etapa 17-B): 100 si el plan del jugador cuesta igual o
/// menos que el de la solución dorada; baja proporcionalmente si cuesta más.
pub fn puntaje(costo_dorado: f64, costo_jugador: f64) -> f64 {
    if costo_jugador <= 0.0 {
        return 100.0;
    }
    ((costo_dorado / costo_jugador) * 100.0).clamp(0.0, 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{self, Company};

    #[tokio::test]
    async fn costo_del_plan_devuelve_un_numero_positivo() {
        let pg = db::init_embedded_postgres()
            .await
            .expect("Postgres embebido debe arrancar");
        let pool = db::load_company(&pg, Company::HospitalArcangel)
            .await
            .expect("Hospital Arcángel debe cargar");

        let costo = costo_del_plan(&pool, "SELECT * FROM pacientes WHERE departamento_id = 1")
            .await
            .expect("EXPLAIN debe ejecutar");
        assert!(costo > 0.0);

        pool.close().await;
        pg.stop().await.expect("Postgres debe detenerse limpiamente");
    }

    #[test]
    fn puntaje_100_si_cuesta_igual_o_menos() {
        assert_eq!(puntaje(10.0, 10.0), 100.0);
        assert_eq!(puntaje(10.0, 5.0), 100.0);
    }

    #[test]
    fn puntaje_baja_proporcionalmente_si_cuesta_mas() {
        assert_eq!(puntaje(10.0, 20.0), 50.0);
    }

    #[test]
    fn puntaje_100_si_costo_jugador_es_cero() {
        assert_eq!(puntaje(10.0, 0.0), 100.0);
    }
}
