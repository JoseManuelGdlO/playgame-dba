# Fase 0 / Plan 1: Esquema y Datos (Hospital Arcángel + Postafeta) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reemplazar el esquema de una sola empresa del spike técnico por la capa de datos real de las 2 empresas del MVP (Etapa 19): Hospital Arcángel expandido a sus 6 tablas canónicas (Etapa 16) y Postafeta construido desde cero, cada una en su propia base de datos dentro del mismo servidor Postgres embebido.

**Architecture:** `app/src-tauri/src/db.rs` (un solo archivo, una sola empresa hardcodeada) se convierte en el módulo `app/src-tauri/src/db/` con un submódulo por empresa (`hospital_arcangel.rs`, `postafeta.rs`) que expone su propio `SCHEMA_SQL`/`SEED_SQL`, y un `mod.rs` con la infraestructura común (arranque de Postgres embebido, carga de una empresa en su propia base de datos, ejecución de queries arbitrarias). Un enum `Company` selecciona qué base de datos cargar — así el resto del código (Tauri commands, futuro motor de tickets) nunca necesita conocer detalles de una empresa específica.

**Tech Stack:** Rust, sqlx 0.9 (postgres), postgresql_embedded 0.20.4, tokio (async test runtime) — mismo stack que el spike, sin dependencias nuevas.

## Global Constraints

- Las 2 empresas del MVP son Hospital Arcángel y Postafeta, nada más (Etapa 19).
- Arco de rango cubierto: Becario → Auxiliar de Sistemas únicamente (Etapa 19); los tickets de este rango solo necesitan `SELECT/WHERE/ORDER BY/JOIN/COUNT/SUM/AVG/GROUP BY` (Etapa 10) — el esquema no necesita soportar nada más avanzado todavía.
- Tamaño de esquema por empresa: ~5-8 tablas, 1-2 saltos de join típicos (Etapa 15, franja Becario/Auxiliar).
- Datos limpios y nombres de columna consistentes — nada de "suciedad" de datos legado; eso es exclusivo de empresas tardías (Etapa 16).
- Cada tabla debe tener comentarios (`COMMENT ON TABLE`/`COMMENT ON COLUMN`) con sabor narrativo, porque alimentan directamente el visor ERD (Etapa 16, Etapa 7).
- Los datos son fijos ("congelados"), no se generan aleatoriamente en runtime (Etapa 16) — se insertan como SQL literal.
- Alcance de lectura únicamente: nada de escritura de datos/DDL en el gameplay (Etapa 14) — este plan solo construye datos, no toca el motor de tickets/scoring (eso es el Plan 2).

---

## File Structure

- Delete: `app/src-tauri/src/db.rs` (se reemplaza por el directorio de abajo)
- Create: `app/src-tauri/src/db/mod.rs` — infraestructura común: arranque de Postgres embebido, enum `Company`, `load_company()`, `run_query()`, `QueryResult`
- Create: `app/src-tauri/src/db/hospital_arcangel.rs` — `DB_NAME`, `SCHEMA_SQL`, `SEED_SQL`, `TICKET_ENUNCIADO`, `TICKET_SOLUCION`, `run_ticket_solution()`, tests de esta empresa
- Create: `app/src-tauri/src/db/postafeta.rs` — `DB_NAME`, `SCHEMA_SQL`, `SEED_SQL`, tests de esta empresa
- Modify: `app/src-tauri/src/lib.rs` — adaptar la llamada de arranque a la nueva API de dos pasos (`init_embedded_postgres()` + `load_company()`)

---

### Task 1: Mover el spike a la estructura de módulo `db/` (refactor puro, sin cambio de comportamiento)

**Files:**
- Delete: `app/src-tauri/src/db.rs`
- Create: `app/src-tauri/src/db/mod.rs`
- Create: `app/src-tauri/src/db/hospital_arcangel.rs`

**Interfaces:**
- Produces: `db::init_embedded_postgres() -> anyhow::Result<(PostgreSQL, PgPool)>`, `db::run_query(pool: &PgPool, sql: &str) -> anyhow::Result<QueryResult>`, `db::run_ticket_solution(pool: &PgPool) -> anyhow::Result<QueryResult>`, `db::TICKET_ENUNCIADO: &str`, `db::QueryResult { rows: Vec<Value> }` — mismas firmas que el `db.rs` actual, solo reubicadas. `lib.rs` no necesita cambios en esta tarea.

- [ ] **Step 1: Crear `app/src-tauri/src/db/hospital_arcangel.rs` con el contenido actual del spike, renombrando la constante de base de datos**

