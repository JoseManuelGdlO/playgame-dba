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

/// Directorio de datos de la app (Etapa 11-G/Menú, Plan 9), resuelto una
/// sola vez en `setup()` — ahí vive el único archivo de guardado.
struct DirectorioGuardado(std::path::PathBuf);

/// Todo lo que el Hub necesita pintar al entrar desde el Menú (Etapa
/// 11-G/Menú, Plan 9): stats del jugador + el turno actual en una sola
/// respuesta. `turno_actual`/`rango_actual` (Plan 7) siguen existiendo para
/// los refrescos puntuales que ya hace el frontend en otros momentos.
#[derive(serde::Serialize)]
struct EstadoJuegoView {
    dinero: i64,
    reputacion: f64,
    rango: tickets::Rango,
    presupuesto_restante: u32,
    pendientes: Vec<tickets::Ticket>,
    fase: FaseArco,
    empresa: db::Company,
}

impl EstadoJuegoView {
    fn construir(jugador: &economia::EstadoJugador, manejado: &TurnoManejado) -> Self {
        EstadoJuegoView {
            dinero: jugador.dinero,
            reputacion: jugador.reputacion,
            rango: jugador.rango,
            presupuesto_restante: manejado.actual.presupuesto_restante,
            pendientes: manejado.actual.pendientes.clone(),
            fase: manejado.fase,
            empresa: manejado.empresa,
        }
    }
}

/// Construye el estado de una partida nueva (Etapa 11-G/Menú, Plan 9):
/// siempre Hospital Arcángel, Becario, dinero/reputación/perks en cero.
/// Reutilizado tanto por `setup()` (estado por defecto al abrir la app) como
/// por el comando `iniciar_partida` (el jugador decide empezar de cero a
/// mitad de sesión).
async fn estado_de_partida_nueva(
    pg: &postgresql_embedded::PostgreSQL,
) -> anyhow::Result<(sqlx::PgPool, economia::EstadoJugador, TurnoManejado)> {
    let pool = db::load_company(pg, db::Company::HospitalArcangel).await?;
    let catalogo = tickets::catalogo(db::Company::HospitalArcangel);
    let jugador = economia::EstadoJugador::default();
    let elegibles = tickets::tickets_elegibles(&catalogo, jugador.rango);
    let (actual, indice_siguiente) = turno::EstadoTurno::nuevo(&elegibles, 0);
    Ok((
        pool,
        jugador,
        TurnoManejado {
            catalogo,
            indice_siguiente,
            actual,
            fase: FaseArco::TrabajoNormal,
            empresa: db::Company::HospitalArcangel,
        },
    ))
}

/// Reconstruye el `Ticket` completo de cada id guardado (Etapa 11-G/Menú,
/// Plan 9), buscándolo en el catálogo que corresponda según la fase: el
/// catálogo completo de la empresa si `fase == TrabajoNormal`, o el lote del
/// mini-boss si el jugador se había quedado a mitad de esa secuencia. Los
/// ids que ya no aparezcan en el catálogo (no debería pasar en la práctica)
/// se descartan en silencio en vez de fallar toda la carga.
fn resolver_tickets_guardados(empresa: db::Company, fase: FaseArco, ids: &[String]) -> Vec<tickets::Ticket> {
    let catalogo = if fase == FaseArco::TrabajoNormal {
        tickets::catalogo(empresa)
    } else {
        tickets::mini_boss_hospital_arcangel()
    };
    ids.iter()
        .filter_map(|id| catalogo.iter().find(|t| t.id == id).cloned())
        .collect()
}

