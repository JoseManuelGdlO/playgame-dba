# Fase 0 / Plan 19: Consola y scoring — presencia Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the SQL console and post-ticket scoring feel alive (live window states, reactive portrait, tiered scoring ritual + toast) without adding tickets or changing Rust rules.

**Architecture:** Frontend-only. Pure JS helpers classify score tiers and drive CSS class state on `#pantalla-consola .ventana-terminal` and `#scoring-overlay`. Rework `mostrarScoring` order (metrics → skin/title → rewards → mentor/ascenso) with skip; extend `ultimoFeedback.tier` for toast copy/classes. No Tauri/Rust changes.

**Tech Stack:** Vanilla JS (ES modules), CSS — same as Plans 11–18. No new dependencies.

**Spec:** `docs/superpowers/specs/2026-07-15-consola-scoring-presencia-design.md`

## Global Constraints

- Strictly frontend: `app/src/index.html`, `app/src/main.js`, `app/src/styles.css` only (optional tiny reuse of existing `sfxExito` / `sfxError` / `sfxTick` from `audio.js` — no new tracks).
- Do **not** add tickets, change validation, economy, ranks, or Tauri commands.
- Retries with `intentos_restantes` stay status-only (no scoring overlay).
- `UMBRAL_SCORE_EXCELENTE = 85` and speed shortcut `>= 95` live as JS constants (not CSS).
- `consola-alerta` when `intentos_restantes ≤ 1` **or** `presupuesto_restante ≤ 20`.
- No automated UI test runner — verify pure helpers with `node -e`; verify UX with manual checklist. Commit after each task.

## File map

| File | Responsibility |
|------|----------------|
| `app/src/index.html` | Minimal hooks: result “running” node; scoring metric/reward row classes |
| `app/src/styles.css` | `consola-*` states, result stagger/shake, scoring skins, toast tiers, retrato reactions |
| `app/src/main.js` | `clasificarTierScore`, console state API, rewrite `mostrarScoring` + skip, toast tier |

---

### Task 1: Markup hooks (resultado running + filas de scoring)

**Files:**
- Modify: `app/src/index.html` (`#pantalla-consola` output section; `#scoring-overlay` panel)

**Interfaces:**
- Consumes: existing IDs (`#result-table`, scoring spans)
- Produces: `#result-running`; classes `scoring-fila scoring-metrica` / `scoring-recompensa` on reward/metric `<p>`s; keep existing span IDs

- [ ] **Step 1: Add running indicator above the result table**

In `#pantalla-consola`, inside `<section class="output">`, change to:

```html
            <section class="output">
              <h2>Resultado</h2>
              <p id="result-running" class="result-running oculto" aria-live="polite">Ejecutando…</p>
              <div id="result-table"></div>
            </section>
```

- [ ] **Step 2: Tag scoring metric vs reward rows**

In `#scoring-overlay` `.scoring-panel`, keep IDs; add classes:

```html
          <h2 id="scoring-titulo">Resultado</h2>
          <p class="scoring-fila scoring-metrica">Correctitud: <span id="scoring-correctitud">0</span></p>
          <p class="scoring-fila scoring-metrica">Velocidad: <span id="scoring-velocidad">0</span></p>
          <p class="scoring-fila scoring-metrica">Buenas prácticas: <span id="scoring-practicas">0</span></p>
          <p class="scoring-fila scoring-recompensa">💰 +<span id="scoring-dinero">0</span> <span id="scoring-dinero-nota" class="scoring-nota"></span></p>
          <p class="scoring-fila scoring-recompensa">⭐ +<span id="scoring-reputacion">0</span></p>
          <p id="scoring-mentor" class="scoring-beat"></p>
          <p id="scoring-ascenso" class="scoring-beat"></p>
          <button id="btn-cerrar-scoring">Cerrar</button>
```

- [ ] **Step 3: Commit**

```bash
git add app/src/index.html
git commit -m "$(cat <<'EOF'
Add markup hooks for console running state and scoring row roles.

EOF
)"
```

---

### Task 2: CSS — consola viva + skins de scoring + toast tiers

**Files:**
- Modify: `app/src/styles.css` (near `.ventana-terminal` / `.scoring-overlay` / `.ticket-toast`)

**Interfaces:**
- Consumes: classes applied by Task 3–5 JS
- Produces: visual rules for `consola-*`, `retrato-reaccion-*`, `scoring-*` skins, toast tiers

