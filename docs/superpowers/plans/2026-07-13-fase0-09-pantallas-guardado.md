# Pantallas (Menú/Hub/Consola) + Guardado Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Separar la app en 3 pantallas navegables (Menú, Hub, Consola) y agregar un sistema de guardado/carga de un solo slot con autoguardado, para que la app deje de sentirse como una sola página web y se sienta como un juego con sesiones persistentes.

**Architecture:** El backend gana un módulo `guardado` (serialización a JSON de un snapshot ligero: tickets pendientes por id, no el `Ticket` completo) y 3 comandos nuevos (`existe_partida_guardada`, `iniciar_partida`, `cargar_partida`) que reemplazan en caliente `AppState`/`Jugador`/`Turno` — el mismo patrón de "reemplazar estado gestionado" que `confirmar_transicion_agencia` (Plan 8) ya estableció. `TurnoManejado` gana un campo `empresa: db::Company` (hoy no había ninguna forma de saber qué empresa está activa como dato, solo el catálogo cargado). El frontend gana 3 bloques de pantalla y una función `mostrarPantalla()` que los alterna — mismo mecanismo `.oculto` ya usado para los overlays.

**Tech Stack:** Rust (Tauri backend), `serde_json` (ya es dependencia, usado en `db::mod`), vanilla JS/HTML (frontend) — sin dependencias nuevas.

## Global Constraints

- Un solo slot de guardado: un archivo `partida.json` en `app.path().app_data_dir()`.
- Autoguardado (sin acción del jugador) después de: `resolver_ticket`, `cerrar_dia`, `confirmar_transicion_agencia`, e `iniciar_partida`. Un fallo de guardado nunca debe interrumpir el flujo del juego (se ignora silenciosamente — perder un autosave puntual es preferible a que el jugador no pueda seguir jugando por un problema de disco).
- Los tickets pendientes se persisten **por id**, nunca el `Ticket` completo — se reconstruyen contra `tickets::catalogo(empresa)` (si `fase == TrabajoNormal`) o `tickets::mini_boss_hospital_arcangel()` (en cualquier otra fase) al cargar.
- Navegación: Menú → Hub → Consola → (✓ Enviar + cerrar scoring, o "‹ Volver") → Hub. El overlay de la Agencia (Plan 8) sigue viviendo sobre el Hub, sin cambios.
- Spec de referencia: `docs/superpowers/specs/2026-07-13-fase0-09-pantallas-guardado-design.md`.

---

### Task 1: Derives de `Deserialize` en los tipos que se van a persistir

**Files:**
- Modify: `app/src-tauri/src/tickets/mod.rs`
- Modify: `app/src-tauri/src/db/mod.rs`
- Modify: `app/src-tauri/src/lib.rs`

**Interfaces:**
- Produces: `tickets::Arquetipo`, `tickets::Rango`, `db::Company`, `FaseArco` ganan `serde::Deserialize` (y `Company` gana también `serde::Serialize`, que no tenía). Sin cambios de comportamiento.

- [ ] **Step 1: `Arquetipo` y `Rango` en `tickets/mod.rs`**

Localizar:

```rust
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
```

Reemplazar:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Arquetipo {
    Select,
    Join,
    Agregacion,
}

/// Rango de carrera del jugador (Etapa 10, Plan 7): determina qué tickets
/// del catálogo puede recibir en su bandeja. El orden de declaración importa
/// — el derive de `Ord` decide qué rango "alcanza" a cuál según ese orden.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, serde::Serialize, serde::Deserialize)]
pub enum Rango {
    #[default]
    Becario,
    AuxiliarDeSistemas,
}
```

- [ ] **Step 2: `Company` en `db/mod.rs`**

Localizar:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Company {
    HospitalArcangel,
    Postafeta,
}
```

Reemplazar:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Company {
    HospitalArcangel,
    Postafeta,
}
```

- [ ] **Step 3: `FaseArco` en `lib.rs`**

Localizar:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
enum FaseArco {
    TrabajoNormal,
    MiniBoss,
    ArcoCompletado,
}
```

Reemplazar:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum FaseArco {
    TrabajoNormal,
    MiniBoss,
    ArcoCompletado,
}
```

- [ ] **Step 4: Correr la suite completa**

Run: `cd app/src-tauri && cargo build && cargo test --lib -- --nocapture`
Expected: build sin errores, 0 tests failed — mismo conteo que antes (este paso es solo derives, sin tests nuevos; el round-trip real de `Deserialize` se prueba en la Tarea 2).

- [ ] **Step 5: Commit**

```bash
git add app/src-tauri/src/tickets/mod.rs app/src-tauri/src/db/mod.rs app/src-tauri/src/lib.rs
git commit -m "Add Deserialize to Arquetipo, Rango, Company, and FaseArco for save persistence"
```

---

### Task 2: Módulo `guardado` (serialización a JSON, un solo slot)

**Files:**
- Create: `app/src-tauri/src/guardado/mod.rs`

**Interfaces:**
- Consumes: `tickets::{Arquetipo, Rango}`, `db::Company`, `crate::FaseArco` (todos visibles desde cualquier módulo del crate por ser hijos de la raíz, sin necesidad de marcarlos `pub` adicionalmente — `FaseArco` ya es accesible como `crate::FaseArco` aunque no tenga `pub` porque `guardado` es un módulo descendiente de la raíz del crate).
- Produces: `pub struct PartidaGuardada { ... }` (Serialize + Deserialize), `pub fn guardar(dir: &Path, partida: &PartidaGuardada) -> anyhow::Result<()>`, `pub fn cargar(dir: &Path) -> anyhow::Result<Option<PartidaGuardada>>`, `pub fn existe(dir: &Path) -> bool`.

- [ ] **Step 1: Escribir `app/src-tauri/src/guardado/mod.rs`**

```rust
use std::path::Path;