/// Autoguarda la partida completa (Etapa 11-G/Menú, Plan 9) — llamado tras
/// `resolver_ticket`, `cerrar_dia`, `confirmar_transicion_agencia`, e
/// `iniciar_partida`. Un fallo de guardado se ignora en silencio: perder un
/// autosave puntual es preferible a que el jugador no pueda seguir jugando
/// por un problema de disco.
fn autoguardar(dir: &std::path::Path, jugador: &economia::EstadoJugador, manejado: &TurnoManejado) {
    let partida = guardado::PartidaGuardada {
        dinero: jugador.dinero,
        reputacion: jugador.reputacion,
        xp_por_arquetipo: jugador.xp_por_arquetipo.clone(),
        rango: jugador.rango,
        perks_desbloqueados: jugador.perks_desbloqueados.iter().map(|s| s.to_string()).collect(),
        perks_equipados: jugador.perks_equipados.iter().map(|s| s.to_string()).collect(),
        empresa: manejado.empresa,
        fase: manejado.fase,
        indice_siguiente: manejado.indice_siguiente,
        presupuesto_restante: manejado.actual.presupuesto_restante,
        pendientes_ids: manejado.actual.pendientes.iter().map(|t| t.id.to_string()).collect(),
    };
    let _ = guardado::guardar(dir, &partida);
}

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
/// selección round-robin simple de un solo "ticket actual" (Plan 3) — la
/// fase del arco de esa empresa (Etapa 7/11-G, Plan 8), y qué empresa es esa
/// (Etapa 11-G/Menú, Plan 9 — antes no había ninguna forma de leer "la
/// empresa activa" como dato, solo el catálogo ya cargado; el guardado
/// necesita poder persistirla).
struct TurnoManejado {
    catalogo: Vec<tickets::Ticket>,
    indice_siguiente: usize,
    actual: turno::EstadoTurno,
    fase: FaseArco,
    empresa: db::Company,
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
    empresa: db::Company,
}

