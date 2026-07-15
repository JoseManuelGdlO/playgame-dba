use crate::tickets::Ticket;

/// Cuántos tickets recibe el jugador en cada turno (Etapa 11-A).
const TAMANO_LOTE: usize = 3;

/// Presupuesto de tiempo de turno base (Modo Turbo suma encima).
pub const PRESUPUESTO_POR_TURNO: u32 = 100;

/// Cuánta reputación se pierde por cada ticket que queda pendiente al cerrar
/// el turno (Etapa 11-A/12) — valor de partida, sujeto a ajuste.
const FACTOR_PENALIZACION_ESCALAMIENTO: f64 = 2.0;

/// Estado de un turno (Etapa 11-A): el lote de tickets pendientes y cuánto
/// presupuesto de tiempo queda.
#[derive(Debug, Clone, serde::Serialize)]
pub struct EstadoTurno {
    pub presupuesto_restante: u32,
    pub pendientes: Vec<Ticket>,
    /// Cuántos intentos incorrectos lleva cada ticket pendiente en este turno
    /// (Plan 17) — nunca se manda al frontend (el conteo de intentos
    /// restantes que ve el jugador viene en la respuesta de `resolver_ticket`,
    /// no de aquí). Se limpia cuando el ticket se resuelve de verdad (acierto
    /// o intentos agotados) para que un id de ticket que reaparezca en un
    /// turno futuro empiece en cero.
    #[serde(skip)]
    pub intentos_usados: std::collections::HashMap<String, u32>,
}

/// Un ticket escalado al cerrar el turno, y cuánta reputación costó.
#[derive(Debug, Clone, PartialEq)]
pub struct Escalamiento {
    pub ticket_id: &'static str,
    pub reputacion_perdida: f64,
}

impl EstadoTurno {
    /// Arranca un turno nuevo: sortea `TAMANO_LOTE` tickets del catálogo
    /// (rotación determinista, sin aleatoriedad) empezando en
    /// `indice_inicial`, y llena el presupuesto de tiempo. Devuelve el nuevo
    /// estado y el índice donde debe empezar el turno siguiente.
    pub fn nuevo(catalogo: &[Ticket], indice_inicial: usize) -> (Self, usize) {
        Self::nuevo_con_presupuesto(catalogo, indice_inicial, PRESUPUESTO_POR_TURNO)
    }

    pub fn nuevo_con_presupuesto(
        catalogo: &[Ticket],
        indice_inicial: usize,
        presupuesto: u32,
    ) -> (Self, usize) {
        let tamano = TAMANO_LOTE.min(catalogo.len());
        let mut pendientes = Vec::with_capacity(tamano);
        // Plan 7: `catalogo` ya no es siempre el catálogo completo de la
        // empresa — el gating por rango (`lib.rs`) puede pasar un slice
        // filtrado de tamaño distinto entre turnos. Normalizar con módulo
        // evita un panic por índice fuera de rango; un catálogo vacío no
        // debe darse en la práctica (Becario siempre tiene tickets
        // elegibles), pero se cubre explícitamente para no dejar un panic
        // latente.
        let mut indice = if catalogo.is_empty() { 0 } else { indice_inicial % catalogo.len() };
        for _ in 0..tamano {
            pendientes.push(catalogo[indice].clone());
            indice = (indice + 1) % catalogo.len();
        }
        (
            EstadoTurno {
                presupuesto_restante: presupuesto,
                pendientes,
                intentos_usados: std::collections::HashMap::new(),
            },
            indice,
        )
    }

    /// Busca un ticket pendiente por id.
    pub fn buscar_pendiente(&self, id: &str) -> Option<&Ticket> {
        self.pendientes.iter().find(|t| t.id == id)
    }

    /// Retira un ticket resuelto del lote y consume `costo` del presupuesto
    /// (puede ser menor que `ticket.costo_tiempo` con Café Cargado).
    pub fn resolver(&mut self, id: &str, costo: u32) -> Option<Ticket> {
        let posicion = self.pendientes.iter().position(|t| t.id == id)?;
        let ticket = self.pendientes.remove(posicion);
        self.presupuesto_restante = self.presupuesto_restante.saturating_sub(costo);
        Some(ticket)
    }

    /// Cuenta un intento incorrecto de `id` y devuelve el nuevo total
    /// (Plan 17) — 1 en el primer registro de ese ticket en este turno.
    pub fn registrar_intento(&mut self, id: &str) -> u32 {
        let contador = self.intentos_usados.entry(id.to_string()).or_insert(0);
        *contador += 1;
        *contador
    }