```rust
pub(crate) const DB_NAME: &str = "query_path_hospital_arcangel";

/// Walking skeleton del esquema de Hospital Arcángel (Etapa 16): 3 tablas,
/// suficientes para probar JOIN, agregación, window functions y CTE recursivo
/// (jerarquía jefe_id) contra un Postgres real.
pub(crate) const SCHEMA_SQL: &str = r#"
CREATE TABLE departamentos (
    id SERIAL PRIMARY KEY,
    nombre TEXT NOT NULL
);

CREATE TABLE empleados (
    id SERIAL PRIMARY KEY,
    nombre TEXT NOT NULL,
    puesto TEXT NOT NULL,
    departamento_id INTEGER NOT NULL REFERENCES departamentos(id),
    jefe_id INTEGER REFERENCES empleados(id),
    salario NUMERIC(10, 2) NOT NULL,
    fecha_contratacion DATE NOT NULL
);

CREATE TABLE pacientes (
    id SERIAL PRIMARY KEY,
    nombre TEXT NOT NULL,
    departamento_id INTEGER NOT NULL REFERENCES departamentos(id),
    fecha_admision DATE NOT NULL,
    motivo TEXT NOT NULL
);
"#;

pub(crate) const SEED_SQL: &str = r#"
INSERT INTO departamentos (id, nombre) VALUES
    (1, 'Cardiología'),
    (2, 'Urgencias'),
    (3, 'Pediatría'),
    (4, 'Dirección General');

INSERT INTO empleados (id, nombre, puesto, departamento_id, jefe_id, salario, fecha_contratacion) VALUES
    (1, 'Dra. Ibarra', 'Directora General', 4, NULL, 95000, '2015-01-10'),
    (2, 'Dr. Salcedo', 'Jefe de Cardiología', 1, 1, 72000, '2017-03-01'),
    (3, 'Dra. Nuño', 'Jefa de Urgencias', 2, 1, 70000, '2018-06-15'),
    (4, 'Dr. Peralta', 'Cardiólogo', 1, 2, 58000, '2019-09-01'),
    (5, 'Dra. Cetina', 'Cardióloga', 1, 2, 61000, '2020-02-20'),
    (6, 'Enf. Rico', 'Enfermero de Urgencias', 2, 3, 32000, '2021-05-11');

INSERT INTO pacientes (id, nombre, departamento_id, fecha_admision, motivo) VALUES
    (1, 'Juan Pérez', 1, '2026-07-01', 'Palpitaciones tras maratón de la serie contable'),
    (2, 'Marta Solís', 1, '2026-07-05', 'Arritmia post junta de las 7am'),
    (3, 'Luis Vega', 1, '2026-06-20', 'Chequeo de rutina, insiste que está "bien"'),
    (4, 'Carla Ríos', 2, '2026-07-02', 'Torcedura de tobillo corriendo a imprimir algo');

SELECT setval('empleados_id_seq', (SELECT max(id) FROM empleados));
SELECT setval('pacientes_id_seq', (SELECT max(id) FROM pacientes));
SELECT setval('departamentos_id_seq', (SELECT max(id) FROM departamentos));
"#;

/// El único ticket del walking skeleton (Etapa 14): rango Becario,
/// solo SELECT/WHERE/ORDER BY (Etapa 10).
pub const TICKET_ENUNCIADO: &str = "Motivo: Contabilidad quiere saber quién ha pisado Cardiología últimamente.\nSolicitud: lista los pacientes admitidos en Cardiología (nombre, fecha de admisión y motivo), del más reciente al más antiguo.";

pub(crate) const TICKET_SOLUCION: &str =
    "SELECT nombre, fecha_admision, motivo FROM pacientes WHERE departamento_id = 1 ORDER BY fecha_admision DESC";

#[cfg(test)]
mod tests {
    use super::super::*;

    /// Prueba de punta a punta del stack (Etapa 18/22): arranca Postgres
    /// embebido, y ejecuta window function, CTE recursivo y EXPLAIN reales —
    /// justo lo que SQLite no puede hacer y por lo que se eligió este stack.
    #[tokio::test]
    async fn walking_skeleton_end_to_end() {
        let (pg, pool) = init_embedded_postgres()
            .await
            .expect("Postgres embebido debe arrancar");

        let ranking = run_query(
            &pool,
            "SELECT nombre, salario, RANK() OVER (PARTITION BY departamento_id ORDER BY salario DESC) AS puesto \
             FROM empleados WHERE departamento_id = 1",
        )
        .await
        .expect("window function debe ejecutar");
        assert_eq!(ranking.rows.len(), 3);

        let cadena = run_query(
            &pool,
            "WITH RECURSIVE cadena AS ( \
                SELECT id, nombre, jefe_id, 1 AS nivel FROM empleados WHERE id = 4 \
                UNION ALL \
                SELECT e.id, e.nombre, e.jefe_id, c.nivel + 1 FROM empleados e JOIN cadena c ON e.id = c.jefe_id \
             ) SELECT nombre, nivel FROM cadena ORDER BY nivel",
        )
        .await
        .expect("CTE recursiva debe ejecutar");
        assert_eq!(cadena.rows.len(), 3, "Dr. Peralta -> Dr. Salcedo -> Dra. Ibarra");

        let plan = run_query(&pool, "EXPLAIN SELECT * FROM pacientes")
            .await
            .expect("EXPLAIN debe ejecutar");
        assert!(!plan.rows.is_empty());

        let jugador = run_query(&pool, TICKET_SOLUCION).await.unwrap();
        let esperado = run_ticket_solution(&pool).await.unwrap();
        assert_eq!(jugador.rows, esperado.rows, "la solución del ticket debe pasar contra sí misma");

        pool.close().await;
        pg.stop().await.expect("Postgres debe detenerse limpiamente");
    }
}
```

- [ ] **Step 2: Crear `app/src-tauri/src/db/mod.rs` con la infraestructura común, delegando a `hospital_arcangel`**

```rust
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
```

- [ ] **Step 3: Borrar el archivo viejo**

```bash
rm app/src-tauri/src/db.rs
```

- [ ] **Step 4: Verificar que compila y el test pasa exactamente igual que antes**

Run: `cd app/src-tauri && cargo test --lib -- --nocapture`
Expected: `test db::hospital_arcangel::tests::walking_skeleton_end_to_end ... ok` — 1 passed; 0 failed.

- [ ] **Step 5: Commit**

```bash
git add app/src-tauri/src/db.rs app/src-tauri/src/db/mod.rs app/src-tauri/src/db/hospital_arcangel.rs
git commit -m "Refactor db.rs into db/ module, one file per company"
```

---

### Task 2: Expandir Hospital Arcángel a sus 6 tablas canónicas (Etapa 16)

**Files:**
- Modify: `app/src-tauri/src/db/hospital_arcangel.rs` (reemplaza `SCHEMA_SQL`, `SEED_SQL`, `TICKET_ENUNCIADO`, `TICKET_SOLUCION` completos; ajusta y añade tests)

**Interfaces:**
- Consumes: `super::super::{init_embedded_postgres, run_query, run_ticket_solution, QueryResult}` (sin cambios desde Task 1)
- Produces: mismo `TICKET_ENUNCIADO`/`TICKET_SOLUCION` pero con columnas renombradas (`fecha_ingreso`, `diagnostico`) — ningún consumidor externo depende todavía de los nombres viejos, solo los tests de este archivo.

- [ ] **Step 1: Reemplazar `SCHEMA_SQL` por las 6 tablas de la Etapa 16, con comentarios de sabor para el ERD**