- [ ] **Step 1: Consola state styles**

Append (or place near `#pantalla-consola` rules):

```css
#pantalla-consola .ventana-terminal {
  transition: box-shadow 160ms ease, border-color 160ms ease;
}

#pantalla-consola .ventana-terminal.consola-querying {
  box-shadow: 0 0 0 2px #c9a227;
}

#pantalla-consola .ventana-terminal.consola-ok {
  box-shadow: 0 0 0 2px #5a8f5a;
}

#pantalla-consola .ventana-terminal.consola-error {
  box-shadow: 0 0 0 2px #a33a3a;
}

#pantalla-consola .ventana-terminal.consola-alerta {
  box-shadow: 0 0 0 2px #c9782a;
}

#pantalla-consola .ventana-terminal.consola-ok.consola-alerta {
  box-shadow: 0 0 0 2px #c9782a, 0 0 0 4px rgba(90, 143, 90, 0.45);
}

#pantalla-consola .ventana-terminal.consola-error.consola-alerta {
  box-shadow: 0 0 0 2px #c9782a, 0 0 0 4px rgba(163, 58, 58, 0.45);
}

.result-running {
  margin: 0 0 0.5rem;
  font-size: 0.85rem;
  color: #c9a227;
  font-style: italic;
}

#result-table.es-flash {
  animation: result-flash 280ms ease;
}

@keyframes result-flash {
  from { filter: brightness(1.35); }
  to { filter: brightness(1); }
}

#result-table .resultado-bloque.es-stagger {
  animation: resultado-entrada 280ms ease both;
}

@keyframes resultado-entrada {
  from { opacity: 0; transform: translateY(6px); }
  to { opacity: 1; transform: translateY(0); }
}

#result-table.es-shake-error {
  animation: resultado-shake 320ms ease;
}

#result-table .resultado-error {
  color: #ffb4b4;
  font-weight: 600;
  background: rgba(120, 30, 30, 0.35);
  padding: 0.4rem 0.55rem;
  border-left: 3px solid #c44;
}

@keyframes resultado-shake {
  0%, 100% { transform: translateX(0); }
  25% { transform: translateX(-4px); }
  75% { transform: translateX(4px); }
}

#ticket-retrato.retrato-reaccion-ok {
  animation: pulso-exito 420ms ease;
}

#ticket-retrato.retrato-reaccion-error {
  animation: resultado-shake 320ms ease;
}
```

- [ ] **Step 2: Scoring skins + reward pop + toast tiers**

```css
#scoring-overlay .ventana-terminal.scoring-excelente {
  box-shadow: 0 0 0 2px #c9a227, 0 8px 28px rgba(201, 162, 39, 0.35);
}

#scoring-overlay .ventana-terminal.scoring-pass {
  box-shadow: 0 0 0 2px #5a8f5a;
}

#scoring-overlay .ventana-terminal.scoring-fail {
  box-shadow: 0 0 0 2px #8a4040;
  filter: saturate(0.75);
}

#scoring-overlay .scoring-metrica.es-debil {
  color: #ffb4b4;
  font-weight: 600;
}

.scoring-fila.scoring-recompensa.es-pop-reward {
  animation: reward-pop 380ms ease;
}

@keyframes reward-pop {
  0% { transform: scale(0.92); opacity: 0.4; }
  60% { transform: scale(1.06); opacity: 1; }
  100% { transform: scale(1); }
}

.ticket-toast.es-excelente {
  background: #3d3210;
  border-color: #c9a227;
}

.ticket-toast.es-pass {
  background: #1f3d1f;
}
```

(Keep existing `.ticket-toast.es-fallo` — do not duplicate; only add the new tier classes above.)

- [ ] **Step 3: Commit**

```bash
git add app/src/styles.css
git commit -m "$(cat <<'EOF'
Add CSS for live console states and tiered scoring feedback.

EOF
)"
```

---

### Task 3: JS — `clasificarTierScore` + toast tipado

**Files:**
- Modify: `app/src/main.js` (constants near other umbrales; `mostrarToastTicket`; `ultimoFeedback` in `submitTicket`)

**Interfaces:**
- Consumes: `score` fields `pass`, `puntaje_correctitud`, `puntaje_velocidad`, `puntaje_practicas`
- Produces:
  - `UMBRAL_SCORE_EXCELENTE = 85`
  - `UMBRAL_VELOCIDAD_EXCELENTE = 95`
  - `clasificarTierScore(score) → "excelente" | "pass" | "fail"`
  - `metricaMasDebil(score) → "correctitud" | "practicas" | "velocidad"`
  - `ultimoFeedback.tier` set on final close
  - toast classes `es-excelente` / `es-pass` / `es-fallo`

