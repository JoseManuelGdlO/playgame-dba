use super::{
    plantilla_depuracion, plantilla_reporte_agregado, plantilla_reporte_join,
    plantilla_reporte_join_agregado, plantilla_reporte_simple, Arquetipo, Ticket,
};
#[cfg(test)]
use super::TipoTicket;

pub(crate) fn catalogo() -> Vec<Ticket> {
    vec![
        plantilla_reporte_simple(
            "hospital_reporte_pacientes_cardiologia",
            "Contabilidad",
            "Contabilidad quiere saber quién ha pisado Cardiología últimamente.",
            "Lista los pacientes admitidos en Cardiología (nombre, fecha de ingreso y diagnóstico), del más reciente al más antiguo.",
            "SELECT nombre, fecha_ingreso, diagnostico FROM pacientes WHERE departamento_id = 1 ORDER BY fecha_ingreso DESC",
            10,
        ),
        plantilla_reporte_simple(
            "hospital_reporte_habitaciones_libres",
            "Enfermería",
            "Enfermería necesita saber qué camas están libres antes de que llegue el siguiente turno de admisiones.",
            "Lista el número y el tipo de cada habitación que esté libre (no ocupada), ordenadas por número.",
            "SELECT numero, tipo FROM habitaciones WHERE ocupada = false ORDER BY numero",
            10,
        ),
        plantilla_reporte_simple(
            "hospital_reporte_pacientes_sin_alta",
            "Auditoría de Calidad",
            "Auditoría de Calidad necesita confirmar cuántos pacientes siguen internados para su reporte semanal de ocupación.",
            "Lista el nombre y la fecha de ingreso de los pacientes que todavía no tienen fecha de alta, del ingreso más antiguo al más reciente.",
            "SELECT nombre, fecha_ingreso FROM pacientes WHERE fecha_alta IS NULL ORDER BY fecha_ingreso, nombre",
            10,
        ),
        plantilla_reporte_agregado(
            "hospital_reporte_costo_por_tipo",
            "Dirección General",
            "El CEO quiere un número para su próxima galleta de la fortuna motivacional sobre gastos médicos.",
            "¿Cuánto hemos gastado en total por cada tipo de tratamiento? Ordena de mayor a menor gasto.",
            "SELECT tipo, SUM(costo) AS costo_total FROM tratamientos GROUP BY tipo ORDER BY costo_total DESC",
            15,
        ),
        plantilla_reporte_join(
            "hospital_reporte_empleados_departamento",
            "Recursos Humanos",
            "RH necesita el directorio actualizado antes de la auditoría de nómina de este trimestre.",
            "Lista el nombre de cada empleado junto con el nombre de su departamento, ordenados por nombre de empleado.",
            "SELECT e.nombre AS empleado, d.nombre AS departamento FROM empleados e JOIN departamentos d ON e.departamento_id = d.id ORDER BY e.nombre",
            15,
        ),
        plantilla_reporte_join_agregado(
            "hospital_reporte_habitaciones_ocupadas",
            "Administración de Instalaciones",
            "Mantenimiento quiere saber cuántas camas siguen ocupadas antes de programar la fumigación trimestral.",
            "¿Cuántas habitaciones ocupadas hay en cada departamento? Ordena por nombre de departamento.",
            "SELECT d.nombre AS departamento, COUNT(*) AS habitaciones_ocupadas FROM habitaciones h JOIN departamentos d ON h.departamento_id = d.id WHERE h.ocupada = true GROUP BY d.nombre ORDER BY d.nombre",
            20,
        ),
        plantilla_depuracion(
            "hospital_depuracion_pacientes_departamento",
            "El Mentor",
            "Un becario anterior escribió este reporte de altas y funciona, pero el Mentor sospecha que hay una forma más rápida.",
            "Optimiza esta consulta para que no tenga que preguntarle a la base de datos una vez por cada paciente.",
            "SELECT nombre, (SELECT d.nombre FROM departamentos d WHERE d.id = pacientes.departamento_id) AS departamento FROM pacientes ORDER BY nombre",
            "SELECT p.nombre, d.nombre AS departamento FROM pacientes p JOIN departamentos d ON p.departamento_id = d.id ORDER BY p.nombre",
            vec![Arquetipo::Join],
            20,
        ),
        plantilla_depuracion(
            "hospital_depuracion_costo_por_tipo",
            "Finanzas",
            "Finanzas heredó este reporte de un consultor externo que ya no trabaja aquí.",
            "Este reporte tarda una eternidad en cargar. Encuentra una forma de obtener el mismo resultado sin tanto rodeo.",
            "SELECT DISTINCT tipo, (SELECT SUM(costo) FROM tratamientos t2 WHERE t2.tipo = t1.tipo) AS costo_total FROM tratamientos t1 ORDER BY costo_total DESC",
            "SELECT tipo, SUM(costo) AS costo_total FROM tratamientos GROUP BY tipo ORDER BY costo_total DESC",
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
    fn catalogo_tiene_6_reportes_y_2_depuraciones() {
        let tickets = catalogo();
        assert_eq!(tickets.len(), 8);
        let reportes = tickets.iter().filter(|t| t.tipo == TipoTicket::ReporteAnalisis).count();
        let depuraciones = tickets.iter().filter(|t| t.tipo == TipoTicket::InvestigacionDepuracion).count();
        assert_eq!(reportes, 6, "4 originales + 2 Select-only agregados para Becario (Plan 7)");
        assert_eq!(depuraciones, 2);
    }

    #[tokio::test]
    async fn todas_las_queries_doradas_ejecutan() {
        let pg = db::init_embedded_postgres().await.expect("Postgres embebido debe arrancar");
        let pool = db::load_company(&pg, Company::HospitalArcangel).await.expect("Hospital Arcángel debe cargar");

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
        let pool = db::load_company(&pg, Company::HospitalArcangel).await.expect("Hospital Arcángel debe cargar");

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
