# Fase 0 / Plan 2: Motor de Validación SQL Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reemplazar la comparación naive `jugador.rows == esperado.rows` del spike por el motor de validación real de la Etapa 17: tres mediciones deterministas (correctitud por conjunto de resultados, velocidad por costo de plan `EXPLAIN`, buenas prácticas por un linter estático sobre el AST de la query), más El Mentor con comentarios pre-escritos atados a esas mediciones.

**Architecture:** Un nuevo módulo `app/src-tauri/src/validation/` con un archivo por componente (`correctness.rs`, `speed.rs`, `practices.rs`, `mentor.rs`) y un `mod.rs` que los combina en una única función `evaluar_entrega()`. Cada componente es independiente y puro donde es posible (practices/correctness/mentor no tocan la base de datos; solo `speed` necesita el pool para correr `EXPLAIN`). `db::run_query` (ya existente) sigue siendo el único punto que ejecuta SQL de jugador contra Postgres.

**Tech Stack:** Rust, el crate `sqlparser` (parser SQL real → AST, dialecto Postgres) para el linter — única dependencia nueva. Reutiliza sqlx/postgresql_embedded ya presentes.

## Global Constraints

- Correctitud: se compara el resultado como conjunto de filas (nunca el texto de la query); solo se compara como secuencia ordenada si el ticket pidió explícitamente un orden. Se tolera que los nombres de columna/alias difieran si los valores coinciden en posición y tipo. Se tolera redondeo menor en decimales (Etapa 17-A).
- Velocidad: se mide con el costo estimado del plan de `EXPLAIN`, nunca con tiempo de reloj real — determinista y reproducible en cualquier máquina (Etapa 17-B).
- Buenas prácticas: un linter estático sobre la estructura (AST) de la query, no sobre su texto literal. Produce un puntaje 0-100, nunca un pase/falla binario (Etapa 17-C).
- Alcance del linter en este plan: solo los conceptos SQL de Becario/Auxiliar de Sistemas (`SELECT/WHERE/ORDER BY/JOIN/COUNT/SUM/AVG/GROUP BY`, Etapa 10/19) — nada de reglas sobre subconsultas, CTEs o window functions todavía.
- El Mentor nunca llama a una IA en vivo; cada comentario está pre-escrito y atado a una regla específica del linter o a un patrón de costo de plan (Etapa 17-D).
- Este plan NO calcula dinero/reputación/XP, ni aplica los pesos de rúbrica por ticket, ni los multiplicadores de perks (Etapa 12/13/14) — solo produce las 3 mediciones crudas 0-100 más el comentario del Mentor. Combinar esas mediciones con pesos y multiplicadores es responsabilidad de un plan posterior.
- Alcance de lectura únicamente — nada de escritura de datos/DDL (Etapa 14); no se toca el esquema de ninguna empresa.

---

## File Structure

- Modify: `app/src-tauri/Cargo.toml` — agrega la dependencia `sqlparser`
- Create: `app/src-tauri/src/validation/mod.rs` — combina los 4 componentes en `evaluar_entrega()`; crece de forma incremental a lo largo de las tareas
- Create: `app/src-tauri/src/validation/practices.rs` — linter estático (Etapa 17-C)
- Create: `app/src-tauri/src/validation/speed.rs` — costo de plan `EXPLAIN` (Etapa 17-B)
- Create: `app/src-tauri/src/validation/correctness.rs` — comparación de resultados (Etapa 17-A)
- Create: `app/src-tauri/src/validation/mentor.rs` — comentarios pre-escritos (Etapa 17-D)
- Modify: `app/src-tauri/src/lib.rs` — agrega `mod validation;`, y en la última tarea rewira `submit_ticket` para usar `validation::evaluar_entrega`

---

### Task 1: Dependencia `sqlparser` + linter de buenas prácticas (Etapa 17-C)

**Files:**
- Modify: `app/src-tauri/Cargo.toml`
- Create: `app/src-tauri/src/validation/mod.rs`
- Create: `app/src-tauri/src/validation/practices.rs`
- Modify: `app/src-tauri/src/lib.rs` (agrega `mod validation;`)

**Interfaces:**
- Produces: `validation::practices::Regla` (enum público: `SelectStar`, `JoinSinCondicion`, `AliasFaltante`), `validation::practices::analizar(sql: &str) -> Vec<Regla>`, `validation::practices::puntaje(violaciones: &[Regla]) -> f64`