```rust
pub(crate) const SCHEMA_SQL: &str = r#"
CREATE TABLE departamentos (
    id SERIAL PRIMARY KEY,
    nombre TEXT NOT NULL,
    piso INTEGER NOT NULL,
    jefe_id INTEGER
);

CREATE TABLE empleados (
    id SERIAL PRIMARY KEY,
    nombre TEXT NOT NULL,
    puesto TEXT NOT NULL,
    departamento_id INTEGER NOT NULL REFERENCES departamentos(id),
    jefe_id INTEGER REFERENCES empleados(id),
    salario NUMERIC(10, 2) NOT NULL,
    fecha_contratacion DATE NOT NULL
);

ALTER TABLE departamentos
    ADD CONSTRAINT departamentos_jefe_id_fkey FOREIGN KEY (jefe_id) REFERENCES empleados(id);

CREATE TABLE seguros (
    id SERIAL PRIMARY KEY,
    aseguradora TEXT NOT NULL,
    cobertura_pct NUMERIC(5, 2) NOT NULL
);

CREATE TABLE pacientes (
    id SERIAL PRIMARY KEY,
    nombre TEXT NOT NULL,
    fecha_nacimiento DATE NOT NULL,
    genero TEXT NOT NULL,
    fecha_ingreso DATE NOT NULL,
    fecha_alta DATE,
    departamento_id INTEGER NOT NULL REFERENCES departamentos(id),
    diagnostico TEXT NOT NULL,
    seguro_id INTEGER REFERENCES seguros(id)
);

CREATE TABLE tratamientos (
    id SERIAL PRIMARY KEY,
    paciente_id INTEGER NOT NULL REFERENCES pacientes(id),
    tipo TEXT NOT NULL,
    fecha DATE NOT NULL,
    costo NUMERIC(10, 2) NOT NULL,
    empleado_id INTEGER NOT NULL REFERENCES empleados(id)
);

CREATE TABLE habitaciones (
    id SERIAL PRIMARY KEY,
    numero INTEGER NOT NULL,
    departamento_id INTEGER NOT NULL REFERENCES departamentos(id),
    tipo TEXT NOT NULL,
    ocupada BOOLEAN NOT NULL DEFAULT false
);

COMMENT ON TABLE departamentos IS 'Las 4 áreas de Hospital Arcángel. Dirección General también cuenta como "departamento" para efectos de nómina, aunque nadie ahí haya visto a un paciente jamás.';
COMMENT ON COLUMN departamentos.jefe_id IS 'Responsable del área ante Dirección General. Puede o no coincidir con el jefe directo de cada empleado.';
COMMENT ON TABLE empleados IS 'Personal médico y administrativo.';
COMMENT ON COLUMN empleados.jefe_id IS 'A quién le reporta este empleado en la cadena de mando real.';
COMMENT ON TABLE seguros IS 'Aseguradoras con convenio. La cobertura real casi nunca coincide con lo que promete el folleto.';
COMMENT ON TABLE pacientes IS 'Historial de admisiones. fecha_alta queda NULL mientras el paciente sigue internado.';
COMMENT ON COLUMN pacientes.diagnostico IS 'Motivo de ingreso redactado por el residente de guardia, casi siempre a las 3am.';
COMMENT ON TABLE tratamientos IS 'Procedimientos/servicios aplicados a cada paciente, uno por fila.';
COMMENT ON TABLE habitaciones IS 'Inventario físico de camas por departamento.';
COMMENT ON COLUMN habitaciones.ocupada IS 'Se actualiza a mano por el personal de piso — a veces con un día de retraso.';
"#;
```

- [ ] **Step 2: Reemplazar `SEED_SQL` por el dataset completo de las 6 tablas**

