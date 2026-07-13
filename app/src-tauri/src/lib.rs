mod db;
mod economia;
mod perks;
mod tickets;
mod validation;

use std::sync::Mutex;
use tauri::Manager;

/// Pool de conexión al Postgres embebido, gestionado por Tauri.
struct AppState {
    pool: sqlx::PgPool,
}

/// Estado de economía del jugador (Etapa 12/13), gestionado por Tauri.
struct Jugador(Mutex<economia::EstadoJugador>);

/// Mantiene vivo el servidor Postgres embebido y permite detenerlo al salir.
struct EmbeddedPostgres(Mutex<Option<postgresql_embedded::PostgreSQL>>);

/// Catálogo de tickets de la empresa activa + índice del ticket actual
/// (Etapa 14). Selección round-robin simple — sin bandeja de entrada ni
/// tiempo de turno todavía (Etapa 11-A, plan de UI/loop posterior).
struct Tickets {
    catalogo: Vec<tickets::Ticket>,
    indice_actual: Mutex<usize>,
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
fn ticket_actual(tickets: tauri::State<'_, Tickets>) -> tickets::Ticket {
    let indice = *tickets.indice_actual.lock().unwrap();
    tickets.catalogo[indice].clone()
}

#[tauri::command]
async fn run_query(state: tauri::State<'_, AppState>, sql: String) -> Result<db::QueryResult, String> {
    db::run_query(&state.pool, &sql).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn submit_ticket(
    state: tauri::State<'_, AppState>,
    jugador: tauri::State<'_, Jugador>,
    tickets: tauri::State<'_, Tickets>,
    sql: String,
) -> Result<ScoreResult, String> {
    let indice = *tickets.indice_actual.lock().unwrap();
    let ticket = tickets.catalogo[indice].clone();

    let evaluacion = validation::evaluar_entrega(&state.pool, &sql, &ticket.sql_dorada, ticket.requiere_orden)
        .await
        .map_err(|e| e.to_string())?;

    let mut estado = jugador.0.lock().unwrap();
    let multiplicador_dinero = estado.multiplicador_dinero(perks::catalogo());
    let multiplicador_reputacion = estado.multiplicador_reputacion(perks::catalogo());
    let resultado = economia::calcular(&evaluacion, &ticket, multiplicador_dinero, multiplicador_reputacion);
    estado.aplicar_resultado(&resultado);

    if evaluacion.correcta {
        let mut indice_mut = tickets.indice_actual.lock().unwrap();
        *indice_mut = (*indice_mut + 1) % tickets.catalogo.len();
    }

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
        mensaje: if evaluacion.correcta {
            "Ticket resuelto. Contabilidad procesará tu pago... eventualmente.".to_string()
        } else {
            "El resultado no coincide con lo que pidió la solicitud. Revisa tu consulta.".to_string()
        },
    })
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
            ticket_actual,
            run_query,
            submit_ticket,
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
                // `submit_ticket` valida el SQL del jugador (ejecutado contra `pool`)
                // contra `sql_dorada` del ticket actual de `Tickets`, así que si
                // alguna vez divergen, se validaría contra el esquema de otra empresa.
                let catalogo = tickets::catalogo(db::Company::HospitalArcangel);
                handle.manage(AppState { pool });
                handle.manage(Jugador(Mutex::new(economia::EstadoJugador::default())));
                handle.manage(Tickets { catalogo, indice_actual: Mutex::new(0) });
                handle.manage(EmbeddedPostgres(Mutex::new(Some(pg))));
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