- [ ] **Step 1: Agregar la dependencia**

Run: `cd app/src-tauri && cargo add sqlparser@0.62`
Expected: `Cargo.toml` gana la línea `sqlparser = "0.62"` y `Cargo.lock` se actualiza.

- [ ] **Step 2: Escribir el linter con sus tests, en `app/src-tauri/src/validation/practices.rs`**

```rust
use sqlparser::ast::{JoinConstraint, JoinOperator, SelectItem, SetExpr, Statement, TableFactor};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;

/// Una regla de buenas prácticas violada (Etapa 17-C). Cada variante tiene un
/// comentario pre-escrito del Mentor asociado (Etapa 17-D, ver `mentor.rs`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Regla {
    /// `SELECT *` en vez de columnas explícitas.
    SelectStar,
    /// JOIN (o comma-join) sin condición de unión — riesgo de producto cartesiano.
    JoinSinCondicion,
    /// Dos o más tablas en la consulta, pero al menos una sin alias.
    AliasFaltante,
}

const PENALIZACION_POR_REGLA: f64 = 30.0;

fn agregar_regla(violaciones: &mut Vec<Regla>, regla: Regla) {
    if !violaciones.contains(&regla) {
        violaciones.push(regla);
    }
}

fn constraint_de(op: &JoinOperator) -> Option<&JoinConstraint> {
    match op {
        JoinOperator::Join(c)
        | JoinOperator::Inner(c)
        | JoinOperator::Left(c)
        | JoinOperator::LeftOuter(c)
        | JoinOperator::Right(c)
        | JoinOperator::RightOuter(c)
        | JoinOperator::FullOuter(c)
        | JoinOperator::CrossJoin(c) => Some(c),
        _ => None,
    }
}

fn sin_alias(factor: &TableFactor) -> bool {
    matches!(factor, TableFactor::Table { alias: None, .. })
}

/// Analiza el AST de la query del jugador y devuelve las reglas violadas.
/// Si la query no parsea como un SELECT simple, no reporta violaciones — la
/// responsabilidad de "SQL inválido" es de la ejecución (Etapa 17-A), no del
/// linter.
pub fn analizar(sql: &str) -> Vec<Regla> {
    let mut violaciones = Vec::new();

    let Ok(statements) = Parser::parse_sql(&PostgreSqlDialect {}, sql) else {
        return violaciones;
    };
    let Some(Statement::Query(query)) = statements.into_iter().next() else {
        return violaciones;
    };
    let SetExpr::Select(select) = *query.body else {
        return violaciones;
    };

    if select
        .projection
        .iter()
        .any(|item| matches!(item, SelectItem::Wildcard(_)))
    {
        agregar_regla(&mut violaciones, Regla::SelectStar);
    }

    let total_tablas: usize =
        select.from.len() + select.from.iter().map(|t| t.joins.len()).sum::<usize>();

    if select.from.len() > 1 {
        agregar_regla(&mut violaciones, Regla::JoinSinCondicion);
    }

    for tabla in &select.from {
        if total_tablas > 1 && sin_alias(&tabla.relation) {
            agregar_regla(&mut violaciones, Regla::AliasFaltante);
        }
        for join in &tabla.joins {
            if let Some(constraint) = constraint_de(&join.join_operator) {
                if matches!(constraint, JoinConstraint::None) {
                    agregar_regla(&mut violaciones, Regla::JoinSinCondicion);
                }
            }
            if total_tablas > 1 && sin_alias(&join.relation) {
                agregar_regla(&mut violaciones, Regla::AliasFaltante);
            }
        }
    }

    violaciones
}

/// Puntaje 0-100: 100 menos 30 por cada regla violada (piso en 0) — nunca
/// pase/falla binario (Etapa 17-C).
pub fn puntaje(violaciones: &[Regla]) -> f64 {
    (100.0 - PENALIZACION_POR_REGLA * violaciones.len() as f64).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detecta_select_star() {
        assert_eq!(analizar("SELECT * FROM pacientes"), vec![Regla::SelectStar]);
    }

    #[test]
    fn query_limpia_no_viola_nada() {
        assert_eq!(
            analizar("SELECT nombre FROM pacientes WHERE departamento_id = 1 ORDER BY fecha_ingreso DESC"),
            Vec::<Regla>::new()
        );
    }

    #[test]
    fn detecta_comma_join_sin_condicion() {
        assert_eq!(
            analizar(
                "SELECT p.nombre, d.nombre FROM pacientes p, departamentos d WHERE p.departamento_id = d.id"
            ),
            vec![Regla::JoinSinCondicion]
        );
    }

    #[test]
    fn detecta_join_sin_on() {
        assert_eq!(
            analizar("SELECT p.nombre FROM pacientes p JOIN departamentos d"),
            vec![Regla::JoinSinCondicion]
        );
    }

    #[test]
    fn detecta_alias_faltante() {
        assert_eq!(
            analizar(
                "SELECT p.nombre FROM pacientes p JOIN departamentos ON p.departamento_id = departamentos.id"
            ),
            vec![Regla::AliasFaltante]
        );
    }

    #[test]
    fn join_correcto_con_alias_no_viola_nada() {
        assert_eq!(
            analizar("SELECT p.nombre FROM pacientes p JOIN departamentos d ON p.departamento_id = d.id"),
            Vec::<Regla>::new()
        );
    }

    #[test]
    fn query_de_3_tablas_bien_escrita_no_viola_nada() {
        assert_eq!(
            analizar(
                "SELECT d.nombre, COUNT(t.id) FROM tratamientos t \
                 JOIN pacientes p ON p.id = t.paciente_id \
                 JOIN departamentos d ON d.id = p.departamento_id \
                 GROUP BY d.nombre"
            ),
            Vec::<Regla>::new()
        );
    }

    #[test]
    fn detecta_varias_violaciones_a_la_vez() {
        assert_eq!(
            analizar("SELECT * FROM pacientes p, departamentos"),
            vec![Regla::SelectStar, Regla::JoinSinCondicion, Regla::AliasFaltante]
        );
    }

    #[test]
    fn puntaje_baja_30_por_regla() {
        assert_eq!(puntaje(&[]), 100.0);
        assert_eq!(puntaje(&[Regla::SelectStar]), 70.0);
        assert_eq!(puntaje(&[Regla::SelectStar, Regla::AliasFaltante]), 40.0);
    }

    #[test]
    fn puntaje_nunca_baja_de_cero() {
        assert_eq!(
            puntaje(&[
                Regla::SelectStar,
                Regla::JoinSinCondicion,
                Regla::AliasFaltante,
                Regla::AliasFaltante
            ]),
            0.0
        );
    }
}
```

