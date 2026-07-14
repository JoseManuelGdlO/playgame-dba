# Vestido Visual (escritorio, terminal, retratos) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Vestir las 3 pantallas que el Plan 9 ya construyó (Menú/Hub/Consola) y los overlays de scoring/Agencia con la capa visual que faltaba: pantalla de título limpia para el Menú, escritorio apagado + tickets/perks como papel + ventana de terminal + retrato del solicitante para Hub/Consola, y la misma ventana de terminal para los overlays.

**Architecture:** Cambios puramente de frontend (CSS/HTML/JS), sin tocar ningún comando de Tauri ni Rust. Perks pasa de un `<select>` a una lista de fichas de papel individuales con botón contextual por ítem (mismo patrón que ya usa la bandeja de tickets). 3 retratos placeholder (SVG inline) seleccionados por `ticket.solicitante`, campo que ya viaja al frontend sin cambios de backend.

**Tech Stack:** vanilla JS/HTML/CSS — sin dependencias nuevas.

## Global Constraints

- Sin cambios en `app/src-tauri/` — este plan es 100% frontend.
- Perks cambia de interacción (dropdown → lista), no solo de estilo — decisión explícita del brainstorming, no un descuido.
- 3 retratos exactos: genérico (default), "El Mentor", "Auditor de Cumplimiento" — mapeados por el string literal de `ticket.solicitante`.
- El Menú NO lleva el fondo de escritorio (pantalla de título limpia); Hub y Consola sí.
- Spec de referencia: `docs/superpowers/specs/2026-07-13-fase0-10-vestido-visual-design.md`.

---

### Task 1: Menú — pantalla de título limpia

**Files:**
- Modify: `app/src/index.html`
- Modify: `app/src/styles.css`

**Interfaces:**
- Produces: `#pantalla-menu` con su propio layout de página completa (sin depender de `.container`); regla genérica `button:disabled`.

- [ ] **Step 1: Quitar `class="container"` de `#pantalla-menu` en `index.html`**

Localizar:

```html
    <div id="pantalla-menu" class="container">
```

Reemplazar:

```html
    <div id="pantalla-menu">
```

- [ ] **Step 2: Agregar los estilos del Menú y `button:disabled` en `styles.css`**

Localizar:

```css
button:active {
  background-color: #45475a;
}
```

Reemplazar:

```css
button:active {
  background-color: #45475a;
}

button:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

#pantalla-menu {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  padding: 2rem;
  background: radial-gradient(ellipse at center, #1e1e2e, #11111b);
}

#pantalla-menu h1 {
  font-family: "SF Mono", Menlo, Consolas, monospace;
  font-size: 2rem;
  letter-spacing: 0.15em;
  color: #a6e3a1;
}

#pantalla-menu .subtitle {
  display: block;
  margin-top: 0.5rem;
  font-size: 0.85rem;
}

#pantalla-menu .actions {
  flex-direction: column;
  align-items: center;
  margin-top: 2rem;
}

#pantalla-menu .actions button {
  width: 220px;
}
```

- [ ] **Step 3: Verificación**

Sin runner de tests de frontend en este proyecto. Revisar que `index.html`/`styles.css` sigan siendo HTML/CSS válidos (sin llaves o etiquetas sin cerrar) leyendo el diff con cuidado. La confirmación visual real queda para la verificación manual guiada al final del plan (Task 3).

- [ ] **Step 4: Commit**

```bash
git add app/src/index.html app/src/styles.css
git commit -m "Style the Menu screen as a clean title screen"
```

---

### Task 2: Hub y Consola — escritorio, papel, ventana de terminal, retratos, Perks como lista

**Files:**
- Modify: `app/src/index.html`
- Modify: `app/src/styles.css`
- Modify: `app/src/main.js`

**Interfaces:**
- Produces: clases `.escritorio`, `.ventana-terminal` (+ `-barra`, `-punto`, `-titulo`, `-cuerpo`), `.papel`, `.papel-perk`, `.retrato`; función `retratoParaSolicitante(solicitante)`; `renderPerks` reescrito sobre una lista (`#lista-perks`) en vez de un `<select>`; nueva `accionPerk(perk)` que reemplaza `desbloquearPerkSeleccionado`/`equiparODesequiparPerkSeleccionado`.

