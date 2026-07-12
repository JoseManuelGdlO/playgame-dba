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
    use super::TICKET_SOLUCION;

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
