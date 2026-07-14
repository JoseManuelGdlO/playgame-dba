mod correctness;
mod mentor;
mod practices;
mod speed;

use sqlx::PgPool;

use crate::db;

/// Resultado de evaluar una entrega (Etapa 17): los 3 componentes en bruto,
/// 0-100 cada uno. Combinarlos con los pesos propios de cada ticket (Etapa
/// 14) y los multiplicadores de perks (Etapa 12/13) es responsabilidad de un
/// plan posterior — este módulo solo produce las 3 mediciones deterministas.
#[derive(Debug, Clone, PartialEq)]
pub struct Evaluacion {
    pub correcta: bool,
    pub puntaje_correctitud: f64,
    pub puntaje_velocidad: f64,
    pub puntaje_practicas: f64,
    pub comentario_mentor: Option<&'static str>,
}

/// Evalúa la query del jugador contra la query dorada del ticket (Etapa 17).
/// `requiere_orden` viene de la metadata del ticket (si la solicitud pidió un
/// orden específico, Etapa 14) — hasta que exista el motor de tickets (plan
/// siguiente), quien llama a esta función decide ese valor directamente.
pub async fn evaluar_entrega(
    pool: &PgPool,
    sql_jugador: &str,
    sql_dorada: &str,
    requiere_orden: bool,
) -> anyhow::Result<Evaluacion> {
    let resultado_jugador = db::run_query(pool, sql_jugador).await?;
    let resultado_dorado = db::run_query(pool, sql_dorada).await?;

    let correcta = correctness::son_equivalentes(
        &resultado_dorado.rows,
        &resultado_jugador.rows,
        requiere_orden,
    );
    let puntaje_correctitud =
        correctness::puntaje(&resultado_dorado.rows, &resultado_jugador.rows, requiere_orden);

    let costo_dorado = speed::costo_del_plan(pool, sql_dorada).await?;
    let costo_jugador = speed::costo_del_plan(pool, sql_jugador).await?;
    let puntaje_velocidad = speed::puntaje(costo_dorado, costo_jugador);

    let violaciones = practices::analizar(sql_jugador);
    let puntaje_practicas = practices::puntaje(&violaciones);

    let comentario_mentor = mentor::comentario(correcta, &violaciones, puntaje_velocidad);

    Ok(Evaluacion {
        correcta,
        puntaje_correctitud,
        puntaje_velocidad,
        puntaje_practicas,
        comentario_mentor,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{self, Company};

    const SQL_DORADA: &str =
        "SELECT nombre, fecha_ingreso, diagnostico FROM pacientes WHERE departamento_id = 1 ORDER BY fecha_ingreso DESC";

    #[tokio::test]
    async fn evaluacion_de_query_correcta_y_limpia() {
        let pg = db::init_embedded_postgres()
            .await
            .expect("Postgres embebido debe arrancar");
        let pool = db::load_company(&pg, Company::HospitalArcangel)
            .await
            .expect("Hospital Arcángel debe cargar");

        let evaluacion = evaluar_entrega(&pool, SQL_DORADA, SQL_DORADA, true)
            .await
            .expect("la evaluación debe ejecutar");

        assert!(evaluacion.correcta);
        assert_eq!(evaluacion.puntaje_correctitud, 100.0);
        assert_eq!(evaluacion.puntaje_practicas, 100.0);
        assert_eq!(evaluacion.puntaje_velocidad, 100.0, "misma query -> mismo costo de plan");
        assert_eq!(evaluacion.comentario_mentor, None);

        pool.close().await;
        pg.stop().await.expect("Postgres debe detenerse limpiamente");
    }

    #[tokio::test]
    async fn evaluacion_de_query_con_resultado_incorrecto() {
        let pg = db::init_embedded_postgres()
            .await
            .expect("Postgres embebido debe arrancar");
        let pool = db::load_company(&pg, Company::HospitalArcangel)
            .await
            .expect("Hospital Arcángel debe cargar");

        let sql_jugador =
            "SELECT nombre, fecha_ingreso, diagnostico FROM pacientes WHERE departamento_id = 2 ORDER BY fecha_ingreso DESC";
        let evaluacion = evaluar_entrega(&pool, sql_jugador, SQL_DORADA, true)
            .await
            .expect("la evaluación debe ejecutar");

        assert!(!evaluacion.correcta);
        assert_eq!(evaluacion.puntaje_correctitud, 0.0);
        assert_eq!(
            evaluacion.comentario_mentor, None,
            "el Mentor nunca debe sugerir que el resultado fue correcto en una entrega fallida"
        );

        pool.close().await;
        pg.stop().await.expect("Postgres debe detenerse limpiamente");
    }

    #[tokio::test]
    async fn evaluacion_de_query_correcta_pero_con_malas_practicas() {
        let pg = db::init_embedded_postgres()
            .await
            .expect("Postgres embebido debe arrancar");
        let pool = db::load_company(&pg, Company::HospitalArcangel)
            .await
            .expect("Hospital Arcángel debe cargar");

        let sql_jugador = "SELECT pacientes.nombre, fecha_ingreso, diagnostico FROM pacientes, departamentos \
             WHERE pacientes.departamento_id = departamentos.id AND departamentos.id = 1 \
             ORDER BY fecha_ingreso DESC";
        let evaluacion = evaluar_entrega(&pool, sql_jugador, SQL_DORADA, true)
            .await
            .expect("la evaluación debe ejecutar");

        assert!(evaluacion.correcta, "el resultado es el mismo aunque la query esté mal escrita");
        assert_eq!(evaluacion.puntaje_correctitud, 100.0);
        assert_eq!(evaluacion.puntaje_practicas, 40.0, "2 reglas violadas: comma-join y sin alias");
        assert!(evaluacion.comentario_mentor.unwrap().contains("JOIN"));

        pool.close().await;
        pg.stop().await.expect("Postgres debe detenerse limpiamente");
    }
}