- [ ] **Step 1: Clases de escritorio, ventana de terminal, papel y retrato en `styles.css`**

Localizar:

```css
.bandeja li {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  background: #313244;
  border-radius: 8px;
  padding: 0.6rem 0.8rem;
}
```

Reemplazar:

```css
.escritorio {
  background: linear-gradient(160deg, #2a2a1f, #1c1c15);
  min-height: 100vh;
}

.ventana-terminal {
  background: #181825;
  border-radius: 10px;
  overflow: hidden;
  margin: 1.5rem 0;
}

.ventana-terminal-barra {
  background: #11111b;
  padding: 0.5rem 0.75rem;
  display: flex;
  align-items: center;
  gap: 0.4rem;
}

.ventana-terminal-punto {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  display: inline-block;
}

.ventana-terminal-punto.rojo {
  background: #f38ba8;
}

.ventana-terminal-punto.amarillo {
  background: #f9e2af;
}

.ventana-terminal-punto.verde {
  background: #a6e3a1;
}

.ventana-terminal-titulo {
  color: #6c7086;
  font-size: 0.8rem;
  margin-left: 0.4rem;
  font-family: "SF Mono", Menlo, Consolas, monospace;
}

.ventana-terminal-cuerpo {
  padding: 1.25rem;
}

.papel {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  background: #e8dcc0;
  color: #3a3a2e;
  border-radius: 2px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
  padding: 0.6rem 0.8rem;
}

.papel:nth-child(odd) {
  transform: rotate(-0.4deg);
}

.papel:nth-child(even) {
  transform: rotate(0.3deg);
}

.papel button {
  background: #3a3a2e;
  color: #e8dcc0;
  border: none;
  border-radius: 2px;
  font-size: 0.8rem;
  padding: 0.3rem 0.7rem;
}

.papel-perk {
  border-left: 4px solid #585b70;
}

.papel-perk.desbloqueado {
  border-left-color: #89b4fa;
}

.papel-perk.equipado {
  border-left-color: #a6e3a1;
}

.retrato {
  width: 64px;
  height: 64px;
  background: #3a3a2e;
  border: 3px solid #6b6b52;
  border-radius: 4px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

.retrato svg {
  width: 48px;
  height: 48px;
}

.ticket-info-con-retrato {
  display: flex;
  gap: 0.75rem;
  align-items: flex-start;
  margin-bottom: 0.75rem;
}

#lista-perks {
  list-style: none;
  padding: 0;
  margin: 0 0 0.75rem;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}
```

- [ ] **Step 2: `#app-shell` gana la clase `escritorio`, en `index.html`**

Localizar:

```html
    <div id="app-shell" class="oculto">
```

Reemplazar:

```html
    <div id="app-shell" class="oculto escritorio">
```

- [ ] **Step 3: Perks — de `<select>` a lista, en `index.html`**

Localizar:

```html
        <section class="perks">
          <h2>Perks</h2>
          <select id="perks-select"></select>
          <div class="actions">
            <button id="btn-unlock-perk">Desbloquear</button>
            <button id="btn-equip-perk">Equipar/Desequipar</button>
          </div>
          <p id="perks-equipados-msg"></p>
        </section>
```

Reemplazar:

```html
        <section class="perks">
          <h2>Perks</h2>
          <ul id="lista-perks"></ul>
          <p id="perks-equipados-msg"></p>
        </section>
```

- [ ] **Step 4: Consola — ventana de terminal + retrato, en `index.html`**

Localizar:

```html
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
          <div id="result-table"></div>
        </section>
      </main>
```

Reemplazar:

```html
      <main class="container oculto" id="pantalla-consola">
        <button id="btn-volver-hub">‹ Volver</button>

        <div class="ventana-terminal">
          <div class="ventana-terminal-barra">
            <span class="ventana-terminal-punto rojo"></span>
            <span class="ventana-terminal-punto amarillo"></span>
            <span class="ventana-terminal-punto verde"></span>
            <span class="ventana-terminal-titulo" id="consola-titulo">query-path</span>
          </div>
          <div class="ventana-terminal-cuerpo">
            <section class="console">
              <div class="ticket-info-con-retrato">
                <div class="retrato" id="ticket-retrato"></div>
                <p id="ticket-activo-info">Elige un ticket de la bandeja para empezar.</p>
              </div>
              <textarea id="sql-input" spellcheck="false" placeholder="SELECT * FROM pacientes;"></textarea>
              <div class="actions">
                <button id="btn-play">▶ Play</button>
                <button id="btn-submit">✓ Enviar ticket</button>
              </div>
            </section>

            <section class="output">
              <h2>Resultado</h2>
              <div id="result-table"></div>
            </section>
          </div>
        </div>
      </main>
```

- [ ] **Step 5: Retratos, variables nuevas, y `seleccionarTicket` en `main.js`**

Localizar:

```js
let sqlInput, statusMsg, resultTable, dineroEl, reputacionEl, rangoEl;
let perksSelect, perksEquipadosMsg;
let presupuestoEl, listaTickets, ticketActivoInfo, bandejaTitulo;
let scoringOverlay, scoringAscenso, agenciaOverlay;
let pantallaMenu, appShell, pantallaHub, pantallaConsola, btnCargarPartida;
```

Reemplazar:

```js
let sqlInput, statusMsg, resultTable, dineroEl, reputacionEl, rangoEl;
let listaPerks, perksEquipadosMsg;
let presupuestoEl, listaTickets, ticketActivoInfo, bandejaTitulo;
let scoringOverlay, scoringAscenso, agenciaOverlay;
let pantallaMenu, appShell, pantallaHub, pantallaConsola, btnCargarPartida;
let ticketRetrato, consolaTitulo;

const RETRATOS = {
  generico: `<svg viewBox="0 0 8 8"><rect width="8" height="8" fill="#4a4a3a"/><rect x="2" y="1" width="4" height="3" fill="#8a8266"/><rect x="1" y="4" width="6" height="3" fill="#6b6b52"/><rect x="3" y="2" width="1" height="1" fill="#2a2a1f"/><rect x="5" y="2" width="1" height="1" fill="#2a2a1f"/></svg>`,
  "El Mentor": `<svg viewBox="0 0 8 8"><rect width="8" height="8" fill="#3a3a2e"/><rect x="2" y="1" width="4" height="3" fill="#9a8a7a"/><rect x="1" y="4" width="6" height="3" fill="#7a6a5a"/><rect x="2" y="2" width="4" height="1" fill="#1c1c15"/></svg>`,
  "Auditor de Cumplimiento": `<svg viewBox="0 0 8 8"><rect width="8" height="8" fill="#2a2a35"/><rect x="2" y="1" width="4" height="3" fill="#7a7a8a"/><rect x="1" y="4" width="6" height="3" fill="#5a5a6a"/><rect x="3" y="4" width="2" height="3" fill="#1c1c22"/></svg>`,
};

function retratoParaSolicitante(solicitante) {
  return RETRATOS[solicitante] || RETRATOS.generico;
}
```

Localizar:

```js
function seleccionarTicket(ticket) {
  ticketActivoId = ticket.id;
  ticketActivoInfo.textContent = `Motivo: ${ticket.motivo}\nSolicitud: ${ticket.solicitud}`;
  sqlInput.value = ticket.sql_inicial || "SELECT * FROM pacientes;";
  mostrarPantalla("consola");
}
```

Reemplazar:

```js
function seleccionarTicket(ticket) {
  ticketActivoId = ticket.id;
  ticketActivoInfo.textContent = `Motivo: ${ticket.motivo}\nSolicitud: ${ticket.solicitud}`;
  sqlInput.value = ticket.sql_inicial || "SELECT * FROM pacientes;";
  ticketRetrato.innerHTML = retratoParaSolicitante(ticket.solicitante);
  consolaTitulo.textContent = `query-path — ${ticket.id}`;
  mostrarPantalla("consola");
}
```

- [ ] **Step 6: `.papel` en cada ticket de la bandeja, en `main.js`**

Localizar:

```js
  for (const ticket of estadoTurno.pendientes) {
    const li = document.createElement("li");
    const info = document.createElement("span");
    info.textContent = `[⏱️ ${ticket.costo_tiempo}] ${ticket.motivo}`;
```

Reemplazar:

```js
  for (const ticket of estadoTurno.pendientes) {
    const li = document.createElement("li");
    li.className = "papel";
    const info = document.createElement("span");
    info.textContent = `[⏱️ ${ticket.costo_tiempo}] ${ticket.motivo}`;
```

- [ ] **Step 7: Reescribir `renderPerks` y reemplazar los 2 manejadores por `accionPerk`, en `main.js`**

Localizar:

```js
function renderPerks(perks) {
  const seleccionado = perksSelect.value;
  perksSelect.innerHTML = "";
  for (const perk of perks) {
    const opt = document.createElement("option");
    opt.value = perk.id;
    const estado = perk.equipado ? "⭐ equipado" : perk.desbloqueado ? "✅ desbloqueado" : "🔒 bloqueado";
    opt.textContent = `${perk.nombre} (${perk.categoria}) — ${estado} — $${perk.costo_dinero}, ⭐${perk.reputacion_minima}`;
    perksSelect.appendChild(opt);
  }
  if (seleccionado) perksSelect.value = seleccionado;

  const equipados = perks.filter((p) => p.equipado).map((p) => p.nombre);
  perksEquipadosMsg.textContent = equipados.length ? `Equipados: ${equipados.join(", ")}` : "Ningún perk equipado.";
}

async function cargarPerks() {
  const perks = await invoke("catalogo_perks");
  renderPerks(perks);
}

async function desbloquearPerkSeleccionado() {
  const id = perksSelect.value;
  if (!id) return;
  try {
    const perks = await invoke("desbloquear_perk", { id });
    renderPerks(perks);
    setStatus("Perk desbloqueado.", "ok");
  } catch (err) {
    setStatus(String(err), "error");
  }
}

async function equiparODesequiparPerkSeleccionado() {
  const id = perksSelect.value;
  if (!id) return;
  const actual = (await invoke("catalogo_perks")).find((p) => p.id === id);
  try {
    const perks = actual && actual.equipado
      ? await invoke("desequipar_perk", { id })
      : await invoke("equipar_perk", { id });
    renderPerks(perks);
  } catch (err) {
    setStatus(String(err), "error");
  }
}
```

Reemplazar:

```js
function renderPerks(perks) {
  listaPerks.innerHTML = "";
  for (const perk of perks) {
    const li = document.createElement("li");
    li.className = `papel papel-perk ${perk.equipado ? "equipado" : perk.desbloqueado ? "desbloqueado" : ""}`.trim();

    const info = document.createElement("span");
    const estado = perk.equipado ? "⭐ equipado" : perk.desbloqueado ? "✅ desbloqueado" : "🔒 bloqueado";
    info.textContent = `${perk.nombre} (${perk.categoria}) — ${estado} — $${perk.costo_dinero}, ⭐${perk.reputacion_minima}`;

    const boton = document.createElement("button");
    boton.textContent = perk.equipado ? "Desequipar" : perk.desbloqueado ? "Equipar" : "Desbloquear";
    boton.addEventListener("click", () => accionPerk(perk));

    li.appendChild(info);
    li.appendChild(boton);
    listaPerks.appendChild(li);
  }

  const equipados = perks.filter((p) => p.equipado).map((p) => p.nombre);
  perksEquipadosMsg.textContent = equipados.length ? `Equipados: ${equipados.join(", ")}` : "Ningún perk equipado.";
}

async function cargarPerks() {
  const perks = await invoke("catalogo_perks");
  renderPerks(perks);
}

async function accionPerk(perk) {
  try {
    let perks;
    if (perk.equipado) {
      perks = await invoke("desequipar_perk", { id: perk.id });
    } else if (perk.desbloqueado) {
      perks = await invoke("equipar_perk", { id: perk.id });
    } else {
      perks = await invoke("desbloquear_perk", { id: perk.id });
      setStatus("Perk desbloqueado.", "ok");
    }
    renderPerks(perks);
  } catch (err) {
    setStatus(String(err), "error");
  }
}
```

