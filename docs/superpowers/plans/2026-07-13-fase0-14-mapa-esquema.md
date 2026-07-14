# Fase 0 / Plan 14: Mapa de Esquema Visual por Empresa Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let the player see a draggable diagram of the active company's real database schema — tables, columns, and foreign-key relationships — read live from Postgres, using the human descriptions that already exist as `COMMENT ON TABLE`/`COMMENT ON COLUMN` in each company's schema SQL.

**Architecture:** A new Rust function introspects `information_schema`/`pg_catalog` against the active company's pool (same access pattern as the existing `run_query`) and returns a plain serializable view. The frontend renders one absolutely-positioned "paper" box per table (hand-placed default position per company) inside a scrollable canvas, with an SVG overlay drawing a line per real foreign key; the player can drag any box, and lines redraw live to follow it.

**Tech Stack:** Rust + sqlx (existing `PgPool`, no new dependencies), Postgres `information_schema`/`pg_catalog` introspection (built-in, no extensions), vanilla JS + inline SVG.

## Global Constraints

- No new Cargo dependencies — `sqlx` already provides everything needed (`Row::try_get` by column name, exactly like the existing `db::run_query`).
- The two Tauri-facing view structs (`EsquemaView`, `TablaEsquema`, `ColumnaEsquema`, `RelacionEsquema`) live in `app/src-tauri/src/db/mod.rs`, following the same precedent as `QueryResult` (also in `db/mod.rs`, also returned directly by its Tauri command with no separate lib.rs wrapper) — there is nothing sensitive to strip from this data, unlike `Ticket`'s `sql_dorada`.
- Self-referencing foreign keys (e.g. `empleados.jefe_id → empleados.id`) are real and must not break the introspection query or the frontend — the frontend explicitly skips drawing a line where `tabla_origen == tabla_destino` (a straight center-to-center line to the same box is not useful to render).
- Default box positions are hand-placed per company in a frontend lookup — no auto-layout algorithm. Only Hospital Arcángel and Postafeta need entries (the only 2 companies that exist).
- Dragging a box only updates its position in memory for as long as the overlay stays open — nothing is persisted (not to disk, not to `localStorage`).
- The schema overlay reuses `.scoring-overlay`/`.ventana-terminal` styling (same visual family as the Plan 13 pause menu) — dark terminal window, not the Hub's warm paper theme, since it's reachable from both Hub and Consola.
- No `.innerHTML` assignment may ever interpolate anything other than: the 3 fixed SVG icon-string constants already used elsewhere in this file, or fields sourced from this plan's own Rust introspection (table/column names, types, comments) — never anything a player types into the SQL editor.

---

### Task 1: Rust — live schema introspection

**Files:**
- Modify: `app/src-tauri/src/db/mod.rs` (new structs + `obtener_esquema` function + tests, appended after the existing `run_query`/tests)
- Modify: `app/src-tauri/src/lib.rs` (new `esquema_actual` Tauri command + command registration)

**Interfaces:**
- Produces (consumed by Tasks 2-3): `db::EsquemaView { tablas: Vec<TablaEsquema>, relaciones: Vec<RelacionEsquema> }`, `db::TablaEsquema { nombre: String, descripcion: Option<String>, columnas: Vec<ColumnaEsquema> }`, `db::ColumnaEsquema { nombre: String, tipo: String, nullable: bool, descripcion: Option<String> }`, `db::RelacionEsquema { tabla_origen: String, columna_origen: String, tabla_destino: String, columna_destino: String }` — all `#[derive(serde::Serialize)]`, all fields serialize under these exact snake_case names (matching every other view struct in this codebase, e.g. `EstadoTurnoView`). The Tauri command `esquema_actual` returns `Result<db::EsquemaView, String>` with no arguments beyond `tauri::State`.

- [ ] **Step 1: Add the view structs and the introspection function to `db/mod.rs`**

In `app/src-tauri/src/db/mod.rs`, right after the existing `run_query` function (before the `#[cfg(test)] mod tests` block), add:

```rust
#[derive(serde::Serialize)]
pub struct ColumnaEsquema {
    pub nombre: String,
    pub tipo: String,
    pub nullable: bool,
    pub descripcion: Option<String>,
}

#[derive(serde::Serialize)]
pub struct TablaEsquema {
    pub nombre: String,
    pub descripcion: Option<String>,
    pub columnas: Vec<ColumnaEsquema>,
}

#[derive(serde::Serialize)]
pub struct RelacionEsquema {
    pub tabla_origen: String,
    pub columna_origen: String,
    pub tabla_destino: String,
    pub columna_destino: String,
}

#[derive(serde::Serialize)]
pub struct EsquemaView {
    pub tablas: Vec<TablaEsquema>,
    pub relaciones: Vec<RelacionEsquema>,
}

/// Introspección en vivo (Etapa 16/Plan 14): tablas, columnas y relaciones
/// reales de la base de datos activa, incluyendo los comentarios humanos que
/// cada esquema ya trae vía `COMMENT ON TABLE`/`COMMENT ON COLUMN` — no se
/// inventa ni duplica ningún texto, se lee directo de Postgres.
pub async fn obtener_esquema(pool: &PgPool) -> anyhow::Result<EsquemaView> {
    let filas_tablas = sqlx::query(
        "SELECT c.relname AS tabla, obj_description(c.oid, 'pg_class') AS descripcion
         FROM pg_class c
         JOIN pg_namespace n ON n.oid = c.relnamespace
         WHERE c.relkind = 'r' AND n.nspname = 'public'
         ORDER BY c.relname",
    )
    .fetch_all(pool)
    .await?;

    let mut tablas: Vec<TablaEsquema> = Vec::new();
    for fila in &filas_tablas {
        let nombre: String = fila.try_get("tabla")?;
        let descripcion: Option<String> = fila.try_get("descripcion")?;
        tablas.push(TablaEsquema { nombre, descripcion, columnas: Vec::new() });
    }

    let filas_columnas = sqlx::query(
        "SELECT c.relname AS tabla, a.attname AS columna,
                format_type(a.atttypid, a.atttypmod) AS tipo,
                NOT a.attnotnull AS nullable,
                col_description(c.oid, a.attnum) AS descripcion
         FROM pg_attribute a
         JOIN pg_class c ON c.oid = a.attrelid
         JOIN pg_namespace n ON n.oid = c.relnamespace
         WHERE c.relkind = 'r' AND n.nspname = 'public' AND a.attnum > 0 AND NOT a.attisdropped
         ORDER BY c.relname, a.attnum",
    )
    .fetch_all(pool)
    .await?;

    for fila in &filas_columnas {
        let tabla: String = fila.try_get("tabla")?;
        let columna = ColumnaEsquema {
            nombre: fila.try_get("columna")?,
            tipo: fila.try_get("tipo")?,
            nullable: fila.try_get("nullable")?,
            descripcion: fila.try_get("descripcion")?,
        };
        if let Some(t) = tablas.iter_mut().find(|t| t.nombre == tabla) {
            t.columnas.push(columna);
        }
    }

    let filas_relaciones = sqlx::query(
        "SELECT tc.table_name AS tabla_origen, kcu.column_name AS columna_origen,
                ccu.table_name AS tabla_destino, ccu.column_name AS columna_destino
         FROM information_schema.table_constraints tc
         JOIN information_schema.key_column_usage kcu
             ON tc.constraint_name = kcu.constraint_name AND tc.table_schema = kcu.table_schema
         JOIN information_schema.constraint_column_usage ccu
             ON tc.constraint_name = ccu.constraint_name AND tc.table_schema = ccu.table_schema
         WHERE tc.constraint_type = 'FOREIGN KEY' AND tc.table_schema = 'public'
         ORDER BY tc.table_name, kcu.column_name",
    )
    .fetch_all(pool)
    .await?;

    let mut relaciones: Vec<RelacionEsquema> = Vec::new();
    for fila in &filas_relaciones {
        relaciones.push(RelacionEsquema {
            tabla_origen: fila.try_get("tabla_origen")?,
            columna_origen: fila.try_get("columna_origen")?,
            tabla_destino: fila.try_get("tabla_destino")?,
            columna_destino: fila.try_get("columna_destino")?,
        });
    }

    Ok(EsquemaView { tablas, relaciones })
}
```

All 3 queries use `pg_catalog`/`information_schema` views and functions that are core Postgres (no extension needed) — `obj_description`/`col_description`/`format_type` are built-in, and the embedded Postgres instance this project already uses is a real Postgres server, so these work exactly as they would against any Postgres install.

- [ ] **Step 2: Add tests to `db/mod.rs`'s existing test module**