```rust
pub(crate) const SEED_SQL: &str = r#"
INSERT INTO departamentos (id, nombre, piso) VALUES
    (1, 'Cardiología', 3),
    (2, 'Urgencias', 1),
    (3, 'Pediatría', 2),
    (4, 'Dirección General', 5);

INSERT INTO empleados (id, nombre, puesto, departamento_id, jefe_id, salario, fecha_contratacion) VALUES
    (1, 'Dra. Ibarra', 'Directora General', 4, NULL, 95000, '2015-01-10'),
    (2, 'Dr. Salcedo', 'Jefe de Cardiología', 1, 1, 72000, '2017-03-01'),
    (3, 'Dra. Nuño', 'Jefa de Urgencias', 2, 1, 70000, '2018-06-15'),
    (4, 'Dr. Peralta', 'Cardiólogo', 1, 2, 58000, '2019-09-01'),
    (5, 'Dra. Cetina', 'Cardióloga', 1, 2, 61000, '2020-02-20'),
    (6, 'Enf. Rico', 'Enfermero de Urgencias', 2, 3, 32000, '2021-05-11'),
    (7, 'Dra. Montes', 'Jefa de Pediatría', 3, 1, 69000, '2016-11-02'),
    (8, 'Dr. Zavala', 'Pediatra', 3, 7, 55000, '2019-04-18'),
    (9, 'Enf. Paredes', 'Enfermera de Pediatría', 3, 7, 31000, '2022-01-09'),
    (10, 'Enf. Cordero', 'Enfermero de Urgencias', 2, 3, 33000, '2020-08-30'),
    (11, 'Dr. Junco', 'Cardiólogo', 1, 2, 60000, '2021-07-14'),
    (12, 'Aux. Reyes', 'Auxiliar Administrativo', 4, 1, 28000, '2023-02-01');

UPDATE departamentos SET jefe_id = 2 WHERE id = 1;
UPDATE departamentos SET jefe_id = 3 WHERE id = 2;
UPDATE departamentos SET jefe_id = 7 WHERE id = 3;
UPDATE departamentos SET jefe_id = 1 WHERE id = 4;

INSERT INTO seguros (id, aseguradora, cobertura_pct) VALUES
    (1, 'MetLife Salud', 80.00),
    (2, 'GNP Vital', 70.00),
    (3, 'AXA Bienestar', 90.00),
    (4, 'Seguro Popular Plus', 50.00),
    (5, 'Sin seguro', 0.00);

INSERT INTO pacientes (id, nombre, fecha_nacimiento, genero, fecha_ingreso, fecha_alta, departamento_id, diagnostico, seguro_id) VALUES
    (1, 'Juan Pérez', '1978-04-12', 'M', '2026-07-01', NULL, 1, 'Palpitaciones tras maratón de la serie contable', 1),
    (2, 'Marta Solís', '1985-11-03', 'F', '2026-07-05', NULL, 1, 'Arritmia post junta de las 7am', 2),
    (3, 'Luis Vega', '1990-02-27', 'M', '2026-06-20', '2026-06-22', 1, 'Chequeo de rutina, insiste que está "bien"', 3),
    (4, 'Carla Ríos', '1999-08-15', 'F', '2026-07-02', '2026-07-03', 2, 'Torcedura de tobillo corriendo a imprimir algo', 1),
    (5, 'Pedro Salas', '1965-01-30', 'M', '2026-06-15', '2026-06-25', 1, 'Cirugía de bypass programada', 3),
    (6, 'Ana Beltrán', '1972-09-09', 'F', '2026-07-08', NULL, 1, 'Dolor torácico tras revisar el estado de cuenta', 2),
    (7, 'Diego Colín', '2015-03-21', 'M', '2026-07-04', '2026-07-05', 3, 'Fiebre alta y berrinche simultáneo', 4),
    (8, 'Sofía Lerma', '2018-06-11', 'F', '2026-07-06', NULL, 3, 'Varicela, contagiada en la guardería', 4),
    (9, 'Emiliano Roa', '2012-12-01', 'M', '2026-06-28', '2026-06-30', 3, 'Fractura de brazo, columpio del patio', 1),
    (10, 'Renata Ibáñez', '2020-05-05', 'F', '2026-07-09', NULL, 3, 'Tos persistente hace tres semanas', 5),
    (11, 'Héctor Camacho', '1955-07-19', 'M', '2026-06-10', '2026-06-14', 1, 'Marcapasos, revisión de rutina', 3),
    (12, 'Gabriela Ponce', '1988-03-03', 'F', '2026-07-10', NULL, 2, 'Corte profundo abriendo una caja de reportes', 2),
    (13, 'Ricardo Fuentes', '1993-10-22', 'M', '2026-07-07', '2026-07-07', 2, 'Reacción alérgica, café de la oficina en mal estado', 1),
    (14, 'Valeria Nuñez', '1980-01-17', 'F', '2026-06-25', '2026-06-27', 1, 'Hipertensión descontrolada, cierre trimestral', 2),
    (15, 'Óscar Beltrán', '2010-11-28', 'M', '2026-07-03', NULL, 3, 'Dolor de oído, natación escolar', 4),
    (16, 'Fernanda Ozuna', '1996-04-04', 'F', '2026-06-18', '2026-06-19', 2, 'Esguince de muñeca, tropezón en la sala de juntas', 5),
    (17, 'Tomás Rangel', '1948-08-08', 'M', '2026-06-05', '2026-06-20', 1, 'Insuficiencia cardiaca, seguimiento prolongado', 3),
    (18, 'Ximena Ledesma', '2005-02-14', 'F', '2026-07-11', NULL, 2, 'Quemadura leve, experimento de café con soplete', 1),
    (19, 'Adrián Cuevas', '1970-06-06', 'M', '2026-06-22', '2026-06-23', 1, 'Chequeo de rutina, obligado por Recursos Humanos', 2),
    (20, 'Paula Montaño', '2017-09-27', 'F', '2026-07-08', NULL, 3, 'Erupción cutánea, alergia a detergente nuevo', 5);

INSERT INTO tratamientos (id, paciente_id, tipo, fecha, costo, empleado_id) VALUES
    (1, 1, 'Electrocardiograma', '2026-07-01', 1200.00, 4),
    (2, 1, 'Consulta', '2026-07-02', 800.00, 2),
    (3, 2, 'Electrocardiograma', '2026-07-05', 1200.00, 5),
    (4, 2, 'Análisis de sangre', '2026-07-05', 450.00, 11),
    (5, 3, 'Consulta', '2026-06-20', 800.00, 2),
    (6, 4, 'Radiografía', '2026-07-02', 950.00, 6),
    (7, 4, 'Sutura', '2026-07-02', 600.00, 6),
    (8, 5, 'Cirugía', '2026-06-16', 45000.00, 2),
    (9, 5, 'Consulta', '2026-06-24', 800.00, 4),
    (10, 6, 'Electrocardiograma', '2026-07-08', 1200.00, 11),
    (11, 7, 'Consulta', '2026-07-04', 700.00, 8),
    (12, 7, 'Nebulización', '2026-07-04', 350.00, 9),
    (13, 8, 'Consulta', '2026-07-06', 700.00, 7),
    (14, 9, 'Radiografía', '2026-06-28', 950.00, 6),
    (15, 9, 'Consulta', '2026-06-28', 700.00, 7),
    (16, 10, 'Consulta', '2026-07-09', 700.00, 8),
    (17, 11, 'Electrocardiograma', '2026-06-11', 1200.00, 5),
    (18, 11, 'Consulta', '2026-06-12', 800.00, 2),
    (19, 12, 'Sutura', '2026-07-10', 600.00, 6),
    (20, 13, 'Consulta', '2026-07-07', 700.00, 10),
    (21, 14, 'Electrocardiograma', '2026-06-25', 1200.00, 4),
    (22, 14, 'Análisis de sangre', '2026-06-26', 450.00, 11),
    (23, 15, 'Consulta', '2026-07-03', 700.00, 7),
    (24, 16, 'Radiografía', '2026-06-18', 950.00, 6),
    (25, 17, 'Electrocardiograma', '2026-06-06', 1200.00, 2),
    (26, 17, 'Consulta', '2026-06-12', 800.00, 5),
    (27, 17, 'Terapia', '2026-06-18', 1500.00, 11),
    (28, 18, 'Consulta', '2026-07-11', 700.00, 10),
    (29, 19, 'Consulta', '2026-06-22', 800.00, 4),
    (30, 20, 'Consulta', '2026-07-08', 700.00, 7),
    (31, 3, 'Análisis de sangre', '2026-06-21', 450.00, 11),
    (32, 6, 'Consulta', '2026-07-08', 800.00, 2),
    (33, 8, 'Vacuna', '2026-07-06', 300.00, 9),
    (34, 10, 'Nebulización', '2026-07-09', 350.00, 9),
    (35, 20, 'Vacuna', '2026-07-08', 300.00, 9);

INSERT INTO habitaciones (id, numero, departamento_id, tipo, ocupada) VALUES
    (1, 101, 1, 'Individual', true),
    (2, 102, 1, 'Individual', true),
    (3, 103, 1, 'UCI', true),
    (4, 104, 1, 'UCI', false),
    (5, 105, 1, 'Compartida', false),
    (6, 201, 2, 'Individual', true),
    (7, 202, 2, 'Compartida', true),
    (8, 203, 2, 'Compartida', false),
    (9, 204, 2, 'UCI', true),
    (10, 301, 3, 'Individual', true),
    (11, 302, 3, 'Compartida', true),
    (12, 303, 3, 'Compartida', false),
    (13, 304, 3, 'Individual', false),
    (14, 305, 3, 'UCI', false);

SELECT setval('departamentos_id_seq', (SELECT max(id) FROM departamentos));
SELECT setval('empleados_id_seq', (SELECT max(id) FROM empleados));
SELECT setval('seguros_id_seq', (SELECT max(id) FROM seguros));
SELECT setval('pacientes_id_seq', (SELECT max(id) FROM pacientes));
SELECT setval('tratamientos_id_seq', (SELECT max(id) FROM tratamientos));
SELECT setval('habitaciones_id_seq', (SELECT max(id) FROM habitaciones));
"#;
```

