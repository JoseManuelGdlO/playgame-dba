mod db;
mod economia;
mod tickets;
mod validation;

use std::sync::Mutex;
use tauri::Manager;

/// Pool de conexión al Postgres embebido, gestionado por Tauri.
struct AppState {
    pool: sqlx::PgPool,
}

/// Estado de economía del jugador (Etapa 12), gestionado por Tauri. El bool
/// de perk es el mismo stub heredado del spike — el sistema RPG real
/// (Etapa 13) lo reemplaza en un plan posterior.
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

#[derive(serde::Serialize)]
struct PerkStatus {
    unlocked: bool,
    dinero_total: i64,
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

    let resultado = economia::calcular(&evaluacion, &ticket, 1.0);

    let mut estado = jugador.0.lock().unwrap();
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
        // Etapa 10, deliberadamente recortado: solo se expone la señal de que
        // la reputación ya cruzó el umbral de ascenso — no dispara ningún
        // evento de ascenso (mini-boss, cambio de rango), que queda para un
        // plan posterior.
        puede_ascender: estado.puede_ascender(),
        mensaje: if evaluacion.correcta {
            "Ticket resuelto. Contabilidad procesará tu pago... eventualmente.".to_string()
        } else {
            "El resultado no coincide con lo que pidió la solicitud. Revisa tu consulta.".to_string()
        },
    })
}

#[tauri::command]
fn unlock_perk(jugador: tauri::State<'_, Jugador>) -> Result<PerkStatus, String> {
    const COSTO: i64 = 300;
    let mut estado = jugador.0.lock().unwrap();
    if estado.perk_desbloqueado {
        return Ok(PerkStatus { unlocked: true, dinero_total: estado.dinero });
    }
    if estado.dinero < COSTO {
        return Err(format!(
            "No tienes suficiente dinero para este perk (cuesta {COSTO}, tienes {}).",
            estado.dinero
        ));
    }
    estado.dinero -= COSTO;
    estado.perk_desbloqueado = true;
    Ok(PerkStatus { unlocked: true, dinero_total: estado.dinero })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            ticket_actual,
            run_query,
            submit_ticket,
            unlock_perk
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