- [ ] **Step 3: Crear `app/src-tauri/src/validation/mod.rs` (versión mínima — solo declara el submódulo por ahora)**

```rust
mod practices;
```

- [ ] **Step 4: Registrar el módulo en `app/src-tauri/src/lib.rs`**

Localizar la línea `mod db;` cerca del inicio de `lib.rs` y agregar justo debajo:

```rust
mod db;
mod validation;
```

- [ ] **Step 5: Correr la suite completa y confirmar que los tests nuevos pasan junto con los ya existentes**

Run: `cd app/src-tauri && cargo test --lib -- --nocapture`
Expected: los 5 tests de `db` (sin cambios) más los 9 tests nuevos de `validation::practices::tests` — 14 passed; 0 failed.

- [ ] **Step 6: Commit**

```bash
git add app/src-tauri/Cargo.toml app/src-tauri/Cargo.lock app/src-tauri/src/validation/mod.rs app/src-tauri/src/validation/practices.rs app/src-tauri/src/lib.rs
git commit -m "Add SQL practices linter (Stage 17-C) via sqlparser AST analysis"
```

---

### Task 2: Costo de plan `EXPLAIN` (Etapa 17-B)

**Files:**
- Create: `app/src-tauri/src/validation/speed.rs`
- Modify: `app/src-tauri/src/validation/mod.rs` (agrega `mod speed;`)

**Interfaces:**
- Consumes: `crate::db::{init_embedded_postgres, load_company, Company}` (para el test de integración)
- Produces: `validation::speed::costo_del_plan(pool: &PgPool, sql: &str) -> anyhow::Result<f64>`, `validation::speed::puntaje(costo_dorado: f64, costo_jugador: f64) -> f64`

- [ ] **Step 1: Escribir `app/src-tauri/src/validation/speed.rs`**