- [ ] **Step 3: Actualizar el ticket para usar los nombres de columna nuevos**

```rust
pub const TICKET_ENUNCIADO: &str = "Motivo: Contabilidad quiere saber quién ha pisado Cardiología últimamente.\nSolicitud: lista los pacientes admitidos en Cardiología (nombre, fecha de ingreso y diagnóstico), del más reciente al más antiguo.";

pub(crate) const TICKET_SOLUCION: &str =
    "SELECT nombre, fecha_ingreso, diagnostico FROM pacientes WHERE departamento_id = 1 ORDER BY fecha_ingreso DESC";
```

- [ ] **Step 4: Ajustar la aserción del test existente (Cardiología ahora tiene 4 empleados, no 3) y correr para confirmar que falla antes del fix**

En `walking_skeleton_end_to_end`, cambiar:

```rust
        assert_eq!(ranking.rows.len(), 3);
```

por:

```rust
        assert_eq!(ranking.rows.len(), 4, "Salcedo, Peralta, Cetina y Junco están en Cardiología");
```

Run: `cd app/src-tauri && cargo test --lib walking_skeleton_end_to_end -- --nocapture`
Expected (antes de aplicar Steps 1-3): FAIL — la aserción vieja (3) no coincide con las 4 filas que produce el nuevo seed.

- [ ] **Step 5: Aplicar los Steps 1-3 (ya mostrados arriba) y correr de nuevo para confirmar que pasa**

Run: `cd app/src-tauri && cargo test --lib walking_skeleton_end_to_end -- --nocapture`
Expected: `test db::hospital_arcangel::tests::walking_skeleton_end_to_end ... ok`

- [ ] **Step 6: Escribir los tests nuevos para las 3 tablas agregadas**

Agregar al mismo `mod tests` de `hospital_arcangel.rs`:

```rust
    #[tokio::test]
    async fn reporte_costos_por_departamento() {
        let (pg, pool) = init_embedded_postgres()
            .await
            .expect("Postgres embebido debe arrancar");

        let resultado = run_query(
            &pool,
            "SELECT d.nombre, COUNT(t.id) AS total_tratamientos, SUM(t.costo) AS costo_total \
             FROM tratamientos t \
             JOIN pacientes p ON p.id = t.paciente_id \
             JOIN departamentos d ON d.id = p.departamento_id \
             GROUP BY d.nombre \
             ORDER BY costo_total DESC",
        )
        .await
        .expect("el reporte por departamento debe ejecutar");

        assert_eq!(resultado.rows.len(), 3, "pacientes solo existen en 3 de los 4 departamentos");

        pool.close().await;
        pg.stop().await.expect("Postgres debe detenerse limpiamente");
    }

    #[tokio::test]
    async fn habitaciones_y_seguros_cargan_correctamente() {
        let (pg, pool) = init_embedded_postgres()
            .await
            .expect("Postgres embebido debe arrancar");

        let habitaciones = run_query(&pool, "SELECT * FROM habitaciones").await.unwrap();
        assert_eq!(habitaciones.rows.len(), 14);

        let seguros = run_query(&pool, "SELECT * FROM seguros").await.unwrap();
        assert_eq!(seguros.rows.len(), 5);

        let pacientes_sin_seguro = run_query(
            &pool,
            "SELECT p.nombre FROM pacientes p JOIN seguros s ON s.id = p.seguro_id WHERE s.aseguradora = 'Sin seguro'",
        )
        .await
        .unwrap();
        assert!(!pacientes_sin_seguro.rows.is_empty());

        pool.close().await;
        pg.stop().await.expect("Postgres debe detenerse limpiamente");
    }
```

- [ ] **Step 7: Correr toda la suite de este módulo y confirmar que los 3 tests pasan**

Run: `cd app/src-tauri && cargo test --lib db::hospital_arcangel -- --nocapture`
Expected: 3 passed; 0 failed.

- [ ] **Step 8: Commit**

```bash
git add app/src-tauri/src/db/hospital_arcangel.rs
git commit -m "Expand Hospital Arcángel to its 6-table schema from Stage 16"
```

---

### Task 3: Construir Postafeta y generalizar la carga a múltiples empresas

**Files:**
- Create: `app/src-tauri/src/db/postafeta.rs`
- Modify: `app/src-tauri/src/db/mod.rs` (agrega `Company` enum y `load_company()`, cambia la firma de `init_embedded_postgres()`)

**Interfaces:**
- Consumes: `hospital_arcangel::{DB_NAME, SCHEMA_SQL, SEED_SQL}` (sin cambios de Task 2), mismo shape para `postafeta`
- Produces: `db::Company` (enum público con variantes `HospitalArcangel`, `Postafeta`), `db::init_embedded_postgres() -> anyhow::Result<PostgreSQL>` (**cambia** — ya no crea la base ni devuelve el pool), `db::load_company(pg: &PostgreSQL, company: Company) -> anyhow::Result<PgPool>` (nuevo). Este cambio de firma rompe a los tests de `hospital_arcangel.rs` (Task 1/2) y a `lib.rs` — se corrigen en los Steps 4 y en la Task 4 respectivamente.

- [ ] **Step 1: Crear `app/src-tauri/src/db/postafeta.rs` con su esquema, datos y comentarios**

