use crate::tickets::{Arquetipo, Ticket};
use crate::validation::Evaluacion;

/// Puntos de XP que otorga usar cada arquetipo SQL una vez, antes de escalar
/// por el puntaje final (Etapa 10: la dificultad del concepto define el XP
/// base — Join vale más que Select, Agregación más que Join).
fn xp_base_por_arquetipo(arquetipo: Arquetipo) -> i64 {
    match arquetipo {
        Arquetipo::Select => 10,
        Arquetipo::Join => 20,
        Arquetipo::Agregacion => 25,
    }
}

/// Resultado de aplicar la fórmula de economía (Etapa 12) a una entrega ya
/// evaluada (Plan 2) contra un ticket (Plan 3).
#[derive(Debug, Clone, PartialEq)]
pub struct Resultado {
    pub puntaje_base: f64,
    pub puntaje_final: f64,
    pub dinero_ganado: i64,
    pub reputacion_ganada: f64,
    pub xp_ganado: Vec<(Arquetipo, i64)>,
}

/// Calcula dinero/reputación/XP ganados por una entrega, siguiendo la
/// fórmula literal de la Etapa 12. `multiplicador_perks` representa el
/// efecto de los perks activos del jugador — fijo en 1.0 hasta que exista el
/// sistema RPG real (Etapa 13, plan posterior). Si la entrega es incorrecta,
/// no se otorga dinero/reputación/XP (la penalización por tickets escalados
/// es solo de reputación y depende del sistema de turnos, Etapa 11-A, no
/// construido — este cálculo no la implementa).
pub fn calcular(evaluacion: &Evaluacion, ticket: &Ticket, multiplicador_perks: f64) -> Resultado {
    let puntaje_base = evaluacion.puntaje_correctitud * ticket.peso_correctitud
        + evaluacion.puntaje_velocidad * ticket.peso_velocidad
        + evaluacion.puntaje_practicas * ticket.peso_practicas;
    let puntaje_final = puntaje_base * multiplicador_perks;

    if !evaluacion.correcta {
        return Resultado {
            puntaje_base,
            puntaje_final,
            dinero_ganado: 0,
            reputacion_ganada: 0.0,
            xp_ganado: Vec::new(),
        };
    }

    let dinero_ganado = (puntaje_final * ticket.valor_base as f64 / 100.0).round() as i64;
    let reputacion_ganada = puntaje_final * ticket.factor_reputacion / 100.0;
    let xp_ganado = ticket
        .arquetipos
        .iter()
        .map(|&arquetipo| {
            let xp = (xp_base_por_arquetipo(arquetipo) as f64 * puntaje_final / 100.0).round() as i64;
            (arquetipo, xp)
        })
        .collect();

    Resultado {
        puntaje_base,
        puntaje_final,
        dinero_ganado,
        reputacion_ganada,
        xp_ganado,
    }
}

/// Estado acumulado del jugador (Etapa 12): dinero, reputación y XP por
/// arquetipo ganados a lo largo de la partida, más el stub de un solo perk
/// heredado del spike original (Etapa 13 lo reemplaza en un plan posterior).
#[derive(Debug, Clone, Default)]
pub struct EstadoJugador {
    pub dinero: i64,
    pub reputacion: f64,
    pub xp_por_arquetipo: Vec<(Arquetipo, i64)>,
    pub perk_desbloqueado: bool,
}

/// Umbral de reputación para ascender de Becario a Auxiliar de Sistemas en
/// Hospital Arcángel (Etapa 10). El ascenso real (superar el mini-boss,
/// cambiar de rango) es responsabilidad de un plan posterior — esta
/// constante solo define cuándo se cumple la condición de reputación.
const UMBRAL_ASCENSO_AUXILIAR: f64 = 500.0;

impl EstadoJugador {
    /// Aplica el resultado de una entrega (Etapa 12): acumula dinero,
    /// reputación y XP por arquetipo sobre el estado existente.
    pub fn aplicar_resultado(&mut self, resultado: &Resultado) {
        self.dinero += resultado.dinero_ganado;
        self.reputacion += resultado.reputacion_ganada;
        for &(arquetipo, xp) in &resultado.xp_ganado {
            match self.xp_por_arquetipo.iter_mut().find(|(a, _)| *a == arquetipo) {
                Some((_, existente)) => *existente += xp,
                None => self.xp_por_arquetipo.push((arquetipo, xp)),
            }
        }
    }

