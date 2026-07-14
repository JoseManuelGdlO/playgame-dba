# Fase 0 / Plan 15: Tutorial de Onboarding con El Mentor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a Mentor-guided, blocking tutorial that walks a first-time player through their very first ticket (`hospital_reporte_pacientes_cardiologia`), teaching `SELECT`/`FROM`/`WHERE`/`ORDER BY` from zero by having them write the real query themselves — with a reusable Papers-Please-style dialogue/blip engine underneath.

**Architecture:** A new standalone ES module `app/src/dialogo.js` (mirrors the existing `app/src/audio.js` pattern: pure engine, no project-specific state) exposes a blocking, spotlight-capable dialogue card with progressive text reveal + blip sound. A new `app/src/tutorial.js` module owns the Plan-15-specific beat script and state machine, built on top of `dialogo.js`. `app/src/main.js` wires a handful of existing functions (`iniciarPartida`, `seleccionarTicket`, `renderBandeja`, the Play/Enviar/Cerrar-scoring button handlers, the SQL textarea, the Esc handler) to notify `tutorial.js` at the right moments. `app/src/index.html`/`app/src/styles.css` gain one new always-visible skip button and the dialogue/spotlight CSS.

**Tech Stack:** Vanilla JS (ES modules), CSS (`position: fixed` + `box-shadow` spotlight trick), Web Audio API (via the existing `audio.js` engine). No new dependencies, no Rust, no new Tauri commands.

## Global Constraints

- Reuse the existing "El Mentor" character (portrait already defined as `RETRATOS["El Mentor"]` in `app/src/main.js`) — no new character.
- The tutorial teaches real SQL from zero using the real first ticket (`hospital_reporte_pacientes_cardiologia`) — the player types every clause themselves; nothing is auto-filled.
- Triggers only after `iniciar_partida()` (brand new game). Never on `cargar_partida()`. No persistence of tutorial progress across app restarts.
- A "Ya sé SQL, saltar" button is visible for the entire duration of the tutorial and ends it immediately when clicked.
- While the tutorial is active, Esc must not open the pause menu.
- Per-clause "advance" checks are a loose, case/whitespace-insensitive substring match against the live textarea content — never a duplicate of the real SQL validation engine, which only runs on the real "Enviar ticket" submission.
- Strictly frontend: only `app/src/*.{js,html,css}` change. No Rust, no new Tauri commands.
- No frontend test runner exists in this project (same as Plans 6-14) — correctness comes from careful diff self-review plus a final manual verification pass in the real running app.

---

### Task 1: Reusable dialogue engine (`dialogo.js`) + blip sound

**Files:**
- Create: `app/src/dialogo.js`
- Modify: `app/src/audio.js` (add `sfxBlip()` after the existing `sfxTecleo()`, lines 57-60)
- Modify: `app/src/styles.css` (append new rules at the end of the file, after line 958)

**Interfaces:**
- Produces (consumed by Task 2): `app/src/dialogo.js` exports `mostrarDialogo(retratoSvg, nombre, texto, opciones)` where `opciones` is `{ permitir?: string[], alContinuar?: () => void }`, `ocultarDialogo()`, and `permitirSiempre(selectores: string[])`.
- Produces (consumed by Task 2): `app/src/audio.js` exports a new `sfxBlip()` alongside its existing exports.

- [ ] **Step 1: Add `sfxBlip()` to the audio engine**

In `app/src/audio.js`, right after the existing `sfxTecleo` function (lines 57-60):

```js
export function sfxTecleo() {
  const variacion = 0.9 + Math.random() * 0.2;
  tono(1200 * variacion, 30, "square", 0.04);
}
```

add:

```js
export function sfxBlip() {
  const variacion = 0.85 + Math.random() * 0.5;
  tono(300 * variacion, 45, "square", 0.05);
}
```

This reuses the existing `tono()` helper already defined above in the same file — no other changes to `audio.js` are needed.

- [ ] **Step 2: Create the dialogue engine module**

Create `app/src/dialogo.js` with this exact content:

```js
import { sfxBlip } from "./audio.js";

let capaBloqueo = null;
let tarjeta = null;
let elementoTexto = null;
let selectoresPermitidos = [];
let siempreVisibles = [];
let callbackContinuar = null;
let intervaloRevelado = null;
let manejadorRedimension = null;
let textoCompleto = "";
let revelando = false;

const VELOCIDAD_REVELADO_MS = 30;

function manejarClickDocumento(evento) {
  const enTarjeta = evento.target.closest(".dialogo-tarjeta");
  const enPermitido = selectoresPermitidos.some((selector) => evento.target.closest(selector));
  const enSiempreVisible = siempreVisibles.some((selector) => evento.target.closest(selector));
  if (enTarjeta || enPermitido || enSiempreVisible) return;
  evento.stopPropagation();
  evento.preventDefault();
}

function detenerRevelado() {
  if (intervaloRevelado) {
    clearInterval(intervaloRevelado);
    intervaloRevelado = null;
  }
  revelando = false;
}

function completarRevelado() {
  detenerRevelado();
  elementoTexto.textContent = textoCompleto;
}

function iniciarRevelado() {
  let indice = 0;
  revelando = true;
  elementoTexto.textContent = "";
  intervaloRevelado = setInterval(() => {
    indice += 1;
    elementoTexto.textContent = textoCompleto.slice(0, indice);
    if (indice % 3 === 0) {
      sfxBlip();
    }
    if (indice >= textoCompleto.length) {
      detenerRevelado();
    }
  }, VELOCIDAD_REVELADO_MS);
}

function actualizarSpotlight(spotlight, selector) {
  const elemento = selector && document.querySelector(selector);
  if (!elemento) {
    spotlight.style.display = "none";
    return;
  }
  const rect = elemento.getBoundingClientRect();
  spotlight.style.display = "block";
  spotlight.style.left = `${rect.left - 6}px`;
  spotlight.style.top = `${rect.top - 6}px`;
  spotlight.style.width = `${rect.width + 12}px`;
  spotlight.style.height = `${rect.height + 12}px`;
}

export function permitirSiempre(selectores) {
  siempreVisibles = selectores;
}

export function mostrarDialogo(retratoSvg, nombre, texto, opciones = {}) {
  ocultarDialogo();

  selectoresPermitidos = opciones.permitir || [];
  callbackContinuar = opciones.alContinuar || null;
  textoCompleto = texto;

  capaBloqueo = document.createElement("div");
  capaBloqueo.className = "dialogo-bloqueo";

  const objetivoSpotlight = selectoresPermitidos[0];
  if (objetivoSpotlight) {
    const spotlight = document.createElement("div");
    spotlight.className = "dialogo-spotlight";
    capaBloqueo.appendChild(spotlight);
    actualizarSpotlight(spotlight, objetivoSpotlight);
    manejadorRedimension = () => actualizarSpotlight(spotlight, objetivoSpotlight);
    window.addEventListener("resize", manejadorRedimension);
  } else {
    const dim = document.createElement("div");
    dim.className = "dialogo-dim";
    capaBloqueo.appendChild(dim);
  }

  tarjeta = document.createElement("div");
  tarjeta.className = "dialogo-tarjeta";
  tarjeta.innerHTML = `
    <div class="retrato">${retratoSvg}</div>
    <div class="dialogo-cuerpo">
      <div class="dialogo-nombre">${nombre}</div>
      <div class="dialogo-texto"></div>
    </div>
  `;
  elementoTexto = tarjeta.querySelector(".dialogo-texto");
  tarjeta.addEventListener("click", () => {
    if (revelando) {
      completarRevelado();
    } else if (callbackContinuar) {
      callbackContinuar();
    }
  });
  capaBloqueo.appendChild(tarjeta);

  document.body.appendChild(capaBloqueo);
  document.addEventListener("click", manejarClickDocumento, true);

  iniciarRevelado();
}

export function ocultarDialogo() {
  detenerRevelado();
  document.removeEventListener("click", manejarClickDocumento, true);
  if (manejadorRedimension) {
    window.removeEventListener("resize", manejadorRedimension);
    manejadorRedimension = null;
  }
  if (capaBloqueo) {
    capaBloqueo.remove();
    capaBloqueo = null;
  }
  tarjeta = null;
  elementoTexto = null;
  selectoresPermitidos = [];
  callbackContinuar = null;
}
```