- [ ] **Step 1: Add helpers after existing umbral constants**

Find `UMBRAL_ASCENSO_AUXILIAR` (or nearby constants) and add:

```js
const UMBRAL_SCORE_EXCELENTE = 85;
const UMBRAL_VELOCIDAD_EXCELENTE = 95;
const PRESUPUESTO_ALERTA = 20;

/** @param {{ pass: boolean, puntaje_correctitud: number, puntaje_velocidad: number, puntaje_practicas: number }} score */
function clasificarTierScore(score) {
  if (!score.pass) return "fail";
  const promedio =
    (Number(score.puntaje_correctitud) +
      Number(score.puntaje_velocidad) +
      Number(score.puntaje_practicas)) /
    3;
  if (promedio >= UMBRAL_SCORE_EXCELENTE || Number(score.puntaje_velocidad) >= UMBRAL_VELOCIDAD_EXCELENTE) {
    return "excelente";
  }
  return "pass";
}

/** @param {{ puntaje_correctitud: number, puntaje_velocidad: number, puntaje_practicas: number }} score */
function metricaMasDebil(score) {
  const pares = [
    ["correctitud", Number(score.puntaje_correctitud)],
    ["practicas", Number(score.puntaje_practicas)],
    ["velocidad", Number(score.puntaje_velocidad)],
  ];
  pares.sort((a, b) => a[1] - b[1]);
  return pares[0][0];
}
```

- [ ] **Step 2: Verify helpers with node**

Run from repo root:

```bash
node -e "
const UMBRAL_SCORE_EXCELENTE=85, UMBRAL_VELOCIDAD_EXCELENTE=95;
function clasificarTierScore(score){
  if(!score.pass) return 'fail';
  const p=(score.puntaje_correctitud+score.puntaje_velocidad+score.puntaje_practicas)/3;
  if(p>=UMBRAL_SCORE_EXCELENTE||score.puntaje_velocidad>=UMBRAL_VELOCIDAD_EXCELENTE) return 'excelente';
  return 'pass';
}
const cases=[
  [{pass:false,puntaje_correctitud:0,puntaje_velocidad:100,puntaje_practicas:100},'fail'],
  [{pass:true,puntaje_correctitud:100,puntaje_velocidad:100,puntaje_practicas:100},'excelente'],
  [{pass:true,puntaje_correctitud:70,puntaje_velocidad:96,puntaje_practicas:70},'excelente'],
  [{pass:true,puntaje_correctitud:80,puntaje_velocidad:80,puntaje_practicas:80},'pass'],
];
for (const [s,e] of cases) {
  const g=clasificarTierScore(s);
  if(g!==e) { console.error('FAIL',s,g,e); process.exit(1); }
}
console.log('ok clasificarTierScore');
"
```

Expected: `ok clasificarTierScore`

- [ ] **Step 3: Wire `ultimoFeedback.tier` in `submitTicket`**

Where `ultimoFeedback = { ... }` is built after a final close, add:

```js
    const tier = clasificarTierScore(score);
    ultimoFeedback = {
      titulo: ticketActivoMotivo || ticketActivoId || "Ticket",
      pass: score.pass,
      tier,
      deltaDinero: score.dinero_ganado,
      deltaRep: score.reputacion_ganada,
      ascendio: score.ascendio,
    };
```

- [ ] **Step 4: Update `mostrarToastTicket` for tiers**

Replace the function body so copy/classes follow `feedback.tier`:

```js
function mostrarToastTicket(feedback) {
  if (!ticketToastEl || !feedback) return;
  const tier = feedback.tier || (feedback.pass ? "pass" : "fail");
  const lineaResultado =
    tier === "excelente" ? "Query limpia" : tier === "pass" ? "Resuelto" : "Incorrecto";
  const partes = [`${lineaResultado} · ${feedback.titulo}`];
  if (feedback.pass) {
    const repTxt = Number(feedback.deltaRep).toFixed(1);
    const dineroTxt =
      feedback.deltaDinero > 0
        ? `+$${feedback.deltaDinero} (al cerrar el día)`
        : `+$0`;
    partes.push(`${dineroTxt} · +${repTxt} rep`);
  }
  ticketToastEl.textContent = partes.join("\n");
  ticketToastEl.classList.remove("es-excelente", "es-pass", "es-fallo");
  ticketToastEl.classList.add(
    tier === "excelente" ? "es-excelente" : tier === "pass" ? "es-pass" : "es-fallo"
  );
  ticketToastEl.classList.remove("oculto");
  if (toastTimer) clearTimeout(toastTimer);
  toastTimer = setTimeout(() => ticketToastEl.classList.add("oculto"), 3000);
}
```