- [ ] **Step 8: Actualizar `DOMContentLoaded` — nuevas refs, quitar los listeners de los botones que ya no existen**

Localizar:

```js
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
```

Reemplazar:

```js
  listaPerks = document.querySelector("#lista-perks");
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
  ticketRetrato = document.querySelector("#ticket-retrato");
  consolaTitulo = document.querySelector("#consola-titulo");
```

Localizar:

```js
  document.querySelector("#btn-unlock-perk").addEventListener("click", desbloquearPerkSeleccionado);
  document.querySelector("#btn-equip-perk").addEventListener("click", equiparODesequiparPerkSeleccionado);
  document.querySelector("#btn-confirmar-agencia").addEventListener("click", confirmarTransicionAgencia);
```

Reemplazar:

```js
  document.querySelector("#btn-confirmar-agencia").addEventListener("click", confirmarTransicionAgencia);
```

- [ ] **Step 9: Verificación**

Sin runner de tests de frontend. Revisar el diff con cuidado: cada `document.querySelector` nuevo debe resolver contra un id real del HTML de este mismo Step; `perksSelect`/`desbloquearPerkSeleccionado`/`equiparODesequiparPerkSeleccionado` no deben quedar ninguna referencia (`grep` para confirmar cero apariciones). La confirmación visual real queda para la verificación manual guiada al final del plan (Task 3).

- [ ] **Step 10: Commit**

```bash
git add app/src/index.html app/src/styles.css app/src/main.js
git commit -m "Dress the Hub and Console screens: desk background, paper tickets/perks, terminal window, portraits"
```

---

### Task 3: Overlays de scoring y Agencia — ventana de terminal sobre el escritorio

**Files:**
- Modify: `app/src/index.html`
- Modify: `app/src/styles.css`

**Interfaces:**
- Consumes: `.ventana-terminal` (+ variantes) de la Tarea 2.
- Produces: `.scoring-overlay` con fondo de escritorio; `.scoring-panel` sin su propio marco (lo hereda de `.ventana-terminal`).

- [ ] **Step 1: Fondo de escritorio para los overlays, y `.scoring-panel` sin su propio marco, en `styles.css`**

Localizar:

```css
.scoring-overlay {
  position: fixed;
  inset: 0;
  background: rgba(17, 17, 27, 0.85);
  display: flex;
  align-items: center;
  justify-content: center;
}

.oculto {
  display: none;
}

.scoring-panel {
  background: #1e1e2e;
  border: 1px solid #45475a;
  border-radius: 12px;
  padding: 1.5rem 2rem;
  min-width: 280px;
  text-align: center;
}
```

Reemplazar:

```css
.scoring-overlay {
  position: fixed;
  inset: 0;
  background: linear-gradient(160deg, rgba(42, 42, 31, 0.92), rgba(28, 28, 21, 0.92));
  display: flex;
  align-items: center;
  justify-content: center;
}

.oculto {
  display: none;
}

.scoring-panel {
  min-width: 280px;
  text-align: center;
}
```

- [ ] **Step 2: Envolver el panel de scoring en la ventana de terminal, en `index.html`**

Localizar:

```html
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
```

Reemplazar:

```html
    <div id="scoring-overlay" class="scoring-overlay oculto">
      <div class="ventana-terminal">
        <div class="ventana-terminal-barra">
          <span class="ventana-terminal-punto rojo"></span>
          <span class="ventana-terminal-punto amarillo"></span>
          <span class="ventana-terminal-punto verde"></span>
          <span class="ventana-terminal-titulo">query-path — resultado</span>
        </div>
        <div class="ventana-terminal-cuerpo scoring-panel">
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
    </div>
```

- [ ] **Step 3: Envolver el panel de la Agencia en la ventana de terminal, en `index.html`**

Localizar:

```html
    <div id="agencia-overlay" class="scoring-overlay oculto">
      <div class="scoring-panel">
        <h2>Grupo Ómega RH — Reasignación</h2>
        <p>Has superado al Auditor de Cumplimiento. Tu siguiente asignación:</p>
        <p><strong>Postafeta</strong> — todo el Slack de la empresa lo administra un becario invisible llamado Kevin; todo viene firmado "- Kevin".</p>
        <button id="btn-confirmar-agencia">Aceptar reasignación</button>
      </div>
    </div>
```

