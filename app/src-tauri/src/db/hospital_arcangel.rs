pub(crate) const DB_NAME: &str = "query_path_hospital_arcangel";

/// Esquema de Hospital Arcángel (Etapa 16): 6 tablas, suficientes para probar
/// JOIN, agregación, window functions y CTE recursivo (jerarquía jefe_id)
/// contra un Postgres real.
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

/// El único ticket de Hospital Arcángel por ahora (Etapa 14): rango Becario,
/// solo SELECT/WHERE/ORDER BY (Etapa 10).
pub const TICKET_ENUNCIADO: &str = "Motivo: Contabilidad quiere saber quién ha pisado Cardiología últimamente.\nSolicitud: lista los pacientes admitidos en Cardiología (nombre, fecha de ingreso y diagnóstico), del más reciente al más antiguo.";

pub(crate) const TICKET_SOLUCION: &str =
    "SELECT nombre, fecha_ingreso, diagnostico FROM pacientes WHERE departamento_id = 1 ORDER BY fecha_ingreso DESC";

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::TICKET_SOLUCION;

    /// Prueba de punta a punta del stack (Etapa 18/22): arranca Postgres
    /// embebido, y ejecuta window function, CTE recursivo y EXPLAIN reales —
    /// justo lo que SQLite no puede hacer y por lo que se eligió este stack.
    #[tokio::test]
    async fn hospital_arcangel_end_to_end() {
        let pg = init_embedded_postgres().await.expect("Postgres embebido debe arrancar");
        let pool = load_company(&pg, Company::HospitalArcangel)
            .await
            .expect("Hospital Arcángel debe cargar");

        let ranking = run_query(
            &pool,
            "SELECT nombre, salario, RANK() OVER (PARTITION BY departamento_id ORDER BY salario DESC) AS puesto \
             FROM empleados WHERE departamento_id = 1",
        )
        .await
        .expect("window function debe ejecutar");
        assert_eq!(ranking.rows.len(), 4, "Salcedo, Peralta, Cetina y Junco están en Cardiología");

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
        let esperado = run_query(&pool, TICKET_SOLUCION).await.unwrap();
        assert_eq!(jugador.rows, esperado.rows, "la solución del ticket debe pasar contra sí misma");

        pool.close().await;
        pg.stop().await.expect("Postgres debe detenerse limpiamente");
    }

    #[tokio::test]
    async fn reporte_costos_por_departamento() {
        let pg = init_embedded_postgres().await.expect("Postgres embebido debe arrancar");
        let pool = load_company(&pg, Company::HospitalArcangel)
            .await
            .expect("Hospital Arcángel debe cargar");

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
        let pg = init_embedded_postgres().await.expect("Postgres embebido debe arrancar");
        let pool = load_company(&pg, Company::HospitalArcangel)
            .await
            .expect("Hospital Arcángel debe cargar");

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
}