```rust
pub(crate) const DB_NAME: &str = "query_path_postafeta";

pub(crate) const SCHEMA_SQL: &str = r#"
CREATE TABLE sucursales (
    id SERIAL PRIMARY KEY,
    nombre TEXT NOT NULL,
    ciudad TEXT NOT NULL,
    direccion TEXT NOT NULL
);

CREATE TABLE empleados (
    id SERIAL PRIMARY KEY,
    nombre TEXT NOT NULL,
    puesto TEXT NOT NULL,
    sucursal_id INTEGER NOT NULL REFERENCES sucursales(id),
    fecha_contratacion DATE NOT NULL,
    salario NUMERIC(10, 2) NOT NULL
);

CREATE TABLE clientes (
    id SERIAL PRIMARY KEY,
    nombre TEXT NOT NULL,
    telefono TEXT NOT NULL,
    ciudad TEXT NOT NULL
);

CREATE TABLE paquetes (
    id SERIAL PRIMARY KEY,
    cliente_id INTEGER NOT NULL REFERENCES clientes(id),
    sucursal_origen_id INTEGER NOT NULL REFERENCES sucursales(id),
    sucursal_destino_id INTEGER NOT NULL REFERENCES sucursales(id),
    repartidor_id INTEGER REFERENCES empleados(id),
    peso_kg NUMERIC(6, 2) NOT NULL,
    fecha_envio DATE NOT NULL,
    fecha_entrega DATE,
    estado TEXT NOT NULL,
    costo_envio NUMERIC(10, 2) NOT NULL
);

CREATE TABLE incidencias (
    id SERIAL PRIMARY KEY,
    paquete_id INTEGER NOT NULL REFERENCES paquetes(id),
    tipo TEXT NOT NULL,
    fecha DATE NOT NULL,
    descripcion TEXT NOT NULL,
    resuelta BOOLEAN NOT NULL DEFAULT false
);

COMMENT ON TABLE sucursales IS 'Puntos de la red Postafeta. Todas reportan a la matriz, que a su vez reporta a Kevin.';
COMMENT ON TABLE empleados IS 'Personal de sucursal: mostradores, gerentes y repartidores.';
COMMENT ON TABLE clientes IS 'Quien manda o recibe un paquete.';
COMMENT ON TABLE paquetes IS 'Un envío de punta a punta.';
COMMENT ON COLUMN paquetes.estado IS 'en_transito, entregado, perdido o devuelto. Kevin lo actualiza a mano desde el Slack — a veces un día tarde.';
COMMENT ON TABLE incidencias IS 'Todo reporte de pérdida o daño pasa primero por Kevin, quien lo documenta y firma "- Kevin" antes de escalarlo.';
"#;

pub(crate) const SEED_SQL: &str = r#"
INSERT INTO sucursales (id, nombre, ciudad, direccion) VALUES
    (1, 'Postafeta Centro', 'Ciudad de México', 'Av. Juárez 120'),
    (2, 'Postafeta Norte', 'Monterrey', 'Av. Constitución 45'),
    (3, 'Postafeta Bajío', 'Guadalajara', 'Av. Vallarta 900'),
    (4, 'Postafeta Golfo', 'Veracruz', 'Blvd. Ávila Camacho 33'),
    (5, 'Postafeta Sureste', 'Mérida', 'Calle 60 #210');

INSERT INTO empleados (id, nombre, puesto, sucursal_id, fecha_contratacion, salario) VALUES
    (1, 'Kevin Marín', 'Becario de Sistemas', 1, '2024-01-08', 12000),
    (2, 'Rosa Elena Tapia', 'Gerente de Sucursal', 1, '2018-03-01', 38000),
    (3, 'Iván Zamudio', 'Repartidor', 1, '2021-05-19', 22000),
    (4, 'Lourdes Aguirre', 'Mostrador', 1, '2020-09-12', 20000),
    (5, 'Marco Nieto', 'Gerente de Sucursal', 2, '2017-11-03', 37000),
    (6, 'Selene Cabrera', 'Repartidor', 2, '2022-02-14', 22000),
    (7, 'Ulises Prado', 'Repartidor', 2, '2022-08-30', 22500),
    (8, 'Karina Ochoa', 'Gerente de Sucursal', 3, '2019-06-21', 36000),
    (9, 'Benjamín Solano', 'Repartidor', 3, '2021-01-11', 21500),
    (10, 'Fabiola Rentería', 'Mostrador', 3, '2023-03-05', 19500),
    (11, 'Gustavo Ibáñez', 'Gerente de Sucursal', 4, '2020-04-17', 35000),
    (12, 'Norma Villaseñor', 'Repartidor', 5, '2021-10-02', 22000);

INSERT INTO clientes (id, nombre, telefono, ciudad) VALUES
    (1, 'Comercial Rovira SA', '555-1023', 'Ciudad de México'),
    (2, 'Marisol Peña', '555-2091', 'Ciudad de México'),
    (3, 'Ferretería Dos Hermanos', '555-3312', 'Monterrey'),
    (4, 'Tomás Elizondo', '555-4420', 'Monterrey'),
    (5, 'Papelería La Central', '555-5108', 'Guadalajara'),
    (6, 'Andrea Bustos', '555-6675', 'Guadalajara'),
    (7, 'Refaccionaria López', '555-7743', 'Veracruz'),
    (8, 'Cecilia Marrufo', '555-8891', 'Veracruz'),
    (9, 'Distribuidora Kann', '555-9012', 'Mérida'),
    (10, 'Rodrigo Pat', '555-0143', 'Mérida'),
    (11, 'Boutique Alameda', '555-1256', 'Ciudad de México'),
    (12, 'Julián Cordero', '555-2367', 'Monterrey'),
    (13, 'Zapatería El Paso', '555-3478', 'Guadalajara'),
    (14, 'Nadia Treviño', '555-4589', 'Veracruz'),
    (15, 'Consultorio Dental Ek', '555-5690', 'Mérida');

INSERT INTO paquetes (id, cliente_id, sucursal_origen_id, sucursal_destino_id, repartidor_id, peso_kg, fecha_envio, fecha_entrega, estado, costo_envio) VALUES
    (1, 1, 1, 2, 3, 2.5, '2026-06-20', '2026-06-22', 'entregado', 180.00),
    (2, 2, 1, 3, 3, 1.0, '2026-06-25', '2026-06-26', 'entregado', 120.00),
    (3, 3, 2, 4, 6, 5.0, '2026-06-18', '2026-06-21', 'entregado', 260.00),
    (4, 4, 2, 1, 6, 0.8, '2026-07-01', NULL, 'en_transito', 110.00),
    (5, 5, 3, 5, 9, 3.2, '2026-06-15', '2026-06-17', 'entregado', 190.00),
    (6, 6, 3, 2, 9, 1.5, '2026-06-30', NULL, 'en_transito', 150.00),
    (7, 7, 4, 4, NULL, 4.0, '2026-06-10', NULL, 'perdido', 230.00),
    (8, 8, 4, 1, NULL, 2.0, '2026-06-05', '2026-06-09', 'entregado', 175.00),
    (9, 9, 5, 5, 12, 1.2, '2026-07-02', NULL, 'en_transito', 130.00),
    (10, 10, 5, 3, 12, 0.5, '2026-06-28', '2026-06-30', 'entregado', 95.00),
    (11, 11, 1, 2, 3, 6.0, '2026-06-12', '2026-06-15', 'entregado', 310.00),
    (12, 12, 2, 4, 7, 2.2, '2026-07-05', NULL, 'en_transito', 165.00),
    (13, 13, 3, 1, 9, 3.5, '2026-06-22', NULL, 'perdido', 200.00),
    (14, 14, 4, 2, NULL, 1.8, '2026-06-08', '2026-06-11', 'entregado', 150.00),
    (15, 15, 5, 4, 12, 0.9, '2026-06-27', '2026-06-29', 'entregado', 110.00),
    (16, 1, 1, 3, 3, 2.0, '2026-07-08', NULL, 'en_transito', 170.00),
    (17, 2, 1, 4, 3, 1.4, '2026-06-14', '2026-06-16', 'entregado', 140.00),
    (18, 3, 2, 5, 6, 4.5, '2026-06-19', '2026-06-23', 'entregado', 255.00),
    (19, 4, 2, 2, 7, 0.6, '2026-07-10', NULL, 'en_transito', 105.00),
    (20, 5, 3, 1, 9, 2.8, '2026-06-24', '2026-06-26', 'devuelto', 185.00),
    (21, 6, 3, 4, 9, 3.0, '2026-06-29', NULL, 'en_transito', 195.00),
    (22, 7, 4, 5, NULL, 5.5, '2026-06-13', '2026-06-18', 'entregado', 280.00),
    (23, 8, 4, 3, NULL, 1.1, '2026-07-03', NULL, 'perdido', 125.00),
    (24, 9, 5, 2, 12, 2.4, '2026-06-16', '2026-06-19', 'entregado', 175.00),
    (25, 10, 5, 1, 12, 0.7, '2026-07-06', NULL, 'en_transito', 115.00),
    (26, 11, 1, 5, 3, 1.9, '2026-06-21', '2026-06-24', 'entregado', 160.00),
    (27, 12, 2, 3, 7, 3.3, '2026-06-26', NULL, 'devuelto', 210.00),
    (28, 13, 3, 2, 9, 2.6, '2026-07-04', NULL, 'en_transito', 180.00),
    (29, 14, 4, 1, NULL, 0.4, '2026-06-17', '2026-06-19', 'entregado', 90.00),
    (30, 15, 5, 3, 12, 1.6, '2026-06-11', '2026-06-13', 'entregado', 145.00);

INSERT INTO incidencias (id, paquete_id, tipo, fecha, descripcion, resuelta) VALUES
    (1, 7, 'perdida', '2026-06-11', 'Paquete no localizado en bodega de Veracruz tras el corte de inventario mensual.', false),
    (2, 13, 'perdida', '2026-06-23', 'Escaneo de salida existe, pero el paquete nunca llegó a Guadalajara.', false),
    (3, 23, 'perdida', '2026-07-04', 'Repartidor reporta que la caja "se veía sospechosamente ligera" al recogerla.', false),
    (4, 20, 'devolucion', '2026-06-25', 'Cliente rechazó el paquete por etiqueta de destino ilegible.', true),
    (5, 27, 'devolucion', '2026-06-27', 'Dirección de entrega no existe según el repartidor; Kevin confirma que el CP estaba mal capturado.', true),
    (6, 3, 'daño', '2026-06-21', 'Caja llegó con la esquina aplastada; cliente aceptó de todas formas.', true),
    (7, 18, 'daño', '2026-06-23', 'Producto frágil sin la etiqueta correspondiente, daño menor reportado por el cliente.', true),
    (8, 7, 'retraso', '2026-06-14', 'Paquete "perdido" reapareció 4 días tarde en la sucursal equivocada, antes de perderse definitivamente otra vez.', false);

SELECT setval('sucursales_id_seq', (SELECT max(id) FROM sucursales));
SELECT setval('empleados_id_seq', (SELECT max(id) FROM empleados));
SELECT setval('clientes_id_seq', (SELECT max(id) FROM clientes));
SELECT setval('paquetes_id_seq', (SELECT max(id) FROM paquetes));
SELECT setval('incidencias_id_seq', (SELECT max(id) FROM incidencias));
"#;

#[cfg(test)]
mod tests {
    use super::super::*;

    #[tokio::test]
    async fn postafeta_carga_y_reporta_estado_de_envios() {
        let pg = init_embedded_postgres().await.expect("Postgres embebido debe arrancar");
        let pool = load_company(&pg, Company::Postafeta).await.expect("Postafeta debe cargar");

        let paquetes = run_query(&pool, "SELECT * FROM paquetes").await.unwrap();
        assert_eq!(paquetes.rows.len(), 30);

        let por_estado = run_query(
            &pool,
            "SELECT estado, COUNT(*) AS total FROM paquetes GROUP BY estado ORDER BY estado",
        )
        .await
        .expect("agrupar por estado debe ejecutar");
        assert_eq!(por_estado.rows.len(), 4, "entregado, en_transito, perdido, devuelto");

        let reporte_perdidos = run_query(
            &pool,
            "SELECT p.id, c.nombre, i.descripcion \
             FROM paquetes p \
             JOIN clientes c ON c.id = p.cliente_id \
             JOIN incidencias i ON i.paquete_id = p.id \
             WHERE p.estado = 'perdido'",
        )
        .await
        .expect("el join de 3 tablas debe ejecutar");
        assert!(!reporte_perdidos.rows.is_empty());

        pool.close().await;
        pg.stop().await.expect("Postgres debe detenerse limpiamente");
    }
}
```