- [ ] **Step 5: Commit**

```bash
git add app/src/main.js
git commit -m "$(cat <<'EOF'
Classify score tiers and surface them on the ticket toast.

EOF
)"
```

---

### Task 4: JS — estados vivos de la consola

**Files:**
- Modify: `app/src/main.js` (`runQuery`, `runAllQueries`, `renderResultados`, `seleccionarTicket`, `submitTicket`, `#btn-volver-hub`, `renderBandeja`)

**Interfaces:**
- Consumes: `intentosRestantesPorTicket`, `PRESUPUESTO_ALERTA`, `#result-running`, `#ticket-retrato`, `#consola-titulo`
- Produces:
  - `let presupuestoRestanteActual = 100;`
  - `ventanaConsolaEl` cached query for `#pantalla-consola .ventana-terminal`
  - `setEstadoConsola(modo)` where `modo` is `"idle" | "querying" | "ok" | "error"`
  - `refrescarAlertaConsola()`
  - `reaccionRetrato(kind)` where `kind` is `"ok" | "error"`
  - Clears console classes when returning to hub

- [ ] **Step 1: Add state helpers (near `renderResultados`)**

```js
let presupuestoRestanteActual = 100;
let ventanaConsolaEl = null;
let resultRunningEl = null;
let consolaModoActual = "idle"; // idle | querying | ok | error

function obtenerVentanaConsola() {
  if (!ventanaConsolaEl) {
    ventanaConsolaEl = document.querySelector("#pantalla-consola .ventana-terminal");
  }
  return ventanaConsolaEl;
}

function refrescarAlertaConsola() {
  const ventana = obtenerVentanaConsola();
  if (!ventana) return;
  const intentos =
    ticketActivoId != null
      ? intentosRestantesPorTicket[ticketActivoId] ?? intentosLimite
      : intentosLimite;
  const enAlerta = intentos <= 1 || presupuestoRestanteActual <= PRESUPUESTO_ALERTA;
  ventana.classList.toggle("consola-alerta", enAlerta);
}

function setEstadoConsola(modo) {
  const ventana = obtenerVentanaConsola();
  if (!ventana) return;
  consolaModoActual = modo;
  ventana.classList.remove("consola-idle", "consola-querying", "consola-ok", "consola-error");
  ventana.classList.add(`consola-${modo}`);
  refrescarAlertaConsola();

  if (!consolaTitulo) return;
  const base = ticketActivoId ? `query-path — ${ticketActivoId}` : "query-path";
  if (modo === "querying") consolaTitulo.textContent = `${base} · ejecutando…`;
  else if (modo === "ok") consolaTitulo.textContent = `${base} · ok`;
  else if (modo === "error") consolaTitulo.textContent = `${base} · error`;
  else consolaTitulo.textContent = base;
}

function limpiarEstadoConsola() {
  const ventana = obtenerVentanaConsola();
  if (ventana) {
    ventana.classList.remove(
      "consola-idle",
      "consola-querying",
      "consola-ok",
      "consola-error",
      "consola-alerta"
    );
  }
  consolaModoActual = "idle";
  if (resultRunningEl) resultRunningEl.classList.add("oculto");
  if (resultTable) {
    resultTable.classList.remove("es-flash", "es-shake-error");
  }
}

function mostrarRunningResultado(visible) {
  if (!resultRunningEl) resultRunningEl = document.querySelector("#result-running");
  if (!resultRunningEl) return;
  resultRunningEl.classList.toggle("oculto", !visible);
}

function reaccionRetrato(kind) {
  if (!ticketRetrato) return;
  ticketRetrato.classList.remove("retrato-reaccion-ok", "retrato-reaccion-error");
  void ticketRetrato.offsetHeight;
  ticketRetrato.classList.add(
    kind === "ok" ? "retrato-reaccion-ok" : "retrato-reaccion-error"
  );
}
```

