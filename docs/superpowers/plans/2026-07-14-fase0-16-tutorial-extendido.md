# Fase 0 / Plan 16: Escalón fácil + Tutorial extendido Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a below-simple ticket tier (SELECT specific columns, no WHERE/ORDER BY) to Hospital Arcángel's catalog, and extend Plan 15's Mentor tutorial from one guided ticket into three continuous segments: the new easy ticket (deeper table/column teaching), the existing Cardiología ticket (now with a comparisons beat before WHERE), and a closing Hub tour explaining money, reputation, perks, and rank-up.

**Architecture:** One new Rust template function (`plantilla_reporte_simple_sin_orden`) plus 3 new catalog entries in `tickets/hospital_arcangel.rs`, inserted before the 3 existing simple tickets — this reorders what a new player's first two tickets are. `app/src/main.js`'s tutorial-start guard becomes a two-id check. `app/src/tutorial.js` is rewritten in place: every existing beat function is kept (some renamed to `...A`/`...B` to disambiguate which ticket they belong to), three new beats are added for the new ticket and three more for the Hub tour, and three of the four `notificar*` entry points (`notificarClicPrimerTicket`, `notificarClicPlay`, `notificarCierreScoring`) become thin dispatchers to a per-tramo "what happens next" function pointer instead of calling one hardcoded function each. `app/src/dialogo.js` and `app/src/audio.js` are untouched.

**Tech Stack:** Rust/sqlx/Tauri (Task 1), vanilla JS ES modules (Task 2) — same stack as Plan 15, no new dependencies.

> **Post-implementation correction (final whole-branch review):** Task 1's steps below insert all 3 new tickets before `hospital_reporte_pacientes_cardiologia`. That is **wrong** and was caught by the final review — a turn only serves `TAMANO_LOTE = 3` tickets (`turno/mod.rs`), so putting all 3 new easy tickets ahead of Cardiología pushes Cardiología out of a new game's opening batch entirely, meaning `pendientes[1]` is never Cardiología and the tutorial's start guard never fires. The shipped, correct order only puts `hospital_reporte_departamentos` before Cardiología; `hospital_reporte_empleados_directorio` and `hospital_reporte_habitaciones_inventario` go **after** the 3 original simple tickets instead. See commit `7036623` ("Fix Hospital Arcangel catalog order so tutorial's Cardiologia guard fires") for the actual fix and its regression test. The step-by-step code below is left as originally written for historical accuracy of what was planned — do not copy Task 1 Step 5's ticket ordering verbatim if referencing this plan later.

## Global Constraints

- Scope is Hospital Arcángel only — Postafeta is untouched.
- The new easy tier is exactly "SELECT specific columns, no WHERE, no ORDER BY" — same Becario rank as the existing simple tickets (`arquetipos: vec![Arquetipo::Select]`).
- Because these new tickets have no `ORDER BY`, Postgres does not guarantee row order, so they must be created with `requiere_orden: false` (multiset comparison), unlike every existing `plantilla_reporte_simple` ticket which has `requiere_orden: true`.
- The tutorial never duplicates the real SQL validation engine — clause-advance checks stay a loose, case/whitespace-insensitive substring match (unchanged from Plan 15).
- The skip button ("Ya sé SQL, saltar") must remain clickable through all three tramos, exactly as it was through Plan 15's single tramo.
- Strictly frontend + one Rust file for the new tickets — no new Tauri commands.
- No frontend test runner exists in this project — correctness for `tutorial.js`/`main.js` comes from careful diff self-review plus manual verification in the running app. Rust changes use `cargo test` (TDD), same as every prior plan that touched Rust.

---

### Task 1: New "sin orden" ticket template + 3 new Hospital Arcángel tickets

**Files:**
- Modify: `app/src-tauri/src/tickets/mod.rs:91-117` (add `plantilla_reporte_simple_sin_orden` right after `plantilla_reporte_simple`), `mod.rs:261-274` and `mod.rs:324-332` (tests)
- Modify: `app/src-tauri/src/tickets/hospital_arcangel.rs:1-32` (imports + 3 new catalog entries), `hospital_arcangel.rs:135-143` (test)

