# Fase 0 / Plan 12: Reskin del Hub — escritorio de carpeta/papel rico Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reskin `#pantalla-hub` as a rich, warm manila-folder/paper desktop — profile card, career-progress card, per-ticket icon+priority badges, an honest "próximamente" stats placeholder, and a bottom tab bar — using only data that already exists (`dinero`, `reputacion`, `rango`, ticket `tipo`/`prioridad`, perks), with zero Rust changes.

**Architecture:** `#pantalla-hub` gains its own textured background and a 3-column CSS grid (profile+career / bandeja+stats-placeholder / perks), replacing plain emoji with hand-drawn inline SVG icons throughout. The shared `<header>` (used by both Hub and Consola) is left completely untouched in markup/styling — Consola must not change — but is hidden via one new class toggle specifically while Hub is the active screen, since Hub now shows its own richer top badges and profile card carrying the same information.

**Tech Stack:** Vanilla JS (ES modules), CSS Grid, inline SVG. No new dependencies, no backend changes.

## Global Constraints

- Only `#pantalla-hub` changes visually. Consola SQL and both overlays (`#scoring-overlay`, `#agencia-overlay`) must render identically to how Plan 10/11 left them — the shared `<header>` inside `#app-shell` must not change its markup, ids, or styling; it is only hidden (via a class toggle) while Hub is active, and shown exactly as before while Consola is active.
- No Rust/Tauri changes. This plan only reads `ticket.tipo` and `ticket.prioridad`, both already serialized to the frontend today (confirmed: `TipoTicket`/`Prioridad` have no `#[serde(skip_serializing)]` and no `rename_all`, so their JSON values are the exact Rust variant names: `"ReporteAnalisis"`, `"InvestigacionDepuracion"`, `"Baja"`, `"Media"`, `"Urgente"`).
- No fabricated data: the career-progress card must only ever show the 2 ranks that actually exist (`Becario`, `AuxiliarDeSistemas`) plus an honest "Próximamente..." line for ranks beyond scope — never invented rank names. The stats panel is an explicitly-labeled placeholder with no numbers.
- No new screens, no tab actually navigating anywhere new: "Dashboard" and "Perks" tabs scroll to their own section within the same Hub page; "Mis Logros" is disabled and shows a status message, matching the existing disabled-button convention already used for "Multijugador (próximamente)" in the Menú.
- Every emoji currently in `#pantalla-hub`'s own markup (not the shared header) is replaced by an inline SVG line icon, matching the placeholder-icon convention already established for the Consola's `RETRATOS`.
- No frontend test runner exists in this project (same as Plans 6-11) — correctness comes from careful diff self-review plus a final manual (visual-only) verification pass in the real running app.

---

### Task 1: Hub shell — folder background, hideable shared header, top badges, profile + career-progress cards

**Files:**
- Modify: `app/src/index.html:26-49` (the `#app-shell` header and the `#pantalla-hub` opening structure)
- Modify: `app/src/styles.css` (new rules, appended)
- Modify: `main.js:5-12` (new module vars), `main.js:26-46` (`mostrarPantalla`), `main.js:48-59` (`NOMBRE_RANGO`/`renderRango`), `main.js:162-169` (`pintarHubDesdeEstadoJuego`), `main.js:203-214` (`confirmarTransicionAgencia`), `main.js:286-303` (`submitTicket`), `main.js:351-377` (`DOMContentLoaded` var assignments)