- [ ] **Step 2: Upgrade `renderResultados` for stagger / error shake**

Replace `renderResultados` with:

```js
function renderResultados(resultados) {
  resultTable.innerHTML = "";
  resultTable.classList.remove("es-shake-error");
  const mostrarEtiquetas = resultados.length > 1;
  let huboError = false;
  resultados.forEach((resultado, indice) => {
    const bloque = document.createElement("div");
    bloque.className = "resultado-bloque es-stagger";
    bloque.style.animationDelay = `${indice * 55}ms`;
    if (mostrarEtiquetas) {
      const etiqueta = document.createElement("h3");
      etiqueta.className = "resultado-etiqueta";
      etiqueta.textContent = `Resultado ${indice + 1}`;
      bloque.appendChild(etiqueta);
    }
    if (resultado.error) {
      huboError = true;
      const error = document.createElement("p");
      error.className = "resultado-error";
      error.textContent = resultado.error;
      bloque.appendChild(error);
    } else {
      bloque.appendChild(crearTablaFilas(resultado.rows));
    }
    resultTable.appendChild(bloque);
  });
  if (huboError) {
    resultTable.classList.add("es-shake-error");
    setEstadoConsola("error");
    reaccionRetrato("error");
  } else {
    resultTable.classList.remove("es-flash");
    void resultTable.offsetHeight;
    resultTable.classList.add("es-flash");
    setEstadoConsola("ok");
    reaccionRetrato("ok");
  }
}
```

- [ ] **Step 3: Wrap `runQuery` / `runAllQueries` with querying + running**

In `runQuery`, after confirm, before invoke:

```js
  setEstadoConsola("querying");
  mostrarRunningResultado(true);
```

In a `finally` block, always:

```js
  mostrarRunningResultado(false);
```

On catch path (when clearing `resultTable`), also:

```js
    setEstadoConsola("error");
    reaccionRetrato("error");
    resultTable.classList.add("es-shake-error");
```

Mirror the same querying/running bookends in `runAllQueries` (running on at start, off after `renderResultados`).

- [ ] **Step 4: Init on ticket select; clear on hub; track presupuesto; submit querying**

In `seleccionarTicket`, after `mostrarPantalla("consola")`:

```js
  setEstadoConsola("idle");
  if (resultTable) resultTable.innerHTML = "";
  mostrarRunningResultado(false);
```

In `renderBandeja`, after reading presupuesto:

```js
  presupuestoRestanteActual = Number(estadoTurno.presupuesto_restante) || 0;
  refrescarAlertaConsola();
```

In `submitTicket`, at start of try (after disabling button):

```js
    setEstadoConsola("querying");
```

On retry path (`intentos_restantes`), call `refrescarAlertaConsola()` after `actualizarEtiquetaIntentos`.

In `#btn-volver-hub` handler, after clearing ticket:

```js
    limpiarEstadoConsola();
```

In DOM boot (where other els are queried), cache:

```js
  resultRunningEl = document.querySelector("#result-running");
  ventanaConsolaEl = document.querySelector("#pantalla-consola .ventana-terminal");
```

- [ ] **Step 5: Manual smoke (consola)**

Run the app (`npm run tauri dev` or project usual), open a Becario ticket:

1. Play with valid SELECT → green ok ring, title `· ok`, portrait pulse, staggered rows.
2. Play with bad SQL → red ring, `· error`, shake, stronger error text.
3. Volver → classes cleared.
4. (Optional) With last attempt or presupuesto ≤ 20 → orange alerta accent remains with ok/error.

- [ ] **Step 6: Commit**

```bash
git add app/src/main.js
git commit -m "$(cat <<'EOF'
Add live console window states for preview runs and pressure alerts.

EOF
)"
```

---

### Task 5: JS — ritual de `mostrarScoring` + skip

**Files:**
- Modify: `app/src/main.js` (`mostrarScoring`, scoring close button / overlay click)

**Interfaces:**
- Consumes: `clasificarTierScore`, `metricaMasDebil`, Task 1 row classes, Task 2 skin classes
- Produces: rewritten async ritual; `scoringSkipRequested` flag; skip on `#btn-cerrar-scoring` click during sequence **or** click on `#scoring-overlay` backdrop (not the inner `.ventana-terminal`)

- [ ] **Step 1: Replace `mostrarScoring` with tiered ritual**

