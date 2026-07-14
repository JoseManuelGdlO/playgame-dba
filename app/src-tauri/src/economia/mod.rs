use crate::perks::{Efecto, Perk};
use crate::tickets::{Arquetipo, Rango, Ticket};
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
/// fórmula literal de la Etapa 12. `multiplicador_dinero`/`multiplicador_reputacion`
/// representan el efecto de los perks de la categoría Billetera y Fama
/// actualmente equipados (Etapa 13, Plan 5) — cada uno 1.0 si no hay ninguno
/// activo. A diferencia del multiplicador único que existía antes de la
/// Etapa 13, cada recurso escala por separado porque los perks reales
/// afectan solo uno de los dos ("Buena Fama" solo reputación, "Bono Bajo la
/// Mesa" solo dinero) — un multiplicador compartido los volvería
/// indistinguibles. Si la entrega es incorrecta, no se otorga
/// dinero/reputación/XP (la penalización por tickets escalados es solo de
/// reputación y depende del sistema de turnos, Etapa 11-A, no construido —
/// este cálculo no la implementa).
pub fn calcular(
    evaluacion: &Evaluacion,
    ticket: &Ticket,
    multiplicador_dinero: f64,
    multiplicador_reputacion: f64,
) -> Resultado {
    let puntaje_base = evaluacion.puntaje_correctitud * ticket.peso_correctitud
        + evaluacion.puntaje_velocidad * ticket.peso_velocidad
        + evaluacion.puntaje_practicas * ticket.peso_practicas;
    // Ya no hay un multiplicador genérico compartido (Etapa 13, Plan 5): los
    // perks con efecto real escalan dinero/reputación por separado, más
    // abajo — puntaje_final se mantiene igual a puntaje_base.
    let puntaje_final = puntaje_base;

    if !evaluacion.correcta {
        return Resultado {
            puntaje_base,
            puntaje_final,
            dinero_ganado: 0,
            reputacion_ganada: 0.0,
            xp_ganado: Vec::new(),
        };
    }

    let dinero_ganado =
        (puntaje_final * ticket.valor_base as f64 / 100.0 * multiplicador_dinero).round() as i64;
    let reputacion_ganada =
        puntaje_final * ticket.factor_reputacion / 100.0 * multiplicador_reputacion;
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

/// Estado acumulado del jugador (Etapa 12/13): dinero, reputación, XP por
/// arquetipo, rango de carrera, y los perks desbloqueados/equipados.
#[derive(Debug, Clone, Default)]
pub struct EstadoJugador {
    pub dinero: i64,
    pub reputacion: f64,
    pub xp_por_arquetipo: Vec<(Arquetipo, i64)>,
    pub rango: Rango,
    pub perks_desbloqueados: Vec<&'static str>,
    pub perks_equipados: Vec<&'static str>,
}

/// Umbral de reputación para ascender de Becario a Auxiliar de Sistemas en
/// Hospital Arcángel (Etapa 10). El ascenso de este plan usa únicamente esta
/// condición — el mini-boss de la empresa (Etapa 11-G) queda fuera de
/// alcance y se puede añadir como condición adicional en un plan posterior
/// sin romper esta lógica (Plan 7).
/// Playtest: ~0.5 rep por ticket Select. Umbral ~2.5 ≈ 4–5 tickets bien hechos
/// en Hospital Arcángel antes del Auditor (los perks piden 3–7; algunos se
/// desbloquean cerca del climax o justo después).
pub(crate) const UMBRAL_ASCENSO_AUXILIAR: f64 = 2.5;

impl EstadoJugador {
    /// Aplica el resultado de una entrega (Etapa 12): acumula dinero,
    /// reputación y XP por arquetipo sobre el estado existente, y dispara el
    /// ascenso automático de rango si la reputación acumulada ya cruzó el
    /// umbral (Etapa 10, Plan 7). Devuelve `true` solo en la entrega exacta
    /// en la que el ascenso ocurre, para que el llamador pueda anunciarlo.
    pub fn aplicar_resultado(&mut self, resultado: &Resultado) -> bool {
        self.dinero += resultado.dinero_ganado;
        self.reputacion += resultado.reputacion_ganada;
        for &(arquetipo, xp) in &resultado.xp_ganado {
            match self.xp_por_arquetipo.iter_mut().find(|(a, _)| *a == arquetipo) {
                Some((_, existente)) => *existente += xp,
                None => self.xp_por_arquetipo.push((arquetipo, xp)),
            }
        }
        if self.rango == Rango::Becario && self.puede_ascender() {
            self.rango = Rango::AuxiliarDeSistemas;
            true
        } else {
            false
        }
    }

    /// Etapa 10: señal de que la reputación ya cruzó el umbral de ascenso —
    /// no dispara ningún cambio de estado por sí sola.
    pub fn puede_ascender(&self) -> bool {
        self.reputacion >= UMBRAL_ASCENSO_AUXILIAR
    }

    /// Máximo de perks equipados simultáneamente (Etapa 13, Plan 7): 2 slots
    /// para Becario, 3 para Auxiliar de Sistemas (hito de slot de esa
    /// etapa). La escalera completa de hasta 7 slots por rango es de un plan
    /// posterior, junto con más rangos (Fase 1+).
    pub fn max_slots(&self) -> usize {
        match self.rango {
            Rango::Becario => 2,
            Rango::AuxiliarDeSistemas => 3,
        }
    }

    /// Aplica una penalización de reputación (Etapa 11-A: tickets escalados
    /// por no atenderse a tiempo en el turno) — nunca baja de 0.
    pub fn aplicar_penalizacion(&mut self, reputacion_perdida: f64) {
        self.reputacion = (self.reputacion - reputacion_perdida).max(0.0);
    }

    /// Etapa 13: un perk se desbloquea cuando se cumplen 3 condiciones a la
    /// vez — dinero suficiente, reputación mínima, y maestría (XP) suficiente
    /// en el arquetipo que ese perk requiere.
    pub fn puede_desbloquear(&self, perk: &Perk) -> bool {
        let xp_en_arquetipo = self
            .xp_por_arquetipo
            .iter()
            .find(|&&(a, _)| a == perk.arquetipo_requerido)
            .map(|&(_, xp)| xp)
            .unwrap_or(0);

        self.dinero >= perk.costo_dinero
            && self.reputacion >= perk.reputacion_minima
            && xp_en_arquetipo >= perk.xp_minimo
    }

    /// Desbloquea un perk permanentemente (Etapa 13): gasta el dinero, no
    /// toca la reputación ni el XP (esos solo se verifican, nunca se
    /// consumen). Idempotente si ya estaba desbloqueado.
    pub fn desbloquear_perk(&mut self, catalogo: &[Perk], id: &str) -> Result<(), String> {
        if self.perks_desbloqueados.contains(&id) {
            return Ok(());
        }
        let perk = catalogo
            .iter()
            .find(|p| p.id == id)
            .ok_or_else(|| format!("perk desconocido: {id}"))?;
        if !self.puede_desbloquear(perk) {
            return Err(format!("No cumples los requisitos para '{}' todavía.", perk.nombre));
        }
        self.dinero -= perk.costo_dinero;
        self.perks_desbloqueados.push(perk.id);
        Ok(())
    }

    /// Equipa un perk ya desbloqueado (Etapa 11-D: equipar es gratis).
    /// Falla si no está desbloqueado, o si ya se ocuparon los slots
    /// disponibles para el rango actual (Etapa 13, Plan 7). Idempotente si
    /// ya estaba equipado.
    pub fn equipar_perk(&mut self, id: &str) -> Result<(), String> {
        if !self.perks_desbloqueados.contains(&id) {
            return Err(format!("'{id}' no está desbloqueado todavía."));
        }
        if self.perks_equipados.contains(&id) {
            return Ok(());
        }
        let max_slots = self.max_slots();
        if self.perks_equipados.len() >= max_slots {
            return Err(format!("Ya tienes {max_slots} perks equipados — desequipa uno primero."));
        }
        let id_estatico = self
            .perks_desbloqueados
            .iter()
            .find(|&&d| d == id)
            .copied()
            .expect("ya se confirmó arriba que está desbloqueado");
        self.perks_equipados.push(id_estatico);
        Ok(())
    }

    /// Desequipa un perk (gratis, Etapa 11-D). No falla si no estaba
    /// equipado.
    pub fn desequipar_perk(&mut self, id: &str) {
        self.perks_equipados.retain(|&equipado| equipado != id);
    }

    /// Multiplicador de dinero (Etapa 12/13) por los perks "Billetera y Fama"
    /// actualmente equipados — 1.0 si ninguno está activo.
    pub fn multiplicador_dinero(&self, catalogo: &[Perk]) -> f64 {
        let mut multiplicador = 1.0;
        for &id in &self.perks_equipados {
            if let Some(perk) = catalogo.iter().find(|p| p.id == id) {
                if let Efecto::BonoDinero(bono) = perk.efecto {
                    multiplicador *= 1.0 + bono;
                }
            }
        }
        multiplicador
    }

    /// Multiplicador de reputación (Etapa 12/13) por los perks "Billetera y
    /// Fama" actualmente equipados — 1.0 si ninguno está activo.
    pub fn multiplicador_reputacion(&self, catalogo: &[Perk]) -> f64 {
        let mut multiplicador = 1.0;
        for &id in &self.perks_equipados {
            if let Some(perk) = catalogo.iter().find(|p| p.id == id) {
                if let Efecto::BonoReputacion(bono) = perk.efecto {
                    multiplicador *= 1.0 + bono;
                }
            }
        }
        multiplicador
    }

    /// Intentos extra por ticket (Plan 17) por los perks equipados que dan
    /// `BonoIntentos` — 0 si ninguno está activo. Se suma, no se multiplica
    /// (a diferencia de dinero/reputación): dos perks de este tipo, si
    /// alguna vez existieran, sumarían sus bonos en vez de componerse.
    pub fn intentos_extra(&self, catalogo: &[Perk]) -> u32 {
        let mut extra = 0;
        for &id in &self.perks_equipados {
            if let Some(perk) = catalogo.iter().find(|p| p.id == id) {
                if let Efecto::BonoIntentos(bono) = perk.efecto {
                    extra += bono;
                }
            }
        }
        extra
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::perks;
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

        let resultado = calcular(&evaluacion, &ticket, 1.0, 1.0);

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

        let resultado = calcular(&evaluacion, &ticket, 1.0, 1.0);

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

        let resultado = calcular(&evaluacion, &ticket, 1.0, 1.0);

        assert_eq!(
            resultado.xp_ganado,
            vec![(Arquetipo::Join, 20), (Arquetipo::Agregacion, 25)]
        );
    }

    #[test]
    fn calcular_aplica_los_multiplicadores_de_dinero_y_reputacion_por_separado() {
        let ticket = ticket_de_prueba(vec![Arquetipo::Select]);
        let evaluacion = evaluacion_perfecta();

        let resultado = calcular(&evaluacion, &ticket, 2.0, 1.5);

        assert_eq!(
            resultado.puntaje_final, 100.0,
            "puntaje_final ya no lleva un multiplicador genérico"
        );
        assert_eq!(resultado.dinero_ganado, 200, "100 (valor_base) * 2.0");
        assert_eq!(resultado.reputacion_ganada, 0.75, "0.5 (factor_reputacion) * 1.5");
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

        estado.reputacion = 2.4;
        assert!(!estado.puede_ascender());

        estado.reputacion = 2.5;
        assert!(estado.puede_ascender());
    }

    #[test]
    fn aplicar_penalizacion_resta_reputacion_sin_bajar_de_cero() {
        let mut estado = EstadoJugador::default();
        estado.reputacion = 5.0;

        estado.aplicar_penalizacion(2.0);
        assert_eq!(estado.reputacion, 3.0);

        estado.aplicar_penalizacion(10.0);
        assert_eq!(estado.reputacion, 0.0, "no debe bajar de 0");
    }

    #[test]
    fn calcular_distingue_cual_peso_multiplica_a_cual_puntaje() {
        let mut ticket = ticket_de_prueba(vec![Arquetipo::Select]);
        ticket.peso_correctitud = 0.5;
        ticket.peso_velocidad = 0.3125;
        ticket.peso_practicas = 0.1875;
        let evaluacion = Evaluacion {
            correcta: true,
            puntaje_correctitud: 80.0,
            puntaje_velocidad: 64.0,
            puntaje_practicas: 32.0,
            comentario_mentor: None,
        };

        let resultado = calcular(&evaluacion, &ticket, 1.0, 1.0);

        assert_eq!(resultado.puntaje_base, 66.0);
        assert_eq!(resultado.puntaje_final, 66.0);
    }

    #[test]
    fn calcular_redondea_dinero_y_xp_cuando_el_puntaje_final_es_fraccionario() {
        let mut ticket = ticket_de_prueba(vec![Arquetipo::Select]);
        ticket.peso_correctitud = 0.625;
        ticket.peso_velocidad = 0.25;
        ticket.peso_practicas = 0.125;
        let evaluacion = Evaluacion {
            correcta: true,
            puntaje_correctitud: 70.0,
            puntaje_velocidad: 51.0,
            puntaje_practicas: 11.0,
            comentario_mentor: None,
        };

        let resultado = calcular(&evaluacion, &ticket, 1.0, 1.0);

        assert_eq!(resultado.puntaje_base, 57.875);
        assert_eq!(resultado.puntaje_final, 57.875);
        assert_eq!(resultado.dinero_ganado, 58);
        assert_eq!(resultado.xp_ganado, vec![(Arquetipo::Select, 6)]);
    }

    #[test]
    fn aplicar_resultado_agrega_arquetipo_nuevo_sin_afectar_los_existentes() {
        let mut estado = EstadoJugador::default();
        let resultado_select = Resultado {
            puntaje_base: 100.0,
            puntaje_final: 100.0,
            dinero_ganado: 100,
            reputacion_ganada: 0.5,
            xp_ganado: vec![(Arquetipo::Select, 10)],
        };
        let resultado_join = Resultado {
            puntaje_base: 100.0,
            puntaje_final: 100.0,
            dinero_ganado: 50,
            reputacion_ganada: 0.3,
            xp_ganado: vec![(Arquetipo::Join, 20)],
        };

        estado.aplicar_resultado(&resultado_select);
        estado.aplicar_resultado(&resultado_join);

        assert_eq!(
            estado.xp_por_arquetipo,
            vec![(Arquetipo::Select, 10), (Arquetipo::Join, 20)],
            "la entrada existente de Select no debe alterarse y Join debe agregarse como nueva entrada"
        );
    }

    #[test]
    fn puede_desbloquear_requiere_dinero_reputacion_y_xp_simultaneamente() {
        let perk = perks::buscar("buena_fama").expect("buena_fama debe existir en el catálogo");

        let mut estado = EstadoJugador::default();
        assert!(!estado.puede_desbloquear(perk), "sin nada, no debe poder desbloquear");

        estado.dinero = perk.costo_dinero;
        assert!(!estado.puede_desbloquear(perk), "dinero solo no basta");

        estado.reputacion = perk.reputacion_minima;
        assert!(!estado.puede_desbloquear(perk), "falta el XP del arquetipo requerido");

        estado.xp_por_arquetipo.push((perk.arquetipo_requerido, perk.xp_minimo));
        assert!(estado.puede_desbloquear(perk), "con las 3 condiciones cumplidas ya debe poder");
    }

    #[test]
    fn desbloquear_perk_gasta_dinero_y_es_idempotente() {
        let catalogo = perks::catalogo();
        let perk = perks::buscar("buena_fama").unwrap();

        let mut estado = EstadoJugador::default();
        estado.dinero = perk.costo_dinero;
        estado.reputacion = perk.reputacion_minima;
        estado.xp_por_arquetipo.push((perk.arquetipo_requerido, perk.xp_minimo));

        estado.desbloquear_perk(catalogo, "buena_fama").expect("debe poder desbloquear");
        assert_eq!(estado.dinero, 0);
        assert!(estado.perks_desbloqueados.contains(&"buena_fama"));

        estado.dinero = 1000;
        estado.desbloquear_perk(catalogo, "buena_fama").expect("ya desbloqueado, no debe fallar");
        assert_eq!(estado.dinero, 1000, "no debe cobrar de nuevo un perk ya desbloqueado");
    }

    #[test]
    fn desbloquear_perk_falla_si_no_cumple_los_requisitos() {
        let catalogo = perks::catalogo();
        let mut estado = EstadoJugador::default();
        assert!(estado.desbloquear_perk(catalogo, "buena_fama").is_err());
    }

    #[test]
    fn equipar_perk_respeta_el_limite_de_2_slots() {
        let mut estado = EstadoJugador::default();
        estado.perks_desbloqueados = vec!["instinto", "rayos_x", "piloto_automatico"];

        estado.equipar_perk("instinto").unwrap();
        estado.equipar_perk("rayos_x").unwrap();
        let resultado = estado.equipar_perk("piloto_automatico");

        assert!(resultado.is_err(), "un tercer perk no debe caber en 2 slots");
        assert_eq!(estado.perks_equipados, vec!["instinto", "rayos_x"]);
    }

    #[test]
    fn equipar_perk_falla_si_no_esta_desbloqueado() {
        let mut estado = EstadoJugador::default();
        assert!(estado.equipar_perk("instinto").is_err());
    }

    #[test]
    fn desequipar_perk_libera_un_slot() {
        let mut estado = EstadoJugador::default();
        estado.perks_desbloqueados = vec!["instinto", "rayos_x"];
        estado.equipar_perk("instinto").unwrap();
        estado.equipar_perk("rayos_x").unwrap();

        estado.desequipar_perk("instinto");
        assert_eq!(estado.perks_equipados, vec!["rayos_x"]);

        estado.perks_desbloqueados.push("piloto_automatico");
        estado.equipar_perk("piloto_automatico").unwrap();
        assert_eq!(estado.perks_equipados, vec!["rayos_x", "piloto_automatico"]);
    }

    #[test]
    fn multiplicador_dinero_solo_cuenta_perks_equipados_no_solo_desbloqueados() {
        let catalogo = perks::catalogo();
        let mut estado = EstadoJugador::default();
        estado.perks_desbloqueados.push("bono_bajo_la_mesa");

        assert_eq!(
            estado.multiplicador_dinero(catalogo),
            1.0,
            "desbloqueado pero no equipado no debe aplicar"
        );

        estado.equipar_perk("bono_bajo_la_mesa").unwrap();
        assert_eq!(estado.multiplicador_dinero(catalogo), 1.2, "equipado, +20%");
    }

    #[test]
    fn multiplicador_reputacion_solo_cuenta_perks_equipados_no_solo_desbloqueados() {
        let catalogo = perks::catalogo();
        let mut estado = EstadoJugador::default();
        estado.perks_desbloqueados.push("buena_fama");

        assert_eq!(estado.multiplicador_reputacion(catalogo), 1.0);

        estado.equipar_perk("buena_fama").unwrap();
        assert_eq!(estado.multiplicador_reputacion(catalogo), 1.2);
    }

    #[test]
    fn intentos_extra_es_0_sin_el_perk_y_2_con_el_equipado() {
        let catalogo = perks::catalogo();
        let mut estado = EstadoJugador::default();
        estado.perks_desbloqueados.push("segunda_opinion");

        assert_eq!(
            estado.intentos_extra(catalogo),
            0,
            "desbloqueado pero no equipado no debe aplicar"
        );

        estado.equipar_perk("segunda_opinion").unwrap();
        assert_eq!(estado.intentos_extra(catalogo), 2, "equipado, +2 intentos");
    }

    #[test]
    fn desbloquear_perk_falla_con_id_desconocido() {
        let mut estado = EstadoJugador::default();

        let resultado = estado.desbloquear_perk(perks::catalogo(), "perk_que_no_existe");

        assert!(resultado.is_err());
        assert!(resultado.unwrap_err().contains("perk_que_no_existe"));
    }

    #[test]
    fn equipar_perk_dos_veces_es_idempotente() {
        let mut estado = EstadoJugador::default();
        estado.perks_desbloqueados = vec!["instinto"];
        estado.equipar_perk("instinto").unwrap();

        let resultado = estado.equipar_perk("instinto");

        assert!(resultado.is_ok());
        assert_eq!(
            estado.perks_equipados,
            vec!["instinto"],
            "re-equipar un perk ya equipado no debe duplicar la entrada"
        );
    }

    #[test]
    fn desequipar_perk_sin_estar_equipado_no_hace_nada() {
        let mut estado = EstadoJugador::default();

        estado.desequipar_perk("instinto");

        assert!(estado.perks_equipados.is_empty());
    }

    #[test]
    fn multiplicadores_de_dinero_y_reputacion_se_calculan_independientemente_con_ambos_perks_equipados(
    ) {
        let catalogo = perks::catalogo();
        let mut estado = EstadoJugador::default();
        estado.perks_desbloqueados = vec!["buena_fama", "bono_bajo_la_mesa"];
        estado.equipar_perk("buena_fama").unwrap();
        estado.equipar_perk("bono_bajo_la_mesa").unwrap();

        assert_eq!(
            estado.multiplicador_dinero(catalogo),
            1.2,
            "solo bono_bajo_la_mesa afecta dinero"
        );
        assert_eq!(
            estado.multiplicador_reputacion(catalogo),
            1.2,
            "solo buena_fama afecta reputación"
        );
    }

    #[test]
    fn aplicar_resultado_asciende_a_auxiliar_al_cruzar_el_umbral_una_sola_vez() {
        let mut estado = EstadoJugador::default();
        assert_eq!(estado.rango, Rango::Becario);
        let resultado = Resultado {
            puntaje_base: 100.0,
            puntaje_final: 100.0,
            dinero_ganado: 100,
            reputacion_ganada: 2.5,
            xp_ganado: vec![],
        };

        let ascendio = estado.aplicar_resultado(&resultado);

        assert!(ascendio, "debe ascender en la entrega exacta que cruza el umbral");
        assert_eq!(estado.rango, Rango::AuxiliarDeSistemas);

        let resultado_siguiente = Resultado {
            puntaje_base: 100.0,
            puntaje_final: 100.0,
            dinero_ganado: 50,
            reputacion_ganada: 1.0,
            xp_ganado: vec![],
        };
        let ascendio_de_nuevo = estado.aplicar_resultado(&resultado_siguiente);
        assert!(!ascendio_de_nuevo, "ya ascendido, no debe volver a dispararse");
    }

    #[test]
    fn aplicar_resultado_asciende_aunque_la_reputacion_salte_muy_por_encima_del_umbral() {
        // Guarda contra que alguien "arregle" `puede_ascender` cambiando el
        // `>=` por un `==`: un solo ticket con una ganancia de reputación
        // grande (muy por encima del umbral de 2.5) debe seguir
        // disparando el ascenso en esa misma entrega.
        let mut estado = EstadoJugador::default();
        let resultado = Resultado {
            puntaje_base: 100.0,
            puntaje_final: 100.0,
            dinero_ganado: 500,
            reputacion_ganada: 750.0,
            xp_ganado: vec![],
        };

        let ascendio = estado.aplicar_resultado(&resultado);

        assert!(ascendio, "un salto de reputación que sobrepasa el umbral en una sola entrega debe ascender");
        assert_eq!(estado.rango, Rango::AuxiliarDeSistemas);
    }

    #[test]
    fn max_slots_es_2_para_becario_y_3_para_auxiliar_de_sistemas() {
        let mut estado = EstadoJugador::default();
        assert_eq!(estado.max_slots(), 2);

        estado.rango = Rango::AuxiliarDeSistemas;
        assert_eq!(estado.max_slots(), 3);
    }

    #[test]
    fn equipar_perk_permite_un_tercer_slot_para_auxiliar_de_sistemas() {
        let mut estado = EstadoJugador::default();
        estado.rango = Rango::AuxiliarDeSistemas;
        estado.perks_desbloqueados = vec!["instinto", "rayos_x", "piloto_automatico"];

        estado.equipar_perk("instinto").unwrap();
        estado.equipar_perk("rayos_x").unwrap();
        estado.equipar_perk("piloto_automatico").unwrap();

        assert_eq!(estado.perks_equipados, vec!["instinto", "rayos_x", "piloto_automatico"]);
    }
}