```rust
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
```

- [ ] **Step 2: Actualizar `app/src-tauri/src/validation/mod.rs`**

```rust
mod practices;
mod speed;
```

- [ ] **Step 3: Correr la suite completa**

Run: `cd app/src-tauri && cargo test --lib -- --nocapture`
Expected: 14 tests previos + 4 nuevos de `validation::speed::tests` — 18 passed; 0 failed.

- [ ] **Step 4: Commit**

```bash
git add app/src-tauri/src/validation/speed.rs app/src-tauri/src/validation/mod.rs
git commit -m "Add EXPLAIN plan-cost measurement (Stage 17-B)"
```

---

### Task 3: Comparación de resultados (Etapa 17-A)

**Files:**
- Create: `app/src-tauri/src/validation/correctness.rs`
- Modify: `app/src-tauri/src/validation/mod.rs` (agrega `mod correctness;`)

**Interfaces:**
- Produces: `validation::correctness::son_equivalentes(doradas: &[serde_json::Value], jugador: &[serde_json::Value], requiere_orden: bool) -> bool`, `validation::correctness::puntaje(doradas: &[Value], jugador: &[Value], requiere_orden: bool) -> f64`

- [ ] **Step 1: Escribir `app/src-tauri/src/validation/correctness.rs`**

```rust
use serde_json::Value;

/// Convierte una fila (objeto JSON con columnas en el orden de la proyección)
/// a una representación normalizada por posición: se ignoran los nombres de
/// columna (Etapa 17-A: "se tolera nombres de columna/alias distintos si los
/// valores coinciden en posición y tipo") y los números se redondean a 2
/// decimales (Etapa 17-A: "se tolera redondeo menor en decimales").
fn normalizar_fila(fila: &Value) -> Vec<String> {
    match fila {
        Value::Object(mapa) => mapa.values().map(normalizar_valor).collect(),
        otro => vec![normalizar_valor(otro)],
    }
}

fn normalizar_valor(valor: &Value) -> String {
    match valor {
        Value::Number(n) => match n.as_f64() {
            Some(f) => format!("{:.2}", f),
            None => n.to_string(),
        },
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "NULL".to_string(),
        otro => otro.to_string(),
    }
}

/// Compara el resultado del jugador contra el resultado dorado (Etapa 17-A).
/// Si `requiere_orden` es true, las filas deben coincidir en el mismo orden
/// (la solicitud del ticket pidió un orden específico); si no, se comparan
/// como conjunto (multiset), ignorando el orden de filas.
pub fn son_equivalentes(doradas: &[Value], jugador: &[Value], requiere_orden: bool) -> bool {
    if doradas.len() != jugador.len() {
        return false;
    }

    let doradas: Vec<Vec<String>> = doradas.iter().map(normalizar_fila).collect();
    let jugador: Vec<Vec<String>> = jugador.iter().map(normalizar_fila).collect();

    if requiere_orden {
        return doradas == jugador;
    }

    let mut restantes = jugador.clone();
    for fila_dorada in &doradas {
        let Some(pos) = restantes.iter().position(|fila| fila == fila_dorada) else {
            return false;
        };
        restantes.remove(pos);
    }
    restantes.is_empty()
}

/// Puntaje binario (Etapa 17-A no describe crédito parcial): 100 si el
/// resultado del jugador es equivalente al dorado, 0 si no.
pub fn puntaje(doradas: &[Value], jugador: &[Value], requiere_orden: bool) -> f64 {
    if son_equivalentes(doradas, jugador, requiere_orden) {
        100.0
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn filas_identicas_son_equivalentes() {
        let doradas = vec![json!({"nombre": "Juan", "edad": 30})];
        let jugador = vec![json!({"nombre": "Juan", "edad": 30})];
        assert!(son_equivalentes(&doradas, &jugador, false));
    }

    #[test]
    fn nombres_de_columna_distintos_no_importan_si_los_valores_coinciden_en_posicion() {
        let doradas = vec![json!({"nombre": "Juan", "edad": 30})];
        let jugador = vec![json!({"nombre_paciente": "Juan", "anios": 30})];
        assert!(son_equivalentes(&doradas, &jugador, false));
    }

    #[test]
    fn tolera_redondeo_menor_en_decimales() {
        let doradas = vec![json!({"costo": 100.001})];
        let jugador = vec![json!({"costo": 100.0})];
        assert!(son_equivalentes(&doradas, &jugador, false));
    }

    #[test]
    fn sin_requerir_orden_las_filas_pueden_venir_en_otro_orden() {
        let doradas = vec![json!({"n": 1}), json!({"n": 2})];
        let jugador = vec![json!({"n": 2}), json!({"n": 1})];
        assert!(son_equivalentes(&doradas, &jugador, false));
    }

    #[test]
    fn requiriendo_orden_el_orden_distinto_falla() {
        let doradas = vec![json!({"n": 1}), json!({"n": 2})];
        let jugador = vec![json!({"n": 2}), json!({"n": 1})];
        assert!(!son_equivalentes(&doradas, &jugador, true));
    }

    #[test]
    fn valores_distintos_no_son_equivalentes() {
        let doradas = vec![json!({"n": 1})];
        let jugador = vec![json!({"n": 2})];
        assert!(!son_equivalentes(&doradas, &jugador, false));
    }

    #[test]
    fn cantidad_de_filas_distinta_no_es_equivalente() {
        let doradas = vec![json!({"n": 1}), json!({"n": 2})];
        let jugador = vec![json!({"n": 1})];
        assert!(!son_equivalentes(&doradas, &jugador, false));
    }

    #[test]
    fn puntaje_es_100_o_0() {
        let doradas = vec![json!({"n": 1})];
        assert_eq!(puntaje(&doradas, &doradas, false), 100.0);
        let jugador = vec![json!({"n": 2})];
        assert_eq!(puntaje(&doradas, &jugador, false), 0.0);
    }
}
```