```js
let scoringSkipRequested = false;
let scoringAnimando = false;

function scoringVentanaEl() {
  return document.querySelector("#scoring-overlay .ventana-terminal");
}

function aplicarSkinScoring(tier) {
  const ventana = scoringVentanaEl();
  if (!ventana) return;
  ventana.classList.remove("scoring-excelente", "scoring-pass", "scoring-fail");
  ventana.classList.add(
    tier === "excelente" ? "scoring-excelente" : tier === "pass" ? "scoring-pass" : "scoring-fail"
  );
}

function tituloPorTier(tier) {
  if (tier === "excelente") return "✨ Query limpia";
  if (tier === "pass") return "✅ Resuelto";
  return "❌ Incorrecto";
}

async function esperarScoring(ms) {
  const paso = 50;
  let restante = ms;
  while (restante > 0) {
    if (scoringSkipRequested) return;
    await esperar(Math.min(paso, restante));
    restante -= paso;
  }
}

async function mostrarScoring(score) {
  const tituloEl = document.querySelector("#scoring-titulo");
  const mentorEl = document.querySelector("#scoring-mentor");
  const tier = clasificarTierScore(score);

  scoringSkipRequested = false;
  scoringAnimando = true;
  btnCerrarScoring.disabled = false; // allow skip via Cerrar
  tituloEl.textContent = "";
  tituloEl.className = "";
  mentorEl.textContent = "";
  scoringAscenso.textContent = "";
  mentorEl.classList.add("linea-oculta");
  scoringAscenso.classList.add("linea-oculta");

  document.querySelectorAll("#scoring-overlay .scoring-metrica").forEach((p) => {
    p.classList.remove("es-debil");
  });

  const notaDineroEl = document.querySelector("#scoring-dinero-nota");
  if (notaDineroEl) {
    notaDineroEl.textContent =
      score.pass && score.dinero_ganado > 0 ? "(se paga al cerrar el día)" : "";
  }

  const metricas = [
    { span: document.querySelector("#scoring-correctitud"), valor: score.puntaje_correctitud, decimales: 0, clave: "correctitud" },
    { span: document.querySelector("#scoring-velocidad"), valor: score.puntaje_velocidad, decimales: 0, clave: "velocidad" },
    { span: document.querySelector("#scoring-practicas"), valor: score.puntaje_practicas, decimales: 0, clave: "practicas" },
  ].map((linea) => ({ ...linea, fila: linea.span.closest("p") }));

  const recompensas = [
    { span: document.querySelector("#scoring-dinero"), valor: score.dinero_ganado, decimales: 0 },
    { span: document.querySelector("#scoring-reputacion"), valor: score.reputacion_ganada, decimales: 1 },
  ].map((linea) => ({ ...linea, fila: linea.span.closest("p") }));

  for (const linea of [...metricas, ...recompensas]) {
    linea.fila.classList.add("linea-oculta");
    linea.fila.classList.remove("es-pop-reward");
    linea.span.textContent = (0).toFixed(linea.decimales);
  }

  aplicarSkinScoring(tier);
  scoringOverlay.classList.remove("oculto");

  const revelarLinea = async (linea, { reward = false } = {}) => {
    if (scoringSkipRequested) {
      linea.fila.classList.remove("linea-oculta");
      linea.span.textContent = Number(linea.valor).toFixed(linea.decimales);
      return;
    }
    linea.fila.classList.remove("linea-oculta");
    if (reward) {
      linea.fila.classList.add("es-pop-reward");
    }
    animarNumero(linea.span, linea.valor, linea.decimales);
    sfxTick();
    await esperarScoring(DURACION_LINEA_SCORING_MS);
  };

  for (const linea of metricas) {
    await revelarLinea(linea);
  }

  if (!scoringSkipRequested) {
    tituloEl.textContent = tituloPorTier(tier);
    tituloEl.className = score.pass ? "pulso" : "shake";
    if (tier === "fail") {
      const debil = metricaMasDebil(score);
      const filaDebil = metricas.find((m) => m.clave === debil)?.fila;
      if (filaDebil) filaDebil.classList.add("es-debil");
      sfxError();
    } else {
      sfxExito();
    }
    await esperarScoring(200);
  }

  for (const linea of recompensas) {
    await revelarLinea(linea, { reward: true });
  }

  if (scoringSkipRequested) {
    for (const linea of [...metricas, ...recompensas]) {
      linea.fila.classList.remove("linea-oculta");
      linea.span.textContent = Number(linea.valor).toFixed(linea.decimales);
    }
    tituloEl.textContent = tituloPorTier(tier);
    tituloEl.className = score.pass ? "pulso" : "shake";
    if (tier === "fail") {
      const debil = metricaMasDebil(score);
      const filaDebil = metricas.find((m) => m.clave === debil)?.fila;
      if (filaDebil) filaDebil.classList.add("es-debil");
    }
  }

  mentorEl.textContent = score.comentario_mentor || "";
  mentorEl.classList.remove("linea-oculta");
  if (score.ascendio) {
    scoringAscenso.textContent = `¡Ascendiste a ${NOMBRE_RANGO[score.rango_actual] || score.rango_actual}! +1 slot de perk. Nuevos tickets disponibles.`;
    scoringAscenso.classList.remove("linea-oculta");
    sfxAscenso();
  } else {
    scoringAscenso.classList.add("linea-oculta");
  }

  scoringAnimando = false;
  scoringSkipRequested = false;
  btnCerrarScoring.disabled = false;
}
```