- [ ] **Step 2: Reemplazar `app/src-tauri/src/db/mod.rs` completo con la versión multi-empresa**

```rust
mod hospital_arcangel;
mod postafeta;

pub use hospital_arcangel::TICKET_ENUNCIADO;

use postgresql_embedded::{PostgreSQL, Settings};
use serde_json::Value;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};

/// Empresa activa: cada una vive en su propia base de datos dentro del mismo
/// servidor Postgres embebido (Etapa 11-G: el esquema cambia por completo al
/// cambiar de empresa; el progreso de rango/perks vive fuera de este módulo).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
```

- [ ] **Step 3: Correr y confirmar que el módulo nuevo compila y el test de convivencia pasa**

Run: `cd app/src-tauri && cargo test --lib db::tests -- --nocapture`
Expected: `test db::tests::ambas_empresas_conviven_en_el_mismo_servidor ... ok`

- [ ] **Step 4: Adaptar los tests de `hospital_arcangel.rs` (Task 1/2) a la nueva API de dos pasos**

En `app/src-tauri/src/db/hospital_arcangel.rs`, dentro de `mod tests`, reemplazar en cada uno de los 3 tests la línea:

```rust
        let (pg, pool) = init_embedded_postgres()
            .await
            .expect("Postgres embebido debe arrancar");
```