In the same file, inside the existing `#[cfg(test)] mod tests { ... }` block (after the existing `ambas_empresas_conviven_en_el_mismo_servidor` test), add:

```rust
    #[tokio::test]
    async fn obtener_esquema_lee_tablas_columnas_y_relaciones_reales() {
        let pg = init_embedded_postgres().await.expect("Postgres embebido debe arrancar");
        let pool = load_company(&pg, Company::HospitalArcangel)
            .await
            .expect("Hospital Arcángel debe cargar");

        let esquema = obtener_esquema(&pool).await.expect("la introspección debe funcionar");

        assert_eq!(esquema.tablas.len(), 6, "Hospital Arcángel tiene 6 tablas");

        let pacientes = esquema
            .tablas
            .iter()
            .find(|t| t.nombre == "pacientes")
            .expect("la tabla pacientes debe aparecer");
        assert_eq!(
            pacientes.descripcion.as_deref(),
            Some("Historial de admisiones. fecha_alta queda NULL mientras el paciente sigue internado.")
        );

        let columna_diagnostico = pacientes
            .columnas
            .iter()
            .find(|c| c.nombre == "diagnostico")
            .expect("la columna diagnostico debe aparecer");
        assert_eq!(columna_diagnostico.tipo, "text");
        assert!(!columna_diagnostico.nullable);
        assert_eq!(
            columna_diagnostico.descripcion.as_deref(),
            Some("Motivo de ingreso redactado por el residente de guardia, casi siempre a las 3am.")
        );

        let columna_fecha_alta = pacientes
            .columnas
            .iter()
            .find(|c| c.nombre == "fecha_alta")
            .expect("la columna fecha_alta debe aparecer");
        assert!(columna_fecha_alta.nullable, "fecha_alta no tiene NOT NULL en el esquema");

        let relacion_paciente_departamento = esquema.relaciones.iter().any(|r| {
            r.tabla_origen == "pacientes"
                && r.columna_origen == "departamento_id"
                && r.tabla_destino == "departamentos"
                && r.columna_destino == "id"
        });
        assert!(relacion_paciente_departamento, "pacientes.departamento_id debe referenciar departamentos.id");

        pool.close().await;
        pg.stop().await.expect("Postgres debe detenerse limpiamente");
    }

    #[tokio::test]
    async fn obtener_esquema_soporta_multiples_fks_en_una_tabla() {
        let pg = init_embedded_postgres().await.expect("Postgres embebido debe arrancar");
        let pool = load_company(&pg, Company::Postafeta)
            .await
            .expect("Postafeta debe cargar");

        let esquema = obtener_esquema(&pool).await.expect("la introspección debe funcionar");

        assert_eq!(esquema.tablas.len(), 5, "Postafeta tiene 5 tablas");

        let relaciones_paquetes: Vec<_> = esquema.relaciones.iter().filter(|r| r.tabla_origen == "paquetes").collect();
        assert_eq!(relaciones_paquetes.len(), 4, "paquetes referencia clientes, sucursales (x2) y empleados");

        let hacia_clientes = relaciones_paquetes
            .iter()
            .any(|r| r.columna_origen == "cliente_id" && r.tabla_destino == "clientes");
        assert!(hacia_clientes);

        pool.close().await;
        pg.stop().await.expect("Postgres debe detenerse limpiamente");
    }
```

- [ ] **Step 3: Run the new tests**

Run: `cd app/src-tauri && cargo test --no-default-features obtener_esquema`
Expected: both new tests pass (`obtener_esquema_lee_tablas_columnas_y_relaciones_reales`, `obtener_esquema_soporta_multiples_fks_en_una_tabla`).

- [ ] **Step 4: Add the Tauri command**

In `app/src-tauri/src/lib.rs`, right after the existing `catalogo_perks` command, add:

```rust
#[tauri::command]
async fn esquema_actual(state: tauri::State<'_, AppState>) -> Result<db::EsquemaView, String> {
    let pool = state.0.lock().unwrap().clone();
    db::obtener_esquema(&pool).await.map_err(|e| e.to_string())
}
```

Then find the `tauri::generate_handler![...]` macro call (it currently lists `catalogo_perks, desbloquear_perk, equipar_perk, desequipar_perk` among its entries) and add `esquema_actual` to that list.

- [ ] **Step 5: Run the full backend test suite**