    /// Etapa 10: señal de que la reputación ya cruzó el umbral de ascenso —
    /// no dispara ningún cambio de estado por sí sola.
    pub fn puede_ascender(&self) -> bool {
        self.reputacion >= UMBRAL_ASCENSO_AUXILIAR
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tickets::{Prioridad, TipoTicket};

    fn ticket_de_prueba(arquetipos: Vec<Arquetipo>) -> Ticket {
        Ticket {
            id: "ticket_de_prueba",
            tipo: TipoTicket::ReporteAnalisis,
            solicitante: "Alguien",
            motivo: "un motivo".to_string(),
            solicitud: "una solicitud".to_string(),
            prioridad: Prioridad::Media,
            costo_tiempo: 10,
            arquetipos,
            sql_dorada: "SELECT 1".to_string(),
            sql_inicial: None,
            requiere_orden: true,
            peso_correctitud: 0.6,
            peso_velocidad: 0.2,
            peso_practicas: 0.2,
            valor_base: 100,
            factor_reputacion: 0.5,
        }
    }

    fn evaluacion_perfecta() -> Evaluacion {
        Evaluacion {
            correcta: true,
            puntaje_correctitud: 100.0,
            puntaje_velocidad: 100.0,
            puntaje_practicas: 100.0,
            comentario_mentor: None,
        }
    }

    #[test]
    fn calcular_ticket_correcto_otorga_recompensa_proporcional() {
        let ticket = ticket_de_prueba(vec![Arquetipo::Select]);
        let evaluacion = evaluacion_perfecta();

        let resultado = calcular(&evaluacion, &ticket, 1.0);

        assert_eq!(resultado.puntaje_base, 100.0);
        assert_eq!(resultado.puntaje_final, 100.0);
        assert_eq!(resultado.dinero_ganado, 100);
        assert_eq!(resultado.reputacion_ganada, 0.5);
        assert_eq!(resultado.xp_ganado, vec![(Arquetipo::Select, 10)]);
    }

    #[test]
    fn calcular_ticket_incorrecto_no_otorga_dinero_ni_reputacion_ni_xp() {
        let ticket = ticket_de_prueba(vec![Arquetipo::Select]);
        let mut evaluacion = evaluacion_perfecta();
        evaluacion.correcta = false;

        let resultado = calcular(&evaluacion, &ticket, 1.0);

        assert_eq!(resultado.dinero_ganado, 0);
        assert_eq!(resultado.reputacion_ganada, 0.0);
        assert!(resultado.xp_ganado.is_empty());
        assert_eq!(
            resultado.puntaje_base, 100.0,
            "el puntaje de calidad se calcula aunque el resultado sea incorrecto"
        );
    }

    #[test]
    fn calcular_reparte_xp_entre_varios_arquetipos() {
        let mut ticket = ticket_de_prueba(vec![Arquetipo::Join, Arquetipo::Agregacion]);
        ticket.peso_correctitud = 0.4;
        ticket.peso_velocidad = 0.3;
        ticket.peso_practicas = 0.3;
        let evaluacion = evaluacion_perfecta();

        let resultado = calcular(&evaluacion, &ticket, 1.0);

        assert_eq!(
            resultado.xp_ganado,
            vec![(Arquetipo::Join, 20), (Arquetipo::Agregacion, 25)]
        );
    }

    #[test]
    fn calcular_aplica_el_multiplicador_de_perks() {
        let ticket = ticket_de_prueba(vec![Arquetipo::Select]);
        let evaluacion = evaluacion_perfecta();

        let resultado = calcular(&evaluacion, &ticket, 2.0);

        assert_eq!(resultado.puntaje_final, 200.0);
        assert_eq!(resultado.dinero_ganado, 200);
    }

    #[test]
    fn aplicar_resultado_acumula_dinero_reputacion_y_xp() {
        let mut estado = EstadoJugador::default();
        let resultado = Resultado {
            puntaje_base: 100.0,
            puntaje_final: 100.0,
            dinero_ganado: 100,
            reputacion_ganada: 0.5,
            xp_ganado: vec![(Arquetipo::Select, 10)],
        };

        estado.aplicar_resultado(&resultado);

        assert_eq!(estado.dinero, 100);
        assert_eq!(estado.reputacion, 0.5);
        assert_eq!(estado.xp_por_arquetipo, vec![(Arquetipo::Select, 10)]);
    }

    #[test]
    fn aplicar_resultado_suma_xp_al_mismo_arquetipo_en_llamadas_sucesivas() {
        let mut estado = EstadoJugador::default();
        let resultado = Resultado {
            puntaje_base: 100.0,
            puntaje_final: 100.0,
            dinero_ganado: 100,
            reputacion_ganada: 0.5,
            xp_ganado: vec![(Arquetipo::Select, 10)],
        };

        estado.aplicar_resultado(&resultado);
        estado.aplicar_resultado(&resultado);

        assert_eq!(estado.dinero, 200);
        assert_eq!(
            estado.xp_por_arquetipo,
            vec![(Arquetipo::Select, 20)],
            "debe acumular en la misma entrada, no duplicarla"
        );
    }

    #[test]
    fn puede_ascender_es_false_bajo_el_umbral_y_true_al_cruzarlo() {
        let mut estado = EstadoJugador::default();
        assert!(!estado.puede_ascender());

        estado.reputacion = 499.9;
        assert!(!estado.puede_ascender());

        estado.reputacion = 500.0;
        assert!(estado.puede_ascender());
    }
}