Design notes for the reviewer:
- `capaBloqueo` (the fixed, full-viewport container) is given `pointer-events: none` in CSS (Step 3) — this lets every click pass straight through it to whatever real element is underneath. The actual blocking is done entirely by `manejarClickDocumento`, a **capture-phase** listener on `document`: capture fires before the click reaches its real target, so `evento.stopPropagation()` there stops a disallowed click before any real button's own listener ever runs. This is why the visual "spotlight cutout" (a `box-shadow: 0 0 0 2000px …` ring) and the actual click-blocking are two independent mechanisms — the box-shadow is purely cosmetic (dims everything outside its own rectangle), and the document-capture listener is what actually enforces "only this element is clickable."
- `.dialogo-tarjeta` overrides back to `pointer-events: auto` in CSS so the card itself stays clickable despite its `pointer-events: none` ancestor.
- `permitirSiempre()` lets a consumer register selectors (like a skip button) that must stay clickable through **every** beat, regardless of that beat's own `opciones.permitir` — Task 2 uses this for the tutorial's skip button.
- `mostrarDialogo` calls `ocultarDialogo()` as its first line, so calling it repeatedly (one call per tutorial beat) never leaks a duplicate `document` listener, a duplicate `resize` listener, or a duplicate `<div class="dialogo-bloqueo">` in the DOM.

- [ ] **Step 3: Add the dialogue/spotlight CSS**

In `app/src/styles.css`, append at the end of the file (after line 958, the last existing rule `.columna-descripcion`):

```css
.dialogo-bloqueo {
  position: fixed;
  inset: 0;
  z-index: 9000;
  pointer-events: none;
}

.dialogo-dim {
  position: fixed;
  inset: 0;
  background: rgba(10, 10, 8, 0.55);
}

.dialogo-spotlight {
  position: fixed;
  display: none;
  border-radius: 4px;
  border: 2px solid #f9e2af;
  box-shadow: 0 0 0 2000px rgba(10, 10, 8, 0.72);
  transition: left 150ms ease, top 150ms ease, width 150ms ease, height 150ms ease;
}

.dialogo-tarjeta {
  position: fixed;
  left: 50%;
  top: 40%;
  transform: translate(-50%, -50%);
  width: 320px;
  max-width: calc(100vw - 2rem);
  pointer-events: auto;
  cursor: pointer;
  display: flex;
  gap: 0.75rem;
  align-items: flex-start;
  background: linear-gradient(160deg, #2a2a1f, #1c1c15);
  border: 2px solid #6b6b52;
  border-radius: 6px;
  padding: 1rem 1.2rem;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.6);
  z-index: 1;
}

.dialogo-nombre {
  color: #f9e2af;
  font-size: 0.7rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 0.25rem;
}

.dialogo-texto {
  font-size: 0.9rem;
  line-height: 1.4;
  color: #cdd6f4;
}
```

`.dialogo-tarjeta` reuses the existing `.retrato` class (already defined earlier in this file, giving the portrait its 64×64 box/border) by pairing `class="retrato"` on the portrait `<div>` inside `mostrarDialogo`'s template — no separate portrait class is needed.

- [ ] **Step 4: Self-review**

