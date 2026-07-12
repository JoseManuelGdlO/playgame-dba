mod db;

use std::sync::Mutex;
use tauri::Manager;

/// Pool de conexión al Postgres embebido, gestionado por Tauri.
struct AppState {
    pool: sqlx::PgPool,
}

/// Estado stub de economía/loadout (Etapa 12/13) — solo para probar la forma
/// del loop en el walking skeleton, sin persistencia entre sesiones.
#[derive(Default)]
struct PerkState {
    unlocked: bool,
    dinero: i64,
}

struct Perk(Mutex<PerkState>);

/// Mantiene vivo el servidor Postgres embebido y permite detenerlo al salir.
struct EmbeddedPostgres(Mutex<Option<postgresql_embedded::PostgreSQL>>);

#[derive(serde::Serialize)]
pub struct QueryResult {
    rows: Vec<serde_json::Value>,
}

#[derive(serde::Serialize)]
struct ScoreResult {
    pass: bool,
    dinero_ganado: i64,
    dinero_total: i64,
    mensaje: String,
}

#[derive(serde::Serialize)]
struct PerkStatus {
    unlocked: bool,
    dinero_total: i64,
}

#[tauri::command]
fn ticket_actual() -> &'static str {
    db::TICKET_ENUNCIADO
}

#[tauri::command]
async fn run_query(state: tauri::State<'_, AppState>, sql: String) -> Result<QueryResult, String> {
    db::run_query(&state.pool, &sql).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn submit_ticket(
    state: tauri::State<'_, AppState>,
    perk: tauri::State<'_, Perk>,
    sql: String,
) -> Result<ScoreResult, String> {
    let jugador = db::run_query(&state.pool, &sql).await.map_err(|e| e.to_string())?;
    let esperado = db::run_ticket_solution(&state.pool).await.map_err(|e| e.to_string())?;
    let pass = jugador.rows == esperado.rows;

    let mut perk_state = perk.0.lock().unwrap();
    let dinero_ganado = if pass { 500 } else { 0 };
    perk_state.dinero += dinero_ganado;

    Ok(ScoreResult {
        pass,
        dinero_ganado,
        dinero_total: perk_state.dinero,
        mensaje: if pass {
            "Ticket resuelto. Contabilidad procesará tu pago... eventualmente.".to_string()
        } else {
            "El resultado no coincide con lo que pidió Contabilidad. Revisa tu WHERE/ORDER BY.".to_string()
        },
    })
}

#[tauri::command]
fn unlock_perk(perk: tauri::State<'_, Perk>) -> Result<PerkStatus, String> {
    const COSTO: i64 = 300;
    let mut perk_state = perk.0.lock().unwrap();
    if perk_state.unlocked {
        return Ok(PerkStatus { unlocked: true, dinero_total: perk_state.dinero });
    }
    if perk_state.dinero < COSTO {
        return Err(format!(
            "No tienes suficiente dinero para este perk (cuesta {COSTO}, tienes {}).",
            perk_state.dinero
        ));
    }
    perk_state.dinero -= COSTO;
    perk_state.unlocked = true;
    Ok(PerkStatus { unlocked: true, dinero_total: perk_state.dinero })
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
                let (pg, pool) = db::init_embedded_postgres()
                    .await
                    .expect("no se pudo inicializar Postgres embebido — spike fallido");
                handle.manage(AppState { pool });
                handle.manage(Perk(Mutex::new(PerkState::default())));
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