    /// Olvida el conteo de intentos de `id` (Plan 17) — se llama cuando el
    /// ticket se resuelve de verdad (acierto o intentos agotados), para que
    /// no quede un conteo obsoleto si ese id reapareciera en un turno futuro.
    pub fn limpiar_intentos(&mut self, id: &str) {
        self.intentos_usados.remove(id);
    }

    /// Deshace un `resolver()` (Plan 17): reembolsa el costo de tiempo del
    /// ticket al presupuesto y lo reinserta en `pendientes` — usado cuando un
    /// intento falla pero todavía quedan reintentos disponibles, para que el
    /// ticket siga abierto sin haber costado nada de tiempo ni de pago.
    pub fn reintentar(&mut self, ticket: Ticket, costo: u32) {
        self.presupuesto_restante = self.presupuesto_restante.saturating_add(costo);
        self.pendientes.push(ticket);
    }

    /// Etapa 11-A: ¿ya no queda presupuesto para ningún ticket pendiente?
    /// (Vacío cuenta como agotado: no hay nada más que hacer este turno.)
    pub fn turno_agotado(&self) -> bool {
        self.turno_agotado_con_factor(1.0)
    }

    pub fn turno_agotado_con_factor(&self, factor_costo: f64) -> bool {
        self.pendientes.iter().all(|t| {
            let costo = crate::economia::costo_tiempo_efectivo(t.costo_tiempo, factor_costo);
            costo > self.presupuesto_restante
        })
    }

