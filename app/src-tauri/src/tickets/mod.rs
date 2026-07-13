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
    #[serde(skip_serializing)]
    pub peso_correctitud: f64,
    #[serde(skip_serializing)]
    pub peso_velocidad: f64,
    #[serde(skip_serializing)]
    pub peso_practicas: f64,
    /// Valor base de dinero (Etapa 12) — sube con prioridad/complejidad del
    /// ticket. Dato interno de la fórmula de economía, sin uso del lado del
    /// cliente.
    #[serde(skip_serializing)]
    pub valor_base: i64,
    /// Factor de reputación (Etapa 12) — mayor en tickets de mayor exigencia.
    #[serde(skip_serializing)]
    pub factor_reputacion: f64,
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

/// Rango de carrera del jugador (Etapa 10, Plan 7): determina qué tickets
/// del catálogo puede recibir en su bandeja. El orden de declaración importa
/// — el derive de `Ord` decide qué rango "alcanza" a cuál según ese orden.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, serde::Serialize)]
pub enum Rango {
    #[default]
    Becario,
    AuxiliarDeSistemas,
}

/// Etapa 10/Plan 7: un ticket requiere Auxiliar de Sistemas si su solución
/// necesita JOIN o agregación — Becario solo domina SELECT/WHERE/ORDER BY.
pub fn rango_requerido(ticket: &Ticket) -> Rango {
    let necesita_auxiliar = ticket
        .arquetipos
        .iter()
        .any(|a| matches!(a, Arquetipo::Join | Arquetipo::Agregacion));
    if necesita_auxiliar {
        Rango::AuxiliarDeSistemas
    } else {
        Rango::Becario
    }
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
        valor_base: 100,
        factor_reputacion: 0.5,
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
        valor_base: 150,
        factor_reputacion: 0.7,
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
        valor_base: 150,
        factor_reputacion: 0.7,
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
        valor_base: 200,
        factor_reputacion: 1.0,
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
        valor_base: 250,
        factor_reputacion: 1.2,
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
        assert_eq!((ticket.valor_base, ticket.factor_reputacion), (100, 0.5));
    }

    #[test]
    fn plantilla_reporte_agregado_arma_un_ticket_de_agregacion() {
        let ticket = plantilla_reporte_agregado("id2", "Alguien", "un motivo", "una solicitud", "SELECT 1", 15);
        assert_eq!(ticket.tipo, TipoTicket::ReporteAnalisis);
        assert_eq!(ticket.arquetipos, vec![Arquetipo::Agregacion]);
        assert_eq!(ticket.prioridad, Prioridad::Baja);
        assert_eq!((ticket.peso_correctitud, ticket.peso_velocidad, ticket.peso_practicas), (0.5, 0.2, 0.3));
        assert_eq!((ticket.valor_base, ticket.factor_reputacion), (150, 0.7));
    }

    #[test]
    fn plantilla_reporte_join_arma_un_ticket_con_join() {
        let ticket = plantilla_reporte_join("id3", "Alguien", "un motivo", "una solicitud", "SELECT 1", 15);
        assert_eq!(ticket.tipo, TipoTicket::ReporteAnalisis);
        assert_eq!(ticket.arquetipos, vec![Arquetipo::Join]);
        assert_eq!((ticket.peso_correctitud, ticket.peso_velocidad, ticket.peso_practicas), (0.5, 0.2, 0.3));
        assert_eq!((ticket.valor_base, ticket.factor_reputacion), (150, 0.7));
    }

    #[test]
    fn plantilla_reporte_join_agregado_arma_un_ticket_con_join_y_agregacion() {
        let ticket = plantilla_reporte_join_agregado("id4", "Alguien", "un motivo", "una solicitud", "SELECT 1", 20);
        assert_eq!(ticket.tipo, TipoTicket::ReporteAnalisis);
        assert_eq!(ticket.arquetipos, vec![Arquetipo::Join, Arquetipo::Agregacion]);
        assert_eq!((ticket.peso_correctitud, ticket.peso_velocidad, ticket.peso_practicas), (0.4, 0.3, 0.3));
        assert_eq!((ticket.valor_base, ticket.factor_reputacion), (200, 1.0));
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
        assert_eq!((ticket.valor_base, ticket.factor_reputacion), (250, 1.2));
    }

    #[test]
    fn catalogo_devuelve_el_tamano_esperado_por_empresa() {
        assert_eq!(
            catalogo(crate::db::Company::HospitalArcangel).len(),
            8,
            "Plan 7 agrega 2 tickets Select-only para que Becario tenga bandeja"
        );
        assert_eq!(catalogo(crate::db::Company::Postafeta).len(), 6);
    }

    #[test]
    fn rango_becario_es_menor_que_auxiliar_de_sistemas() {
        assert!(Rango::Becario < Rango::AuxiliarDeSistemas);
    }

    #[test]
    fn rango_requerido_es_becario_para_tickets_solo_select() {
        let ticket = plantilla_reporte_simple("id_becario", "Alguien", "un motivo", "una solicitud", "SELECT 1", 10);
        assert_eq!(rango_requerido(&ticket), Rango::Becario);
    }

    #[test]
    fn rango_requerido_es_auxiliar_si_incluye_join_o_agregacion() {
        let con_join = plantilla_reporte_join("id_join", "Alguien", "un motivo", "una solicitud", "SELECT 1", 10);
        let con_agregacion =
            plantilla_reporte_agregado("id_agg", "Alguien", "un motivo", "una solicitud", "SELECT 1", 10);
        let con_ambos =
            plantilla_reporte_join_agregado("id_both", "Alguien", "un motivo", "una solicitud", "SELECT 1", 10);

        assert_eq!(rango_requerido(&con_join), Rango::AuxiliarDeSistemas);
        assert_eq!(rango_requerido(&con_agregacion), Rango::AuxiliarDeSistemas);
        assert_eq!(rango_requerido(&con_ambos), Rango::AuxiliarDeSistemas);
    }

    #[test]
    fn catalogo_de_hospital_arcangel_tiene_3_tickets_elegibles_para_becario() {
        let elegibles = catalogo(crate::db::Company::HospitalArcangel)
            .into_iter()
            .filter(|t| rango_requerido(t) <= Rango::Becario)
            .count();
        assert_eq!(elegibles, 3, "el ticket original de Select + los 2 agregados en la Tarea 1");
    }
}
