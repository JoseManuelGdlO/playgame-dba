/// Un ticket concreto (Etapa 14): la unidad de trabajo que el jugador
/// resuelve escribiendo SQL. Generado por una plantilla paramétrica (las
/// funciones `plantilla_*` de este archivo), nunca escrito a mano ticket por
/// ticket (Pilar 5).
#[derive(Debug, Clone, serde::Serialize)]
pub struct Ticket {
    pub id: &'static str,
    pub tipo: TipoTicket,
    pub solicitante: &'static str,
    pub motivo: String,
    pub solicitud: String,
    pub prioridad: Prioridad,
    pub costo_tiempo: u32,
    pub arquetipos: Vec<Arquetipo>,
    /// Nunca debe llegar al cliente: es la respuesta correcta del puzzle.
    #[serde(skip_serializing)]
    pub sql_dorada: String,
    pub sql_inicial: Option<String>,
    pub requiere_orden: bool,
    // Ya no se serializan (no deben llegar al cliente) y aún no las lee
    // ningún código de producción — solo los tests de este módulo — por lo
    // que el análisis de código muerto las marcaría sin este allow.
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    pub peso_correctitud: f64,
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    pub peso_velocidad: f64,
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    pub peso_practicas: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum TipoTicket {
    ReporteAnalisis,
    InvestigacionDepuracion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Prioridad {
    Baja,
    Media,
    // Ningún ticket de los catálogos actuales (Hospital Arcángel/Postafeta,
    // Tareas 2-3) usa esta prioridad todavía — queda reservada para tickets
    // futuros de mayor urgencia.
    #[allow(dead_code)]
    Urgente,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Arquetipo {
    Select,
    Join,
    Agregacion,
}

/// Plantilla "reporte simple": filtra y ordena una tabla por una columna,
/// sin JOIN ni agregación (Becario: SELECT/WHERE/ORDER BY, Etapa 10).
fn plantilla_reporte_simple(
    id: &'static str,
    solicitante: &'static str,
    motivo: impl Into<String>,
    solicitud: impl Into<String>,
    sql_dorada: impl Into<String>,
    costo_tiempo: u32,
) -> Ticket {
    Ticket {
        id,
        tipo: TipoTicket::ReporteAnalisis,
        solicitante,
        motivo: motivo.into(),
        solicitud: solicitud.into(),
        prioridad: Prioridad::Media,
        costo_tiempo,
        arquetipos: vec![Arquetipo::Select],
        sql_dorada: sql_dorada.into(),
        sql_inicial: None,
        requiere_orden: true,
        peso_correctitud: 0.6,
        peso_velocidad: 0.2,
        peso_practicas: 0.2,
    }
}

/// Plantilla "reporte agregado": agrupa una tabla por una columna y calcula
/// una métrica (Auxiliar: GROUP BY + COUNT/SUM, Etapa 10).
fn plantilla_reporte_agregado(
    id: &'static str,
    solicitante: &'static str,
    motivo: impl Into<String>,
    solicitud: impl Into<String>,
    sql_dorada: impl Into<String>,
    costo_tiempo: u32,
) -> Ticket {
    Ticket {
        id,
        tipo: TipoTicket::ReporteAnalisis,
        solicitante,
        motivo: motivo.into(),
        solicitud: solicitud.into(),
        prioridad: Prioridad::Baja,
        costo_tiempo,
        arquetipos: vec![Arquetipo::Agregacion],
        sql_dorada: sql_dorada.into(),
        sql_inicial: None,
        requiere_orden: true,
        peso_correctitud: 0.5,
        peso_velocidad: 0.2,
        peso_practicas: 0.3,
    }
}

/// Plantilla "reporte con JOIN": combina 2 tablas y lista resultados
/// (Auxiliar: JOIN inner, Etapa 10).
fn plantilla_reporte_join(
    id: &'static str,
    solicitante: &'static str,
    motivo: impl Into<String>,
    solicitud: impl Into<String>,
    sql_dorada: impl Into<String>,
    costo_tiempo: u32,
) -> Ticket {
    Ticket {
        id,
        tipo: TipoTicket::ReporteAnalisis,
        solicitante,
        motivo: motivo.into(),
        solicitud: solicitud.into(),
        prioridad: Prioridad::Media,
        costo_tiempo,
        arquetipos: vec![Arquetipo::Join],
        sql_dorada: sql_dorada.into(),
        sql_inicial: None,
        requiere_orden: true,
        peso_correctitud: 0.5,
        peso_velocidad: 0.2,
        peso_practicas: 0.3,
    }
}

/// Plantilla "reporte con JOIN + agregación": combina 2 tablas, agrupa y
/// calcula una métrica (Auxiliar: JOIN + GROUP BY + COUNT, Etapa 10).
fn plantilla_reporte_join_agregado(
    id: &'static str,
    solicitante: &'static str,
    motivo: impl Into<String>,
    solicitud: impl Into<String>,
    sql_dorada: impl Into<String>,
    costo_tiempo: u32,
) -> Ticket {
    Ticket {
        id,
        tipo: TipoTicket::ReporteAnalisis,
        solicitante,
        motivo: motivo.into(),
        solicitud: solicitud.into(),
        prioridad: Prioridad::Media,
        costo_tiempo,
        arquetipos: vec![Arquetipo::Join, Arquetipo::Agregacion],
        sql_dorada: sql_dorada.into(),
        sql_inicial: None,
        requiere_orden: true,
        peso_correctitud: 0.4,
        peso_velocidad: 0.3,
        peso_practicas: 0.3,
    }
}

/// Plantilla "Investigación/Depuración" (Etapa 14): se entrega una query ya
/// escrita — lenta pero con el mismo resultado correcto, nunca con un
/// resultado de negocio distinto — que el jugador debe optimizar. Conecta
/// con la fantasía de maestría y El Mentor (Etapa 5).
fn plantilla_depuracion(
    id: &'static str,
    solicitante: &'static str,
    motivo: impl Into<String>,
    solicitud: impl Into<String>,
    sql_inicial: impl Into<String>,
    sql_dorada: impl Into<String>,
    arquetipos: Vec<Arquetipo>,
    costo_tiempo: u32,
) -> Ticket {
    Ticket {
        id,
        tipo: TipoTicket::InvestigacionDepuracion,
        solicitante,
        motivo: motivo.into(),
        solicitud: solicitud.into(),
        prioridad: Prioridad::Baja,
        costo_tiempo,
        arquetipos,
        sql_dorada: sql_dorada.into(),
        sql_inicial: Some(sql_inicial.into()),
        requiere_orden: true,
        peso_correctitud: 0.3,
        peso_velocidad: 0.5,
        peso_practicas: 0.2,
    }
}

mod hospital_arcangel;
mod postafeta;

/// Catálogo de tickets de `company` (Etapa 14) — generado por las plantillas
/// paramétricas de este módulo, nunca escrito a mano ticket por ticket.
pub fn catalogo(company: crate::db::Company) -> Vec<Ticket> {
    match company {
        crate::db::Company::HospitalArcangel => hospital_arcangel::catalogo(),
        crate::db::Company::Postafeta => postafeta::catalogo(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plantilla_reporte_simple_arma_un_ticket_de_reporte_sin_join() {
        let ticket = plantilla_reporte_simple("id1", "Alguien", "un motivo", "una solicitud", "SELECT 1", 10);
        assert_eq!(ticket.tipo, TipoTicket::ReporteAnalisis);
        assert_eq!(ticket.arquetipos, vec![Arquetipo::Select]);
        assert!(ticket.sql_inicial.is_none());
        assert!(ticket.requiere_orden);
        assert_eq!((ticket.peso_correctitud, ticket.peso_velocidad, ticket.peso_practicas), (0.6, 0.2, 0.2));
    }

    #[test]
    fn plantilla_reporte_agregado_arma_un_ticket_de_agregacion() {
        let ticket = plantilla_reporte_agregado("id2", "Alguien", "un motivo", "una solicitud", "SELECT 1", 15);
        assert_eq!(ticket.tipo, TipoTicket::ReporteAnalisis);
        assert_eq!(ticket.arquetipos, vec![Arquetipo::Agregacion]);
        assert_eq!(ticket.prioridad, Prioridad::Baja);
        assert_eq!((ticket.peso_correctitud, ticket.peso_velocidad, ticket.peso_practicas), (0.5, 0.2, 0.3));
    }

    #[test]
    fn plantilla_reporte_join_arma_un_ticket_con_join() {
        let ticket = plantilla_reporte_join("id3", "Alguien", "un motivo", "una solicitud", "SELECT 1", 15);
        assert_eq!(ticket.tipo, TipoTicket::ReporteAnalisis);
        assert_eq!(ticket.arquetipos, vec![Arquetipo::Join]);
        assert_eq!((ticket.peso_correctitud, ticket.peso_velocidad, ticket.peso_practicas), (0.5, 0.2, 0.3));
    }

    #[test]
    fn plantilla_reporte_join_agregado_arma_un_ticket_con_join_y_agregacion() {
        let ticket = plantilla_reporte_join_agregado("id4", "Alguien", "un motivo", "una solicitud", "SELECT 1", 20);
        assert_eq!(ticket.tipo, TipoTicket::ReporteAnalisis);
        assert_eq!(ticket.arquetipos, vec![Arquetipo::Join, Arquetipo::Agregacion]);
        assert_eq!((ticket.peso_correctitud, ticket.peso_velocidad, ticket.peso_practicas), (0.4, 0.3, 0.3));
    }

    #[test]
    fn plantilla_depuracion_arma_un_ticket_con_sql_inicial() {
        let ticket = plantilla_depuracion(
            "id5",
            "Alguien",
            "un motivo",
            "una solicitud",
            "SELECT lenta",
            "SELECT rapida",
            vec![Arquetipo::Join],
            20,
        );
        assert_eq!(ticket.tipo, TipoTicket::InvestigacionDepuracion);
        assert_eq!(ticket.sql_inicial, Some("SELECT lenta".to_string()));
        assert_eq!(ticket.sql_dorada, "SELECT rapida");
        assert_eq!(ticket.arquetipos, vec![Arquetipo::Join]);
        assert_eq!((ticket.peso_correctitud, ticket.peso_velocidad, ticket.peso_practicas), (0.3, 0.5, 0.2));
    }

    #[test]
    fn catalogo_devuelve_6_tickets_para_cada_empresa() {
        assert_eq!(catalogo(crate::db::Company::HospitalArcangel).len(), 6);
        assert_eq!(catalogo(crate::db::Company::Postafeta).len(), 6);
    }
}