**Interfaces:**
- Produces (consumed by Tasks 2-3): the `.hub-grid` 3-column layout with `.hub-columna-bandeja` and `.hub-columna-perks` as the two grid cells later tasks add content into. `actualizarDinero(valor)` and `actualizarReputacion(valorFormateado)` helper functions (Tasks 2-3 don't need these directly, but must not reintroduce direct `dineroEl.textContent=`/`reputacionEl.textContent=` assignments if they touch these areas).

- [ ] **Step 1: Give the shared header a stable id, without changing anything else about it**

In `app/src/index.html`, change line 27 from:

```html
      <header>
```

to:

```html
      <header id="header-app-shell">
```

This is the ONLY change to the shared header in this entire plan — its contents (`<h1>`, `.stats`, `#dinero`/`#reputacion`/`#rango`) stay byte-for-byte identical, since Consola relies on them unchanged.

- [ ] **Step 2: Restructure `#pantalla-hub`'s opening markup**

Still in `app/src/index.html`, replace this block (currently lines ~36-48):

```html
      <main class="container" id="pantalla-hub">
        <section class="bandeja">
          <h2 id="bandeja-titulo">Bandeja — turno actual</h2>
          <p>⏱️ Presupuesto de tiempo: <span id="presupuesto">0</span></p>
          <ul id="lista-tickets"></ul>
          <button id="btn-cerrar-dia">Cerrar día</button>
        </section>

        <section class="perks">
          <h2>Perks</h2>
          <ul id="lista-perks"></ul>
          <p id="perks-equipados-msg"></p>
        </section>
      </main>
```

with:

```html
      <main class="container hub-folder" id="pantalla-hub">
        <div class="hub-topbar">
          <div class="hub-badge">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="9"/><path d="M12 7v10M9 9.5c0-1 1-1.5 3-1.5s3 1 3 2-1 1.5-3 1.5-3 .5-3 1.5 1 2 3 2 3-.5 3-1.5"/></svg>
            <span id="dinero-hub">0</span>
          </div>
          <div class="hub-badge">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round"><path d="M12 2l2.9 6.3 6.9.6-5.2 4.6 1.6 6.8L12 17l-6.2 3.3 1.6-6.8L2.2 8.9l6.9-.6z"/></svg>
            <span id="reputacion-hub">0</span>
          </div>
        </div>

        <div class="hub-grid">
          <div class="hub-columna-perfil">
            <div class="tarjeta-perfil">
              <div class="clip-papel"></div>
              <div class="tarjeta-perfil-titulo">Tu perfil</div>
              <div class="tarjeta-perfil-avatar">
                <svg viewBox="0 0 8 8"><rect width="8" height="8" fill="#4a4a3a"/><rect x="2" y="1" width="4" height="3" fill="#8a8266"/><rect x="1" y="4" width="6" height="3" fill="#6b6b52"/></svg>
              </div>
              <div class="tarjeta-perfil-rango" id="rango-perfil">Becario</div>
              <div class="tarjeta-perfil-rango-label">Rango actual</div>
            </div>

            <div class="tarjeta-progreso-carrera">
              <div class="tarjeta-progreso-titulo">Progreso de carrera</div>
              <div class="progreso-item progreso-actual">
                <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6L9 17l-5-5"/></svg>
                <span id="progreso-rango-actual-texto">Becario</span>
              </div>
              <div class="progreso-item progreso-siguiente" id="progreso-rango-siguiente">➜ Auxiliar de Sistemas</div>
              <div class="progreso-item progreso-futuro">
                <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round"><rect x="5" y="11" width="14" height="9" rx="1"/><path d="M8 11V7a4 4 0 0 1 8 0v4"/></svg>
                Próximamente...
              </div>
            </div>
          </div>

          <section class="bandeja hub-columna-bandeja">
            <h2 id="bandeja-titulo">Bandeja — turno actual</h2>
            <p>⏱️ Presupuesto de tiempo: <span id="presupuesto">0</span></p>
            <ul id="lista-tickets"></ul>
            <button id="btn-cerrar-dia">Cerrar día</button>
          </section>

          <section class="perks hub-columna-perks">
            <h2>Perks</h2>
            <ul id="lista-perks"></ul>
            <p id="perks-equipados-msg"></p>
          </section>
        </div>
      </main>
```

Note: the `<section class="bandeja hub-columna-bandeja">` and `<section class="perks hub-columna-perks">` blocks are intentionally unchanged inside (Tasks 2-3 will modify their inner content) — this step only moves them into the new grid wrapper and adds the two new sibling elements (top badges, profile+career column).

- [ ] **Step 3: Add the Task 1 CSS**

In `app/src/styles.css`, append at the end of the file:

```css
.hub-folder {
  background: linear-gradient(155deg, #c9a876, #b8965f);
  border-radius: 10px;
  color: #2a2018;
  position: relative;
}

.hub-topbar {
  display: flex;
  justify-content: flex-end;
  gap: 0.5rem;
  margin-bottom: 1rem;
}

.hub-badge {
  background: #f0e6d2;
  border: 2px solid #2a2018;
  border-radius: 6px;
  padding: 0.35rem 0.75rem;
  display: flex;
  align-items: center;
  gap: 0.4rem;
  font-family: "SF Mono", Menlo, Consolas, monospace;
  font-size: 0.75rem;
  color: #2a2018;
  box-shadow: 2px 2px 0 rgba(0, 0, 0, 0.2);
}

.hub-grid {
  display: grid;
  grid-template-columns: 170px 1fr 220px;
  gap: 1rem;
  align-items: start;
}

.tarjeta-perfil {
  background: #f7f0dc;
  border-radius: 2px;
  padding: 0.9rem;
  box-shadow: 3px 4px 10px rgba(0, 0, 0, 0.35);
  transform: rotate(-1.2deg);
  margin-bottom: 0.9rem;
  position: relative;
}

.tarjeta-perfil-titulo {
  font-size: 0.5rem;
  text-transform: uppercase;
  color: #8a7355;
  letter-spacing: 0.08em;
  margin-bottom: 0.5rem;
}

.tarjeta-perfil-avatar {
  width: 60px;
  height: 60px;
  background: #3a3a2e;
  border: 3px solid #6b6b52;
  border-radius: 4px;
  margin: 0 auto 0.5rem;
  display: flex;
  align-items: center;
  justify-content: center;
}

.tarjeta-perfil-avatar svg {
  width: 42px;
  height: 42px;
}

.tarjeta-perfil-rango {
  text-align: center;
  font-size: 0.7rem;
  color: #2a2018;
  font-weight: 700;
}

.tarjeta-perfil-rango-label {
  text-align: center;
  font-size: 0.5rem;
  color: #8a7355;
  margin-top: 0.1rem;
}

.tarjeta-progreso-carrera {
  background: #f5e28a;
  border-radius: 2px;
  padding: 0.6rem 0.75rem;
  box-shadow: 2px 3px 8px rgba(0, 0, 0, 0.3);
  transform: rotate(0.8deg);
}

.tarjeta-progreso-titulo {
  font-size: 0.5rem;
  text-transform: uppercase;
  color: #6b5a2a;
  letter-spacing: 0.06em;
  margin-bottom: 0.4rem;
}

.progreso-item {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  font-size: 0.65rem;
  margin-bottom: 0.3rem;
}

.progreso-actual {
  color: #2a4a1a;
}

.progreso-siguiente {
  color: #2a2018;
  font-weight: 700;
}

.progreso-futuro {
  color: #9a9370;
  opacity: 0.7;
}

.clip-papel {
  position: absolute;
  top: -8px;
  left: 50%;
  transform: translateX(-50%);
  width: 20px;
  height: 12px;
  background: #c0c0c0;
  border-radius: 2px;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.4);
}
```

- [ ] **Step 4: Add module-level variables in `main.js`**

Change `app/src/main.js` lines 5-12 from:

```js
let sqlInput, statusMsg, resultTable, dineroEl, reputacionEl, rangoEl;
let listaPerks, perksEquipadosMsg;
let presupuestoEl, listaTickets, ticketActivoInfo, bandejaTitulo;
let scoringOverlay, scoringAscenso, agenciaOverlay;
let pantallaMenu, appShell, pantallaHub, pantallaConsola, btnCargarPartida;
let ticketRetrato, consolaTitulo;
let btnMuteMusica, btnMuteEfectos;
let btnCerrarScoring;
```

to:

```js
let sqlInput, statusMsg, resultTable, dineroEl, reputacionEl, rangoEl;
let listaPerks, perksEquipadosMsg;
let presupuestoEl, listaTickets, ticketActivoInfo, bandejaTitulo;
let scoringOverlay, scoringAscenso, agenciaOverlay;
let pantallaMenu, appShell, pantallaHub, pantallaConsola, btnCargarPartida;
let ticketRetrato, consolaTitulo;
let btnMuteMusica, btnMuteEfectos;
let btnCerrarScoring;
let headerAppShell, dineroHubEl, reputacionHubEl;
let rangoPerfilEl, progresoRangoActualTextoEl, progresoRangoSiguienteEl;
```

- [ ] **Step 5: Hide the shared header specifically while Hub is active**

Change `mostrarPantalla` (`main.js:41-46`) from:

```js
function mostrarPantalla(nombre) {
  alternarPantalla(pantallaMenu, nombre === "menu");
  alternarPantalla(appShell, nombre !== "menu");
  alternarPantalla(pantallaHub, nombre === "hub");
  alternarPantalla(pantallaConsola, nombre === "consola");
}
```

to:

```js
function mostrarPantalla(nombre) {
  alternarPantalla(pantallaMenu, nombre === "menu");
  alternarPantalla(appShell, nombre !== "menu");
  alternarPantalla(pantallaHub, nombre === "hub");
  alternarPantalla(pantallaConsola, nombre === "consola");
  headerAppShell.classList.toggle("oculto", nombre === "hub");
}
```

`headerAppShell` uses the plain (non-fading) `.oculto { display: none; }` global rule directly — it does not need the cross-fade treatment `alternarPantalla` gives the 4 screen elements, since it's shown/hidden instantly alongside whichever screen is fading in.

- [ ] **Step 6: Add dinero/reputación update helpers, and use them everywhere dinero/reputación are set**

In `app/src/main.js`, add these two functions right after `renderRango` (after line 59, before `let ticketActivoId = null;`):

```js
function actualizarDinero(valor) {
  dineroEl.textContent = valor;
  dineroHubEl.textContent = valor;
}

function actualizarReputacion(valorFormateado) {
  reputacionEl.textContent = valorFormateado;
  reputacionHubEl.textContent = valorFormateado;
}
```

Then replace these 4 direct assignments:

In `pintarHubDesdeEstadoJuego` (`main.js:162-169`), change:

```js
function pintarHubDesdeEstadoJuego(estadoJuego) {
  dineroEl.textContent = estadoJuego.dinero;
  reputacionEl.textContent = estadoJuego.reputacion.toFixed(1);
  renderRango(estadoJuego.rango);
  renderBandeja(estadoJuego);
  ticketActivoId = null;
  mostrarPantalla("hub");
}
```

to:

```js
function pintarHubDesdeEstadoJuego(estadoJuego) {
  actualizarDinero(estadoJuego.dinero);
  actualizarReputacion(estadoJuego.reputacion.toFixed(1));
  renderRango(estadoJuego.rango);
  renderBandeja(estadoJuego);
  ticketActivoId = null;
  mostrarPantalla("hub");
}
```

In `confirmarTransicionAgencia` (`main.js:203-214`), change the line `reputacionEl.textContent = "0.0";` to `actualizarReputacion("0.0");`.

In `submitTicket` (`main.js:286-303`), change:

```js
    dineroEl.textContent = score.dinero_total;
    reputacionEl.textContent = score.reputacion_total.toFixed(1);
```

to:

```js
    actualizarDinero(score.dinero_total);
    actualizarReputacion(score.reputacion_total.toFixed(1));
```

- [ ] **Step 7: Extend `renderRango` to update the profile card and career-progress card**

The 2 existing ranks in declaration order double as the "reachable order" for the career-progress card. Change `NOMBRE_RANGO`/`renderRango` (`main.js:52-59`) from:

```js
const NOMBRE_RANGO = {
  Becario: "Becario",
  AuxiliarDeSistemas: "Auxiliar de Sistemas",
};

function renderRango(rango) {
  rangoEl.textContent = NOMBRE_RANGO[rango] || rango;
}
```

to:

```js
const NOMBRE_RANGO = {
  Becario: "Becario",
  AuxiliarDeSistemas: "Auxiliar de Sistemas",
};

const ORDEN_RANGOS = ["Becario", "AuxiliarDeSistemas"];

function renderRango(rango) {
  const nombre = NOMBRE_RANGO[rango] || rango;
  rangoEl.textContent = nombre;
  rangoPerfilEl.textContent = nombre;
  progresoRangoActualTextoEl.textContent = nombre;

  const indiceActual = ORDEN_RANGOS.indexOf(rango);
  const siguienteRango = ORDEN_RANGOS[indiceActual + 1];
  progresoRangoSiguienteEl.textContent = siguienteRango
    ? `➜ ${NOMBRE_RANGO[siguienteRango]}`
    : "Alcanzaste el máximo rango disponible";
}
```

- [ ] **Step 8: Wire the new elements in `DOMContentLoaded`**

In `app/src/main.js`'s `DOMContentLoaded` handler, right after the existing `btnCerrarScoring = document.querySelector("#btn-cerrar-scoring");` line (`main.js:376`), add:

```js
  headerAppShell = document.querySelector("#header-app-shell");
  dineroHubEl = document.querySelector("#dinero-hub");
  reputacionHubEl = document.querySelector("#reputacion-hub");
  rangoPerfilEl = document.querySelector("#rango-perfil");
  progresoRangoActualTextoEl = document.querySelector("#progreso-rango-actual-texto");
  progresoRangoSiguienteEl = document.querySelector("#progreso-rango-siguiente");
```

- [ ] **Step 9: Self-review**

Read the full diff. Confirm the shared `<header>`'s inner markup (`<h1>`, `.stats`, `#dinero`/`#reputacion`/`#rango`) is byte-for-byte unchanged except for the added `id="header-app-shell"` on the `<header>` tag itself. Confirm `renderRango` is still called from every existing call site (`pintarHubDesdeEstadoJuego`, `submitTicket` via `renderRango(score.rango_actual)`) without signature changes. Confirm `actualizarDinero`/`actualizarReputacion` fully replace the 4 direct assignments listed in Step 6 — grep for `dineroEl.textContent =` and `reputacionEl.textContent =` in the final file; the only remaining occurrences should be inside the two new helper functions themselves.

- [ ] **Step 10: Commit**

```bash
git add app/src/index.html app/src/styles.css app/src/main.js
git commit -m "Give the Hub its own folder-textured shell: top badges, profile card, career-progress card"
```

---

### Task 2: Bandeja — ticket icon-by-tipo, priority label, moon icon on Cerrar día

**Files:**
- Modify: `app/src/main.js` (new lookups, `renderBandeja` rewrite, `cerrarDia`'s button markup is untouched — only its icon in HTML)
- Modify: `app/src/index.html` (Cerrar día button icon)
- Modify: `app/src/styles.css` (new rules for ticket icon badge + priority label + paperclip)

**Interfaces:**
- Consumes: `.hub-columna-bandeja` from Task 1 (the `<section class="bandeja hub-columna-bandeja">` element, untouched in structure except its `<ul id="lista-tickets">` children).
- Produces: nothing consumed by Task 3.

- [ ] **Step 1: Add the ticket-type icon and priority lookups**

In `app/src/main.js`, add these two constants right after the `RETRATOS`/`retratoParaSolicitante` block (after line 22, before `const DURACION_TRANSICION_MS = 250;`):

```js
const ICONOS_TIPO_TICKET = {
  ReporteAnalisis: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round" stroke-linecap="round"><rect x="4" y="3" width="16" height="18" rx="1"/><path d="M8 12v4M12 9v7M16 13v3"/></svg>`,
  InvestigacionDepuracion: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="10" cy="10" r="6"/><path d="M20 20l-5.5-5.5"/></svg>`,
};

const PRIORIDAD_INFO = {
  Baja: { color: "#5a7a3a", etiqueta: "BAJA" },
  Media: { color: "#9a7a2a", etiqueta: "MEDIA" },
  Urgente: { color: "#a13a54", etiqueta: "URGENTE" },
};
```

- [ ] **Step 2: Rewrite `renderBandeja`'s ticket-card construction**

Change `renderBandeja` (`main.js`, the `estadoTurno.pendientes.forEach` block) from:

```js
  estadoTurno.pendientes.forEach((ticket, indice) => {
    const li = document.createElement("li");
    li.className = "papel papel-entrando";
    li.style.animationDelay = `${indice * 60}ms`;
    const info = document.createElement("span");
    info.textContent = `[⏱️ ${ticket.costo_tiempo}] ${ticket.motivo}`;
    const boton = document.createElement("button");
    boton.textContent = ticket.id === ticketActivoId ? "En curso" : "Trabajar en este";
    boton.addEventListener("click", () => seleccionarTicket(ticket));
    li.appendChild(info);
    li.appendChild(boton);
    listaTickets.appendChild(li);
  });
```

to:

```js
  estadoTurno.pendientes.forEach((ticket, indice) => {
    const li = document.createElement("li");
    li.className = "papel papel-entrando papel-ticket";
    li.style.animationDelay = `${indice * 60}ms`;

    const clip = document.createElement("div");
    clip.className = "clip-papel";
    li.appendChild(clip);

    const icono = document.createElement("div");
    icono.className = "icono-tipo-ticket";
    icono.innerHTML = ICONOS_TIPO_TICKET[ticket.tipo] || ICONOS_TIPO_TICKET.ReporteAnalisis;
    li.appendChild(icono);

    const detalle = document.createElement("div");
    detalle.className = "papel-ticket-detalle";
    const info = document.createElement("div");
    info.className = "papel-ticket-motivo";
    info.textContent = `[⏱️ ${ticket.costo_tiempo}] ${ticket.motivo}`;
    const prioridad = PRIORIDAD_INFO[ticket.prioridad] || PRIORIDAD_INFO.Baja;
    const etiquetaPrioridad = document.createElement("div");
    etiquetaPrioridad.className = "papel-ticket-prioridad";
    etiquetaPrioridad.style.color = prioridad.color;
    etiquetaPrioridad.textContent = `● ${prioridad.etiqueta}`;
    detalle.appendChild(info);
    detalle.appendChild(etiquetaPrioridad);
    li.appendChild(detalle);

    const boton = document.createElement("button");
    boton.textContent = ticket.id === ticketActivoId ? "En curso" : "Trabajar en este";
    boton.addEventListener("click", () => seleccionarTicket(ticket));
    li.appendChild(boton);

    listaTickets.appendChild(li);
  });
```

- [ ] **Step 3: Add a moon icon to "Cerrar día"**

In `app/src/index.html`, change (inside `<section class="bandeja hub-columna-bandeja">`, from Task 1's markup):

```html
            <button id="btn-cerrar-dia">Cerrar día</button>
```

to:

```html
            <button id="btn-cerrar-dia">
              Cerrar día
              <svg width="13" height="13" viewBox="0 0 24 24" fill="currentColor"><path d="M20 14.5A8.5 8.5 0 1 1 9.5 4a7 7 0 0 0 10.5 10.5z"/></svg>
            </button>
```

- [ ] **Step 4: Add the CSS for the ticket icon badge, priority label, and paperclip**

In `app/src/styles.css`, append:

```css
.papel-ticket {
  position: relative;
  align-items: flex-start;
}

.papel-ticket .clip-papel {
  top: -6px;
  left: 14px;
  transform: none;
}

.icono-tipo-ticket {
  width: 32px;
  height: 32px;
  border-radius: 6px;
  background: #e8a5b8;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  color: #5a2035;
}

.icono-tipo-ticket svg {
  width: 18px;
  height: 18px;
}

.papel-ticket-detalle {
  flex: 1;
}

.papel-ticket-motivo {
  font-size: 0.7rem;
}

.papel-ticket-prioridad {
  font-size: 0.6rem;
  margin-top: 0.15rem;
}

#btn-cerrar-dia {
  display: inline-flex;
  align-items: center;
  gap: 0.5rem;
}
```

- [ ] **Step 5: Self-review**

Read the diff. Confirm `ticket.tipo`/`ticket.prioridad` are read directly off the `ticket` objects already passed into `renderBandeja` via `estadoTurno.pendientes` (no new invoke call needed — these fields already arrive with every ticket, confirmed against `app/src-tauri/src/tickets/mod.rs`'s `Ticket` struct). Confirm the fallback (`|| ICONOS_TIPO_TICKET.ReporteAnalisis` / `|| PRIORIDAD_INFO.Baja`) only matters defensively and doesn't mask a real bug — both `TipoTicket` and `Prioridad` are exhaustive 2-variant/3-variant Rust enums with no `#[serde(skip_serializing)]`, so every ticket should always carry a valid value matching a lookup key exactly. Confirm the click handler on `boton` (`seleccionarTicket(ticket)`) is unchanged.

- [ ] **Step 6: Commit**

```bash
git add app/src/main.js app/src/index.html app/src/styles.css
git commit -m "Give ticket cards an icon-by-type badge and a real priority label"
```

---

### Task 3: Perks restyle, stats placeholder, bottom tab bar, decorative stamp

**Files:**
- Modify: `app/src/main.js` (`renderPerks` icon swap, tab bar wiring, `DOMContentLoaded`)
- Modify: `app/src/index.html` (stats placeholder inside the bandeja column, tab bar + stamp at the end of `#pantalla-hub`)
- Modify: `app/src/styles.css` (perk icon/coffee-stain rules, stats placeholder, tab bar, stamp)

**Interfaces:**
- Consumes: `.hub-columna-bandeja`/`.hub-columna-perks` from Task 1, `ICONOS_TIPO_TICKET`-style lookup pattern established in Task 2 (this task adds its own separate lookups for perk state icons, not reusing Task 2's).
- Produces: nothing consumed by later work — this is the last task in the plan.

- [ ] **Step 1: Add the stats placeholder panel**

In `app/src/index.html`, inside `<section class="bandeja hub-columna-bandeja">`, Task 2 left the Cerrar día button as:

```html
            <button id="btn-cerrar-dia">
              Cerrar día
              <svg width="13" height="13" viewBox="0 0 24 24" fill="currentColor"><path d="M20 14.5A8.5 8.5 0 1 1 9.5 4a7 7 0 0 0 10.5 10.5z"/></svg>
            </button>
```

Add this right after that button's closing `</button>` tag (still inside `<section class="bandeja hub-columna-bandeja">`):

```html
            <div class="panel-stats-placeholder">
              <div class="panel-stats-titulo">Estadísticas (próximamente)</div>
              <div class="panel-stats-texto">Consultas resueltas, precisión, tiempo promedio — llegan en un plan futuro</div>
            </div>
```

- [ ] **Step 2: Add the coffee-stain decoration to the perks column**

In `app/src/index.html`, inside `<section class="perks hub-columna-perks">` (from Task 1), add this right after the existing `<p id="perks-equipados-msg"></p>` line:

```html
            <div class="mancha-cafe"></div>
```

- [ ] **Step 3: Add the bottom tab bar and the decorative stamp**

In `app/src/index.html`, right after the closing `</div>` of `.hub-grid` (still inside `<main id="pantalla-hub">`, before that `</main>` tag), add:

```html
        <div class="barra-pestanas">
          <div class="pestana pestana-activa" id="tab-dashboard">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/></svg>
            Dashboard
          </div>
          <div class="pestana" id="tab-perks">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round"><path d="M12 2l7 4v6c0 5-3 8-7 10-4-2-7-5-7-10V6z"/></svg>
            Perks
          </div>
          <div class="pestana pestana-bloqueada" id="tab-logros" title="Próximamente">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round" stroke-linecap="round"><path d="M8 4h8v4a4 4 0 0 1-8 0z"/><path d="M8 5H5a2 2 0 0 0 2 4M16 5h3a2 2 0 0 1-2 4"/><path d="M12 12v3M9 20h6M10 17h4l1 3H9z"/></svg>
            Mis Logros
          </div>
        </div>

        <div class="sello-confidencial">CONFIDENCIAL</div>
```

- [ ] **Step 4: Replace the perk status emoji with SVG icons**

Change `renderPerks` in `app/src/main.js` from:

```js
    const info = document.createElement("span");
    const estado = perk.equipado ? "⭐ equipado" : perk.desbloqueado ? "✅ desbloqueado" : "🔒 bloqueado";
    info.textContent = `${perk.nombre} (${perk.categoria}) — ${estado} — $${perk.costo_dinero}, ⭐${perk.reputacion_minima}`;
```

to:

```js
    const ICONO_EQUIPADO = `<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round"><path d="M12 2l2.9 6.3 6.9.6-5.2 4.6 1.6 6.8L12 17l-6.2 3.3 1.6-6.8L2.2 8.9l6.9-.6z"/></svg>`;
    const ICONO_DESBLOQUEADO = `<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6L9 17l-5-5"/></svg>`;
    const ICONO_BLOQUEADO = `<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round"><rect x="5" y="11" width="14" height="9" rx="1"/><path d="M8 11V7a4 4 0 0 1 8 0v4"/></svg>`;

    const info = document.createElement("span");
    const iconoEstado = perk.equipado ? ICONO_EQUIPADO : perk.desbloqueado ? ICONO_DESBLOQUEADO : ICONO_BLOQUEADO;
    const textoEstado = perk.equipado ? "equipado" : perk.desbloqueado ? "desbloqueado" : "bloqueado";
    info.innerHTML = `${iconoEstado} ${perk.nombre} (${perk.categoria}) — ${textoEstado} — $${perk.costo_dinero}, ⭐${perk.reputacion_minima}`;
```

`info` changes from a `<span>` populated via `.textContent` to one populated via `.innerHTML` — this is safe here because every value interpolated into the template (`iconoEstado` is one of the 3 fixed constants above; `perk.nombre`/`perk.categoria`/`perk.costo_dinero`/`perk.reputacion_minima` all come from our own Rust-side perk catalog, never from user-typed input) — there is no injected untrusted string.

- [ ] **Step 5: Wire the tab bar clicks**

In `app/src/main.js`'s `DOMContentLoaded` handler, add this after the `btnMuteEfectos.addEventListener(...)` block (the last block in the file, right before the closing `});`):

```js
  document.querySelector("#tab-dashboard").addEventListener("click", () => {
    document.querySelector(".hub-columna-bandeja").scrollIntoView({ behavior: "smooth", block: "start" });
  });

  document.querySelector("#tab-perks").addEventListener("click", () => {
    document.querySelector(".hub-columna-perks").scrollIntoView({ behavior: "smooth", block: "start" });
  });

  document.querySelector("#tab-logros").addEventListener("click", () => {
    setStatus("Próximamente.", "");
  });
```

- [ ] **Step 6: Add the Task 3 CSS**

In `app/src/styles.css`, append:

```css
.panel-stats-placeholder {
  background: #f7f0dc;
  border-radius: 2px;
  padding: 0.75rem 0.9rem;
  box-shadow: 2px 3px 8px rgba(0, 0, 0, 0.25);
  opacity: 0.55;
  margin-top: 0.75rem;
}

.panel-stats-titulo {
  font-size: 0.5rem;
  text-transform: uppercase;
  color: #8a7355;
  letter-spacing: 0.06em;
  margin-bottom: 0.4rem;
}

.panel-stats-texto {
  font-size: 0.6rem;
  color: #8a7355;
}

.mancha-cafe {
  width: 50px;
  height: 50px;
  border: 3px solid rgba(90, 60, 30, 0.25);
  border-radius: 50%;
  margin: 1.25rem auto 0;
}

.barra-pestanas {
  display: flex;
  gap: 0.4rem;
  margin-top: 1.1rem;
}

.pestana {
  background: #6b5a45;
  color: #a89878;
  padding: 0.5rem 1rem;
  border-radius: 4px 4px 0 0;
  font-size: 0.65rem;
  display: flex;
  align-items: center;
  gap: 0.4rem;
  cursor: pointer;
  opacity: 0.75;
}

.pestana:hover {
  opacity: 1;
}

.pestana-activa {
  background: #2a2018;
  color: #e8dcc0;
  opacity: 1;
}

.pestana-bloqueada {
  opacity: 0.5;
  cursor: not-allowed;
}

.sello-confidencial {
  position: absolute;
  bottom: 3.5rem;
  right: 1.25rem;
  border: 2px solid #a13a3a;
  color: #a13a3a;
  padding: 0.15rem 0.5rem;
  font-size: 0.55rem;
  font-weight: 700;
  transform: rotate(-8deg);
  opacity: 0.55;
  pointer-events: none;
}
```

- [ ] **Step 7: Self-review**

Read the diff. Confirm `.hub-folder` (added in Task 1, already committed) carries `position: relative;` — this is what makes `.sello-confidencial`'s `position: absolute` anchor correctly within the Hub's own box instead of the full-viewport `#app-shell` (which also has `position: relative` via `.escritorio`, and would otherwise "win" as the containing block). Confirm `#tab-logros`'s click handler does not attempt to invoke any Tauri command (it must only call `setStatus`, matching the disabled-but-clickable "Multijugador" convention — this tab is not a real `<button disabled>`, so it still needs a click listener guarding it from doing anything real, which Step 5 provides). Confirm the `info.innerHTML` change in `renderPerks` only interpolates the 3 fixed icon constants and Rust-catalog-sourced perk fields, never anything user-typed.

- [ ] **Step 8: Commit**

```bash
git add app/src/main.js app/src/index.html app/src/styles.css
git commit -m "Restyle Perks with icon-based state, add stats placeholder, tab bar, and decorative stamp"
```

---

## Manual Verification (after all 3 tasks)

Same pattern as Plans 6-11 — guided verification in the real running app via `screencapture`, purely visual this time (Plan 11's audio is untouched). Cover:
- Menu and Consola/overlays look **exactly as before** — this is the critical regression check, since the shared header must be provably unaffected in Consola.
- Hub: folder-texture background, top badges (💰/⭐ with icons), profile card (avatar + rango), career-progress card (correct current/next rank, "Próximamente..." line).
- Bandeja: each ticket shows the right icon for its `tipo` and the right color/label for its `prioridad`; "Cerrar día" has its moon icon.
- Stats placeholder panel reads clearly as "próximamente", not as real data.
- Perks: icon-based state (locked/unlocked/equipped) instead of emoji; coffee-stain visible.
- Tab bar: "Dashboard"/"Perks" scroll smoothly to their section; "Mis Logros" shows the "Próximamente." status message and does nothing else.
- Resize/scroll the window if needed to confirm the 3-column grid doesn't overlap or clip at the app's default window size.