Run: `cd app/src-tauri && cargo test --no-default-features`
Expected: all tests pass (108 existing + 2 new = 110), 0 failed.

- [ ] **Step 6: Self-review**

Read the diff. Confirm `esquema_actual` is registered in `tauri::generate_handler![...]` (a command not registered there is silently unreachable from the frontend — `invoke("esquema_actual")` would fail at runtime with no compile-time warning). Confirm every `fila.try_get(...)` uses `?` (propagating via `anyhow::Result`), not `.unwrap()` — a malformed row should return an `Err`, not panic. Confirm the struct field names are exactly `nombre`/`descripcion`/`columnas`/`tipo`/`nullable`/`tabla_origen`/`columna_origen`/`tabla_destino`/`columna_destino`/`tablas`/`relaciones` (Task 2's frontend code depends on these exact JSON keys).

- [ ] **Step 7: Commit**

```bash
git add app/src-tauri/src/db/mod.rs app/src-tauri/src/lib.rs
git commit -m "Add live schema introspection: esquema_actual reads tables/columns/FKs from Postgres"
```

---

### Task 2: Frontend — schema diagram markup, styling, and static rendering

**Files:**
- Modify: `app/src/index.html` (new `#esquema-overlay`, "Ver esquema" button in Consola, "Base de Datos" tab in the Hub's tab bar)
- Modify: `app/src/styles.css` (new rules for the overlay's canvas, table boxes, and relationship lines)
- Modify: `app/src/main.js` (new `POSICIONES_TABLAS` lookup, `empresaActual` tracking, `mostrarEsquema()` rendering function — no drag yet, that's Task 3)

**Interfaces:**
- Consumes: `db::EsquemaView`'s exact JSON shape from Task 1 (`{ tablas: [{ nombre, descripcion, columnas: [{ nombre, tipo, nullable, descripcion }] }], relaciones: [{ tabla_origen, columna_origen, tabla_destino, columna_destino }] }`).
- Produces (consumed by Task 3): module-level `esquemaLienzo`/`esquemaSvg` element references; `posicionesActuales` (an object mapping table name → `{ x, y }`, the single source of truth for where each box currently sits); `dibujarRelaciones(relaciones)` (redraws every SVG line from `posicionesActuales` and each box's real rendered size — Task 3 calls this again on every drag movement); `esquemaRelacionesActuales` (the last-fetched relaciones array, re-passed to `dibujarRelaciones` on each drag frame since dragging doesn't refetch from the backend).

- [ ] **Step 1: Add the schema overlay markup**

In `app/src/index.html`, right after the closing `</div>` of `#pausa-overlay` (the last overlay in the file, immediately before `</body>`), add:

```html
    <div id="esquema-overlay" class="scoring-overlay oculto">
      <div class="ventana-terminal ventana-esquema">
        <div class="ventana-terminal-barra">
          <span class="ventana-terminal-punto rojo"></span>
          <span class="ventana-terminal-punto amarillo"></span>
          <span class="ventana-terminal-punto verde"></span>
          <span class="ventana-terminal-titulo">query-path — esquema</span>
        </div>
        <div class="ventana-terminal-cuerpo esquema-cuerpo">
          <div class="esquema-lienzo" id="esquema-lienzo">
            <svg class="esquema-svg" id="esquema-svg"></svg>
          </div>
          <button id="btn-cerrar-esquema">Cerrar</button>
        </div>
      </div>
    </div>
```

- [ ] **Step 2: Add the "Ver esquema" button to the Consola**

In `app/src/index.html`, inside `<section class="console">`, find this block:

```html
              <div class="actions">
                <button id="btn-play">▶ Play</button>
                <button id="btn-submit">✓ Enviar ticket</button>
              </div>
```

and change it to:

```html
              <div class="actions">
                <button id="btn-play">▶ Play</button>
                <button id="btn-submit">✓ Enviar ticket</button>
                <button id="btn-ver-esquema">Ver esquema</button>
              </div>
```

- [ ] **Step 3: Add the "Base de Datos" tab to the Hub's tab bar**

In `app/src/index.html`, inside `.barra-pestanas`, find this block (the existing "Mis Logros" tab, currently the last one):

```html
          <div class="pestana pestana-bloqueada" id="tab-logros" title="Próximamente">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round" stroke-linecap="round"><path d="M8 4h8v4a4 4 0 0 1-8 0z"/><path d="M8 5H5a2 2 0 0 0 2 4M16 5h3a2 2 0 0 1-2 4"/><path d="M12 12v3M9 20h6M10 17h4l1 3H9z"/></svg>
            Mis Logros
          </div>
```

and add a new tab right before it:

```html
          <div class="pestana" id="tab-base-datos">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><ellipse cx="12" cy="5" rx="8" ry="3"/><path d="M4 5v6c0 1.7 3.6 3 8 3s8-1.3 8-3V5"/><path d="M4 11v6c0 1.7 3.6 3 8 3s8-1.3 8-3v-6"/></svg>
            Base de Datos
          </div>
          <div class="pestana pestana-bloqueada" id="tab-logros" title="Próximamente">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round" stroke-linecap="round"><path d="M8 4h8v4a4 4 0 0 1-8 0z"/><path d="M8 5H5a2 2 0 0 0 2 4M16 5h3a2 2 0 0 1-2 4"/><path d="M12 12v3M9 20h6M10 17h4l1 3H9z"/></svg>
            Mis Logros
          </div>
```

- [ ] **Step 4: Add the CSS**

In `app/src/styles.css`, append at the end of the file:

```css
.ventana-esquema {
  width: 90vw;
  max-width: 1100px;
}

.esquema-cuerpo {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1rem;
}

.esquema-lienzo {
  position: relative;
  width: 100%;
  height: 70vh;
  min-width: 1100px;
  min-height: 700px;
  overflow: auto;
  background: #11111b;
  border-radius: 6px;
}

.esquema-svg {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
}

.esquema-linea-relacion {
  stroke: #6c7086;
  stroke-width: 1.5;
}

.caja-tabla {
  position: absolute;
  width: 220px;
  background: #e8dcc0;
  color: #3a3a2e;
  border-radius: 4px;
  box-shadow: 0 3px 8px rgba(0, 0, 0, 0.4);
  padding: 0.6rem 0.7rem;
  cursor: grab;
  user-select: none;
}

.caja-tabla:active {
  cursor: grabbing;
}

.caja-tabla-titulo {
  font-weight: 700;
  font-size: 0.85rem;
  font-family: "SF Mono", Menlo, Consolas, monospace;
  margin-bottom: 0.3rem;
}

.caja-tabla-descripcion {
  font-size: 0.65rem;
  color: #5a4a35;
  margin-bottom: 0.5rem;
  line-height: 1.4;
}

.caja-tabla-columnas {
  list-style: none;
  margin: 0;
  padding: 0;
}

.caja-tabla-columnas li {
  font-size: 0.68rem;
  padding: 0.2rem 0;
  border-top: 1px solid rgba(58, 58, 46, 0.15);
}

.caja-tabla-columnas li:first-child {
  border-top: none;
}

.columna-nombre {
  font-family: "SF Mono", Menlo, Consolas, monospace;
  font-weight: 700;
}

.columna-tipo {
  color: #7a6a55;
  font-size: 0.62rem;
}

.columna-descripcion {
  font-size: 0.6rem;
  color: #8a7355;
  line-height: 1.3;
  margin-top: 0.1rem;
}
```

- [ ] **Step 5: Add `empresaActual` tracking**

`POSICIONES_TABLAS` (Step 6) is keyed by company, so the render function needs to know which company is currently active. In `app/src/main.js`, add a new module-level variable right after the existing `let ticketActivoId = null;` line:

```js
let empresaActual = null;
```

Then in `renderBandeja` (the function that already reads `estadoTurno.empresa` for the company-info card), change:

```js
function renderBandeja(estadoTurno) {
  presupuestoEl.textContent = estadoTurno.presupuesto_restante;
  bandejaTitulo.textContent = TITULO_FASE[estadoTurno.fase] || "Bandeja — turno actual";
  const empresa = EMPRESA_INFO[estadoTurno.empresa];
  if (empresa) {
    empresaNombreEl.textContent = empresa.nombre;
    empresaDescripcionEl.textContent = empresa.descripcion;
  }
```

to:

```js
function renderBandeja(estadoTurno) {
  presupuestoEl.textContent = estadoTurno.presupuesto_restante;
  bandejaTitulo.textContent = TITULO_FASE[estadoTurno.fase] || "Bandeja — turno actual";
  empresaActual = estadoTurno.empresa;
  const empresa = EMPRESA_INFO[estadoTurno.empresa];
  if (empresa) {
    empresaNombreEl.textContent = empresa.nombre;
    empresaDescripcionEl.textContent = empresa.descripcion;
  }
```

`renderBandeja` runs on every Hub refresh (game start/load, closing a day, resolving a ticket, the Agencia transition) — so `empresaActual` is always current by the time the player could possibly open the schema diagram.

- [ ] **Step 6: Add the position lookup, module vars, and the rendering function**

In `app/src/main.js`, add this near the other per-entity lookup constants (right after the `EMPRESA_INFO` constant):

```js
const POSICIONES_TABLAS = {
  HospitalArcangel: {
    departamentos: { x: 80, y: 80 },
    empleados: { x: 420, y: 60 },
    seguros: { x: 780, y: 80 },
    habitaciones: { x: 80, y: 380 },
    pacientes: { x: 420, y: 320 },
    tratamientos: { x: 420, y: 560 },
  },
  Postafeta: {
    sucursales: { x: 420, y: 60 },
    empleados: { x: 80, y: 300 },
    clientes: { x: 780, y: 300 },
    paquetes: { x: 420, y: 320 },
    incidencias: { x: 420, y: 560 },
  },
};
```

Add these module-level variables alongside the other element-reference declarations (near `let tooltipGlobal;`):

```js
let esquemaOverlay, esquemaLienzo, esquemaSvg;
let posicionesActuales = {};
let esquemaRelacionesActuales = [];
```

Then add the rendering functions (place them after `renderPerks`/`cargarPerks`, before the `window.addEventListener("DOMContentLoaded", ...)` block):

```js
function dibujarRelaciones(relaciones) {
  esquemaRelacionesActuales = relaciones;
  esquemaSvg.innerHTML = "";
  relaciones.forEach((rel) => {
    if (rel.tabla_origen === rel.tabla_destino) return;
    const origen = posicionesActuales[rel.tabla_origen];
    const destino = posicionesActuales[rel.tabla_destino];
    if (!origen || !destino) return;
    const cajaOrigen = esquemaLienzo.querySelector(`[data-tabla="${rel.tabla_origen}"]`);
    const cajaDestino = esquemaLienzo.querySelector(`[data-tabla="${rel.tabla_destino}"]`);
    if (!cajaOrigen || !cajaDestino) return;

    const x1 = origen.x + cajaOrigen.offsetWidth / 2;
    const y1 = origen.y + cajaOrigen.offsetHeight / 2;
    const x2 = destino.x + cajaDestino.offsetWidth / 2;
    const y2 = destino.y + cajaDestino.offsetHeight / 2;

    const linea = document.createElementNS("http://www.w3.org/2000/svg", "line");
    linea.setAttribute("x1", x1);
    linea.setAttribute("y1", y1);
    linea.setAttribute("x2", x2);
    linea.setAttribute("y2", y2);
    linea.setAttribute("class", "esquema-linea-relacion");
    esquemaSvg.appendChild(linea);
  });
}

function crearCajaTabla(tabla, posicion) {
  const caja = document.createElement("div");
  caja.className = "caja-tabla";
  caja.dataset.tabla = tabla.nombre;
  caja.style.left = `${posicion.x}px`;
  caja.style.top = `${posicion.y}px`;

  const titulo = document.createElement("div");
  titulo.className = "caja-tabla-titulo";
  titulo.textContent = tabla.nombre;
  caja.appendChild(titulo);

  if (tabla.descripcion) {
    const descripcion = document.createElement("div");
    descripcion.className = "caja-tabla-descripcion";
    descripcion.textContent = tabla.descripcion;
    caja.appendChild(descripcion);
  }

  const listaColumnas = document.createElement("ul");
  listaColumnas.className = "caja-tabla-columnas";
  tabla.columnas.forEach((columna) => {
    const li = document.createElement("li");
    const nulo = columna.nullable ? "" : " NOT NULL";
    li.innerHTML = `<span class="columna-nombre">${columna.nombre}</span> <span class="columna-tipo">${columna.tipo}${nulo}</span>`;
    if (columna.descripcion) {
      const descripcion = document.createElement("div");
      descripcion.className = "columna-descripcion";
      descripcion.textContent = columna.descripcion;
      li.appendChild(descripcion);
    }
    listaColumnas.appendChild(li);
  });
  caja.appendChild(listaColumnas);

  return caja;
}

async function mostrarEsquema() {
  const esquema = await invoke("esquema_actual");
  const posiciones = POSICIONES_TABLAS[empresaActual] || {};

  posicionesActuales = {};
  esquemaLienzo.querySelectorAll(".caja-tabla").forEach((el) => el.remove());

  esquema.tablas.forEach((tabla, indice) => {
    const posicion = posiciones[tabla.nombre] || { x: 40 + indice * 260, y: 40 };
    posicionesActuales[tabla.nombre] = { ...posicion };
    esquemaLienzo.appendChild(crearCajaTabla(tabla, posicion));
  });

  dibujarRelaciones(esquema.relaciones);
  esquemaOverlay.classList.remove("oculto");
}
```

`li.innerHTML` above only ever interpolates `columna.nombre`/`columna.tipo` (both sourced from Task 1's own Postgres introspection of our own schema, never player-typed SQL) plus the fixed literal `" NOT NULL"` — safe, per this plan's Global Constraints.

- [ ] **Step 7: Wire the element references and the 3 entry-point buttons in `DOMContentLoaded`**

In `app/src/main.js`'s `DOMContentLoaded` handler, add this right after the existing `agenciaOverlay = document.querySelector("#agencia-overlay");` line:

```js
  esquemaOverlay = document.querySelector("#esquema-overlay");
  esquemaLienzo = document.querySelector("#esquema-lienzo");
  esquemaSvg = document.querySelector("#esquema-svg");
```

Then, after the existing `document.querySelector("#tab-logros").addEventListener(...)` block, add:

```js
  document.querySelector("#btn-ver-esquema").addEventListener("click", mostrarEsquema);

  document.querySelector("#tab-base-datos").addEventListener("click", mostrarEsquema);

  document.querySelector("#btn-cerrar-esquema").addEventListener("click", () => {
    esquemaOverlay.classList.add("oculto");
  });
```

- [ ] **Step 8: Self-review**

Read the diff. Confirm `esquemaLienzo`/`esquemaSvg`/`esquemaOverlay` are assigned in `DOMContentLoaded` BEFORE any button click could call `mostrarEsquema` (they're plain top-level assignments that run synchronously before any listener fires, so this holds automatically — verify no listener registration accidentally moved above these 3 lines). Confirm `POSICIONES_TABLAS`'s two top-level keys (`HospitalArcangel`, `Postafeta`) exactly match the `Company` enum's serialized variant names (no `rename_all` on that enum — confirmed in Task 1 — so these are the literal Rust variant names). Confirm the fallback position (`{ x: 40 + indice * 260, y: 40 }`) only ever triggers for a table name missing from the lookup — cross-check `POSICIONES_TABLAS`'s entries against the exact table names in `app/src-tauri/src/db/hospital_arcangel.rs`/`postafeta.rs` (`departamentos`, `empleados`, `seguros`, `pacientes`, `tratamientos`, `habitaciones` for Hospital Arcángel; `sucursales`, `empleados`, `clientes`, `paquetes`, `incidencias` for Postafeta) to confirm every real table has an explicit entry.

- [ ] **Step 9: Commit**

```bash
git add app/src/index.html app/src/styles.css app/src/main.js
git commit -m "Render a static schema diagram: table boxes + FK relationship lines"
```

---

### Task 3: Frontend — dragging table boxes

**Files:**
- Modify: `app/src/main.js` (drag state + mouse event listeners)

**Interfaces:**
- Consumes: `esquemaLienzo`, `posicionesActuales`, `dibujarRelaciones`, `esquemaRelacionesActuales` — all produced by Task 2.
- Produces: nothing consumed by later work — this is the last task in the plan.

- [ ] **Step 1: Add drag state and the mousedown handler on each table box**

In `app/src/main.js`, `crearCajaTabla` (from Task 2) builds each box but does not yet make it draggable. Add these module-level variables near the other Task 2 additions (right after `let esquemaRelacionesActuales = [];`):

```js
let cajaArrastrando = null;
let offsetArrastreX = 0;
let offsetArrastreY = 0;
```

Then, in `crearCajaTabla`, right before the final `return caja;` line, add:

```js
  caja.addEventListener("mousedown", (evento) => {
    cajaArrastrando = tabla.nombre;
    const rect = esquemaLienzo.getBoundingClientRect();
    const posicion = posicionesActuales[tabla.nombre];
    offsetArrastreX = evento.clientX - rect.left + esquemaLienzo.scrollLeft - posicion.x;
    offsetArrastreY = evento.clientY - rect.top + esquemaLienzo.scrollTop - posicion.y;
    evento.preventDefault();
  });
```

The offset is computed relative to `esquemaLienzo`'s own box (via `getBoundingClientRect()`), plus its current scroll position (`scrollLeft`/`scrollTop`) — not raw `evento.clientX`/`clientY` directly. `caja.style.left`/`top` are relative to `esquemaLienzo` (the nearest `position: relative` ancestor, set in Task 2's CSS), which is NOT the same coordinate space as viewport-relative mouse coordinates unless the canvas happens to sit at the viewport's top-left corner with no scroll — an assumption that won't generally hold. `preventDefault()` stops the browser's native text-selection drag gesture from interfering.

- [ ] **Step 2: Add the mousemove/mouseup listeners**

In `app/src/main.js`'s `DOMContentLoaded` handler, add this after the `#btn-cerrar-esquema` listener from Task 2:

```js
  document.addEventListener("mousemove", (evento) => {
    if (!cajaArrastrando) return;
    const rect = esquemaLienzo.getBoundingClientRect();
    const nuevaX = evento.clientX - rect.left + esquemaLienzo.scrollLeft - offsetArrastreX;
    const nuevaY = evento.clientY - rect.top + esquemaLienzo.scrollTop - offsetArrastreY;
    posicionesActuales[cajaArrastrando] = { x: nuevaX, y: nuevaY };
    const caja = esquemaLienzo.querySelector(`[data-tabla="${cajaArrastrando}"]`);
    caja.style.left = `${nuevaX}px`;
    caja.style.top = `${nuevaY}px`;
    dibujarRelaciones(esquemaRelacionesActuales);
  });

  document.addEventListener("mouseup", () => {
    cajaArrastrando = null;
  });
```

`mousemove`/`mouseup` are registered on `document` (not on the individual box) so dragging keeps working even if the mouse briefly leaves the box's own bounding area mid-drag — a common expectation for drag interactions, and the reason `cajaArrastrando` (a module-level "which box is being dragged" flag) exists instead of tracking drag state per-element.

- [ ] **Step 3: Self-review**

Read the diff. Confirm `dibujarRelaciones` is called on every `mousemove` while dragging — this is what makes the relationship lines visibly follow the box in real time, not just jump to the correct place after the drag ends. Confirm `mouseup` resets `cajaArrastrando` to `null` unconditionally (no missed case where a fast mouse-up outside any element could leave a box stuck "following" the cursor forever). Confirm this task adds no persistence of any kind (no `localStorage`, no new Tauri command) — dragging only mutates the in-memory `posicionesActuales` object, matching this plan's Global Constraints.

- [ ] **Step 4: Commit**

```bash
git add app/src/main.js
git commit -m "Make schema diagram table boxes draggable, with live-following relationship lines"
```

---

## Manual Verification (after all 3 tasks)

Same pattern as prior plans — guided verification in the real running app via `screencapture`, this time covering both companies (Hospital Arcángel from a fresh game, Postafeta after triggering the Agencia transition, or by checking the diagram reflects whichever company is currently loaded). Cover:
- "Ver esquema" (Consola) and "Base de Datos" (Hub tab) both open the same overlay with the same content.
- All 6 Hospital Arcángel tables appear with their real names, columns, types, and the real `COMMENT` text — spot-check `pacientes` and its `diagnostico`/`fecha_alta` columns against the actual schema SQL.
- Relationship lines connect the correct boxes (e.g. `pacientes` ↔ `departamentos`, `pacientes` ↔ `seguros`, `tratamientos` ↔ `pacientes`, `tratamientos` ↔ `empleados`) — no line drawn from `empleados` to itself despite the real `jefe_id` self-reference.
- Dragging a table box moves it smoothly and its connected lines follow in real time; releasing the mouse outside any box still ends the drag correctly.
- Closing and reopening the overlay resets every box to its default position (no persistence).
- Postafeta's `paquetes` table (4 foreign keys: `clientes`, `sucursales` ×2, `empleados`) renders all 4 lines correctly, including the two separate lines to `sucursales` (origen/destino) not merging into one.