    /// Escala todos los tickets pendientes (turno agotado o cierre manual del
    /// día, Etapa 11-A): cada uno pierde `factor_reputacion × 2.0` de
    /// reputación. Devuelve la lista de escalamientos para que el llamador
    /// aplique la penalización.
    pub fn escalar_pendientes(&self) -> Vec<Escalamiento> {
        self.pendientes
            .iter()
            .map(|t| Escalamiento {
                ticket_id: t.id,
                reputacion_perdida: t.factor_reputacion * FACTOR_PENALIZACION_ESCALAMIENTO,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tickets::{Arquetipo, Prioridad, TipoTicket};

    fn ticket_de_prueba(id: &'static str, costo_tiempo: u32, factor_reputacion: f64) -> Ticket {
        Ticket {
            id,
            tipo: TipoTicket::ReporteAnalisis,
            solicitante: "Alguien",
            motivo: "un motivo".to_string(),
            solicitud: "una solicitud".to_string(),
            prioridad: Prioridad::Media,
            costo_tiempo,
            arquetipos: vec![Arquetipo::Select],
            sql_dorada: "SELECT 1".to_string(),
            sql_inicial: None,
            requiere_orden: true,
            peso_correctitud: 0.6,
            peso_velocidad: 0.2,
            peso_practicas: 0.2,
            valor_base: 100,
            factor_reputacion,
        }
    }

    fn catalogo_de_prueba() -> Vec<Ticket> {
        vec![
            ticket_de_prueba("t1", 30, 0.5),
            ticket_de_prueba("t2", 30, 0.7),
            ticket_de_prueba("t3", 30, 1.0),
            ticket_de_prueba("t4", 30, 0.5),
            ticket_de_prueba("t5", 30, 0.7),
        ]
    }

    #[test]
    fn nuevo_saca_un_lote_de_3_y_llena_el_presupuesto() {
        let catalogo = catalogo_de_prueba();
        let (turno, siguiente_indice) = EstadoTurno::nuevo(&catalogo, 0);

        assert_eq!(turno.presupuesto_restante, 100);
        assert_eq!(turno.pendientes.len(), 3);
        assert_eq!(
            turno.pendientes.iter().map(|t| t.id).collect::<Vec<_>>(),
            vec!["t1", "t2", "t3"]
        );
        assert_eq!(siguiente_indice, 3);
    }

    #[test]
    fn nuevo_rota_el_catalogo_dando_la_vuelta() {
        let catalogo = catalogo_de_prueba();
        let (turno, siguiente_indice) = EstadoTurno::nuevo(&catalogo, 4);

        assert_eq!(
            turno.pendientes.iter().map(|t| t.id).collect::<Vec<_>>(),
            vec!["t5", "t1", "t2"]
        );
        assert_eq!(siguiente_indice, 2);
    }

    #[test]
    fn resolver_quita_el_ticket_y_consume_su_costo_de_tiempo() {
        let catalogo = catalogo_de_prueba();
        let (mut turno, _) = EstadoTurno::nuevo(&catalogo, 0);

        let resuelto = turno.resolver("t2", 30).expect("t2 debe estar pendiente");

        assert_eq!(resuelto.id, "t2");
        assert_eq!(turno.presupuesto_restante, 70);
        assert_eq!(
            turno.pendientes.iter().map(|t| t.id).collect::<Vec<_>>(),
            vec!["t1", "t3"]
        );
    }

    #[test]
    fn resolver_devuelve_none_si_el_id_no_esta_pendiente() {
        let catalogo = catalogo_de_prueba();
        let (mut turno, _) = EstadoTurno::nuevo(&catalogo, 0);

        assert!(turno.resolver("no_existe", 30).is_none());
        assert_eq!(turno.pendientes.len(), 3, "no debe alterar el lote si el id no está pendiente");
    }

    #[test]
    fn reintentar_reembolsa_el_tiempo_y_reinserta_el_ticket() {
        let catalogo = catalogo_de_prueba();
        let (mut turno, _) = EstadoTurno::nuevo(&catalogo, 0);

        let resuelto = turno.resolver("t2", 30).expect("t2 debe estar pendiente");
        assert_eq!(turno.presupuesto_restante, 70, "resolver ya descontó el costo de t2 (30)");

        turno.reintentar(resuelto, 30);

        assert_eq!(turno.presupuesto_restante, 100, "reintentar debe reembolsar el costo de tiempo");
        assert_eq!(
            turno.pendientes.iter().map(|t| t.id).collect::<Vec<_>>(),
            vec!["t1", "t3", "t2"],
            "el ticket reintentado se reinserta al final de pendientes"
        );
    }

    #[test]
    fn registrar_intento_incrementa_y_devuelve_el_conteo_por_ticket() {
        let catalogo = catalogo_de_prueba();
        let (mut turno, _) = EstadoTurno::nuevo(&catalogo, 0);

        assert_eq!(turno.registrar_intento("t1"), 1);
        assert_eq!(turno.registrar_intento("t1"), 2);
        assert_eq!(turno.registrar_intento("t2"), 1, "el conteo es independiente por ticket");
    }

    #[test]
    fn limpiar_intentos_borra_el_conteo_de_ese_ticket() {
        let catalogo = catalogo_de_prueba();
        let (mut turno, _) = EstadoTurno::nuevo(&catalogo, 0);

        turno.registrar_intento("t1");
        turno.registrar_intento("t1");
        turno.limpiar_intentos("t1");

        assert_eq!(
            turno.registrar_intento("t1"),
            1,
            "tras limpiar, el siguiente registro debe volver a empezar en 1"
        );
    }

    #[test]
    fn turno_agotado_es_false_si_algun_pendiente_es_costeable() {
        let catalogo = catalogo_de_prueba();
        let (turno, _) = EstadoTurno::nuevo(&catalogo, 0);
        assert!(!turno.turno_agotado(), "recién empezado, con 100 de presupuesto y tickets de 30");
    }

    #[test]
    fn turno_agotado_es_true_cuando_ningun_pendiente_es_costeable() {
        let catalogo = catalogo_de_prueba();
        let (mut turno, _) = EstadoTurno::nuevo(&catalogo, 0);
        turno.presupuesto_restante = 10;
        assert!(turno.turno_agotado(), "los 3 pendientes cuestan 30 cada uno, más que los 10 restantes");
    }

    #[test]
    fn turno_agotado_es_true_si_no_quedan_pendientes() {
        let catalogo = catalogo_de_prueba();
        let (mut turno, _) = EstadoTurno::nuevo(&catalogo, 0);
        turno.resolver("t1", 30);
        turno.resolver("t2", 30);
        turno.resolver("t3", 30);
        assert!(turno.turno_agotado());
    }

    #[test]
    fn escalar_pendientes_calcula_la_penalizacion_por_ticket() {
        let catalogo = catalogo_de_prueba();
        let (turno, _) = EstadoTurno::nuevo(&catalogo, 0);

        let escalamientos = turno.escalar_pendientes();

        assert_eq!(
            escalamientos,
            vec![
                Escalamiento { ticket_id: "t1", reputacion_perdida: 1.0 },
                Escalamiento { ticket_id: "t2", reputacion_perdida: 1.4 },
                Escalamiento { ticket_id: "t3", reputacion_perdida: 2.0 },
            ]
        );
    }

    #[test]
    fn nuevo_normaliza_un_indice_inicial_fuera_de_rango_con_modulo() {
        let catalogo = catalogo_de_prueba();
        let (turno, _) = EstadoTurno::nuevo(&catalogo, 7);
        assert_eq!(
            turno.pendientes.iter().map(|t| t.id).collect::<Vec<_>>(),
            vec!["t3", "t4", "t5"],
            "7 % 5 == 2, debe empezar en t3"
        );
    }

    #[test]
    fn nuevo_no_entra_en_panico_con_catalogo_vacio() {
        let catalogo: Vec<Ticket> = vec![];
        let (turno, siguiente_indice) = EstadoTurno::nuevo(&catalogo, 0);
        assert!(turno.pendientes.is_empty());
        assert_eq!(siguiente_indice, 0);
    }
}
