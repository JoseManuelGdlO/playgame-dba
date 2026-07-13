mod db;
mod economia;
mod guardado;
mod perks;
mod tickets;
// `buscar_pendiente` (usado antes por `resolver_ticket`) quedó sin llamadores
// tras el fix de doble envío en `resolver_ticket` (ver ese comando): ahora se
// usa `resolver` como operación atómica de "check-and-remove" en su lugar.
// Se permite dead_code en vez de borrar el método de `turno/mod.rs` — ese
// archivo queda fuera del alcance de este fix y el método sigue siendo una
// utilidad de lectura legítima de `EstadoTurno`.
#[allow(dead_code)]
mod turno;
mod validation;

use std::sync::Mutex;
use tauri::Manager;

/// Pool de conexión al Postgres embebido, gestionado por Tauri. `Mutex` en
/// vez de un campo suelto (Etapa 11-G, Plan 8) porque
/// `confirmar_transicion_agencia` necesita poder reemplazarlo en caliente al
/// cambiar de empresa; `PgPool` es barato de clonar (handle basado en Arc
/// internamente), así que los comandos que lo usan clonan la copia vigente
/// en vez de tener que mantener el lock abierto durante un `.await`.
struct AppState(Mutex<sqlx::PgPool>);

/// Estado de economía del jugador (Etapa 12/13), gestionado por Tauri.
struct Jugador(Mutex<economia::EstadoJugador>);

/// Mantiene vivo el servidor Postgres embebido y permite detenerlo al salir.
struct EmbeddedPostgres(Mutex<Option<postgresql_embedded::PostgreSQL>>);

/// Evita que dos llamadas concurrentes a `confirmar_transicion_agencia`
/// corran la transición dos veces (Etapa 11-G, Plan 8 — hallazgo de
/// revisión): sin esto, dos invocaciones simultáneas podrían pasar ambas el
/// check de `fase == ArcoCompletado` antes de que la primera termine de
/// escribir el nuevo estado. `AtomicBool` en vez de otro `Mutex` porque
/// necesitamos poder consultarlo/liberarlo sin arrastrar un guard a través
/// del `.await` de la transición.
struct TransicionEnCurso(std::sync::atomic::AtomicBool);

/// Fase del arco de la empresa activa (Etapa 7/11-G, Plan 8): trabajo
/// normal, el lote dedicado del mini-boss, o el arco ya completo (esperando
/// que el jugador confirme la transición de la Agencia). Deliberadamente
/// específica del único mini-boss del MVP — no es un sistema genérico para
/// más empresas (Fase 1+).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum FaseArco {
    TrabajoNormal,
    MiniBoss,
    ArcoCompletado,
}

/// El catálogo completo de la empresa activa, el índice de rotación para el
/// próximo turno, el turno (bandeja) actual (Etapa 11-A) — reemplaza la
/// selección round-robin simple de un solo "ticket actual" (Plan 3) — y la
/// fase del arco de esa empresa (Etapa 7/11-G, Plan 8).
struct TurnoManejado {
    catalogo: Vec<tickets::Ticket>,
    indice_siguiente: usize,
    actual: turno::EstadoTurno,
    fase: FaseArco,
}

impl TurnoManejado {
    /// Escala los tickets pendientes del turno actual (penaliza reputación)
    /// y arranca el turno siguiente — usado tanto cuando el presupuesto se
    /// agota como cuando el jugador cierra el día manualmente (Etapa 11-A).
    /// El lote nuevo se filtra por el rango actual del jugador (Etapa 10,
    /// Plan 7): un ascenso a mitad de turno no reordena la bandeja ya
    /// mostrada, pero el turno siguiente ya refleja el catálogo desbloqueado.
    fn escalar_y_avanzar(&mut self, jugador: &mut economia::EstadoJugador) {
        for escalamiento in self.actual.escalar_pendientes() {
            jugador.aplicar_penalizacion(escalamiento.reputacion_perdida);
        }
        let elegibles = tickets::tickets_elegibles(&self.catalogo, jugador.rango);
        let (nuevo_turno, siguiente_indice) = turno::EstadoTurno::nuevo(&elegibles, self.indice_siguiente);
        self.actual = nuevo_turno;
        self.indice_siguiente = siguiente_indice;
    }

