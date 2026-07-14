mod hospital_arcangel;
mod postafeta;

use postgresql_embedded::{PostgreSQL, Settings};
use serde_json::{json, Map, Value};
use sqlparser::ast::{Expr, SelectItem, SetExpr, Statement};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};
use std::collections::HashMap;

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
    // Default del crate es 5s; en Windows el primer `initdb` suele tardar mucho
    // más (antivirus / cold start). 120s da margen sin dejar el arranque colgado.
    let mut settings = Settings::new();
    settings.timeout = Some(std::time::Duration::from_secs(120));
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
///
/// Con proyección explícita, `row_to_json(ROW(q.*))` + rename evita que dos
/// columnas `nombre` se pisen (como pasaba con `row_to_json(q)`).
/// Con `SELECT *` (u otra proyección que no parseamos), usamos `row_to_json(q)`
/// para conservar los nombres reales de Postgres en vez de `col_1`, `col_2`, …
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
                json!({ "QUERY PLAN": plan_line })
            })
            .collect()
    } else {
        let nombres = nombres_proyeccion_unicos(trimmed);
        let wrapped = if nombres.is_empty() {
            format!(
                "SELECT coalesce(json_agg(row_to_json(q)), '[]'::json) AS result FROM ({trimmed}) AS q"
            )
        } else {
            format!(
                "SELECT coalesce(json_agg(row_to_json(ROW(q.*))), '[]'::json) AS result FROM ({trimmed}) AS q"
            )
        };
        let row = sqlx::query(sqlx::AssertSqlSafe(wrapped))
            .fetch_one(pool)
            .await?;
        let value: Value = row.try_get(0)?;
        let filas = match value {
            Value::Array(items) => items,
            other => vec![other],
        };
        if nombres.is_empty() {
            filas
        } else {
            filas
                .into_iter()
                .map(|fila| renombrar_fila_anonima(fila, &nombres))
                .collect()
        }
    };

    Ok(QueryResult { rows })
}

fn nombre_de_expr(expr: &Expr) -> String {
    match expr {
        Expr::Identifier(ident) => ident.value.clone(),
        Expr::CompoundIdentifier(partes) => partes
            .last()
            .map(|p| p.value.clone())
            .unwrap_or_else(|| "col".to_string()),
        _ => "col".to_string(),
    }
}

/// Nombres visibles de la proyección del SELECT, uniquificados (`nombre`,
/// `nombre_2`, …) para no perder columnas homónimas al pintar/validar.
fn nombres_proyeccion_unicos(sql: &str) -> Vec<String> {
    let Ok(statements) = Parser::parse_sql(&PostgreSqlDialect {}, sql) else {
        return Vec::new();
    };
    let Some(Statement::Query(query)) = statements.into_iter().next() else {
        return Vec::new();
    };
    let SetExpr::Select(select) = *query.body else {
        return Vec::new();
    };

    let mut crudos = Vec::new();
    for item in &select.projection {
        match item {
            SelectItem::ExprWithAlias { alias, .. } => crudos.push(alias.value.clone()),
            SelectItem::UnnamedExpr(expr) => crudos.push(nombre_de_expr(expr)),
            // Wildcards / multi-alias: no conocemos un nombre estable aquí.
            SelectItem::Wildcard(_)
            | SelectItem::QualifiedWildcard(_, _)
            | SelectItem::ExprWithAliases { .. } => {
                return Vec::new();
            }
        }
    }

    uniquificar_nombres(crudos)
}

fn uniquificar_nombres(crudos: Vec<String>) -> Vec<String> {
    let mut vistos: HashMap<String, usize> = HashMap::new();
    let mut unicos = Vec::with_capacity(crudos.len());
    for nombre in crudos {
        let contador = vistos.entry(nombre.clone()).or_insert(0);
        *contador += 1;
        if *contador == 1 {
            unicos.push(nombre);
        } else {
            unicos.push(format!("{nombre}_{contador}"));
        }
    }
    unicos
}

fn renombrar_fila_anonima(fila: Value, nombres: &[String]) -> Value {
    let Value::Object(mapa) = fila else {
        return fila;
    };

    let mut ordenados: Vec<(usize, Value)> = mapa
        .into_iter()
        .filter_map(|(clave, valor)| {
            let numero = clave.strip_prefix('f')?.parse::<usize>().ok()?;
            Some((numero, valor))
        })
        .collect();
    ordenados.sort_by_key(|(numero, _)| *numero);

    let mut renombrada = Map::new();
    for (indice, (_, valor)) in ordenados.into_iter().enumerate() {
        let nombre = nombres
            .get(indice)
            .cloned()
            .unwrap_or_else(|| format!("col_{}", indice + 1));
        renombrada.insert(nombre, valor);
    }
    Value::Object(renombrada)
}

#[derive(serde::Serialize)]
pub struct ColumnaEsquema {
    pub nombre: String,
    pub tipo: String,
    pub nullable: bool,
    pub descripcion: Option<String>,
}