por:

```rust
        let pg = init_embedded_postgres().await.expect("Postgres embebido debe arrancar");
        let pool = load_company(&pg, Company::HospitalArcangel)
            .await
            .expect("Hospital Arcángel debe cargar");
```

- [ ] **Step 5: Correr toda la suite de `db` y confirmar que los 5 tests pasan (3 de Hospital Arcángel + 1 de Postafeta + 1 de convivencia)**

Run: `cd app/src-tauri && cargo test --lib db:: -- --nocapture`
Expected: 5 passed; 0 failed.

Nota: este comando no incluye todavía `lib.rs`, que sigue sin compilar hasta la Task 4 (usa la firma vieja de `init_embedded_postgres`). Es esperado — `cargo test --lib db::` compila y prueba solo el módulo `db`; `cargo build`/`cargo test --lib` sin filtro fallará hasta la Task 4.

- [ ] **Step 6: Commit**

```bash
git add app/src-tauri/src/db/mod.rs app/src-tauri/src/db/postafeta.rs app/src-tauri/src/db/hospital_arcangel.rs
git commit -m "Add Postafeta and generalize db module to support multiple companies"
```

---

### Task 4: Conectar la app Tauri a la nueva API y verificar regresión completa

**Files:**
- Modify: `app/src-tauri/src/lib.rs:83-92` (bloque `setup` que arranca Postgres)

**Interfaces:**
- Consumes: `db::init_embedded_postgres() -> anyhow::Result<PostgreSQL>`, `db::load_company(&PostgreSQL, db::Company) -> anyhow::Result<PgPool>` (de Task 3)
- Produces: ninguna interfaz nueva — la app sigue exponiendo los mismos comandos Tauri (`ticket_actual`, `run_query`, `submit_ticket`, `unlock_perk`) sin cambios, ahora corriendo contra Hospital Arcángel vía la nueva API.

- [ ] **Step 1: Actualizar el bloque `setup` en `lib.rs`**

Localizar en `app/src-tauri/src/lib.rs`:

```rust
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                let (pg, pool) = db::init_embedded_postgres()
                    .await
                    .expect("no se pudo inicializar Postgres embebido — spike fallido");
                handle.manage(AppState { pool });
                handle.manage(Perk(Mutex::new(PerkState::default())));
                handle.manage(EmbeddedPostgres(Mutex::new(Some(pg))));
            });
            Ok(())
        })
```

Reemplazar por:

```rust
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                let pg = db::init_embedded_postgres()
                    .await
                    .expect("no se pudo inicializar Postgres embebido");
                let pool = db::load_company(&pg, db::Company::HospitalArcangel)
                    .await
                    .expect("no se pudo cargar Hospital Arcángel");
                handle.manage(AppState { pool });
                handle.manage(Perk(Mutex::new(PerkState::default())));
                handle.manage(EmbeddedPostgres(Mutex::new(Some(pg))));
            });
            Ok(())
        })
```

- [ ] **Step 2: Verificar que todo el crate compila**

Run: `cd app/src-tauri && cargo check`
Expected: `Finished` sin errores.

- [ ] **Step 3: Correr la suite completa de tests**

Run: `cd app/src-tauri && cargo test --lib -- --nocapture`
Expected: 5 passed; 0 failed (los 3 de `hospital_arcangel`, el de `postafeta`, el de convivencia en `db::tests`).

- [ ] **Step 4: Smoke test de la app real**

Run: `cd app && npm run tauri dev`
Expected: la ventana abre, el ticket de Hospital Arcángel se muestra en `#ticket-enunciado` con el texto actualizado ("fecha de ingreso y diagnóstico"), `SELECT * FROM pacientes;` en la consola devuelve 20 filas. Cerrar la ventana para terminar el proceso.

- [ ] **Step 5: Commit**

```bash
git add app/src-tauri/src/lib.rs
git commit -m "Wire Tauri app to the multi-company db API"
```

---

## Self-Review Notes

- **Cobertura del spec:** Etapa 19 (2 empresas: Hospital Arcángel ✓ Postafeta ✓) — Etapa 16 (6 tablas Hospital Arcángel ✓, datos limpios ✓, comentarios ERD ✓, "congelados" como SQL literal ✓) — Etapa 15 (tamaño 5-8 tablas/1-2 joins: Hospital 6 tablas ✓, Postafeta 5 tablas ✓) — Etapa 11-G (cada empresa en su propia base de datos, lista para cambiar de "esquema y elenco" sin tocar progreso ✓, aunque el progreso de rango/perks en sí es Plan 4/5, fuera de este plan).
- **Fuera de alcance deliberado (para planes siguientes):** motor de validación de tickets (Plan 2), generación de tickets por plantilla (Plan 3), economía/RPG (Plan 4/5) — este plan solo entrega datos, ningún ticket nuevo más allá de ajustar el texto del ya existente para que compile contra el esquema nuevo.
- **Consistencia de tipos:** `Company` se usa igual en `mod.rs` (definición), `hospital_arcangel.rs`/`postafeta.rs` (tests) y `lib.rs` (`db::Company::HospitalArcangel`) — mismo path calificado en los tres lugares.

---

## Execution Handoff

Plan completo y guardado en `docs/superpowers/plans/2026-07-12-fase0-01-esquema-datos.md`. Dos opciones de ejecución:

1. **Subagent-Driven (recomendado)** — despacho un subagente fresco por tarea, reviso entre tareas, iteración rápida
2. **Ejecución inline** — ejecuto las tareas en esta sesión con executing-plans, ejecución por lotes con checkpoints

¿Cuál prefieres?