**Interfaces:**
- Produces (consumed by Task 2 only in the sense that the frontend now expects these ids to exist): three new ticket ids — `"hospital_reporte_departamentos"`, `"hospital_reporte_empleados_directorio"`, `"hospital_reporte_habitaciones_inventario"` — and a new `pub(crate)`-visibility-equivalent function `plantilla_reporte_simple_sin_orden(id, solicitante, motivo, solicitud, sql_dorada, costo_tiempo) -> Ticket` (same signature as `plantilla_reporte_simple`, just `requiere_orden: false`).

- [ ] **Step 1: Write the failing test for the new template function**

In `app/src-tauri/src/tickets/mod.rs`, find the existing test (lines 265-274):

```rust
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
```

Right after it, add:

```rust
    #[test]
    fn plantilla_reporte_simple_sin_orden_arma_un_ticket_sin_requerir_orden() {
        let ticket =
            plantilla_reporte_simple_sin_orden("id6", "Alguien", "un motivo", "una solicitud", "SELECT 1", 10);
        assert_eq!(ticket.tipo, TipoTicket::ReporteAnalisis);
        assert_eq!(ticket.arquetipos, vec![Arquetipo::Select]);
        assert!(ticket.sql_inicial.is_none());
        assert!(
            !ticket.requiere_orden,
            "sin ORDER BY, la comparación debe ser por conjunto, no por orden exacto"
        );
        assert_eq!((ticket.peso_correctitud, ticket.peso_velocidad, ticket.peso_practicas), (0.6, 0.2, 0.2));
        assert_eq!((ticket.valor_base, ticket.factor_reputacion), (100, 0.5));
    }
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cd app/src-tauri && "$HOME/.cargo/bin/cargo.exe" test --quiet --lib tickets::tests::plantilla_reporte_simple_sin_orden_arma_un_ticket_sin_requerir_orden`
Expected: compile error — `plantilla_reporte_simple_sin_orden` doesn't exist yet (`error[E0425]: cannot find function`).

- [ ] **Step 3: Implement the new template function**

In `app/src-tauri/src/tickets/mod.rs`, right after the existing `plantilla_reporte_simple` function (ends at line 117, right before the `/// Plantilla "reporte agregado"` doc comment on line 119), add:

```rust
/// Plantilla "reporte simple sin orden": igual que `plantilla_reporte_simple`
/// pero para solicitudes que no piden ningún orden particular — sin `ORDER
/// BY`, Postgres no garantiza el orden de las filas, así que la comparación
/// de correctitud debe ser por conjunto (`requiere_orden: false`), no por
/// secuencia exacta (Plan 16).
fn plantilla_reporte_simple_sin_orden(
    id: &'static str,
    solicitante: &'static str,
    motivo: impl Into<String>,
    solicitud: impl Into<String>,
    sql_dorada: impl Into<String>,
    costo_tiempo: u32,
) -> Ticket {
    Ticket {
        requiere_orden: false,
        ..plantilla_reporte_simple(id, solicitante, motivo, solicitud, sql_dorada, costo_tiempo)
    }
}
```

This reuses `plantilla_reporte_simple` via Rust's struct-update syntax (`..plantilla_reporte_simple(...)`) rather than duplicating all 15 fields — the only field that actually differs is `requiere_orden`.

- [ ] **Step 4: Run the test to verify it passes**

Run: `cd app/src-tauri && "$HOME/.cargo/bin/cargo.exe" test --quiet --lib tickets::tests::plantilla_reporte_simple_sin_orden_arma_un_ticket_sin_requerir_orden`
Expected: `test result: ok. 1 passed`

- [ ] **Step 5: Add the 3 new tickets to Hospital Arcángel's catalog**

In `app/src-tauri/src/tickets/hospital_arcangel.rs`, change the imports (lines 1-5) from:

```rust
use super::{
    plantilla_depuracion, plantilla_reporte_agregado, plantilla_reporte_join,
    plantilla_reporte_join_agregado, plantilla_reporte_simple, Arquetipo, Prioridad, Ticket,
    TipoTicket,
};
```

to:

```rust
use super::{
    plantilla_depuracion, plantilla_reporte_agregado, plantilla_reporte_join,
    plantilla_reporte_join_agregado, plantilla_reporte_simple, plantilla_reporte_simple_sin_orden,
    Arquetipo, Prioridad, Ticket, TipoTicket,
};
```

Then, in the same file, change the start of `catalogo()` (lines 7-9) from:

```rust
pub(crate) fn catalogo() -> Vec<Ticket> {
    vec![
        plantilla_reporte_simple(
            "hospital_reporte_pacientes_cardiologia",
```

to:

```rust
pub(crate) fn catalogo() -> Vec<Ticket> {
    vec![
        plantilla_reporte_simple_sin_orden(
            "hospital_reporte_departamentos",
            "Recursos Humanos",
            "RH necesita confirmar el directorio de áreas antes de actualizar el organigrama.",
            "Lista el nombre y el piso de cada departamento.",
            "SELECT nombre, piso FROM departamentos",
            10,
        ),
        plantilla_reporte_simple_sin_orden(
            "hospital_reporte_empleados_directorio",
            "Recursos Humanos",
            "RH quiere el directorio de personal a la mano antes de la reunión de la tarde.",
            "Lista el nombre y el puesto de cada empleado.",
            "SELECT nombre, puesto FROM empleados",
            10,
        ),
        plantilla_reporte_simple_sin_orden(
            "hospital_reporte_habitaciones_inventario",
            "Administración de Instalaciones",
            "Mantenimiento necesita el inventario de habitaciones para su checklist mensual.",
            "Lista el número y el tipo de cada habitación.",
            "SELECT numero, tipo FROM habitaciones",
            10,
        ),
        plantilla_reporte_simple(
            "hospital_reporte_pacientes_cardiologia",
```

Every other entry in `catalogo()` (the existing `hospital_reporte_habitaciones_libres` onward) stays exactly as it is, just now positioned after these 3 new entries instead of after the original first one.

- [ ] **Step 6: Update the two catalog-size tests**

In `app/src-tauri/src/tickets/hospital_arcangel.rs`, change the test at lines 135-143 from:

```rust
    #[test]
    fn catalogo_tiene_6_reportes_y_2_depuraciones() {
        let tickets = catalogo();
        assert_eq!(tickets.len(), 8);
        let reportes = tickets.iter().filter(|t| t.tipo == TipoTicket::ReporteAnalisis).count();
        let depuraciones = tickets.iter().filter(|t| t.tipo == TipoTicket::InvestigacionDepuracion).count();
        assert_eq!(reportes, 6, "4 originales + 2 Select-only agregados para Becario (Plan 7)");
        assert_eq!(depuraciones, 2);
    }
```

to:

```rust
    #[test]
    fn catalogo_tiene_9_reportes_y_2_depuraciones() {
        let tickets = catalogo();
        assert_eq!(tickets.len(), 11);
        let reportes = tickets.iter().filter(|t| t.tipo == TipoTicket::ReporteAnalisis).count();
        let depuraciones = tickets.iter().filter(|t| t.tipo == TipoTicket::InvestigacionDepuracion).count();
        assert_eq!(
            reportes, 9,
            "4 originales + 2 Select-only (Plan 7) + 3 Select-sin-orden (Plan 16)"
        );
        assert_eq!(depuraciones, 2);
    }
```

In `app/src-tauri/src/tickets/mod.rs`, change the test at lines 324-332 from:

```rust
    #[test]
    fn catalogo_devuelve_el_tamano_esperado_por_empresa() {
        assert_eq!(
            catalogo(crate::db::Company::HospitalArcangel).len(),
            8,
            "Plan 7 agrega 2 tickets Select-only para que Becario tenga bandeja"
        );
        assert_eq!(catalogo(crate::db::Company::Postafeta).len(), 6);
    }
```

to:

```rust
    #[test]
    fn catalogo_devuelve_el_tamano_esperado_por_empresa() {
        assert_eq!(
            catalogo(crate::db::Company::HospitalArcangel).len(),
            11,
            "Plan 7 agrega 2 Select-only, Plan 16 agrega 3 Select-sin-orden"
        );
        assert_eq!(catalogo(crate::db::Company::Postafeta).len(), 6);
    }
```

- [ ] **Step 7: Run the full tickets test suite**

Run: `cd app/src-tauri && "$HOME/.cargo/bin/cargo.exe" test --quiet --lib tickets::`
Expected: all tests pass, including `todas_las_queries_doradas_ejecutan` (which iterates `catalogo()` and now also runs the 3 new golden queries against the real embedded Postgres schema) and `los_tickets_de_depuracion_son_correctos_pero_lentos` (unaffected — it filters on `sql_inicial.is_some()`, and the new tickets have no `sql_inicial`).

- [ ] **Step 8: Self-review**

Read the diff. Confirm the 3 new tickets are inserted *before* `hospital_reporte_pacientes_cardiologia` in `catalogo()`'s `vec![...]` (catalog order is serving order — Plan 7 — so this is what makes them the first tickets a new Becario receives). Confirm all 3 new tickets use `plantilla_reporte_simple_sin_orden` (not `plantilla_reporte_simple`) — using the wrong one would reintroduce the row-order flakiness this task exists to avoid. Confirm the new template function's struct-update syntax (`..plantilla_reporte_simple(...)`) compiles cleanly with only `requiere_orden` overridden.

- [ ] **Step 9: Commit**

```bash
git add app/src-tauri/src/tickets/mod.rs app/src-tauri/src/tickets/hospital_arcangel.rs
git commit -m "Add a below-simple ticket tier (SELECT columns, no WHERE/ORDER BY) to Hospital Arcangel"
```

---

### Task 2: Extend the tutorial into 3 tramos + update the two-ticket start guard

**Files:**
- Modify: `app/src/main.js:59` (`TICKET_TUTORIAL_ID` constant), `main.js:374-386` (`iniciarPartida`)
- Modify: `app/src/tutorial.js` (full-file rewrite — every existing function is kept or renamed, several new ones added)

**Interfaces:**
- Consumes: the 4 exported functions from `app/src/dialogo.js` (unchanged from Plan 15: `mostrarDialogo`, `ocultarDialogo`, `permitirSiempre`) and the 3 new ticket ids from Task 1 (`hospital_reporte_departamentos`, `hospital_reporte_empleados_directorio`, `hospital_reporte_habitaciones_inventario` — this task only references the first one by id).
- Produces: `app/src/tutorial.js` keeps the exact same exported names Plan 15 already wired into `main.js` — `iniciarTutorial(retratoSvg, alFinalizar)`, `tutorialActivo()`, `saltarTutorial()`, `notificarClicPrimerTicket()`, `notificarSqlCambiado(valorSql)`, `notificarClicPlay()`, `notificarClicEnviar()`, `notificarCierreScoring()` — their call sites in `main.js` (the Play/Submit button listeners, `seleccionarTicket`, `renderBandeja`'s `data-primer-ticket` tagging, the Esc handler, `btnCerrarScoring`'s listener) do **not** need to change at all in this task; only `iniciarPartida`'s own guard changes.

- [ ] **Step 1: Update the tutorial-start guard to check two tickets**

In `app/src/main.js`, change line 59 from:

```js
const TICKET_TUTORIAL_ID = "hospital_reporte_pacientes_cardiologia";
```

to:

```js
const TICKET_TUTORIAL_ID_PASO1 = "hospital_reporte_departamentos";
const TICKET_TUTORIAL_ID_PASO2 = "hospital_reporte_pacientes_cardiologia";
```

Then change `iniciarPartida` (currently lines 374-386):

```js
async function iniciarPartida() {
  const estadoJuego = await invoke("iniciar_partida");
  pintarHubDesdeEstadoJuego(estadoJuego);
  await cargarPerks();
  setStatus("Partida nueva iniciada.", "ok");
  const primerTicket = estadoJuego.pendientes && estadoJuego.pendientes[0];
  if (primerTicket && primerTicket.id === TICKET_TUTORIAL_ID) {
    btnSaltarTutorial.classList.remove("oculto");
    iniciarTutorial(RETRATOS["El Mentor"], () => {
      btnSaltarTutorial.classList.add("oculto");
    });
  }
}
```

to:

```js
async function iniciarPartida() {
  const estadoJuego = await invoke("iniciar_partida");
  pintarHubDesdeEstadoJuego(estadoJuego);
  await cargarPerks();
  setStatus("Partida nueva iniciada.", "ok");
  const pendientes = estadoJuego.pendientes || [];
  const primerTicket = pendientes[0];
  const segundoTicket = pendientes[1];
  const esInicioDeTutorial =
    primerTicket &&
    primerTicket.id === TICKET_TUTORIAL_ID_PASO1 &&
    segundoTicket &&
    segundoTicket.id === TICKET_TUTORIAL_ID_PASO2;
  if (esInicioDeTutorial) {
    btnSaltarTutorial.classList.remove("oculto");
    iniciarTutorial(RETRATOS["El Mentor"], () => {
      btnSaltarTutorial.classList.add("oculto");
    });
  }
}
```

This now only starts the tutorial when the tray's first two tickets are exactly the two Task 1 guarantees will be first in the catalog — if Task 1 wasn't applied (or the catalog order changes again later), the tutorial simply doesn't start, same defensive behavior as before.

- [ ] **Step 2: Replace the entire contents of `app/src/tutorial.js`**

Replace the whole file with this exact content:

```js
import { mostrarDialogo, ocultarDialogo, permitirSiempre } from "./dialogo.js";

const NOMBRE_MENTOR = "El Mentor";
const SELECTOR_BOTON_SALTAR = "#btn-saltar-tutorial";

let activo = false;
let esperandoCierreScoring = false;
let clausulaObjetivoActual = null;
let pasoActualAlEscribir = null;
let retratoMentorSvg = "";
let callbackAlFinalizar = null;
let manejarClicPrimerTicket = null;
let manejarClicPlay = null;
let manejarCierreScoring = null;

function normalizar(texto) {
  return texto.toLowerCase().replace(/\s+/g, " ").trim();
}

function mostrarPaso(texto, opciones) {
  mostrarDialogo(retratoMentorSvg, NOMBRE_MENTOR, texto, opciones);
}

function pasoEscribirClausula(texto, clausulaObjetivo, siguientePaso) {
  clausulaObjetivoActual = clausulaObjetivo;
  pasoActualAlEscribir = siguientePaso;
  mostrarPaso(texto, { permitir: ["#sql-input"] });
}

// --- Tramo A: hospital_reporte_departamentos (Plan 16) ---

function paso0Bienvenida() {
  mostrarPaso(
    "Bienvenido a tu primer día en Hospital Arcángel. Aquí vas a recibir pedidos reales de otros equipos — tickets — y tu trabajo es resolverlos escribiendo SQL de verdad contra la base de datos de la empresa.",
    { alContinuar: paso1Bandeja }
  );
}

function paso1Bandeja() {
  mostrarPaso("Esa es tu bandeja. Ahí llegan tus pendientes. Dale click al primero para abrir ese ticket.", {
    permitir: ["[data-primer-ticket] button"],
  });
}

function pasoConceptoTablaA() {
  mostrarPaso(
    "Antes de escribir nada: una tabla es como una hoja de cálculo. Cada fila es un registro — en este caso, un departamento — y cada columna es un dato de ese registro, como su nombre o en qué piso está.",
    { permitir: ["#ticket-activo-info"], alContinuar: pasoLeerTicketA }
  );
}

function pasoLeerTicketA() {
  mostrarPaso(
    "Recursos Humanos quiere el directorio de áreas: el nombre y el piso de cada departamento. No piden filtrar nada ni ordenarlo — solo mostrar esos dos datos de todos los departamentos.",
    { permitir: ["#ticket-activo-info"], alContinuar: pasoClausulaSelectA }
  );
}

function pasoClausulaSelectA() {
  pasoEscribirClausula(
    "Empieza diciendo qué columnas quieres ver. Escribe: SELECT nombre, piso",
    "select nombre, piso",
    pasoClausulaFromA
  );
}

function pasoClausulaFromA() {
  pasoEscribirClausula("Ahora dile de qué tabla sacar esos datos. Agrega: FROM departamentos", "from departamentos", pasoPlayA);
}

function pasoPlayA() {
  clausulaObjetivoActual = null;
  pasoActualAlEscribir = null;
  manejarClicPlay = pasoEnviarA;
  mostrarPaso("Dale ▶ Play para probarlo contra la base de datos real.", { permitir: ["#btn-play"] });
}

function pasoEnviarA() {
  manejarCierreScoring = pasoTransicionAB;
  mostrarPaso(
    "Si el resultado se ve bien, dale ✓ Enviar ticket — así es como se resuelve cada encargo en este trabajo.",
    { permitir: ["#btn-submit"] }
  );
}

function pasoTransicionAB() {
  manejarClicPrimerTicket = pasoLeerTicketB;
  mostrarPaso(
    "Bien, ya viste tu primer reporte. El siguiente te va a pedir además un filtro — dale click para abrirlo.",
    { permitir: ["[data-primer-ticket] button"] }
  );
}

// --- Tramo B: hospital_reporte_pacientes_cardiologia (Plan 15, ampliado en Plan 16) ---

function pasoLeerTicketB() {
  mostrarPaso(
    "Contabilidad quiere un reporte de los pacientes de Cardiología. Cardiología es el departamento número 1 — vas a pedirle a la base de datos: de la tabla de pacientes, tráeme algunos datos, pero solo los del departamento 1.",
    { permitir: ["#ticket-activo-info"], alContinuar: pasoClausulaSelectB }
  );
}

function pasoClausulaSelectB() {
  pasoEscribirClausula(
    "Empieza diciendo qué columnas quieres ver. Escribe: SELECT nombre, fecha_ingreso, diagnostico",
    "select nombre, fecha_ingreso, diagnostico",
    pasoClausulaFromB
  );
}

function pasoClausulaFromB() {
  pasoEscribirClausula(
    "Ahora dile de qué tabla — cada tabla es como una hoja de cálculo, y pacientes es la hoja con un renglón por paciente. Agrega: FROM pacientes",
    "from pacientes",
    pasoComparaciones
  );
}

function pasoComparaciones() {
  mostrarPaso(
    "Antes de filtrar: un filtro compara cada fila contra una condición. El signo = significa 'igual a' — también existen > y < para comparar números o fechas, aunque este ticket solo necesita =.",
    { permitir: ["#sql-input"], alContinuar: pasoClausulaWhereB }
  );
}

function pasoClausulaWhereB() {
  pasoEscribirClausula(
    "Contabilidad solo quiere Cardiología, que es el departamento número 1. Agrega: WHERE departamento_id = 1",
    "where departamento_id = 1",
    pasoClausulaOrderByB
  );
}

function pasoClausulaOrderByB() {
  pasoEscribirClausula(
    "Y lo quieren del ingreso más reciente al más antiguo. Agrega: ORDER BY fecha_ingreso DESC",
    "order by fecha_ingreso desc",
    pasoPlayB
  );
}

function pasoPlayB() {
  clausulaObjetivoActual = null;
  pasoActualAlEscribir = null;
  manejarClicPlay = pasoEnviarB;
  mostrarPaso("Dale ▶ Play para probarlo contra la base de datos real.", { permitir: ["#btn-play"] });
}

function pasoEnviarB() {
  manejarCierreScoring = pasoTourHub1;
  mostrarPaso(
    "Si el resultado se ve bien, dale ✓ Enviar ticket — así es como se resuelve cada encargo en este trabajo.",
    { permitir: ["#btn-submit"] }
  );
}

// --- Tramo C: tour del Hub (Plan 16) ---

function pasoTourHub1() {
  mostrarPaso(
    "El dinero lo usas para desbloquear perks. La reputación además determina qué perks y qué rango puedes alcanzar.",
    { permitir: [".hub-topbar"], alContinuar: pasoTourHub2 }
  );
}

function pasoTourHub2() {
  mostrarPaso(
    "Los perks son bonos permanentes — algunos te dan más dinero o reputación por ticket resuelto. Cada uno cuesta dinero y pide una reputación mínima para desbloquearse.",
    { permitir: [".hub-columna-perks"], alContinuar: pasoTourHub3 }
  );
}

function pasoTourHub3() {
  mostrarPaso(
    "Cada ticket bien resuelto suma reputación. Al llegar al umbral necesario subes de rango — de Becario a Auxiliar de Sistemas, por ejemplo — lo que desbloquea tickets nuevos y un slot más de perk.",
    { permitir: [".tarjeta-progreso-carrera"], alContinuar: pasoCierre }
  );
}

function pasoCierre() {
  mostrarPaso(
    "Bien hecho — ya resolviste tus dos primeros tickets y ya conoces lo esencial. El resto de tu bandeja funciona igual: lee lo que piden, escribe la query, pruébala, y envíala. Ahí te dejo.",
    { alContinuar: finalizarTutorial }
  );
}

function finalizarTutorial() {
  activo = false;
  esperandoCierreScoring = false;
  clausulaObjetivoActual = null;
  pasoActualAlEscribir = null;
  manejarClicPrimerTicket = null;
  manejarClicPlay = null;
  manejarCierreScoring = null;
  permitirSiempre([]);
  ocultarDialogo();
  if (callbackAlFinalizar) callbackAlFinalizar();
}

export function iniciarTutorial(retratoSvg, alFinalizar) {
  activo = true;
  esperandoCierreScoring = false;
  clausulaObjetivoActual = null;
  pasoActualAlEscribir = null;
  retratoMentorSvg = retratoSvg;
  callbackAlFinalizar = alFinalizar || null;
  manejarClicPrimerTicket = pasoConceptoTablaA;
  manejarClicPlay = null;
  manejarCierreScoring = null;
  permitirSiempre([SELECTOR_BOTON_SALTAR]);
  paso0Bienvenida();
}

export function tutorialActivo() {
  return activo;
}

export function saltarTutorial() {
  if (!activo) return;
  finalizarTutorial();
}

export function notificarClicPrimerTicket() {
  if (!activo || !manejarClicPrimerTicket) return;
  manejarClicPrimerTicket();
}

export function notificarSqlCambiado(valorSql) {
  if (!activo || !clausulaObjetivoActual) return;
  if (normalizar(valorSql).includes(clausulaObjetivoActual)) {
    const siguiente = pasoActualAlEscribir;
    clausulaObjetivoActual = null;
    pasoActualAlEscribir = null;
    siguiente();
    notificarSqlCambiado(valorSql);
  }
}

export function notificarClicPlay() {
  if (!activo || !manejarClicPlay) return;
  manejarClicPlay();
}

export function notificarClicEnviar() {
  if (!activo) return;
  ocultarDialogo();
  esperandoCierreScoring = true;
}

export function notificarCierreScoring() {
  if (!activo || !esperandoCierreScoring || !manejarCierreScoring) return;
  esperandoCierreScoring = false;
  manejarCierreScoring();
}
```

Design notes for the reviewer:
- `notificarClicPrimerTicket`, `notificarClicPlay`, and `notificarCierreScoring` are now thin dispatchers to module-level function-pointer variables (`manejarClicPrimerTicket`/`manejarClicPlay`/`manejarCierreScoring`) instead of calling one hardcoded function each — this is what lets the *same* real UI event (clicking the first tray ticket, clicking ▶ Play, closing the scoring overlay) mean something different depending on which tramo the tutorial is currently in. Each `paso*` function that's about to wait for one of these events sets the relevant pointer just before showing its dialogue (e.g. `pasoPlayA` sets `manejarClicPlay = pasoEnviarA`, `pasoPlayB` later overwrites it with `manejarClicPlay = pasoEnviarB`).
- `notificarClicPrimerTicket` firing a *second* time (after ticket A is resolved and removed from the tray) works because `renderBandeja` in `main.js` re-tags whichever ticket is now at index 0 with `data-primer-ticket="true"` on every render — since resolving ticket A removes it from `pendientes`, the Cardiología ticket becomes the new index 0, so the exact same `[data-primer-ticket] button` selector and the exact same `seleccionarTicket` → `notificarClicPrimerTicket()` call site (both untouched from Plan 15) correctly trigger `pasoLeerTicketB` this time instead of `pasoConceptoTablaA`. **This only holds with the corrected catalog order** (see the post-implementation correction note at the top of this plan) — Cardiología must be the very next Becario-eligible ticket after `hospital_reporte_departamentos`, with nothing else in between, or the new index 0 after resolving ticket A would be some other ticket instead.
- `notificarClicEnviar` itself does not change at all — hiding the dialogue and arming `esperandoCierreScoring` is identical regardless of which ticket was just submitted; only what happens *after* the real scoring overlay closes (`manejarCierreScoring`) differs per tramo.
- `pasoComparaciones` is a pure "explain, then continue" beat (`alContinuar`, no clause to type) inserted between FROM and WHERE in Tramo B — `permitir: ["#sql-input"]` on it is only for the spotlight (keeping visual focus on the editor while explaining what's about to be typed there), not because anything is required to be typed on this beat.

- [ ] **Step 3: Syntax-check both changed JS files**

Run: `node --check app/src/main.js && node --check app/src/tutorial.js`
Expected: no output, exit code 0.

- [ ] **Step 4: Self-review**

Read the full diff. Confirm every `paso*` function referenced as an `alContinuar` value, a `pasoEscribirClausula` third argument, or a `manejarClicPlay`/`manejarClicPrimerTicket`/`manejarCierreScoring` assignment target is actually defined somewhere in the file (a typo'd function name here fails silently at call time, not at parse time, since these are all just plain function references passed as values). Confirm `pasoPlayA`/`pasoPlayB` both null out `clausulaObjetivoActual`/`pasoActualAlEscribir` before setting `manejarClicPlay` (matching the original Plan 15 `paso7Play` behavior) — otherwise a stray `sql-input` `input` event after reaching the Play beat could still attempt to advance a stale clause target. Confirm `TICKET_TUTORIAL_ID_PASO1`/`TICKET_TUTORIAL_ID_PASO2` in `main.js` match the exact ticket ids Task 1 added to the Rust catalog (`hospital_reporte_departamentos`, `hospital_reporte_pacientes_cardiologia`) — a mismatch here means the tutorial silently never starts for a new game.

- [ ] **Step 5: Commit**

```bash
git add app/src/main.js app/src/tutorial.js
git commit -m "Extend the Mentor tutorial into 3 tramos: the new easy ticket, Cardiologia with deeper WHERE/comparisons teaching, and a Hub tour"
```

---

## Manual Verification (after both tasks)

Same pattern as Plans 6-15 — guided verification in the real running app:

- Start a **brand new game**: the very first ticket in the tray is now "Lista el nombre y el piso de cada departamento" (`hospital_reporte_departamentos`), and the tutorial's welcome beat fires as before.
- Tramo A: the concept-of-a-table beat appears before any typing is requested; SELECT/FROM are typed clause by clause exactly like Plan 15's flow; Play/Enviar work; after closing the scoring overlay, the Mentor's transition line appears and spotlights the *new* first tray ticket (now Cardiología) without any gap or flash of the old closing message.
- Tramo B: clicking that spotlighted ticket opens Cardiología; SELECT/FROM proceed as in Plan 15; the new comparisons beat appears before WHERE is requested; WHERE/ORDER BY/Play/Enviar proceed as before.
- Tramo C: after closing the second scoring overlay, three Hub-tour beats appear in sequence, each spotlighting a different real Hub element (money/reputation badges, the Perks column, the career-progress card) with the rest of the Hub blocked; the final "ahí te dejo" beat ends the tutorial (skip button disappears, Esc opens pause again).
- Skip button: still visible and functional at every beat across all 3 tramos, including immediately after each scoring overlay closes (the moments where `esperandoCierreScoring` was previously true).
- Confirm the two other new easy tickets (`hospital_reporte_empleados_directorio`, `hospital_reporte_habitaciones_inventario`) appear later in the tray as ordinary, unguided tickets once the tutorial (or a skip) is done.
- Confirm **loading an existing save** never triggers the tutorial (unchanged from Plan 15 — `cargarPartida` is never touched by this plan).
