use super::{
    plantilla_depuracion, plantilla_reporte_agregado, plantilla_reporte_join_agregado,
    plantilla_reporte_simple, Arquetipo, Ticket, TipoTicket,
};

pub(crate) fn catalogo() -> Vec<Ticket> {
    vec![
        plantilla_reporte_simple(
            "postafeta_reporte_paquetes_centro",
            "Gerencia de Operaciones",
            "Kevin reporta que la gerencia quiere ver el estado de los envíos antes de la junta semanal.",
            "Lista todos los paquetes enviados desde la sucursal Centro (id 1), con su estado, del más reciente al más antiguo.",
            "SELECT id, estado, fecha_envio FROM paquetes WHERE sucursal_origen_id = 1 ORDER BY fecha_envio DESC",
            10,
        ),
        plantilla_reporte_agregado(
            "postafeta_reporte_incidencias_por_tipo",
            "Kevin",
            "Kevin necesita un resumen antes de firmarlo todo con \"- Kevin\" y mandarlo a la matriz.",
            "¿Cuántas incidencias hay de cada tipo? Ordena de mayor a menor.",
            "SELECT tipo, COUNT(*) AS total FROM incidencias GROUP BY tipo ORDER BY total DESC",
            15,
        ),
        plantilla_reporte_join_agregado(
            "postafeta_reporte_repartidor_top",
            "Recursos Humanos",
            "RH quiere reconocer (o interrogar) al repartidor más ocupado del mes.",
            "Lista cada repartidor junto con cuántos paquetes ha entregado, ordenado de mayor a menor.",
            "SELECT e.nombre AS repartidor, COUNT(p.id) AS total_paquetes FROM empleados e JOIN paquetes p ON p.repartidor_id = e.id GROUP BY e.nombre ORDER BY total_paquetes DESC",
            20,
        ),
        plantilla_reporte_agregado(
            "postafeta_reporte_clientes_por_ciudad",
            "Marketing",
            "Marketing quiere saber en qué ciudades concentrar la próxima campaña.",
            "¿Cuántos clientes tenemos registrados en cada ciudad? Ordena de mayor a menor.",
            "SELECT ciudad, COUNT(*) AS total_clientes FROM clientes GROUP BY ciudad ORDER BY total_clientes DESC",
            15,
        ),
        plantilla_depuracion(
            "postafeta_depuracion_paquetes_cliente",
            "Kevin",
            "Kevin heredó este reporte de otro becario que ya no está (nadie sabe quién).",
            "Esta consulta tarda demasiado. Encuentra una forma más directa de obtener lo mismo.",
            "SELECT id, (SELECT c.nombre FROM clientes c WHERE c.id = paquetes.cliente_id) AS cliente, estado FROM paquetes ORDER BY id",
            "SELECT p.id, c.nombre AS cliente, p.estado FROM paquetes p JOIN clientes c ON p.cliente_id = c.id ORDER BY p.id",
            vec![Arquetipo::Join],
            20,
        ),
        plantilla_depuracion(
            "postafeta_depuracion_incidencias_por_tipo",
            "Finanzas",
            "Finanzas quiere este reporte, pero Kevin dice que \"últimamente se cuelga\".",
            "Optimiza este reporte para que no repita el mismo conteo una y otra vez.",
            "SELECT DISTINCT tipo, (SELECT COUNT(*) FROM incidencias i2 WHERE i2.tipo = i1.tipo) AS total FROM incidencias i1 ORDER BY total DESC",
            "SELECT tipo, COUNT(*) AS total FROM incidencias GROUP BY tipo ORDER BY total DESC",
            vec![Arquetipo::Agregacion],
            20,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{self, Company};
    use crate::validation;

    #[test]
    fn catalogo_tiene_4_reportes_y_2_depuraciones() {
        let tickets = catalogo();
        assert_eq!(tickets.len(), 6);
        let reportes = tickets.iter().filter(|t| t.tipo == TipoTicket::ReporteAnalisis).count();
        let depuraciones = tickets.iter().filter(|t| t.tipo == TipoTicket::InvestigacionDepuracion).count();
        assert_eq!(reportes, 4);
        assert_eq!(depuraciones, 2);
    }

    #[tokio::test]
    async fn todas_las_queries_doradas_ejecutan() {
        let pg = db::init_embedded_postgres().await.expect("Postgres embebido debe arrancar");
        let pool = db::load_company(&pg, Company::Postafeta).await.expect("Postafeta debe cargar");

        for ticket in catalogo() {
            db::run_query(&pool, &ticket.sql_dorada)
                .await
                .unwrap_or_else(|e| panic!("la query dorada de '{}' debe ejecutar: {e}", ticket.id));
        }

        pool.close().await;
        pg.stop().await.expect("Postgres debe detenerse limpiamente");
    }

    #[tokio::test]
    async fn los_tickets_de_depuracion_son_correctos_pero_lentos() {
        let pg = db::init_embedded_postgres().await.expect("Postgres embebido debe arrancar");
        let pool = db::load_company(&pg, Company::Postafeta).await.expect("Postafeta debe cargar");

        for ticket in catalogo().into_iter().filter(|t| t.sql_inicial.is_some()) {
            let sql_inicial = ticket.sql_inicial.clone().unwrap();
            let evaluacion = validation::evaluar_entrega(&pool, &sql_inicial, &ticket.sql_dorada, ticket.requiere_orden)
                .await
                .unwrap_or_else(|e| panic!("la evaluación de '{}' debe ejecutar: {e}", ticket.id));

            assert!(evaluacion.correcta, "'{}': sql_inicial debe dar el mismo resultado que sql_dorada", ticket.id);
            assert!(
                evaluacion.puntaje_velocidad < 100.0,
                "'{}': sql_inicial debe costar más que sql_dorada (puntaje_velocidad={})",
                ticket.id,
                evaluacion.puntaje_velocidad
            );
        }

        pool.close().await;
        pg.stop().await.expect("Postgres debe detenerse limpiamente");
    }
}