const ARCHIVO_GUARDADO: &str = "partida.json";

/// Snapshot serializable de todo el estado del juego (Etapa 11-G/Menú, Plan
/// 9) — se persiste como JSON en el directorio de datos de la app. Los
/// tickets pendientes se guardan por id, no como `Ticket` completo: evita
/// tener que exponer/reconstruir campos internos como `sql_dorada` en el
/// archivo de guardado, y los ids ya alcanzan para reconstruir el `Ticket`
/// completo contra el catálogo correcto al cargar (ver `lib.rs`).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PartidaGuardada {
    pub dinero: i64,
    pub reputacion: f64,
    pub xp_por_arquetipo: Vec<(crate::tickets::Arquetipo, i64)>,
    pub rango: crate::tickets::Rango,
    pub perks_desbloqueados: Vec<String>,
    pub perks_equipados: Vec<String>,
    pub empresa: crate::db::Company,
    pub fase: crate::FaseArco,
    pub indice_siguiente: usize,
    pub presupuesto_restante: u32,
    pub pendientes_ids: Vec<String>,
}

/// Escribe `partida` como JSON en `dir/partida.json`, sobreescribiendo
/// cualquier guardado anterior (un solo slot, Etapa 11-G/Menú, Plan 9).
pub fn guardar(dir: &Path, partida: &PartidaGuardada) -> anyhow::Result<()> {
    std::fs::create_dir_all(dir)?;
    let json = serde_json::to_string_pretty(partida)?;
    std::fs::write(dir.join(ARCHIVO_GUARDADO), json)?;
    Ok(())
}

/// Lee el guardado de `dir/partida.json` si existe. `Ok(None)` si todavía no
/// hay ningún archivo de guardado (primera vez que se abre la app).
pub fn cargar(dir: &Path) -> anyhow::Result<Option<PartidaGuardada>> {
    let ruta = dir.join(ARCHIVO_GUARDADO);
    if !ruta.exists() {
        return Ok(None);
    }
    let json = std::fs::read_to_string(ruta)?;
    Ok(Some(serde_json::from_str(&json)?))
}