#[derive(serde::Serialize)]
pub struct TablaEsquema {
    pub nombre: String,
    pub descripcion: Option<String>,
    pub columnas: Vec<ColumnaEsquema>,
}

#[derive(serde::Serialize)]
pub struct RelacionEsquema {
    pub tabla_origen: String,
    pub columna_origen: String,
    pub tabla_destino: String,
    pub columna_destino: String,
}

#[derive(serde::Serialize)]
pub struct EsquemaView {
    pub tablas: Vec<TablaEsquema>,
    pub relaciones: Vec<RelacionEsquema>,
}

/// Introspección en vivo (Etapa 16/Plan 14): tablas, columnas y relaciones
/// reales de la base de datos activa, incluyendo los comentarios humanos que
/// cada esquema ya trae vía `COMMENT ON TABLE`/`COMMENT ON COLUMN` — no se
/// inventa ni duplica ningún texto, se lee directo de Postgres.
pub async fn obtener_esquema(pool: &PgPool) -> anyhow::Result<EsquemaView> {
    let filas_tablas = sqlx::query(
        "SELECT c.relname AS tabla, obj_description(c.oid, 'pg_class') AS descripcion
         FROM pg_class c
         JOIN pg_namespace n ON n.oid = c.relnamespace
         WHERE c.relkind = 'r' AND n.nspname = 'public'
         ORDER BY c.relname",
    )
    .fetch_all(pool)
    .await?;

    let mut tablas: Vec<TablaEsquema> = Vec::new();
    for fila in &filas_tablas {
        let nombre: String = fila.try_get("tabla")?;
        let descripcion: Option<String> = fila.try_get("descripcion")?;
        tablas.push(TablaEsquema { nombre, descripcion, columnas: Vec::new() });
    }

    let filas_columnas = sqlx::query(
        "SELECT c.relname AS tabla, a.attname AS columna,
                format_type(a.atttypid, a.atttypmod) AS tipo,
                NOT a.attnotnull AS nullable,
                col_description(c.oid, a.attnum) AS descripcion
         FROM pg_attribute a
         JOIN pg_class c ON c.oid = a.attrelid
         JOIN pg_namespace n ON n.oid = c.relnamespace
         WHERE c.relkind = 'r' AND n.nspname = 'public' AND a.attnum > 0 AND NOT a.attisdropped
         ORDER BY c.relname, a.attnum",
    )
    .fetch_all(pool)
    .await?;

    for fila in &filas_columnas {
        let tabla: String = fila.try_get("tabla")?;
        let columna = ColumnaEsquema {
            nombre: fila.try_get("columna")?,
            tipo: fila.try_get("tipo")?,
            nullable: fila.try_get("nullable")?,
            descripcion: fila.try_get("descripcion")?,
        };
        if let Some(t) = tablas.iter_mut().find(|t| t.nombre == tabla) {
            t.columnas.push(columna);
        }
    }

    let filas_relaciones = sqlx::query(
        "SELECT tc.table_name AS tabla_origen, kcu.column_name AS columna_origen,
                ccu.table_name AS tabla_destino, ccu.column_name AS columna_destino
         FROM information_schema.table_constraints tc
         JOIN information_schema.key_column_usage kcu
             ON tc.constraint_name = kcu.constraint_name AND tc.table_schema = kcu.table_schema
         JOIN information_schema.constraint_column_usage ccu
             ON tc.constraint_name = ccu.constraint_name AND tc.table_schema = ccu.table_schema
         WHERE tc.constraint_type = 'FOREIGN KEY' AND tc.table_schema = 'public'
         ORDER BY tc.table_name, kcu.column_name",
    )
    .fetch_all(pool)
    .await?;

    let mut relaciones: Vec<RelacionEsquema> = Vec::new();
    for fila in &filas_relaciones {
        relaciones.push(RelacionEsquema {
            tabla_origen: fila.try_get("tabla_origen")?,
            columna_origen: fila.try_get("columna_origen")?,
            tabla_destino: fila.try_get("tabla_destino")?,
            columna_destino: fila.try_get("columna_destino")?,
        });
    }

    Ok(EsquemaView { tablas, relaciones })
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

    #[tokio::test]
    async fn obtener_esquema_lee_tablas_columnas_y_relaciones_reales() {
        let pg = init_embedded_postgres().await.expect("Postgres embebido debe arrancar");
        let pool = load_company(&pg, Company::HospitalArcangel)
            .await
            .expect("Hospital Arcángel debe cargar");

        let esquema = obtener_esquema(&pool).await.expect("la introspección debe funcionar");

        assert_eq!(esquema.tablas.len(), 6, "Hospital Arcángel tiene 6 tablas");

        let pacientes = esquema
            .tablas
            .iter()
            .find(|t| t.nombre == "pacientes")
            .expect("la tabla pacientes debe aparecer");
        assert_eq!(
            pacientes.descripcion.as_deref(),
            Some("Historial de admisiones. fecha_alta queda NULL mientras el paciente sigue internado.")
        );

        let columna_diagnostico = pacientes
            .columnas
            .iter()
            .find(|c| c.nombre == "diagnostico")
            .expect("la columna diagnostico debe aparecer");
        assert_eq!(columna_diagnostico.tipo, "text");
        assert!(!columna_diagnostico.nullable);
        assert_eq!(
            columna_diagnostico.descripcion.as_deref(),
            Some("Motivo de ingreso redactado por el residente de guardia, casi siempre a las 3am.")
        );

        let columna_fecha_alta = pacientes
            .columnas
            .iter()
            .find(|c| c.nombre == "fecha_alta")
            .expect("la columna fecha_alta debe aparecer");
        assert!(columna_fecha_alta.nullable, "fecha_alta no tiene NOT NULL en el esquema");

        let relacion_paciente_departamento = esquema.relaciones.iter().any(|r| {
            r.tabla_origen == "pacientes"
                && r.columna_origen == "departamento_id"
                && r.tabla_destino == "departamentos"
                && r.columna_destino == "id"
        });
        assert!(relacion_paciente_departamento, "pacientes.departamento_id debe referenciar departamentos.id");

        pool.close().await;
        pg.stop().await.expect("Postgres debe detenerse limpiamente");
    }

    #[tokio::test]
    async fn obtener_esquema_soporta_multiples_fks_en_una_tabla() {
        let pg = init_embedded_postgres().await.expect("Postgres embebido debe arrancar");
        let pool = load_company(&pg, Company::Postafeta)
            .await
            .expect("Postafeta debe cargar");

        let esquema = obtener_esquema(&pool).await.expect("la introspección debe funcionar");

        assert_eq!(esquema.tablas.len(), 5, "Postafeta tiene 5 tablas");

        let relaciones_paquetes: Vec<_> = esquema.relaciones.iter().filter(|r| r.tabla_origen == "paquetes").collect();
        assert_eq!(relaciones_paquetes.len(), 4, "paquetes referencia clientes, sucursales (x2) y empleados");

        let hacia_clientes = relaciones_paquetes
            .iter()
            .any(|r| r.columna_origen == "cliente_id" && r.tabla_destino == "clientes");
        assert!(hacia_clientes);

        pool.close().await;
        pg.stop().await.expect("Postgres debe detenerse limpiamente");
    }

    #[test]
    fn uniquificar_nombres_distingue_columnas_homonimas() {
        assert_eq!(
            uniquificar_nombres(vec!["nombre".into(), "nombre".into(), "id".into()]),
            vec!["nombre", "nombre_2", "id"]
        );
    }

    #[tokio::test]
    async fn run_query_conserva_dos_columnas_llamadas_nombre() {
        let pg = init_embedded_postgres().await.expect("Postgres embebido debe arrancar");
        let pool = load_company(&pg, Company::HospitalArcangel)
            .await
            .expect("Hospital Arcángel debe cargar");

        let resultado = run_query(
            &pool,
            "SELECT pa.nombre, de.nombre FROM pacientes pa \
             INNER JOIN departamentos de ON de.id = pa.departamento_id \
             WHERE seguro_id = 5",
        )
        .await
        .expect("la query debe ejecutarse");

        assert!(!resultado.rows.is_empty());
        let primera = resultado.rows[0].as_object().expect("cada fila es un objeto");
        assert!(primera.contains_key("nombre"), "debe conservar el nombre del paciente");
        assert!(
            primera.contains_key("nombre_2"),
            "la segunda columna nombre no debe pisar a la primera"
        );
        assert_eq!(primera.len(), 2);

        pool.close().await;
        pg.stop().await.expect("Postgres debe detenerse limpiamente");
    }

    #[tokio::test]
    async fn run_query_select_estrella_conserva_nombres_reales() {
        let pg = init_embedded_postgres().await.expect("Postgres embebido debe arrancar");
        let pool = load_company(&pg, Company::HospitalArcangel)
            .await
            .expect("Hospital Arcángel debe cargar");

        let resultado = run_query(&pool, "SELECT * FROM habitaciones WHERE ocupada = false")
            .await
            .expect("la query debe ejecutarse");

        assert!(!resultado.rows.is_empty());
        let primera = resultado.rows[0].as_object().expect("cada fila es un objeto");
        assert!(
            primera.contains_key("numero") && primera.contains_key("tipo"),
            "SELECT * debe mostrar nombres reales, no col_N; claves={:?}",
            primera.keys().collect::<Vec<_>>()
        );
        assert!(
            !primera.keys().any(|k| k.starts_with("col_")),
            "no debe caer en col_1/col_2"
        );

        pool.close().await;
        pg.stop().await.expect("Postgres debe detenerse limpiamente");
    }
}