- [ ] **Step 2: Wire skip on Cerrar + overlay backdrop**

Change the `#btn-cerrar-scoring` listener to:

```js
  btnCerrarScoring.addEventListener("click", () => {
    if (scoringAnimando) {
      scoringSkipRequested = true;
      return;
    }
    scoringOverlay.classList.add("oculto");
    mostrarPantalla("hub");
    aplicarFeedbackEnHub();
    notificarCierreScoring();
    considerarSubtramaEmpleo();
  });

  scoringOverlay.addEventListener("click", (evento) => {
    if (evento.target !== scoringOverlay) return;
    if (scoringAnimando) {
      scoringSkipRequested = true;
    }
  });
```

- [ ] **Step 3: Manual smoke (scoring)**

1. Perfect / near-perfect clear → `scoring-excelente`, title “Query limpia”, rewards after metrics, toast `es-excelente`.
2. Mediocre pass → `scoring-pass` / “Resuelto”.
3. Fail final → cold skin, weak metric highlighted, toast fail.
4. Mid-cascade click Cerrar → jumps to filled final; second click closes to hub.
5. Ascenso still shows beat + SFX.
6. Retry with `intentos_restantes` never opens overlay.

- [ ] **Step 4: Commit**

```bash
git add app/src/main.js
git commit -m "$(cat <<'EOF'
Rewrite scoring overlay ritual with tiers and skippable cascade.

EOF
)"
```

---

### Task 6: Checklist final de playtest

**Files:** none (verification only)

- [ ] **Step 1: Run full checklist from spec**

- [ ] Preview ok / error in Hospital Becario
- [ ] Submit pass excelente, pass normal, fail final
- [ ] Retry with `intentos_restantes` (no celebration overlay)
- [ ] Skip mid-cascade
- [ ] Toast + pops after closing scoring
- [ ] Ascenso (`ascendio`) still shows final beat
- [ ] Console alert with low attempts and/or presupuesto ≤ 20
- [ ] Return to hub clears console states
- [ ] Music mute / MiniBoss / pause still work

- [ ] **Step 2: Fix any gaps found; commit only if code changed**

```bash
git status
# if dirty:
git add app/src/main.js app/src/styles.css app/src/index.html
git commit -m "$(cat <<'EOF'
Polish console/scoring presence after playtest checklist.

EOF
)"
```

---

## Spec coverage (self-review)

| Spec item | Task |
|-----------|------|
| Console states idle/querying/ok/error + alerta | 2, 4 |
| Title bar reflects state | 4 |
| Play presence (running, stagger, error shake) | 1, 2, 4 |
| Retrato reaction | 2, 4 |
| Submit querying; hub clears state | 4 |
| Scoring cascade metrics → skin/title → rewards → mentor/ascenso | 5 |
| Three skins + copy | 2, 5 |
| Fail emphasizes weakest metric | 5 |
| Skip during cascade | 5 |
| Toast by tier | 2, 3 |
| JS umbral constants | 3 |
| No Rust / no new tickets | Global constraints |
| Manual checklist | 6 |

## Placeholder / consistency check

- Function names stable across tasks: `clasificarTierScore`, `setEstadoConsola`, `mostrarScoring`, `ultimoFeedback.tier`.
- CSS class names match JS: `consola-*`, `scoring-excelente|pass|fail`, toast `es-excelente|es-pass|es-fallo`.
- No TBD/TODO left in steps.