/// `true` si ya existe un archivo de guardado en `dir` (Etapa 11-G/Menú,
/// Plan 9) — usado para habilitar "Cargar partida" en el Menú sin tener que
/// leer/parsear el archivo completo.
pub fn existe(dir: &Path) -> bool {
    dir.join(ARCHIVO_GUARDADO).exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Company;
    use crate::tickets::{Arquetipo, Rango};
    use crate::FaseArco;

    fn partida_de_prueba() -> PartidaGuardada {
        PartidaGuardada {
            dinero: 500,
            reputacion: 12.5,
            xp_por_arquetipo: vec![(Arquetipo::Select, 30), (Arquetipo::Join, 20)],
            rango: Rango::AuxiliarDeSistemas,
            perks_desbloqueados: vec!["instinto".to_string()],
            perks_equipados: vec!["instinto".to_string()],
            empresa: Company::Postafeta,
            fase: FaseArco::TrabajoNormal,
            indice_siguiente: 2,
            presupuesto_restante: 80,
            pendientes_ids: vec!["postafeta_reporte_paquetes_centro".to_string()],
        }
    }

    fn dir_de_prueba(nombre: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("query_path_test_guardado_{nombre}_{}", std::process::id()))
    }

    #[test]
    fn guardar_y_cargar_hace_round_trip_exacto() {
        let dir = dir_de_prueba("round_trip");
        let partida = partida_de_prueba();

        guardar(&dir, &partida).expect("debe poder guardar");
        let cargada = cargar(&dir).expect("debe poder cargar").expect("debe existir un guardado");

        assert_eq!(cargada.dinero, partida.dinero);
        assert_eq!(cargada.reputacion, partida.reputacion);
        assert_eq!(cargada.xp_por_arquetipo, partida.xp_por_arquetipo);
        assert_eq!(cargada.rango, partida.rango);
        assert_eq!(cargada.perks_desbloqueados, partida.perks_desbloqueados);
        assert_eq!(cargada.perks_equipados, partida.perks_equipados);
        assert_eq!(cargada.empresa, partida.empresa);
        assert_eq!(cargada.fase, partida.fase);
        assert_eq!(cargada.indice_siguiente, partida.indice_siguiente);
        assert_eq!(cargada.presupuesto_restante, partida.presupuesto_restante);
        assert_eq!(cargada.pendientes_ids, partida.pendientes_ids);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn existe_es_false_sin_guardado_y_true_despues_de_guardar() {
        let dir = dir_de_prueba("existe");
        assert!(!existe(&dir));

        guardar(&dir, &partida_de_prueba()).expect("debe poder guardar");
        assert!(existe(&dir));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn cargar_devuelve_none_si_no_existe_el_archivo() {
        let dir = dir_de_prueba("none");
        assert!(cargar(&dir).expect("no debe fallar, solo no encontrar nada").is_none());
    }
}
```

- [ ] **Step 2: Registrar el módulo en `app/src-tauri/src/lib.rs`**

Localizar:

```rust
mod db;
mod economia;
mod perks;
mod tickets;
```

Reemplazar:

```rust
mod db;
mod economia;
mod guardado;
mod perks;
mod tickets;
```

- [ ] **Step 3: Correr la suite completa**

Run: `cd app/src-tauri && cargo test --lib -- --nocapture`
Expected: 0 failed, incluyendo los 3 tests nuevos de `guardado::tests`.

- [ ] **Step 4: Commit**

```bash
git add app/src-tauri/src/guardado/mod.rs app/src-tauri/src/lib.rs
git commit -m "Add guardado module for single-slot JSON save/load"
```

---

### Task 3: Wiring en `lib.rs` — empresa activa, autoguardado, comandos de Menú

**Files:**
- Modify: `app/src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: `guardado::{PartidaGuardada, guardar, cargar, existe}` (Tarea 2).
- Produces: `TurnoManejado.empresa: db::Company` (campo nuevo); `struct EstadoJuegoView { dinero, reputacion, rango, presupuesto_restante, pendientes, fase }`; comandos `existe_partida_guardada() -> bool`, `iniciar_partida() -> Result<EstadoJuegoView, String>`, `cargar_partida() -> Result<EstadoJuegoView, String>`.

- [ ] **Step 1: Agregar el campo `empresa` a `TurnoManejado`**

Localizar:

```rust
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
```

Reemplazar:

```rust
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
```

- [ ] **Step 2: Agregar `empresa` a la construcción de `TurnoManejado` en `transicionar_a_empresa`**

Localizar:

```rust
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
```

Reemplazar:

```rust
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
```

- [ ] **Step 3: Agregar `empresa` a las 5 construcciones de `TurnoManejado` en los tests ya existentes**

Localizar (test `escalar_y_avanzar_refleja_el_catalogo_desbloqueado_tras_un_ascenso`):

```rust
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
```

Reemplazar:

```rust
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
```

Localizar (test `actualizar_fase_dispara_el_lote_del_mini_boss_al_ascender`):

```rust
        let mut manejado = TurnoManejado {
            catalogo,
            indice_siguiente,
            actual: turno_inicial,
            fase: FaseArco::TrabajoNormal,
        };
        let mut jugador = economia::EstadoJugador::default();

        manejado.actualizar_fase(true, &mut jugador);
```

Reemplazar:

```rust
        let mut manejado = TurnoManejado {
            catalogo,
            indice_siguiente,
            actual: turno_inicial,
            fase: FaseArco::TrabajoNormal,
            empresa: db::Company::HospitalArcangel,
        };
        let mut jugador = economia::EstadoJugador::default();

        manejado.actualizar_fase(true, &mut jugador);
```

Localizar (test `actualizar_fase_completa_el_arco_al_vaciar_el_lote_del_mini_boss`):

```rust
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
```

Reemplazar:

```rust
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
```

Localizar (test `actualizar_fase_avanza_el_turno_normal_cuando_no_hay_ascenso_ni_mini_boss`):

```rust
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
```

Reemplazar:

```rust
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
```

Localizar (test `actualizar_fase_no_hace_nada_en_arco_completado`):

```rust
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
```

Reemplazar:

```rust
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
```

- [ ] **Step 4: Agregar `DirectorioGuardado`, `EstadoJuegoView`, `estado_de_partida_nueva`, y `autoguardar`**

Localizar:

```rust
/// Evita que dos llamadas concurrentes a `confirmar_transicion_agencia`
/// corran la transición dos veces (Etapa 11-G, Plan 8 — hallazgo de
/// revisión): sin esto, dos invocaciones simultáneas podrían pasar ambas el
/// check de `fase == ArcoCompletado` antes de que la primera termine de
/// escribir el nuevo estado. `AtomicBool` en vez de otro `Mutex` porque
/// necesitamos poder consultarlo/liberarlo sin arrastrar un guard a través
/// del `.await` de la transición.
struct TransicionEnCurso(std::sync::atomic::AtomicBool);
```

Reemplazar:

```rust
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
```

- [ ] **Step 5: Autoguardar al final de `resolver_ticket`**

Localizar:

```rust
async fn resolver_ticket(
    state: tauri::State<'_, AppState>,
    jugador: tauri::State<'_, Jugador>,
    turno_state: tauri::State<'_, Turno>,
    id: String,
    sql: String,
) -> Result<ScoreResult, String> {
```

Reemplazar:

```rust
async fn resolver_ticket(
    state: tauri::State<'_, AppState>,
    jugador: tauri::State<'_, Jugador>,
    turno_state: tauri::State<'_, Turno>,
    dir: tauri::State<'_, DirectorioGuardado>,
    id: String,
    sql: String,
) -> Result<ScoreResult, String> {
```

Localizar:

```rust
    let mut manejado = turno_state.0.lock().unwrap();
    manejado.actualizar_fase(ascendio, &mut estado);

    Ok(ScoreResult {
```

Reemplazar:

```rust
    let mut manejado = turno_state.0.lock().unwrap();
    manejado.actualizar_fase(ascendio, &mut estado);
    autoguardar(&dir.0, &estado, &manejado);

    Ok(ScoreResult {
```

- [ ] **Step 6: Autoguardar al final de `cerrar_dia`**

Localizar:

```rust
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
```

Reemplazar:

```rust
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
```

- [ ] **Step 7: Autoguardar al final de `confirmar_transicion_agencia`**

Localizar:

```rust
#[tauri::command]
async fn confirmar_transicion_agencia(
    state: tauri::State<'_, AppState>,
    jugador: tauri::State<'_, Jugador>,
    turno_state: tauri::State<'_, Turno>,
    embedded: tauri::State<'_, EmbeddedPostgres>,
    transicion: tauri::State<'_, TransicionEnCurso>,
) -> Result<EstadoTurnoView, String> {
```

Reemplazar:

```rust
#[tauri::command]
async fn confirmar_transicion_agencia(
    state: tauri::State<'_, AppState>,
    jugador: tauri::State<'_, Jugador>,
    turno_state: tauri::State<'_, Turno>,
    embedded: tauri::State<'_, EmbeddedPostgres>,
    transicion: tauri::State<'_, TransicionEnCurso>,
    dir: tauri::State<'_, DirectorioGuardado>,
) -> Result<EstadoTurnoView, String> {
```

Localizar:

```rust
        jugador.0.lock().unwrap().reputacion = 0.0;
        *state.0.lock().unwrap() = pool;
        *turno_state.0.lock().unwrap() = nuevo_manejado;

        Ok(EstadoTurnoView::from(&*turno_state.0.lock().unwrap()))
    }
    .await;
```

Reemplazar:

```rust
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
```

- [ ] **Step 8: Comandos `existe_partida_guardada`, `iniciar_partida`, `cargar_partida`**

Localizar:

```rust
#[tauri::command]
fn catalogo_perks(jugador: tauri::State<'_, Jugador>) -> Vec<PerkConEstado> {
```

Reemplazar:

```rust
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
    dir: tauri::State<'_, DirectorioGuardado>,
) -> Result<EstadoJuegoView, String> {
    let pg = embedded
        .0
        .lock()
        .unwrap()
        .take()
        .ok_or_else(|| "Postgres embebido no está disponible.".to_string())?;
    let resultado = estado_de_partida_nueva(&pg).await;
    embedded.0.lock().unwrap().replace(pg);
    let (pool, jugador_nuevo, manejado_nuevo) = resultado.map_err(|e| e.to_string())?;

    *jugador.0.lock().unwrap() = jugador_nuevo;
    *state.0.lock().unwrap() = pool;
    *turno_state.0.lock().unwrap() = manejado_nuevo;

    let estado = jugador.0.lock().unwrap();
    let manejado = turno_state.0.lock().unwrap();
    autoguardar(&dir.0, &estado, &manejado);
    Ok(EstadoJuegoView::construir(&estado, &manejado))
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
    dir: tauri::State<'_, DirectorioGuardado>,
) -> Result<EstadoJuegoView, String> {
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

#[tauri::command]
fn catalogo_perks(jugador: tauri::State<'_, Jugador>) -> Vec<PerkConEstado> {
```

- [ ] **Step 9: Registrar los comandos nuevos**

Localizar:

```rust
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
```

Reemplazar:

```rust
        .invoke_handler(tauri::generate_handler![
            turno_actual,
            rango_actual,
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
            desequipar_perk
        ])
```

- [ ] **Step 10: `setup()` usa `estado_de_partida_nueva` y gestiona `DirectorioGuardado`**

Localizar:

```rust
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
```

Reemplazar:

```rust
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
```

- [ ] **Step 11: Compilar y correr la suite completa**

Run: `cd app/src-tauri && cargo build && cargo test --lib -- --nocapture`
Expected: build sin errores, 0 tests failed.

- [ ] **Step 12: Commit**

```bash
git add app/src-tauri/src/lib.rs
git commit -m "Wire save/load commands, active-company tracking, and autosave hooks"
```

---

### Task 4: Frontend — pantallas de Menú/Hub/Consola y navegación

**Files:**
- Modify: `app/src/index.html`
- Modify: `app/src/main.js`
- Modify: `app/src/styles.css`

**Interfaces:**
- Consumes: comandos `existe_partida_guardada`, `iniciar_partida`, `cargar_partida` (Tarea 3); campo `fase` ya existente en las respuestas de turno (Plan 8).
- Produces: navegación `mostrarPantalla("menu" | "hub" | "consola")`; botón "‹ Volver"; el Hub y la Consola quedan en secciones separadas.

- [ ] **Step 1: Regla CSS genérica de `.oculto`, en `app/src/styles.css`**

`.oculto` hoy solo tiene efecto combinado con `.scoring-overlay`; las pantallas nuevas necesitan poder ocultarse por sí solas.

Localizar:

```css
.scoring-overlay.oculto {
  display: none;
}
```

Reemplazar:

```css
.oculto {
  display: none;
}
```

(Esta regla por sí sola ya cubre el caso de `.scoring-overlay.oculto` — cualquier elemento con `.oculto` se oculta sin importar qué otras clases tenga.)

- [ ] **Step 2: Reestructurar `app/src/index.html` en 3 pantallas**

Localizar (todo el `<body>`):

```html
  <body>
    <main class="container">
      <header>
        <h1>Query Path <span class="subtitle">— spike técnico (Tauri + Postgres embebido)</span></h1>
        <div class="stats">
          <span>💰 <span id="dinero">0</span></span>
          <span>⭐ <span id="reputacion">0</span></span>
          <span>🎓 <span id="rango">Becario</span></span>
        </div>
      </header>

      <section class="bandeja">
        <h2 id="bandeja-titulo">Bandeja — turno actual</h2>
        <p>⏱️ Presupuesto de tiempo: <span id="presupuesto">0</span></p>
        <ul id="lista-tickets"></ul>
        <button id="btn-cerrar-dia">Cerrar día</button>
      </section>

      <section class="console">
        <p id="ticket-activo-info">Elige un ticket de la bandeja para empezar.</p>
        <textarea id="sql-input" spellcheck="false" placeholder="SELECT * FROM pacientes;"></textarea>
        <div class="actions">
          <button id="btn-play">▶ Play</button>
          <button id="btn-submit">✓ Enviar ticket</button>
        </div>
      </section>

      <section class="perks">
        <h2>Perks</h2>
        <select id="perks-select"></select>
        <div class="actions">
          <button id="btn-unlock-perk">Desbloquear</button>
          <button id="btn-equip-perk">Equipar/Desequipar</button>
        </div>
        <p id="perks-equipados-msg"></p>
      </section>

      <section class="output">
        <h2>Resultado</h2>
        <p id="status-msg"></p>
        <div id="result-table"></div>
      </section>
    </main>

    <div id="scoring-overlay" class="scoring-overlay oculto">
      <div class="scoring-panel">
        <h2 id="scoring-titulo">Resultado</h2>
        <p>Correctitud: <span id="scoring-correctitud">0</span></p>
        <p>Velocidad: <span id="scoring-velocidad">0</span></p>
        <p>Buenas prácticas: <span id="scoring-practicas">0</span></p>
        <p>💰 +<span id="scoring-dinero">0</span></p>
        <p>⭐ +<span id="scoring-reputacion">0</span></p>
        <p id="scoring-mentor"></p>
        <p id="scoring-ascenso"></p>
        <button id="btn-cerrar-scoring">Cerrar</button>
      </div>
    </div>

    <div id="agencia-overlay" class="scoring-overlay oculto">
      <div class="scoring-panel">
        <h2>Grupo Ómega RH — Reasignación</h2>
        <p>Has superado al Auditor de Cumplimiento. Tu siguiente asignación:</p>
        <p><strong>Postafeta</strong> — todo el Slack de la empresa lo administra un becario invisible llamado Kevin; todo viene firmado "- Kevin".</p>
        <button id="btn-confirmar-agencia">Aceptar reasignación</button>
      </div>
    </div>
  </body>
```

Reemplazar:

```html
  <body>
    <div id="pantalla-menu" class="container">
      <header>
        <h1>Query Path <span class="subtitle">— spike técnico (Tauri + Postgres embebido)</span></h1>
      </header>
      <div class="actions">
        <button id="btn-cargar-partida" disabled>Cargar partida</button>
        <button id="btn-iniciar-partida">Iniciar partida</button>
        <button id="btn-multijugador" disabled>Multijugador (próximamente)</button>
      </div>
    </div>

    <div id="app-shell" class="oculto">
      <header>
        <h1>Query Path <span class="subtitle">— spike técnico (Tauri + Postgres embebido)</span></h1>
        <div class="stats">
          <span>💰 <span id="dinero">0</span></span>
          <span>⭐ <span id="reputacion">0</span></span>
          <span>🎓 <span id="rango">Becario</span></span>
        </div>
      </header>

      <main class="container" id="pantalla-hub">
        <section class="bandeja">
          <h2 id="bandeja-titulo">Bandeja — turno actual</h2>
          <p>⏱️ Presupuesto de tiempo: <span id="presupuesto">0</span></p>
          <ul id="lista-tickets"></ul>
          <button id="btn-cerrar-dia">Cerrar día</button>
        </section>

        <section class="perks">
          <h2>Perks</h2>
          <select id="perks-select"></select>
          <div class="actions">
            <button id="btn-unlock-perk">Desbloquear</button>
            <button id="btn-equip-perk">Equipar/Desequipar</button>
          </div>
          <p id="perks-equipados-msg"></p>
        </section>
      </main>

      <main class="container oculto" id="pantalla-consola">
        <button id="btn-volver-hub">‹ Volver</button>

        <section class="console">
          <p id="ticket-activo-info">Elige un ticket de la bandeja para empezar.</p>
          <textarea id="sql-input" spellcheck="false" placeholder="SELECT * FROM pacientes;"></textarea>
          <div class="actions">
            <button id="btn-play">▶ Play</button>
            <button id="btn-submit">✓ Enviar ticket</button>
          </div>
        </section>

        <section class="output">
          <h2>Resultado</h2>
          <p id="status-msg"></p>
          <div id="result-table"></div>
        </section>
      </main>
    </div>

    <div id="scoring-overlay" class="scoring-overlay oculto">
      <div class="scoring-panel">
        <h2 id="scoring-titulo">Resultado</h2>
        <p>Correctitud: <span id="scoring-correctitud">0</span></p>
        <p>Velocidad: <span id="scoring-velocidad">0</span></p>
        <p>Buenas prácticas: <span id="scoring-practicas">0</span></p>
        <p>💰 +<span id="scoring-dinero">0</span></p>
        <p>⭐ +<span id="scoring-reputacion">0</span></p>
        <p id="scoring-mentor"></p>
        <p id="scoring-ascenso"></p>
        <button id="btn-cerrar-scoring">Cerrar</button>
      </div>
    </div>

    <div id="agencia-overlay" class="scoring-overlay oculto">
      <div class="scoring-panel">
        <h2>Grupo Ómega RH — Reasignación</h2>
        <p>Has superado al Auditor de Cumplimiento. Tu siguiente asignación:</p>
        <p><strong>Postafeta</strong> — todo el Slack de la empresa lo administra un becario invisible llamado Kevin; todo viene firmado "- Kevin".</p>
        <button id="btn-confirmar-agencia">Aceptar reasignación</button>
      </div>
    </div>
  </body>
```

- [ ] **Step 3: `mostrarPantalla`, refs nuevas, y quitar `cargarRango` (queda sin llamadores)**

Localizar:

```js
let sqlInput, statusMsg, resultTable, dineroEl, reputacionEl, rangoEl;
let perksSelect, perksEquipadosMsg;
let presupuestoEl, listaTickets, ticketActivoInfo, bandejaTitulo;
let scoringOverlay, scoringAscenso, agenciaOverlay;
```

Reemplazar:

```js
let sqlInput, statusMsg, resultTable, dineroEl, reputacionEl, rangoEl;
let perksSelect, perksEquipadosMsg;
let presupuestoEl, listaTickets, ticketActivoInfo, bandejaTitulo;
let scoringOverlay, scoringAscenso, agenciaOverlay;
let pantallaMenu, appShell, pantallaHub, pantallaConsola, btnCargarPartida;

function mostrarPantalla(nombre) {
  pantallaMenu.classList.toggle("oculto", nombre !== "menu");
  appShell.classList.toggle("oculto", nombre === "menu");
  pantallaHub.classList.toggle("oculto", nombre !== "hub");
  pantallaConsola.classList.toggle("oculto", nombre !== "consola");
}
```

Localizar:

```js
async function cargarRango() {
  const rango = await invoke("rango_actual");
  renderRango(rango);
}
```

Reemplazar: (se elimina — sin llamadores tras este plan; `renderRango` se sigue usando desde otros puntos)

```js
```

- [ ] **Step 4: `seleccionarTicket` navega a la Consola**

Localizar:

```js
function seleccionarTicket(ticket) {
  ticketActivoId = ticket.id;
  ticketActivoInfo.textContent = `Motivo: ${ticket.motivo}\nSolicitud: ${ticket.solicitud}`;
  sqlInput.value = ticket.sql_inicial || "SELECT * FROM pacientes;";
}
```

Reemplazar:

```js
function seleccionarTicket(ticket) {
  ticketActivoId = ticket.id;
  ticketActivoInfo.textContent = `Motivo: ${ticket.motivo}\nSolicitud: ${ticket.solicitud}`;
  sqlInput.value = ticket.sql_inicial || "SELECT * FROM pacientes;";
  mostrarPantalla("consola");
}
```

- [ ] **Step 5: Cerrar el scoring vuelve al Hub; agregar `pintarHubDesdeEstadoJuego`, `iniciarPartida`, `cargarPartida`, `mostrarMenu`**

Localizar:

```js
async function cargarTurno() {
  const estadoTurno = await invoke("turno_actual");
  renderBandeja(estadoTurno);
}
```

Reemplazar:

```js
async function cargarTurno() {
  const estadoTurno = await invoke("turno_actual");
  renderBandeja(estadoTurno);
}

function pintarHubDesdeEstadoJuego(estadoJuego) {
  dineroEl.textContent = estadoJuego.dinero;
  reputacionEl.textContent = estadoJuego.reputacion.toFixed(1);
  renderRango(estadoJuego.rango);
  renderBandeja(estadoJuego);
  ticketActivoId = null;
  mostrarPantalla("hub");
}

async function mostrarMenu() {
  mostrarPantalla("menu");
  const existePartida = await invoke("existe_partida_guardada");
  btnCargarPartida.disabled = !existePartida;
}

async function iniciarPartida() {
  const estadoJuego = await invoke("iniciar_partida");
  pintarHubDesdeEstadoJuego(estadoJuego);
  await cargarPerks();
  setStatus("Partida nueva iniciada.", "ok");
}

async function cargarPartida() {
  try {
    const estadoJuego = await invoke("cargar_partida");
    pintarHubDesdeEstadoJuego(estadoJuego);
    await cargarPerks();
    setStatus("Partida cargada.", "ok");
  } catch (err) {
    setStatus(String(err), "error");
  }
}
```

- [ ] **Step 6: DOMContentLoaded — arranca en el Menú, enlaza los botones nuevos**

Localizar:

```js
window.addEventListener("DOMContentLoaded", async () => {
  sqlInput = document.querySelector("#sql-input");
  statusMsg = document.querySelector("#status-msg");
  resultTable = document.querySelector("#result-table");
  dineroEl = document.querySelector("#dinero");
  reputacionEl = document.querySelector("#reputacion");
  rangoEl = document.querySelector("#rango");
  perksSelect = document.querySelector("#perks-select");
  perksEquipadosMsg = document.querySelector("#perks-equipados-msg");
  presupuestoEl = document.querySelector("#presupuesto");
  listaTickets = document.querySelector("#lista-tickets");
  ticketActivoInfo = document.querySelector("#ticket-activo-info");
  bandejaTitulo = document.querySelector("#bandeja-titulo");
  scoringOverlay = document.querySelector("#scoring-overlay");
  scoringAscenso = document.querySelector("#scoring-ascenso");
  agenciaOverlay = document.querySelector("#agencia-overlay");

  await cargarTurno();
  await cargarRango();
  await cargarPerks();

  document.querySelector("#btn-play").addEventListener("click", runQuery);
  document.querySelector("#btn-submit").addEventListener("click", submitTicket);
  document.querySelector("#btn-cerrar-dia").addEventListener("click", cerrarDia);
  document.querySelector("#btn-cerrar-scoring").addEventListener("click", () => scoringOverlay.classList.add("oculto"));
  document.querySelector("#btn-unlock-perk").addEventListener("click", desbloquearPerkSeleccionado);
  document.querySelector("#btn-equip-perk").addEventListener("click", equiparODesequiparPerkSeleccionado);
  document.querySelector("#btn-confirmar-agencia").addEventListener("click", confirmarTransicionAgencia);
});
```

Reemplazar:

```js
window.addEventListener("DOMContentLoaded", async () => {
  sqlInput = document.querySelector("#sql-input");
  statusMsg = document.querySelector("#status-msg");
  resultTable = document.querySelector("#result-table");
  dineroEl = document.querySelector("#dinero");
  reputacionEl = document.querySelector("#reputacion");
  rangoEl = document.querySelector("#rango");
  perksSelect = document.querySelector("#perks-select");
  perksEquipadosMsg = document.querySelector("#perks-equipados-msg");
  presupuestoEl = document.querySelector("#presupuesto");
  listaTickets = document.querySelector("#lista-tickets");
  ticketActivoInfo = document.querySelector("#ticket-activo-info");
  bandejaTitulo = document.querySelector("#bandeja-titulo");
  scoringOverlay = document.querySelector("#scoring-overlay");
  scoringAscenso = document.querySelector("#scoring-ascenso");
  agenciaOverlay = document.querySelector("#agencia-overlay");
  pantallaMenu = document.querySelector("#pantalla-menu");
  appShell = document.querySelector("#app-shell");
  pantallaHub = document.querySelector("#pantalla-hub");
  pantallaConsola = document.querySelector("#pantalla-consola");
  btnCargarPartida = document.querySelector("#btn-cargar-partida");

  await mostrarMenu();

  document.querySelector("#btn-play").addEventListener("click", runQuery);
  document.querySelector("#btn-submit").addEventListener("click", submitTicket);
  document.querySelector("#btn-cerrar-dia").addEventListener("click", cerrarDia);
  document.querySelector("#btn-cerrar-scoring").addEventListener("click", () => {
    scoringOverlay.classList.add("oculto");
    mostrarPantalla("hub");
  });
  document.querySelector("#btn-unlock-perk").addEventListener("click", desbloquearPerkSeleccionado);
  document.querySelector("#btn-equip-perk").addEventListener("click", equiparODesequiparPerkSeleccionado);
  document.querySelector("#btn-confirmar-agencia").addEventListener("click", confirmarTransicionAgencia);
  document.querySelector("#btn-iniciar-partida").addEventListener("click", iniciarPartida);
  document.querySelector("#btn-cargar-partida").addEventListener("click", cargarPartida);
  document.querySelector("#btn-volver-hub").addEventListener("click", () => {
    ticketActivoId = null;
    ticketActivoInfo.textContent = "Elige un ticket de la bandeja para empezar.";
    mostrarPantalla("hub");
  });
});
```

- [ ] **Step 7: Verificación**

Este proyecto no tiene runner de tests de frontend, y la app corre en una ventana nativa de Tauri sin herramienta de captura automatizada consistente en todos los entornos (mismo patrón ya usado en Planes 6/7/8) — la corrección se cubre con revisión cuidadosa del diff (ids consistentes entre HTML/JS, cada pantalla se oculta/muestra correctamente) más la verificación manual guiada al final del plan: abrir la app → ver el Menú → "Iniciar partida" → Hub → clic en un ticket → Consola → "‹ Volver" → Hub → clic en un ticket → resolverlo → Hub de nuevo → cerrar la app → reabrir → Menú con "Cargar partida" habilitado → cargar y confirmar que el estado coincide.

- [ ] **Step 8: Commit**

```bash
git add app/src/index.html app/src/main.js app/src/styles.css
git commit -m "Add Menu/Hub/Console screen navigation and wire save/load to the frontend"
```

---

## Self-Review Notes

- **Cobertura del spec:** las 3 pantallas y su navegación (Menú→Hub→Consola→Hub, botón "‹ Volver") ✓ (Tarea 4), guardado de un solo slot con autoguardado en los 4 puntos acordados ✓ (Tareas 2-3), persistencia de tickets por id en vez del `Ticket` completo ✓ (Tarea 2/3), Menú gatea Cargar/Iniciar según exista o no un guardado ✓ (Tareas 3-4).
- **Refinamiento respecto al spec de diseño:** el spec decía "`setup()` deja de construir el turno inicial automáticamente" — durante la planeación se determinó que es más simple mantener `setup()` construyendo SIEMPRE un estado de partida nueva por defecto (vía `estado_de_partida_nueva`, la misma función que usa `iniciar_partida`), y que el Menú simplemente decide si lo deja así o lo reemplaza con `cargar_partida`. El comportamiento observable descrito en el spec (el Menú gatea el acceso al Hub) no cambia — es una simplificación de implementación, no una desviación de comportamiento.
- **Hueco cerrado durante la planeación (no estaba en el spec original):** el spec no mencionaba que el Hub necesita ver dinero/reputación/rango *inmediatamente* al entrar desde el Menú (antes de resolver cualquier ticket) — hoy esos valores solo se actualizan dentro de `submitTicket`. Se agregó `EstadoJuegoView` (Tarea 3) para que `iniciar_partida`/`cargar_partida` devuelvan todo lo que el Hub necesita en una sola respuesta.
- **Placeholders:** ninguno — cada Step tiene código completo, comandos exactos, y salida esperada.
- **Consistencia de tipos:** `TurnoManejado.empresa` se agrega una sola vez y se actualiza en las 6 construcciones existentes (1 de código + 5 de tests) en el mismo Step; `EstadoJuegoView`/`PartidaGuardada` comparten los mismos nombres de campo donde corresponde (`fase`, `rango`, `dinero`, `reputacion`) sin renombrados a mitad de camino.
- **Concurrencia:** los 3 comandos async nuevos (`iniciar_partida`, `cargar_partida`, y el autoguardado agregado a `confirmar_transicion_agencia`) siguen el mismo patrón ya verificado en el Plan 8 — ningún `MutexGuard` se mantiene retenido a través de un `.await`; cada lock se toma y suelta en un bloque síncrono antes de cualquier punto de espera async.
- **Alcance:** 4 tareas con su propio ciclo de test y commit. El vestido visual (escritorio, retratos, marco de terminal) queda fuera — es el Plan 10, que se construye encima de estas mismas 3 pantallas sin tocar su estructura ni sus comandos.

## Execution Handoff

Plan completo y guardado en `docs/superpowers/plans/2026-07-13-fase0-09-pantallas-guardado.md`. Dos opciones de ejecución:

1. **Subagent-Driven (recomendado)** — despacho un subagente fresco por tarea, reviso el resultado entre cada una antes de seguir
2. **Ejecución inline** — ejecuto las tareas en esta sesión con executing-plans, ejecución por lotes con checkpoints

¿Cuál prefieres?