    /// Aplica las consecuencias de haber resuelto un ticket sobre la fase
    /// del arco (Etapa 7/11-G, Plan 8), llamado desde `resolver_ticket`
    /// justo después de `aplicar_resultado`: si `ascendio` es `true`, el lote
    /// normal se reemplaza de inmediato por los 2 tickets del mini-boss (sin
    /// mezclarse con lo que quedaba pendiente); si ya estábamos en el lote
    /// del mini-boss y se acaba de vaciar, el arco queda completo; en
    /// cualquier otro caso, se comporta como antes (`escalar_y_avanzar`
    /// cuando el turno normal se agota o se vacía).
    fn actualizar_fase(&mut self, ascendio: bool, jugador: &mut economia::EstadoJugador) {
        if ascendio {
            let (turno_mini_boss, _) = turno::EstadoTurno::nuevo(&tickets::mini_boss_hospital_arcangel(), 0);
            self.actual = turno_mini_boss;
            self.fase = FaseArco::MiniBoss;
        } else if self.fase == FaseArco::MiniBoss && self.actual.pendientes.is_empty() {
            self.fase = FaseArco::ArcoCompletado;
        } else if self.fase == FaseArco::TrabajoNormal
            && (self.actual.pendientes.is_empty() || self.actual.turno_agotado())
        {
            self.escalar_y_avanzar(jugador);
        }
    }
}

struct Turno(Mutex<TurnoManejado>);

/// Vista de `EstadoTurno` (módulo `turno`) más la fase del arco (Etapa
/// 7/11-G, Plan 8) — `turno::EstadoTurno` se queda sin saber nada de
/// empresas/mini-boss, así que esta vista combinada vive aquí, no ahí.
#[derive(serde::Serialize)]
struct EstadoTurnoView {
    presupuesto_restante: u32,
    pendientes: Vec<tickets::Ticket>,
    fase: FaseArco,
}

impl From<&TurnoManejado> for EstadoTurnoView {
    fn from(manejado: &TurnoManejado) -> Self {
        EstadoTurnoView {
            presupuesto_restante: manejado.actual.presupuesto_restante,
            pendientes: manejado.actual.pendientes.clone(),
            fase: manejado.fase,
        }
    }
}

#[derive(serde::Serialize)]
struct ScoreResult {
    pass: bool,
    puntaje_correctitud: f64,
    puntaje_velocidad: f64,
    puntaje_practicas: f64,
    puntaje_base: f64,
    puntaje_final: f64,
    comentario_mentor: Option<String>,
    dinero_ganado: i64,
    dinero_total: i64,
    reputacion_ganada: f64,
    reputacion_total: f64,
    xp_ganado: Vec<(tickets::Arquetipo, i64)>,
    puede_ascender: bool,
    /// Etapa 10, Plan 7: `true` solo en la entrega exacta en la que el
    /// ascenso de rango ocurrió, para que el frontend muestre el anuncio.
    ascendio: bool,
    rango_actual: tickets::Rango,
    mensaje: String,
}

/// Vista combinada de un perk (Etapa 13): datos estáticos del catálogo +
/// estado dinámico del jugador frente a él.
#[derive(serde::Serialize)]
struct PerkConEstado {
    id: &'static str,
    nombre: &'static str,
    categoria: perks::Categoria,
    descripcion: &'static str,
    costo_dinero: i64,
    reputacion_minima: f64,
    desbloqueado: bool,
    equipado: bool,
}

fn vista_perks(estado: &economia::EstadoJugador) -> Vec<PerkConEstado> {
    perks::catalogo()
        .iter()
        .map(|perk| PerkConEstado {
            id: perk.id,
            nombre: perk.nombre,
            categoria: perk.categoria,
            descripcion: perk.descripcion,
            costo_dinero: perk.costo_dinero,
            reputacion_minima: perk.reputacion_minima,
            desbloqueado: estado.perks_desbloqueados.contains(&perk.id),
            equipado: estado.perks_equipados.contains(&perk.id),
        })
        .collect()
}

#[tauri::command]
fn turno_actual(turno: tauri::State<'_, Turno>) -> EstadoTurnoView {
    EstadoTurnoView::from(&*turno.0.lock().unwrap())
}