Reemplazar:

```html
    <div id="agencia-overlay" class="scoring-overlay oculto">
      <div class="ventana-terminal">
        <div class="ventana-terminal-barra">
          <span class="ventana-terminal-punto rojo"></span>
          <span class="ventana-terminal-punto amarillo"></span>
          <span class="ventana-terminal-punto verde"></span>
          <span class="ventana-terminal-titulo">query-path — agencia</span>
        </div>
        <div class="ventana-terminal-cuerpo scoring-panel">
          <h2>Grupo Ómega RH — Reasignación</h2>
          <p>Has superado al Auditor de Cumplimiento. Tu siguiente asignación:</p>
          <p><strong>Postafeta</strong> — todo el Slack de la empresa lo administra un becario invisible llamado Kevin; todo viene firmado "- Kevin".</p>
          <button id="btn-confirmar-agencia">Aceptar reasignación</button>
        </div>
      </div>
    </div>
```

- [ ] **Step 4: Verificación manual guiada en la app real**

A diferencia de planes anteriores, esta vez sí se puede confirmar visualmente en este entorno (`screencapture` ya se usó con éxito para verificar los Planes 6-8). Lanzar la app (`npm run tauri dev` desde `app/`) y confirmar con capturas de pantalla:
- Menú: pantalla de título limpia, sin escritorio, "Cargar partida" deshabilitado si no hay guardado.
- Hub: fondo de escritorio, tickets y perks como fichas de papel con botón contextual por perk.
- Consola: ventana de terminal con semáforo, retrato del solicitante junto al motivo/solicitud (probar con un ticket de "El Mentor" o del Auditor de Cumplimiento si el turno actual lo permite, y con uno genérico).
- Scoring y overlay de Agencia: ventana de terminal sobre el fondo de escritorio.

- [ ] **Step 5: Commit**

```bash
git add app/src/index.html app/src/styles.css
git commit -m "Dress the scoring and Agencia overlays as terminal windows over the desk background"
```

---

## Self-Review Notes

- **Cobertura del spec:** Menú como pantalla de título limpia ✓ (Tarea 1), escritorio + papel + ventana de terminal + retratos + Perks como lista en Hub/Consola ✓ (Tarea 2), overlays vestidos igual ✓ (Tarea 3).
- **Orden de tareas pensado para nunca dejar la app rota a mitad de plan:** cada tarea landea CSS+HTML+JS juntos cuando son interdependientes (Tarea 2 no se separó en "CSS luego HTML luego JS" porque eso habría dejado, por ejemplo, `renderPerks` referenciando un `<select>` ya eliminado entre commits).
- **Placeholders:** ninguno — los 3 retratos son SVG completos, no marcadores de posición vacíos; toda regla CSS y bloque HTML/JS está completo.
- **Consistencia de tipos/nombres:** `.papel`/`.papel-perk`/`.retrato`/`.ventana-terminal*` se definen una sola vez (Tarea 2, Step 1) y se reutilizan sin renombrar en las Tareas 2 y 3. `retratoParaSolicitante` usa las mismas llaves de texto (`"El Mentor"`, `"Auditor de Cumplimiento"`) que ya existen como `solicitante` en los tickets reales (`tickets/hospital_arcangel.rs`).
- **Alcance:** 3 tareas, estrictamente frontend, sin tocar comandos de Tauri ni navegación/guardado (eso ya lo cerró el Plan 9). La restructuración de Perks (dropdown → lista) fue una decisión explícita de alcance confirmada con el usuario antes de escribir este plan, no un descubrimiento a mitad de implementación.

## Execution Handoff

Plan completo y guardado en `docs/superpowers/plans/2026-07-13-fase0-10-vestido-visual.md`. Dos opciones de ejecución:

1. **Subagent-Driven (recomendado)** — despacho un subagente fresco por tarea, reviso el resultado entre cada una antes de seguir
2. **Ejecución inline** — ejecuto las tareas en esta sesión con executing-plans, ejecución por lotes con checkpoints

¿Cuál prefieres?