impl From<&TurnoManejado> for EstadoTurnoView {
    fn from(manejado: &TurnoManejado) -> Self {
        EstadoTurnoView {
            presupuesto_restante: manejado.actual.presupuesto_restante,
            pendientes: manejado.actual.pendientes.clone(),
            fase: manejado.fase,
            empresa: manejado.empresa,
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

/// Vista de la lista de perks junto con el límite de slots del rango actual
/// (Etapa 13/Plan 7: `EstadoJugador::max_slots`) — para que el Hub pueda
/// mostrar "Perks equipados: X/Y" sin adivinar el límite en el frontend.
#[derive(serde::Serialize)]
struct PerksView {
    perks: Vec<PerkConEstado>,
    max_slots: usize,
}

fn vista_perks_con_slots(estado: &economia::EstadoJugador) -> PerksView {
    PerksView {
        perks: vista_perks(estado),
        max_slots: estado.max_slots(),
    }
}

#[tauri::command]
fn turno_actual(turno: tauri::State<'_, Turno>) -> EstadoTurnoView {
    EstadoTurnoView::from(&*turno.0.lock().unwrap())
}

/// Cierra la aplicación por completo (menú principal y menú de pausa) —
/// el autoguardado ya corre tras cada acción relevante, así que no hace
/// falta guardar nada aquí antes de salir.
#[tauri::command]
fn salir_del_juego() {
    std::process::exit(0);
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
    dir: tauri::State<'_, DirectorioGuardado>,
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
    autoguardar(&dir.0, &estado, &manejado);

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
fn cerrar_dia(
    jugador: tauri::State<'_, Jugador>,
    turno_state: tauri::State<'_, Turno>,
    dir: tauri::State<'_, DirectorioGuardado>,
) -> EstadoTurnoView {
    let mut estado = jugador.0.lock().unwrap();
    let mut manejado = turno_state.0.lock().unwrap();
    // Etapa 7/11-G, Plan 8: cerrar el día no tiene sentido narrativo (ni
    // mecánico) durante el mini-boss o esperando la Agencia — el jugador no
    // puede simplemente saltárselos, así que fuera de `TrabajoNormal` esto
    // no hace nada.
    if manejado.fase == FaseArco::TrabajoNormal {
        manejado.escalar_y_avanzar(&mut estado);
    }
    autoguardar(&dir.0, &estado, &manejado);
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
            empresa: company,
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
    dir: tauri::State<'_, DirectorioGuardado>,
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

        {
            let estado = jugador.0.lock().unwrap();
            let manejado = turno_state.0.lock().unwrap();
            autoguardar(&dir.0, &estado, &manejado);
        }

        Ok(EstadoTurnoView::from(&*turno_state.0.lock().unwrap()))
    }
    .await;

    transicion.0.store(false, std::sync::atomic::Ordering::SeqCst);
    resultado
}

#[tauri::command]
fn existe_partida_guardada(dir: tauri::State<'_, DirectorioGuardado>) -> bool {
    guardado::existe(&dir.0)
}

/// Empieza una partida nueva a mitad de sesión (Etapa 11-G/Menú, Plan 9):
/// siempre Hospital Arcángel, Becario, dinero/reputación/perks en cero —
/// reemplaza `AppState`/`Jugador`/`Turno` en caliente, mismo patrón que
/// `confirmar_transicion_agencia` (Plan 8) ya usa para el pool.
#[tauri::command]
async fn iniciar_partida(
    state: tauri::State<'_, AppState>,
    jugador: tauri::State<'_, Jugador>,
    turno_state: tauri::State<'_, Turno>,
    embedded: tauri::State<'_, EmbeddedPostgres>,
    transicion: tauri::State<'_, TransicionEnCurso>,
    dir: tauri::State<'_, DirectorioGuardado>,
) -> Result<EstadoJuegoView, String> {
    if transicion.0.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return Err("Ya hay una transición de partida en curso.".to_string());
    }

    let resultado = async {
        let pg = embedded
            .0
            .lock()
            .unwrap()
            .take()
            .ok_or_else(|| "Postgres embebido no está disponible.".to_string())?;
        let resultado_estado = estado_de_partida_nueva(&pg).await;
        embedded.0.lock().unwrap().replace(pg);
        let (pool, jugador_nuevo, manejado_nuevo) = resultado_estado.map_err(|e| e.to_string())?;

        *jugador.0.lock().unwrap() = jugador_nuevo;
        *state.0.lock().unwrap() = pool;
        *turno_state.0.lock().unwrap() = manejado_nuevo;

        let estado = jugador.0.lock().unwrap();
        let manejado = turno_state.0.lock().unwrap();
        autoguardar(&dir.0, &estado, &manejado);
        Ok(EstadoJuegoView::construir(&estado, &manejado))
    }
    .await;

    transicion.0.store(false, std::sync::atomic::Ordering::SeqCst);
    resultado
}

/// Carga el único slot de guardado (Etapa 11-G/Menú, Plan 9). Falla si no
/// hay ningún guardado. Reconstruye los tickets pendientes por id contra el
/// catálogo/mini-boss de la empresa guardada, y reemplaza
/// `AppState`/`Jugador`/`Turno` en caliente.
#[tauri::command]
async fn cargar_partida(
    state: tauri::State<'_, AppState>,
    jugador: tauri::State<'_, Jugador>,
    turno_state: tauri::State<'_, Turno>,
    embedded: tauri::State<'_, EmbeddedPostgres>,
    transicion: tauri::State<'_, TransicionEnCurso>,
    dir: tauri::State<'_, DirectorioGuardado>,
) -> Result<EstadoJuegoView, String> {
    if transicion.0.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return Err("Ya hay una transición de partida en curso.".to_string());
    }

    let resultado = async {
        let partida = guardado::cargar(&dir.0)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "No hay ninguna partida guardada.".to_string())?;

        let pg = embedded
            .0
            .lock()
            .unwrap()
            .take()
            .ok_or_else(|| "Postgres embebido no está disponible.".to_string())?;
        let resultado_pool = db::load_company(&pg, partida.empresa).await;
        embedded.0.lock().unwrap().replace(pg);
        let pool = resultado_pool.map_err(|e| e.to_string())?;

        let catalogo_completo = tickets::catalogo(partida.empresa);
        let pendientes = resolver_tickets_guardados(partida.empresa, partida.fase, &partida.pendientes_ids);

        let jugador_cargado = economia::EstadoJugador {
            dinero: partida.dinero,
            reputacion: partida.reputacion,
            xp_por_arquetipo: partida.xp_por_arquetipo,
            rango: partida.rango,
            perks_desbloqueados: partida.perks_desbloqueados.iter().filter_map(|id| perks::buscar(id)).map(|p| p.id).collect(),
            perks_equipados: partida.perks_equipados.iter().filter_map(|id| perks::buscar(id)).map(|p| p.id).collect(),
        };

        let manejado_cargado = TurnoManejado {
            catalogo: catalogo_completo,
            indice_siguiente: partida.indice_siguiente,
            actual: turno::EstadoTurno {
                presupuesto_restante: partida.presupuesto_restante,
                pendientes,
                intentos_usados: std::collections::HashMap::new(),
            },
            fase: partida.fase,
            empresa: partida.empresa,
        };

        *state.0.lock().unwrap() = pool;
        *jugador.0.lock().unwrap() = jugador_cargado;
        *turno_state.0.lock().unwrap() = manejado_cargado;

        let estado = jugador.0.lock().unwrap();
        let manejado = turno_state.0.lock().unwrap();
        Ok(EstadoJuegoView::construir(&estado, &manejado))
    }
    .await;