- [ ] **Step 2: Actualizar `app/src-tauri/src/validation/mod.rs`**

```rust
mod correctness;
mod practices;
mod speed;
```

- [ ] **Step 3: Correr la suite completa**

Run: `cd app/src-tauri && cargo test --lib -- --nocapture`
Expected: 18 tests previos + 8 nuevos de `validation::correctness::tests` — 26 passed; 0 failed.

- [ ] **Step 4: Commit**

```bash
git add app/src-tauri/src/validation/correctness.rs app/src-tauri/src/validation/mod.rs
git commit -m "Add result-set correctness comparison (Stage 17-A)"
```

---

### Task 4: El Mentor (Etapa 17-D)

**Files:**
- Create: `app/src-tauri/src/validation/mentor.rs`
- Modify: `app/src-tauri/src/validation/mod.rs` (agrega `mod mentor;`)

**Interfaces:**
- Consumes: `super::practices::Regla` (de la Tarea 1)
- Produces: `validation::mentor::comentario(violaciones: &[practices::Regla], puntaje_velocidad: f64) -> Option<&'static str>`

- [ ] **Step 1: Escribir `app/src-tauri/src/validation/mentor.rs`**

```rust
use super::practices::Regla;

const UMBRAL_VELOCIDAD_BAJA: f64 = 70.0;

/// Comentario del Mentor (Etapa 17-D): nunca se genera con IA en vivo — cada
/// regla de buenas prácticas (o un patrón de costo de plan) tiene un
/// comentario pre-escrito. Se muestra el de la primera regla violada; si
/// ninguna regla falló pero el plan es notablemente costoso, se muestra un
/// comentario sobre el plan. Nunca dos comentarios a la vez (Etapa 11-E: "no
/// en cada ticket, se volvería ruido").
pub fn comentario(violaciones: &[Regla], puntaje_velocidad: f64) -> Option<&'static str> {
    for regla in violaciones {
        let texto = match regla {
            Regla::SelectStar => {
                "Vi que usaste SELECT *. Funciona, pero listar las columnas que de verdad \
                 necesitas hace la query más clara y evita traer datos de más."
            }
            Regla::JoinSinCondicion => {
                "Tu JOIN no tiene una condición clara de unión — eso puede generar un \
                 producto cartesiano (cada fila de una tabla contra cada fila de la otra). \
                 Revisa tu ON."
            }
            Regla::AliasFaltante => {
                "Cuando unes varias tablas, ponerles un alias corto a cada una hace la query \
                 mucho más fácil de leer — la tuya funciona, pero le vendría bien."
            }
        };
        return Some(texto);
    }

    if puntaje_velocidad < UMBRAL_VELOCIDAD_BAJA {
        return Some(
            "Tu query encontró la respuesta correcta, pero el plan de ejecución cuesta más \
             de lo necesario. Vale la pena revisar si hay una forma más directa de llegar al \
             mismo resultado.",
        );
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn muestra_comentario_de_la_primera_regla_violada() {
        assert!(comentario(&[Regla::SelectStar], 100.0)
            .unwrap()
            .contains("SELECT *"));
    }

    #[test]
    fn prioriza_la_primera_regla_sobre_las_siguientes() {
        assert!(comentario(&[Regla::JoinSinCondicion, Regla::AliasFaltante], 100.0)
            .unwrap()
            .contains("JOIN"));
    }

    #[test]
    fn muestra_comentario_de_velocidad_si_no_hay_reglas_violadas_pero_el_plan_es_costoso() {
        assert!(comentario(&[], 40.0).is_some());
    }

    #[test]
    fn no_hay_comentario_si_todo_esta_bien() {
        assert_eq!(comentario(&[], 100.0), None);
    }
}
```

