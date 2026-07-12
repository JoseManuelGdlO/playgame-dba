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