    transicion.0.store(false, std::sync::atomic::Ordering::SeqCst);
    resultado
}

#[tauri::command]
fn catalogo_perks(jugador: tauri::State<'_, Jugador>) -> PerksView {
    let estado = jugador.0.lock().unwrap();
    vista_perks_con_slots(&estado)
}

#[tauri::command]
async fn esquema_actual(state: tauri::State<'_, AppState>) -> Result<db::EsquemaView, String> {
    let pool = state.0.lock().unwrap().clone();
    db::obtener_esquema(&pool).await.map_err(|e| e.to_string())
}

#[tauri::command]
fn desbloquear_perk(jugador: tauri::State<'_, Jugador>, id: String) -> Result<PerksView, String> {
    let mut estado = jugador.0.lock().unwrap();
    estado.desbloquear_perk(perks::catalogo(), &id)?;
    Ok(vista_perks_con_slots(&estado))
}

#[tauri::command]
fn equipar_perk(jugador: tauri::State<'_, Jugador>, id: String) -> Result<PerksView, String> {
    let mut estado = jugador.0.lock().unwrap();
    estado.equipar_perk(&id)?;
    Ok(vista_perks_con_slots(&estado))
}

#[tauri::command]
fn desequipar_perk(jugador: tauri::State<'_, Jugador>, id: String) -> PerksView {
    let mut estado = jugador.0.lock().unwrap();
    estado.desequipar_perk(&id);
    vista_perks_con_slots(&estado)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            turno_actual,
            rango_actual,
            salir_del_juego,
            run_query,
            resolver_ticket,
            cerrar_dia,
            confirmar_transicion_agencia,
            existe_partida_guardada,
            iniciar_partida,
            cargar_partida,
            catalogo_perks,
            desbloquear_perk,
            equipar_perk,
            desequipar_perk,
            esquema_actual
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            let dir_guardado = handle
                .path()
                .app_data_dir()
                .expect("no se pudo resolver el directorio de datos de la app");
            tauri::async_runtime::block_on(async move {
                let pg = db::init_embedded_postgres()
                    .await
                    .expect("no se pudo inicializar Postgres embebido");
                // Estado por defecto al abrir la app (Etapa 11-G/Menú, Plan 9):
                // el Menú decide si se queda así ("Iniciar partida", ya es el
                // estado correcto) o lo reemplaza con un guardado ("Cargar
                // partida", vía el comando `cargar_partida`).
                let (pool, jugador_inicial, turno_inicial) = estado_de_partida_nueva(&pg)
                    .await
                    .expect("no se pudo iniciar la partida por defecto");
                handle.manage(AppState(Mutex::new(pool)));
                handle.manage(Jugador(Mutex::new(jugador_inicial)));
                handle.manage(Turno(Mutex::new(turno_inicial)));
                handle.manage(EmbeddedPostgres(Mutex::new(Some(pg))));
                handle.manage(TransicionEnCurso(std::sync::atomic::AtomicBool::new(false)));
                handle.manage(DirectorioGuardado(dir_guardado));
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
            empresa: db::Company::HospitalArcangel,
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
            empresa: db::Company::HospitalArcangel,
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
            empresa: db::Company::HospitalArcangel,
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
            empresa: db::Company::HospitalArcangel,
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
            empresa: db::Company::HospitalArcangel,
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

    /// Plan 9 (finding de code review): `resolver_tickets_guardados` no tenía
    /// cobertura propia. En `TrabajoNormal` debe reconstruir contra el
    /// catálogo completo de la empresa, preservando el orden de los ids
    /// guardados (no el orden del catálogo).
    #[test]
    fn resolver_tickets_guardados_en_trabajo_normal_usa_el_catalogo_completo() {
        let ids = vec![
            "hospital_reporte_habitaciones_libres".to_string(),
            "hospital_reporte_pacientes_cardiologia".to_string(),
        ];

        let resueltos = resolver_tickets_guardados(db::Company::HospitalArcangel, FaseArco::TrabajoNormal, &ids);

        assert_eq!(resueltos.len(), 2);
        assert_eq!(resueltos[0].id, "hospital_reporte_habitaciones_libres");
        assert_eq!(resueltos[1].id, "hospital_reporte_pacientes_cardiologia");
    }

    /// Cualquier fase distinta de `TrabajoNormal` (tanto `MiniBoss` como
    /// `ArcoCompletado`, el jugador puede haber guardado a mitad de
    /// cualquiera de las dos) debe resolver contra el lote del mini-boss,
    /// no contra el catálogo normal de 8 tickets.
    #[test]
    fn resolver_tickets_guardados_fuera_de_trabajo_normal_usa_el_lote_del_mini_boss() {
        let ids_mini_boss = vec![
            "hospital_miniboss_pacientes_sin_seguro".to_string(),
            "hospital_miniboss_tratamientos_por_tipo".to_string(),
        ];

        for fase in [FaseArco::MiniBoss, FaseArco::ArcoCompletado] {
            let resueltos = resolver_tickets_guardados(db::Company::HospitalArcangel, fase, &ids_mini_boss);
            assert_eq!(resueltos.len(), 2, "fase {fase:?} debe resolver ambos ids del mini-boss");
            assert_eq!(resueltos[0].id, "hospital_miniboss_pacientes_sin_seguro");
            assert_eq!(resueltos[1].id, "hospital_miniboss_tratamientos_por_tipo");
        }

        // Un id que sólo existe en el catálogo normal no debe resolver
        // mientras la fase no sea TrabajoNormal: confirma que de verdad se
        // está buscando en el lote del mini-boss y no en el catálogo de 8.
        let ids_catalogo_normal = vec!["hospital_reporte_pacientes_cardiologia".to_string()];
        let resueltos = resolver_tickets_guardados(db::Company::HospitalArcangel, FaseArco::MiniBoss, &ids_catalogo_normal);
        assert!(
            resueltos.is_empty(),
            "un id exclusivo del catálogo normal no debe resolver contra el lote del mini-boss"
        );
    }

    /// Un id guardado que ya no existe en el catálogo correspondiente (por
    /// ejemplo, un save stale de una versión anterior del contenido) debe
    /// descartarse en silencio en vez de hacer panic o propagar un error.
    #[test]
    fn resolver_tickets_guardados_descarta_en_silencio_los_ids_desconocidos() {
        let ids = vec![
            "hospital_reporte_pacientes_cardiologia".to_string(),
            "id_que_no_existe".to_string(),
        ];

        let resueltos = resolver_tickets_guardados(db::Company::HospitalArcangel, FaseArco::TrabajoNormal, &ids);

        assert_eq!(resueltos.len(), 1);
        assert_eq!(resueltos[0].id, "hospital_reporte_pacientes_cardiologia");
    }
}