/// Etapa 10, Plan 7: expone el rango vigente para que el frontend pinte el
/// badge apenas carga, sin depender de haber resuelto un ticket primero.
#[tauri::command]
fn rango_actual(jugador: tauri::State<'_, Jugador>) -> tickets::Rango {
    jugador.0.lock().unwrap().rango
}

#[tauri::command]
async fn run_query(state: tauri::State<'_, AppState>, sql: String) -> Result<db::QueryResult, String> {
    let pool = state.0.lock().unwrap().clone();
    db::run_query(&pool, &sql).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn resolver_ticket(
    state: tauri::State<'_, AppState>,
    jugador: tauri::State<'_, Jugador>,
    turno_state: tauri::State<'_, Turno>,
    id: String,
    sql: String,
) -> Result<ScoreResult, String> {
    // Se retira el ticket de `pendientes` ANTES de validar/premiar (en vez de
    // solo consultarlo con `buscar_pendiente`) para que un doble envío
    // concurrente del mismo ticket (p. ej. doble clic en "✓ Enviar ticket"
    // antes de que resuelva la primera promesa) sea imposible de premiar dos
    // veces: `resolver` es la operación atómica de "check-and-remove" bajo
    // este mismo lock, así que solo la primera llamada puede obtener
    // `Some(ticket)` — la segunda ve `None` y falla aquí, antes de tocar
    // `Jugador` o correr validación/economía.
    let ticket = {
        let mut manejado = turno_state.0.lock().unwrap();
        manejado
            .actual
            .resolver(&id)
            .ok_or_else(|| format!("'{id}' ya fue resuelto o ya no está pendiente."))?
    };

    let pool = state.0.lock().unwrap().clone();
    let evaluacion = validation::evaluar_entrega(&pool, &sql, &ticket.sql_dorada, ticket.requiere_orden)
        .await
        .map_err(|e| e.to_string())?;

    let mut estado = jugador.0.lock().unwrap();
    let multiplicador_dinero = estado.multiplicador_dinero(perks::catalogo());
    let multiplicador_reputacion = estado.multiplicador_reputacion(perks::catalogo());
    let resultado = economia::calcular(&evaluacion, &ticket, multiplicador_dinero, multiplicador_reputacion);
    let ascendio = estado.aplicar_resultado(&resultado);

    let mut manejado = turno_state.0.lock().unwrap();
    manejado.actualizar_fase(ascendio, &mut estado);

    Ok(ScoreResult {
        pass: evaluacion.correcta,
        puntaje_correctitud: evaluacion.puntaje_correctitud,
        puntaje_velocidad: evaluacion.puntaje_velocidad,
        puntaje_practicas: evaluacion.puntaje_practicas,
        puntaje_base: resultado.puntaje_base,
        puntaje_final: resultado.puntaje_final,
        comentario_mentor: evaluacion.comentario_mentor.map(str::to_string),
        dinero_ganado: resultado.dinero_ganado,
        dinero_total: estado.dinero,
        reputacion_ganada: resultado.reputacion_ganada,
        reputacion_total: estado.reputacion,
        xp_ganado: resultado.xp_ganado,
        puede_ascender: estado.puede_ascender(),
        ascendio,
        rango_actual: estado.rango,
        mensaje: if evaluacion.correcta {
            "Ticket resuelto. Contabilidad procesará tu pago... eventualmente.".to_string()
        } else {
            "El resultado no coincide con lo que pidió la solicitud. Revisa tu consulta.".to_string()
        },
    })
}

#[tauri::command]
fn cerrar_dia(jugador: tauri::State<'_, Jugador>, turno_state: tauri::State<'_, Turno>) -> EstadoTurnoView {
    let mut estado = jugador.0.lock().unwrap();
    let mut manejado = turno_state.0.lock().unwrap();
    // Etapa 7/11-G, Plan 8: cerrar el día no tiene sentido narrativo (ni
    // mecánico) durante el mini-boss o esperando la Agencia — el jugador no
    // puede simplemente saltárselos, así que fuera de `TrabajoNormal` esto
    // no hace nada.
    if manejado.fase == FaseArco::TrabajoNormal {
        manejado.escalar_y_avanzar(&mut estado);
    }
    EstadoTurnoView::from(&*manejado)
}

/// Carga `company` (Etapa 11-G, Plan 8) y reconstruye el turno/catálogo para
/// ella — aislado de `confirmar_transicion_agencia` para poder probarse
/// contra Postgres embebido real sin pasar por el estado de Tauri. No toca
/// `EstadoJugador` directamente (solo lee `rango`, por valor) para que el
/// llamador nunca necesite mantener el lock de `Jugador` abierto durante el
/// `.await`.
async fn transicionar_a_empresa(
    pg: &postgresql_embedded::PostgreSQL,
    company: db::Company,
    rango: tickets::Rango,
) -> anyhow::Result<(sqlx::PgPool, TurnoManejado)> {
    let pool = db::load_company(pg, company).await?;
    let catalogo = tickets::catalogo(company);
    let elegibles = tickets::tickets_elegibles(&catalogo, rango);
    let (actual, indice_siguiente) = turno::EstadoTurno::nuevo(&elegibles, 0);
    Ok((
        pool,
        TurnoManejado {
            catalogo,
            indice_siguiente,
            actual,
            fase: FaseArco::TrabajoNormal,
        },
    ))
}

/// Confirma la transición de la Agencia (Etapa 9/11-G, Plan 8): solo el
/// único destino del MVP, Postafeta. Falla si el arco todavía no está
/// completo. Resetea la reputación a 0 (Etapa 12: "eres el nuevo") — dinero,
/// XP por arquetipo, perks y rango se mantienen intactos.
#[tauri::command]
async fn confirmar_transicion_agencia(
    state: tauri::State<'_, AppState>,
    jugador: tauri::State<'_, Jugador>,
    turno_state: tauri::State<'_, Turno>,
    embedded: tauri::State<'_, EmbeddedPostgres>,
    transicion: tauri::State<'_, TransicionEnCurso>,
) -> Result<EstadoTurnoView, String> {
    if transicion.0.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return Err("Ya hay una transición de Agencia en curso.".to_string());
    }

    let resultado = async {
        {
            let manejado = turno_state.0.lock().unwrap();
            if manejado.fase != FaseArco::ArcoCompletado {
                return Err("El arco de la empresa todavía no está completo.".to_string());
            }
        }

        let rango = jugador.0.lock().unwrap().rango;

        let pg = embedded
            .0
            .lock()
            .unwrap()
            .take()
            .ok_or_else(|| "Postgres embebido no está disponible.".to_string())?;
        let resultado_transicion = transicionar_a_empresa(&pg, db::Company::Postafeta, rango).await;
        embedded.0.lock().unwrap().replace(pg);
        let (pool, nuevo_manejado) = resultado_transicion.map_err(|e| e.to_string())?;

        jugador.0.lock().unwrap().reputacion = 0.0;
        *state.0.lock().unwrap() = pool;
        *turno_state.0.lock().unwrap() = nuevo_manejado;

        Ok(EstadoTurnoView::from(&*turno_state.0.lock().unwrap()))
    }
    .await;

    transicion.0.store(false, std::sync::atomic::Ordering::SeqCst);
    resultado
}