- [ ] **Step 2: Actualizar `app/src-tauri/src/validation/mod.rs`**

```rust
mod correctness;
mod mentor;
mod practices;
mod speed;
```

- [ ] **Step 3: Correr la suite completa**

Run: `cd app/src-tauri && cargo test --lib -- --nocapture`
Expected: los tests previos (el conteo exacto puede variar levemente respecto a lo escrito en este plan; usa el número real reportado por la corrida anterior) + 4 nuevos de `validation::mentor::tests`, todos en verde, 0 failed.

- [ ] **Step 4: Commit**

```bash
git add app/src-tauri/src/validation/mentor.rs app/src-tauri/src/validation/mod.rs
git commit -m "Add pre-written Mentor comments tied to linter rules (Stage 17-D)"
```

---

### Task 5: Combinar los 4 componentes en `evaluar_entrega()`

**Files:**
- Modify: `app/src-tauri/src/validation/mod.rs` (reemplaza la lista de `mod` por la implementación completa)

**Interfaces:**
- Consumes: `crate::db::{run_query, init_embedded_postgres, load_company, Company}` (Plan 1), `correctness::son_equivalentes`, `speed::{costo_del_plan, puntaje}`, `practices::{analizar, puntaje}`, `mentor::comentario`
- Produces: `pub struct Evaluacion { pub correcta: bool, pub puntaje_correctitud: f64, pub puntaje_velocidad: f64, pub puntaje_practicas: f64, pub comentario_mentor: Option<&'static str> }`, `pub async fn evaluar_entrega(pool: &PgPool, sql_jugador: &str, sql_dorada: &str, requiere_orden: bool) -> anyhow::Result<Evaluacion>`

- [ ] **Step 1: Reemplazar `app/src-tauri/src/validation/mod.rs` completo**

```rust
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
    let puntaje_correctitud = if correcta { 100.0 } else { 0.0 };

    let costo_dorado = speed::costo_del_plan(pool, sql_dorada).await?;
    let costo_jugador = speed::costo_del_plan(pool, sql_jugador).await?;
    let puntaje_velocidad = speed::puntaje(costo_dorado, costo_jugador);

    let violaciones = practices::analizar(sql_jugador);
    let puntaje_practicas = practices::puntaje(&violaciones);

    let comentario_mentor = mentor::comentario(&violaciones, puntaje_velocidad);

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

        let sql_jugador = "SELECT nombre, fecha_ingreso, diagnostico FROM pacientes, departamentos \
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
```

- [ ] **Step 2: Correr la suite completa**

Run: `cd app/src-tauri && cargo test --lib -- --nocapture`
Expected: los tests previos (usa el número real reportado por la corrida anterior) + 3 nuevos de `validation::tests`, todos en verde, 0 failed.

- [ ] **Step 3: Commit**

```bash
git add app/src-tauri/src/validation/mod.rs
git commit -m "Combine correctness, speed, and practices into evaluar_entrega()"
```

---

### Task 6: Conectar `submit_ticket` al motor de validación real