Read the full diff. Confirm: `dialogo.js` never references `window.__TAURI__`, `RETRATOS`, or any other project-specific state (it only takes a pre-built `retratoSvg` string as a parameter — it's a fully standalone module, like `audio.js`). Confirm every exported function name (`mostrarDialogo`, `ocultarDialogo`, `permitirSiempre`) is spelled identically to how Task 2 will import it. Confirm `.dialogo-bloqueo` has `pointer-events: none` and `.dialogo-tarjeta` has `pointer-events: auto` — if either is wrong, either every underlying button becomes unclickable forever, or blocking silently does nothing.

- [ ] **Step 5: Commit**

```bash
git add app/src/dialogo.js app/src/audio.js app/src/styles.css
git commit -m "Add a reusable blocking dialogue engine with spotlight cutout and blip sound"
```

---

### Task 2: The Mentor tutorial (`tutorial.js`) + wiring into the real game

**Files:**
- Create: `app/src/tutorial.js`
- Modify: `app/src/index.html` (add `#btn-saltar-tutorial`, right after `<p id="status-msg"></p>` at line 12)
- Modify: `app/src/styles.css` (append the skip-button rule)
- Modify: `app/src/main.js`:
  - `main.js:1` (import line)
  - `main.js:5-23` (add one `let` for the skip button)
  - `main.js:274-281` (`seleccionarTicket`)
  - `main.js:293` (`renderBandeja`'s ticket loop — tag the first ticket)
  - `main.js:357-362` (`iniciarPartida`)
  - `main.js:629-787` (`DOMContentLoaded`: var assignment, Play/Submit/Cerrar-scoring listeners, new `input` listener on `sqlInput`, skip-button listener, Esc handler)

**Interfaces:**
- Consumes: `mostrarDialogo`, `ocultarDialogo`, `permitirSiempre` from `app/src/dialogo.js` (Task 1).
- Produces: `app/src/tutorial.js` exports `iniciarTutorial(retratoSvg, alFinalizar)`, `tutorialActivo()`, `saltarTutorial()`, `notificarClicPrimerTicket()`, `notificarSqlCambiado(valorSql)`, `notificarClicPlay()`, `notificarClicEnviar()`, `notificarCierreScoring()` — all consumed by `main.js` in this same task.

- [ ] **Step 1: Write the tutorial beat script and state machine**

Create `app/src/tutorial.js` with this exact content:

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

function paso2LeerTicket() {
  mostrarPaso(
    "Contabilidad quiere un reporte de los pacientes de Cardiología. Cardiología es el departamento número 1 — vas a pedirle a la base de datos: de la tabla de pacientes, tráeme algunos datos, pero solo los del departamento 1.",
    { permitir: ["#ticket-activo-info"], alContinuar: paso3ClausulaSelect }
  );
}

function paso3ClausulaSelect() {
  pasoEscribirClausula(
    "Empieza diciendo qué columnas quieres ver. Escribe: SELECT nombre, fecha_ingreso, diagnostico",
    "select nombre, fecha_ingreso, diagnostico",
    paso4ClausulaFrom
  );
}

function paso4ClausulaFrom() {
  pasoEscribirClausula(
    "Ahora dile de qué tabla — cada tabla es como una hoja de cálculo, y pacientes es la hoja con un renglón por paciente. Agrega: FROM pacientes",
    "from pacientes",
    paso5ClausulaWhere
  );
}

function paso5ClausulaWhere() {
  pasoEscribirClausula(
    "Contabilidad solo quiere Cardiología, que es el departamento número 1. Agrega: WHERE departamento_id = 1",
    "where departamento_id = 1",
    paso6ClausulaOrderBy
  );
}

function paso6ClausulaOrderBy() {
  pasoEscribirClausula(
    "Y lo quieren del ingreso más reciente al más antiguo. Agrega: ORDER BY fecha_ingreso DESC",
    "order by fecha_ingreso desc",
    paso7Play
  );
}

function paso7Play() {
  clausulaObjetivoActual = null;
  pasoActualAlEscribir = null;
  mostrarPaso("Dale ▶ Play para probarlo contra la base de datos real.", { permitir: ["#btn-play"] });
}

function paso8Enviar() {
  mostrarPaso("Si el resultado se ve bien, dale ✓ Enviar ticket — así es como se resuelve cada encargo en este trabajo.", {
    permitir: ["#btn-submit"],
  });
}

function pasoCierre() {
  mostrarPaso(
    "Bien hecho — ese es tu primer ticket resuelto. El resto de tu bandeja funciona igual: lee lo que piden, escribe la query, pruébala, y envíala. Ahí te dejo.",
    { alContinuar: finalizarTutorial }
  );
}

function finalizarTutorial() {
  activo = false;
  esperandoCierreScoring = false;
  clausulaObjetivoActual = null;
  pasoActualAlEscribir = null;
  permitirSiempre([]);
  ocultarDialogo();
  if (callbackAlFinalizar) callbackAlFinalizar();
}

export function iniciarTutorial(retratoSvg, alFinalizar) {
  activo = true;
  esperandoCierreScoring = false;
  retratoMentorSvg = retratoSvg;
  callbackAlFinalizar = alFinalizar || null;
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
  if (!activo) return;
  paso2LeerTicket();
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
  if (!activo) return;
  paso8Enviar();
}

export function notificarClicEnviar() {
  if (!activo) return;
  ocultarDialogo();
  esperandoCierreScoring = true;
}

export function notificarCierreScoring() {
  if (!activo || !esperandoCierreScoring) return;
  esperandoCierreScoring = false;
  pasoCierre();
}
```

Design notes for the reviewer:
- `notificarSqlCambiado` re-invokes itself (`notificarSqlCambiado(valorSql)`) right after advancing to the next clause step. This is deliberate: if a player who already knows SQL pastes or types the entire final query in one go, the very first `input` event should walk through every remaining clause check in one synchronous pass (each match advances to the next beat's target clause and immediately re-checks the same string), landing on the "▶ Play" beat instantly instead of waiting for further keystrokes. The recursion terminates on its own once `clausulaObjetivoActual` is `null` (i.e., once `paso7Play` — which sets no new clause target — is reached).
- `notificarClicEnviar` deliberately does **not** show a new dialogue — it hides the current one and sets `esperandoCierreScoring = true`, so the real scoring overlay (unrelated to this tutorial) is the only thing on screen while the ticket is actually being scored. Only once the player closes that real scoring overlay does `notificarCierreScoring()` (called from `main.js`) show the tutorial's closing remark.
- `permitirSiempre([SELECTOR_BOTON_SALTAR])` is set once in `iniciarTutorial` and stays in effect for the rest of the tutorial (every subsequent `mostrarDialogo` call in Task 1 keeps whatever `siempreVisibles` was last set to — `mostrarDialogo` never touches it). It's reset to `[]` in `finalizarTutorial` for hygiene.

- [ ] **Step 2: Add the skip button to the HTML**

In `app/src/index.html`, right after line 12 (`<p id="status-msg"></p>`):

```html
    <button id="btn-saltar-tutorial" class="oculto">Ya sé SQL, saltar</button>
```

- [ ] **Step 3: Style the skip button**

In `app/src/styles.css`, append after the rules added in Task 1:

```css
#btn-saltar-tutorial {
  position: fixed;
  top: 0.75rem;
  left: 0.75rem;
  z-index: 9500;
}
```

- [ ] **Step 4: Tag the first rendered ticket**

In `app/src/main.js`, in `renderBandeja`'s ticket loop (line 293), change:

```js
  estadoTurno.pendientes.forEach((ticket, indice) => {
    const li = document.createElement("li");
    li.className = "papel papel-entrando papel-ticket";
    li.style.animationDelay = `${indice * 60}ms`;
```

to:

```js
  estadoTurno.pendientes.forEach((ticket, indice) => {
    const li = document.createElement("li");
    li.className = "papel papel-entrando papel-ticket";
    li.style.animationDelay = `${indice * 60}ms`;
    if (indice === 0) {
      li.dataset.primerTicket = "true";
    }
```

This `data-primer-ticket` attribute is harmless outside the tutorial — it's only ever read by `tutorial.js`'s `permitir: ["[data-primer-ticket] button"]` selector, and has no effect on styling or normal gameplay.

- [ ] **Step 5: Notify on first-ticket click**

In `app/src/main.js`, `seleccionarTicket` (lines 274-281), add the notify call as the last line:

```js
function seleccionarTicket(ticket) {
  ticketActivoId = ticket.id;
  ticketActivoInfo.textContent = `Motivo: ${ticket.motivo}\nSolicitud: ${ticket.solicitud}`;
  sqlInput.value = ticket.sql_inicial || "SELECT * FROM pacientes;";
  ticketRetrato.innerHTML = retratoParaSolicitante(ticket.solicitante);
  consolaTitulo.textContent = `query-path — ${ticket.id}`;
  mostrarPantalla("consola");
  notificarClicPrimerTicket();
}
```

`notificarClicPrimerTicket` no-ops unless the tutorial is active, so this is safe to call unconditionally on every ticket selection, tutorial or not.

- [ ] **Step 6: Start the tutorial after a brand new game**

In `app/src/main.js`, `iniciarPartida` (lines 357-362), change:

```js
async function iniciarPartida() {
  const estadoJuego = await invoke("iniciar_partida");
  pintarHubDesdeEstadoJuego(estadoJuego);
  await cargarPerks();
  setStatus("Partida nueva iniciada.", "ok");
}
```

to:

```js
async function iniciarPartida() {
  const estadoJuego = await invoke("iniciar_partida");
  pintarHubDesdeEstadoJuego(estadoJuego);
  await cargarPerks();
  setStatus("Partida nueva iniciada.", "ok");
  btnSaltarTutorial.classList.remove("oculto");
  iniciarTutorial(RETRATOS["El Mentor"], () => {
    btnSaltarTutorial.classList.add("oculto");
  });
}
```

Note `cargarPartida` (the "Load Game" path, directly below `iniciarPartida` in the same file) is intentionally **not** touched — the tutorial must never trigger there.

- [ ] **Step 7: Import, module variable, and DOMContentLoaded wiring**

At the very top of `app/src/main.js` (line 1), change:

```js
import { sfxClick, sfxTecleo, sfxCierreDia, sfxTick, sfxExito, sfxError, sfxAscenso, iniciarAmbiente, alternarMusica, alternarEfectos } from "./audio.js";
```

to:

```js
import { sfxClick, sfxTecleo, sfxCierreDia, sfxTick, sfxExito, sfxError, sfxAscenso, iniciarAmbiente, alternarMusica, alternarEfectos } from "./audio.js";
import {
  iniciarTutorial,
  tutorialActivo,
  saltarTutorial,
  notificarClicPrimerTicket,
  notificarSqlCambiado,
  notificarClicPlay,
  notificarClicEnviar,
  notificarCierreScoring,
} from "./tutorial.js";
```

Change the `let` declaration block (line 12-13) from:

```js
let btnMuteMusica, btnMuteEfectos;
let btnCerrarScoring;
```

to:

```js
let btnMuteMusica, btnMuteEfectos;
let btnCerrarScoring;
let btnSaltarTutorial;
```

In `DOMContentLoaded`, right after the existing `btnCerrarScoring = document.querySelector("#btn-cerrar-scoring");` line, add:

```js
  btnSaltarTutorial = document.querySelector("#btn-saltar-tutorial");
```

Change the Play button listener from:

```js
  document.querySelector("#btn-play").addEventListener("click", runQuery);
```

to:

```js
  document.querySelector("#btn-play").addEventListener("click", () => {
    runQuery();
    notificarClicPlay();
  });
```

Change the Submit button listener from:

```js
  document.querySelector("#btn-submit").addEventListener("click", submitTicket);
```

to:

```js
  document.querySelector("#btn-submit").addEventListener("click", () => {
    submitTicket();
    notificarClicEnviar();
  });
```

Change the `btnCerrarScoring` listener from:

```js
  btnCerrarScoring.addEventListener("click", () => {
    scoringOverlay.classList.add("oculto");
    mostrarPantalla("hub");
  });
```

to:

```js
  btnCerrarScoring.addEventListener("click", () => {
    scoringOverlay.classList.add("oculto");
    mostrarPantalla("hub");
    notificarCierreScoring();
  });
```

Right after the existing `sqlInput.addEventListener("keydown", () => sfxTecleo());` line, add:

```js
  sqlInput.addEventListener("input", () => notificarSqlCambiado(sqlInput.value));
```

Add the skip-button listener — anywhere after `btnSaltarTutorial` is assigned, e.g. right after the `btnMuteEfectos` listener block:

```js
  btnSaltarTutorial.addEventListener("click", () => {
    saltarTutorial();
    btnSaltarTutorial.classList.add("oculto");
  });
```

Finally, change the Esc handler from:

```js
  document.addEventListener("keydown", (evento) => {
    if (evento.key !== "Escape") return;
    if (appShell.classList.contains("oculto")) return;
    const hayOverlayResultado = !scoringOverlay.classList.contains("oculto");
    const hayOverlayAgencia = !agenciaOverlay.classList.contains("oculto");
    if (hayOverlayResultado || hayOverlayAgencia) return;
    pausaOverlay.classList.toggle("oculto");
  });
```

to:

```js
  document.addEventListener("keydown", (evento) => {
    if (evento.key !== "Escape") return;
    if (appShell.classList.contains("oculto")) return;
    if (tutorialActivo()) return;
    const hayOverlayResultado = !scoringOverlay.classList.contains("oculto");
    const hayOverlayAgencia = !agenciaOverlay.classList.contains("oculto");
    if (hayOverlayResultado || hayOverlayAgencia) return;
    pausaOverlay.classList.toggle("oculto");
  });
```

- [ ] **Step 8: Self-review**

Read the full diff. Confirm every function imported from `./tutorial.js` in `main.js` matches an exported name in `tutorial.js` exactly (`iniciarTutorial`, `tutorialActivo`, `saltarTutorial`, `notificarClicPrimerTicket`, `notificarSqlCambiado`, `notificarClicPlay`, `notificarClicEnviar`, `notificarCierreScoring`). Confirm `btnSaltarTutorial` is assigned in `DOMContentLoaded` *before* its listener is registered later in the same function body (check the existing pattern — all `document.querySelector` assignments happen in one block near the top of `DOMContentLoaded`, listeners after). Confirm `iniciarPartida` calls `iniciarTutorial` only after `pintarHubDesdeEstadoJuego`/`cargarPerks` have already run (so the ticket tray — and its `data-primer-ticket` tag — exists before the first tutorial beat can reference it). Confirm `cargarPartida` was not touched. Confirm the Play/Submit button listeners still call the original `runQuery`/`submitTicket` functions (not just the `notificar*` calls) — a common mistake when wrapping an existing listener in a new arrow function is dropping the original call.

- [ ] **Step 9: Commit**

```bash
git add app/src/tutorial.js app/src/index.html app/src/styles.css app/src/main.js
git commit -m "Add a Mentor-guided tutorial for a player's first ticket"
```

---

## Manual Verification (after both tasks)

Same pattern as Plans 6-14 — guided verification in the real running app:

- Start a **brand new game**: El Mentor's welcome beat appears immediately, fully blocking (clicking anywhere else does nothing), and the "Ya sé SQL, saltar" button is visible and clickable even while blocked.
- Click through the welcome beat → the bandeja beat spotlights the first ticket's "Trabajar en este" button; every other button/tab is unclickable; clicking the spotlighted button opens the console.
- The "leer el ticket" beat spotlights the ticket info panel; clicking the dialogue card advances to the SELECT beat.
- Typing `SELECT nombre, fecha_ingreso, diagnostico` (case/spacing-insensitive) auto-advances to the FROM beat, then WHERE, then ORDER BY, as each clause is typed into the real textarea — confirm the SQL editor itself is the only interactive element throughout these four beats.
- Pasting the entire final query at once (instead of typing clause by clause) jumps straight to the "▶ Play" beat in one step.
- Clicking ▶ Play advances to the "✓ Enviar ticket" beat; clicking Enviar hides the tutorial dialogue entirely while the real scoring overlay plays out normally; closing that scoring overlay brings up El Mentor's closing remark; clicking it ends the tutorial (skip button disappears, Esc now opens the pause menu again).
- Skip button: start another new game, click "Ya sé SQL, saltar" at any beat (try at least the welcome beat and one of the clause-writing beats) — the tutorial ends immediately, all blocking disappears, and the game is fully usable.
- Confirm Esc does nothing while the tutorial is active, and works normally both before it starts and after it ends/is skipped.
- Confirm **loading an existing save** never triggers the tutorial.
- Listen for the blip sound during every beat's text reveal, and confirm the existing mute-effects toggle silences it too.