#[tauri::command]
fn catalogo_perks(jugador: tauri::State<'_, Jugador>) -> Vec<PerkConEstado> {
    let estado = jugador.0.lock().unwrap();
    vista_perks(&estado)
}

#[tauri::command]
fn desbloquear_perk(jugador: tauri::State<'_, Jugador>, id: String) -> Result<Vec<PerkConEstado>, String> {
    let mut estado = jugador.0.lock().unwrap();
    estado.desbloquear_perk(perks::catalogo(), &id)?;
    Ok(vista_perks(&estado))
}

#[tauri::command]
fn equipar_perk(jugador: tauri::State<'_, Jugador>, id: String) -> Result<Vec<PerkConEstado>, String> {
    let mut estado = jugador.0.lock().unwrap();
    estado.equipar_perk(&id)?;
    Ok(vista_perks(&estado))
}

#[tauri::command]
fn desequipar_perk(jugador: tauri::State<'_, Jugador>, id: String) -> Vec<PerkConEstado> {
    let mut estado = jugador.0.lock().unwrap();
    estado.desequipar_perk(&id);
    vista_perks(&estado)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            turno_actual,
            rango_actual,
            run_query,
            resolver_ticket,
            cerrar_dia,
            confirmar_transicion_agencia,
            catalogo_perks,
            desbloquear_perk,
            equipar_perk,
            desequipar_perk
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                let pg = db::init_embedded_postgres()
                    .await
                    .expect("no se pudo inicializar Postgres embebido");
                let pool = db::load_company(&pg, db::Company::HospitalArcangel)
                    .await
                    .expect("no se pudo cargar Hospital Arcángel");
                // `pool` y `catalogo` deben cargarse siempre con la misma `Company`:
                // `resolver_ticket` valida el SQL del jugador (ejecutado contra
                // `pool`) contra `sql_dorada` de un ticket de `Turno`, así que si
                // alguna vez divergen, se validaría contra el esquema de otra empresa.
                let catalogo = tickets::catalogo(db::Company::HospitalArcangel);
                let jugador_inicial = economia::EstadoJugador::default();
                // Etapa 10, Plan 7: el turno inicial ya se filtra por rango —
                // un Becario recién llegado no debe ver tickets de
                // Join/Agregación en su primera bandeja.
                let elegibles = tickets::tickets_elegibles(&catalogo, jugador_inicial.rango);
                let (turno_inicial, indice_siguiente) = turno::EstadoTurno::nuevo(&elegibles, 0);
                handle.manage(AppState(Mutex::new(pool)));
                handle.manage(Jugador(Mutex::new(jugador_inicial)));
                handle.manage(Turno(Mutex::new(TurnoManejado {
                    catalogo,
                    indice_siguiente,
                    actual: turno_inicial,
                    fase: FaseArco::TrabajoNormal,
                })));
                handle.manage(EmbeddedPostgres(Mutex::new(Some(pg))));
                handle.manage(TransicionEnCurso(std::sync::atomic::AtomicBool::new(false)));
            });
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| {
        if !matches!(event, tauri::RunEvent::Exit) {
            return;
        }
        let Some(embedded) = app_handle.try_state::<EmbeddedPostgres>() else {
            return;
        };
        let Some(pg) = embedded.0.lock().unwrap().take() else {
            return;
        };
        tauri::async_runtime::block_on(async move {
            let _ = pg.stop().await;
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use tickets::Rango;

    /// Etapa 10, Plan 7: cubre el caso más delicado del ascenso — un jugador
    /// que asciende a mitad de turno no debe reordenar la bandeja ya
    /// mostrada, pero el turno *siguiente* (armado por `escalar_y_avanzar`)
    /// ya debe reflejar el catálogo desbloqueado, incluyendo tickets de
    /// Join/Agregación que un Becario nunca vería.
    #[test]
    fn escalar_y_avanzar_refleja_el_catalogo_desbloqueado_tras_un_ascenso() {
        let catalogo = tickets::catalogo(db::Company::HospitalArcangel);
        let elegibles_becario = tickets::tickets_elegibles(&catalogo, Rango::Becario);
        // Nota: se arranca en el índice 1 (no 0) a propósito. Con exactamente
        // 3 tickets elegibles para Becario y TAMANO_LOTE = 3, empezar en 0
        // consume el lote completo y la rotación vuelve a 0 — que, aplicado
        // después sobre el catálogo completo de 8, apunta de nuevo a los
        // mismos 3 tickets Select-only por pura coincidencia aritmética
        // (0 % 3 == 0 % 8). Arrancar en 1 simula un jugador que ya llevaba
        // turnos jugados antes de ascender, y es lo que deja ver el
        // comportamiento real que este test cubre: el turno siguiente
        // avanza sobre el catálogo ya desbloqueado, no sobre el filtrado
        // viejo.
        let (turno_inicial, indice_siguiente) = turno::EstadoTurno::nuevo(&elegibles_becario, 1);

        let mut manejado = TurnoManejado {
            catalogo,
            indice_siguiente,
            actual: turno_inicial,
            fase: FaseArco::TrabajoNormal,
        };
        let mut jugador = economia::EstadoJugador {
            rango: Rango::AuxiliarDeSistemas,
            ..economia::EstadoJugador::default()
        };

        manejado.escalar_y_avanzar(&mut jugador);

        assert!(
            manejado
                .actual
                .pendientes
                .iter()
                .any(|t| tickets::rango_requerido(t) == Rango::AuxiliarDeSistemas),
            "tras ascender, el turno siguiente debe poder incluir tickets de Join/Agregación \
             que un Becario nunca vería"
        );
    }

    #[test]
    fn actualizar_fase_dispara_el_lote_del_mini_boss_al_ascender() {
        let catalogo = tickets::catalogo(db::Company::HospitalArcangel);
        let elegibles_becario = tickets::tickets_elegibles(&catalogo, Rango::Becario);
        let (turno_inicial, indice_siguiente) = turno::EstadoTurno::nuevo(&elegibles_becario, 0);
        let mut manejado = TurnoManejado {
            catalogo,
            indice_siguiente,
            actual: turno_inicial,
            fase: FaseArco::TrabajoNormal,
        };
        let mut jugador = economia::EstadoJugador::default();

        manejado.actualizar_fase(true, &mut jugador);

        assert_eq!(manejado.fase, FaseArco::MiniBoss);
        assert_eq!(manejado.actual.pendientes.len(), 2);
        assert!(
            manejado
                .actual
                .pendientes
                .iter()
                .all(|t| tickets::rango_requerido(t) == Rango::AuxiliarDeSistemas),
            "los 2 tickets del mini-boss son Auxiliar-tier"
        );
    }

    #[test]
    fn actualizar_fase_completa_el_arco_al_vaciar_el_lote_del_mini_boss() {
        let catalogo = tickets::catalogo(db::Company::HospitalArcangel);
        let (turno_vacio, _) = turno::EstadoTurno::nuevo(&[], 0);
        let mut manejado = TurnoManejado {
            catalogo,
            indice_siguiente: 0,
            actual: turno_vacio,
            fase: FaseArco::MiniBoss,
        };
        let mut jugador = economia::EstadoJugador::default();

        manejado.actualizar_fase(false, &mut jugador);

        assert_eq!(manejado.fase, FaseArco::ArcoCompletado);
        assert!(manejado.actual.pendientes.is_empty(), "no debe dibujarse un turno normal");
    }

    #[test]
    fn actualizar_fase_avanza_el_turno_normal_cuando_no_hay_ascenso_ni_mini_boss() {
        let catalogo = tickets::catalogo(db::Company::HospitalArcangel);
        let (turno_vacio, indice_siguiente) = turno::EstadoTurno::nuevo(&[], 0);
        let mut manejado = TurnoManejado {
            catalogo,
            indice_siguiente,
            actual: turno_vacio,
            fase: FaseArco::TrabajoNormal,
        };
        let mut jugador = economia::EstadoJugador::default();

        manejado.actualizar_fase(false, &mut jugador);

        assert_eq!(manejado.fase, FaseArco::TrabajoNormal);
        assert!(!manejado.actual.pendientes.is_empty(), "debe dibujar un turno normal nuevo");
    }

    #[test]
    fn actualizar_fase_no_hace_nada_en_arco_completado() {
        let catalogo = tickets::catalogo(db::Company::HospitalArcangel);
        let (turno_vacio, _) = turno::EstadoTurno::nuevo(&[], 0);
        let mut manejado = TurnoManejado {
            catalogo,
            indice_siguiente: 0,
            actual: turno_vacio,
            fase: FaseArco::ArcoCompletado,
        };
        let mut jugador = economia::EstadoJugador::default();

        manejado.actualizar_fase(false, &mut jugador);

        assert_eq!(manejado.fase, FaseArco::ArcoCompletado);
        assert!(manejado.actual.pendientes.is_empty());
    }

    #[tokio::test]
    async fn transicionar_a_empresa_resetea_el_turno_y_arma_el_catalogo_de_la_nueva_empresa() {
        let pg = db::init_embedded_postgres().await.expect("Postgres embebido debe arrancar");

        let (pool, manejado) = transicionar_a_empresa(&pg, db::Company::Postafeta, Rango::AuxiliarDeSistemas)
            .await
            .expect("la transición a Postafeta debe completarse");

        assert_eq!(manejado.fase, FaseArco::TrabajoNormal);
        assert_eq!(manejado.catalogo.len(), 6, "catálogo completo de Postafeta");
        assert!(!manejado.actual.pendientes.is_empty(), "debe armar un turno inicial en la empresa nueva");

        let paquetes = db::run_query(&pool, "SELECT * FROM paquetes").await.expect("Postafeta debe responder queries");
        assert_eq!(paquetes.rows.len(), 30, "el pool devuelto debe apuntar a la base de datos de Postafeta");

        pool.close().await;
        pg.stop().await.expect("Postgres debe detenerse limpiamente");
    }
}