**Files:**
- Modify: `app/src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: `validation::evaluar_entrega(pool, sql_jugador, sql_dorada, requiere_orden) -> anyhow::Result<validation::Evaluacion>` (de la Tarea 5), `db::hospital_arcangel::TICKET_SOLUCION` (vía `db::run_ticket_solution`, ya existente) — el ticket único del spike sigue siendo el único caso de uso hasta el plan de generación de tickets
- Produces: `ScoreResult` ahora expone el desglose de 3 componentes + el comentario del Mentor; el comando Tauri `submit_ticket` sigue teniendo la misma firma externa (mismo nombre, mismos argumentos)

- [ ] **Step 1: Localizar y reemplazar `ScoreResult` y `submit_ticket` en `app/src-tauri/src/lib.rs`**

Buscar la definición actual de `ScoreResult`:

```rust
#[derive(serde::Serialize)]
struct ScoreResult {
    pass: bool,
    dinero_ganado: i64,
    dinero_total: i64,
    mensaje: String,
}
```

Reemplazar por:

```rust
#[derive(serde::Serialize)]
struct ScoreResult {
    pass: bool,
    puntaje_correctitud: f64,
    puntaje_velocidad: f64,
    puntaje_practicas: f64,
    comentario_mentor: Option<String>,
    dinero_ganado: i64,
    dinero_total: i64,
    mensaje: String,
}
```

Buscar el comando `submit_ticket` actual:

```rust
#[tauri::command]
async fn submit_ticket(
    state: tauri::State<'_, AppState>,
    perk: tauri::State<'_, Perk>,
    sql: String,
) -> Result<ScoreResult, String> {
    let jugador = db::run_query(&state.pool, &sql).await.map_err(|e| e.to_string())?;
    let esperado = db::run_ticket_solution(&state.pool).await.map_err(|e| e.to_string())?;
    let pass = jugador.rows == esperado.rows;

    let mut perk_state = perk.0.lock().unwrap();
    let dinero_ganado = if pass { 500 } else { 0 };
    perk_state.dinero += dinero_ganado;

    Ok(ScoreResult {
        pass,
        dinero_ganado,
        dinero_total: perk_state.dinero,
        mensaje: if pass {
            "Ticket resuelto. Contabilidad procesará tu pago... eventualmente.".to_string()
        } else {
            "El resultado no coincide con lo que pidió Contabilidad. Revisa tu WHERE/ORDER BY.".to_string()
        },
    })
}
```

Reemplazar por:

```rust
#[tauri::command]
async fn submit_ticket(
    state: tauri::State<'_, AppState>,
    perk: tauri::State<'_, Perk>,
    sql: String,
) -> Result<ScoreResult, String> {
    let evaluacion = validation::evaluar_entrega(&state.pool, &sql, db::TICKET_SOLUCION_ACTUAL, true)
        .await
        .map_err(|e| e.to_string())?;

    let mut perk_state = perk.0.lock().unwrap();
    let dinero_ganado = if evaluacion.correcta { 500 } else { 0 };
    perk_state.dinero += dinero_ganado;

    Ok(ScoreResult {
        pass: evaluacion.correcta,
        puntaje_correctitud: evaluacion.puntaje_correctitud,
        puntaje_velocidad: evaluacion.puntaje_velocidad,
        puntaje_practicas: evaluacion.puntaje_practicas,
        comentario_mentor: evaluacion.comentario_mentor.map(str::to_string),
        dinero_ganado,
        dinero_total: perk_state.dinero,
        mensaje: if evaluacion.correcta {
            "Ticket resuelto. Contabilidad procesará tu pago... eventualmente.".to_string()
        } else {
            "El resultado no coincide con lo que pidió Contabilidad. Revisa tu WHERE/ORDER BY.".to_string()
        },
    })
}
```

Nota: este paso introduce `db::TICKET_SOLUCION_ACTUAL`, que no existe todavía — el Step 2 lo agrega.

- [ ] **Step 2: Exponer el texto de la solución dorada actual desde `db`**

`db::run_ticket_solution` (ya existente) ejecuta `hospital_arcangel::TICKET_SOLUCION`, pero ese texto es `pub(crate)` dentro de un submódulo privado (`hospital_arcangel`), así que no es alcanzable desde `lib.rs` como `db::hospital_arcangel::TICKET_SOLUCION`. En `app/src-tauri/src/db/mod.rs`, buscar la línea existente:

```rust
pub use hospital_arcangel::TICKET_ENUNCIADO;
```

y **reemplazarla** (no agregar una línea nueva aparte — dejaría `TICKET_ENUNCIADO` importado dos veces y no compila) por:

```rust
pub(crate) use hospital_arcangel::{TICKET_ENUNCIADO, TICKET_SOLUCION as TICKET_SOLUCION_ACTUAL};
```

Nota: tiene que ser `pub(crate) use`, no `pub use` — `TICKET_SOLUCION` está declarado `pub(crate)` en `hospital_arcangel.rs` (Tarea 1 del Plan 1), y Rust rechaza con error E0364 un `pub use` que intente re-exportar como totalmente público algo que solo es público dentro del crate. `pub(crate) use` es suficiente porque `lib.rs` (el único consumidor) está en el mismo crate.

(El renombrado a `TICKET_SOLUCION_ACTUAL` dinamiza el nombre para dejar claro que es el único ticket vigente del MVP, no una API general de múltiples tickets — el motor de generación de tickets del plan siguiente reemplazará este re-export por algo por-ticket.)

- [ ] **Step 3: Verificar que compila**

Run: `cd app/src-tauri && cargo check`
Expected: `Finished` sin errores.

- [ ] **Step 4: Correr la suite completa**

Run: `cd app/src-tauri && cargo test --lib -- --nocapture`
Expected: todos los tests en verde, 0 failed (sin tests nuevos en este paso — es solo wiring de `lib.rs`; el conteo total exacto depende del real de la tarea anterior).

- [ ] **Step 5: Smoke test de la app real**

Run: `cd app && npm run tauri dev` en segundo plano; esperar a que compile y arranque (buscar la línea `Running \`target/debug/app\`` en el log y confirmar con `ps aux` que el proceso vive), sin pánico. Detener limpiamente el proceso (`npm`/`tauri dev`/`target/debug/app`) antes de terminar — no dejar la ventana abierta.

- [ ] **Step 6: Commit**

```bash
git add app/src-tauri/src/lib.rs app/src-tauri/src/db/mod.rs
git commit -m "Wire submit_ticket to the real validation engine (Stage 17)"
```

---

## Self-Review Notes

- **Cobertura del spec:** Etapa 17-A (comparación por conjunto, tolerante a alias/redondeo, orden solo si se pide) ✓ — Etapa 17-B (costo de EXPLAIN, determinista) ✓ — Etapa 17-C (linter AST, puntaje no binario) ✓, acotado a los conceptos SQL de Becario/Auxiliar (Etapa 10/19) por diseño — Etapa 17-D (Mentor con comentarios pre-escritos, sin IA en vivo, atados a una regla o a un patrón de plan) ✓.
- **Fuera de alcance deliberado (para planes siguientes):** pesos de rúbrica por ticket y generación de tickets por plantilla (Plan de Etapa 14), fórmula de dinero/reputación/XP y multiplicadores de perks (Plan de Etapa 12/13) — este plan solo produce las 3 mediciones crudas.
- **Consistencia de tipos:** `Evaluacion` (Tarea 5) usa exactamente los tipos que producen `correctness::puntaje`/`speed::puntaje`/`practices::puntaje`/`mentor::comentario` (Tareas 1-4) — mismos nombres, mismas firmas, sin conversiones sorpresa.
- **Riesgo técnico ya verificado antes de escribir este plan** (no queda como incógnita para el implementador): se prototipó y corrió el linter de la Tarea 1 contra el AST real de `sqlparser 0.62` (confirmando los nombres exactos `SelectItem::Wildcard`, `TableFactor::Table { alias, .. }`, `JoinOperator::{Join,Inner,...}(JoinConstraint)`, `JoinConstraint::None`) y la extracción de costo de la Tarea 2 contra un Postgres embebido real (`EXPLAIN (FORMAT JSON)` → `plan[0]["Plan"]["Total Cost"]`) — ambos con los mismos casos de prueba que aparecen en este documento, y ambos con resultado exitoso.

---

## Execution Handoff

Plan completo y guardado en `docs/superpowers/plans/2026-07-12-fase0-02-motor-validacion.md`. Dos opciones de ejecución:

1. **Subagent-Driven (recomendado)** — despacho un subagente fresco por tarea, reviso el resultado entre cada una antes de seguir
2. **Ejecución inline** — ejecuto las tareas en esta sesión con executing-plans, ejecución por lotes con checkpoints

¿Cuál prefieres?
