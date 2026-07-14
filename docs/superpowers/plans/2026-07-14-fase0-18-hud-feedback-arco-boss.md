# Fase 0 / Plan 18: HUD — feedback de ticket, progreso de arco y señal de boss Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** After closing a ticket, the hub clearly shows what changed (toast + badge pops + arc bar), how far the player is from the Auditor (reputation / 500 + tray counters), and that a boss fight started (banner + alert skin + tense procedural music) — without changing Rust progression rules.

**Architecture:** Frontend-only layer on top of existing `ScoreResult` and `turno_actual`/`fase`. JS keeps `ultimoFeedback` and boss-mode flags; `renderBandeja` / scoring-close refresh the arc panel and fire toast/pops; `audio.js` gains an environment/boss music mode switch using the same Web Audio bus. No new Tauri commands and no Rust changes.

**Tech Stack:** Vanilla JS (ES modules), CSS, Web Audio API — same as Plans 11–12. No new dependencies.

**Spec:** `docs/superpowers/specs/2026-07-14-fase0-18-hud-feedback-arco-boss-design.md`

## Global Constraints

- Strictly frontend: only `app/src/index.html`, `app/src/main.js`, `app/src/styles.css`, `app/src/audio.js`.
- Boss trigger stays `ascendio` / `fase === "MiniBoss"` (Plan 8). Do **not** invent a finite company ticket catalog.
- Toast/pops fire only on **final** ticket close (after scoring overlay closes). Retries with `intentos_restantes` stay status-only (Plan 17).
- Arc path UI (“Camino al Auditor”, 500 tee) only for Hospital Arcángel while Becario or during MiniBoss. Postafeta: turno counters only.
- Audio remains 100% procedural (Plan 11). Respect existing music mute.
- No frontend test runner — verify by app run + checklist at the end. Commit after each task.

## File map

| File | Responsibility |
|------|----------------|
| `app/src/index.html` | Arc panel under empresa card; toast; boss banner; pop spans inside hub badges |
| `app/src/styles.css` | Toast, pops, arc bar, `.hub-boss` skin, boss banner, mini-boss paper accent |
| `app/src/main.js` | `ultimoFeedback`, panel updates, scoring-close hooks, boss flags / sync |
| `app/src/audio.js` | `establecerModoMusica("ambiente" \| "boss")` + tense boss pattern |

---

### Task 1: Markup — panel de arco, toast, banner, pops en badges

**Files:**
- Modify: `app/src/index.html` (hub topbar badges, bandeja/empresa, body-level overlays near scoring)

- [ ] **Step 1: Add pop spans inside hub badges**

Inside `#pantalla-hub` `.hub-topbar`, change the two badge blocks so each value has a sibling pop span:

```html
          <div class="hub-badge" data-tooltip="Dinero disponible — se gana resolviendo tickets y se gasta desbloqueando perks.">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="9"/><path d="M12 7v10M9 9.5c0-1 1-1.5 3-1.5s3 1 3 2-1 1.5-3 1.5-3 .5-3 1.5 1 2 3 2 3-.5 3-1.5"/></svg>
            <span id="dinero-hub">0</span>
            <span id="dinero-hub-pop" class="hub-badge-pop oculto" aria-hidden="true"></span>
          </div>
          <div class="hub-badge" data-tooltip="Reputación — sube al resolver tickets bien y determina qué perks y rangos puedes alcanzar.">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round"><path d="M12 2l2.9 6.3 6.9.6-5.2 4.6 1.6 6.8L12 17l-6.2 3.3 1.6-6.8L2.2 8.9l6.9-.6z"/></svg>
            <span id="reputacion-hub">0</span>
            <span id="reputacion-hub-pop" class="hub-badge-pop oculto" aria-hidden="true"></span>
          </div>
```

- [ ] **Step 2: Add arc panel under the empresa card**

Immediately after `</div>` that closes `.tarjeta-empresa` (before `<h2 id="bandeja-titulo">`), insert:

```html
            <div class="panel-arco" id="panel-arco">
              <div class="panel-arco-camino" id="panel-arco-camino">
                <div class="panel-arco-label" id="panel-arco-label">Camino al Auditor</div>
                <div class="panel-arco-barra" aria-hidden="true">
                  <div class="panel-arco-fill" id="panel-arco-fill"></div>
                </div>
                <div class="panel-arco-rep" id="panel-arco-rep">0 / 500 rep</div>
              </div>
              <div class="panel-arco-turno" id="panel-arco-turno">Bandeja · 0 pendientes · presupuesto 0</div>
            </div>
```

- [ ] **Step 3: Add toast + boss banner near other overlays**

Place these as siblings of `#scoring-overlay` (same level, before or after it is fine):

```html
    <div id="ticket-toast" class="ticket-toast oculto" role="status" aria-live="polite"></div>

    <div id="boss-banner" class="boss-banner oculto" role="alert">
      <div class="boss-banner-titulo">Ascenso · El Auditor te espera</div>
      <div class="boss-banner-sub">Prepárate. Los tickets normales quedaron atrás.</div>
    </div>
```

- [ ] **Step 4: Commit**

```bash
git add app/src/index.html
git commit -m "$(cat <<'EOF'
Add hub markup for arc panel, ticket toast, and boss banner.

EOF
)"
```

---

### Task 2: Styles — toast, pops, arc bar, boss skin

**Files:**
- Modify: `app/src/styles.css` (append after existing `.hub-folder` / badge rules, or at end of file)

- [ ] **Step 1: Add CSS for pops, arc panel, toast, banner, and `.hub-boss`**

Append to `app/src/styles.css`:

```css
.hub-badge-pop {
  color: #2f6b2f;
  font-size: 0.65rem;
  font-weight: 700;
  margin-left: 0.25rem;
  animation: hub-pop-fade 1.6s ease-out forwards;
}

.hub-badge-pop.es-negativo {
  color: #8a2a2a;
}

@keyframes hub-pop-fade {
  0% { opacity: 0; transform: translateY(4px); }
  15% { opacity: 1; transform: translateY(0); }
  70% { opacity: 1; }
  100% { opacity: 0; transform: translateY(-4px); }
}

.panel-arco {
  background: #ebe0c8;
  border: 1px solid #2a2018;
  border-radius: 2px;
  padding: 0.55rem 0.7rem;
  margin-bottom: 0.75rem;
  box-shadow: 1px 2px 0 rgba(0, 0, 0, 0.15);
}

.panel-arco-label {
  font-size: 0.6rem;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  color: #5a4a35;
  margin-bottom: 0.3rem;
}

.panel-arco-barra {
  height: 10px;
  background: #d4c6a8;
  border: 1px solid #2a2018;
  border-radius: 2px;
  overflow: hidden;
}

.panel-arco-fill {
  height: 100%;
  width: 0%;
  background: #3d6b3d;
  transition: width 0.45s ease-out;
}

.panel-arco-fill.es-completo {
  background: #8a2a2a;
}

.panel-arco-rep {
  font-size: 0.62rem;
  margin-top: 0.25rem;
  font-family: "SF Mono", Menlo, Consolas, monospace;
  color: #2a2018;
}

.panel-arco-turno {
  font-size: 0.62rem;
  margin-top: 0.35rem;
  color: #5a4a35;
}

.panel-arco-camino.oculto {
  display: none;
}

.ticket-toast {
  position: fixed;
  right: 1.25rem;
  bottom: 1.25rem;
  z-index: 60;
  max-width: 260px;
  background: #1f3d1f;
  color: #f3f0e6;
  border: 2px solid #2a2018;
  border-radius: 6px;
  padding: 0.65rem 0.85rem;
  font-size: 0.75rem;
  line-height: 1.4;
  box-shadow: 3px 3px 0 rgba(0, 0, 0, 0.35);
  animation: toast-in 0.25s ease-out;
}

.ticket-toast.es-fallo {
  background: #4a1c1c;
}

@keyframes toast-in {
  from { opacity: 0; transform: translateY(8px); }
  to { opacity: 1; transform: translateY(0); }
}

.boss-banner {
  position: fixed;
  inset: 0;
  z-index: 70;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  background: rgba(20, 8, 8, 0.82);
  color: #f5d0d0;
  text-align: center;
  pointer-events: none;
  animation: boss-banner-in 0.35s ease-out;
}

.boss-banner-titulo {
  font-size: 1.35rem;
  font-weight: 700;
  letter-spacing: 0.02em;
}

.boss-banner-sub {
  font-size: 0.85rem;
  opacity: 0.9;
}

@keyframes boss-banner-in {
  from { opacity: 0; }
  to { opacity: 1; }
}

.hub-folder.hub-boss {
  box-shadow: inset 0 0 0 3px #8a2a2a, 0 0 0 1px #5a1515;
}

.hub-folder.hub-boss .tarjeta-empresa {
  border: 1px solid #8a2a2a;
  background: #f3e0d0;
}

.hub-folder.hub-boss #bandeja-titulo {
  color: #6a1a1a;
}

.hub-folder.hub-boss .papel-ticket {
  box-shadow: 2px 2px 0 rgba(138, 42, 42, 0.35);
  border-color: #8a2a2a;
}
```

- [ ] **Step 2: Smoke-check in browser/dev**

Open the hub HTML mentally / run app: elements exist, `.oculto` hides toast/banner, panel sits under empresa card without breaking grid.

- [ ] **Step 3: Commit**

```bash
git add app/src/styles.css
git commit -m "$(cat <<'EOF'
Style arc panel, ticket toast, badge pops, and boss hub skin.

EOF
)"
```

---

### Task 3: Audio — modo `ambiente` vs `boss`

**Files:**
- Modify: `app/src/audio.js`

**Interfaces:**
- Produces: `export function establecerModoMusica(modo)` where `modo` is `"ambiente"` or `"boss"`. Switching modes only changes which pattern the existing recursive scheduler plays next; does not restart `AudioContext` or ignore mute.

- [ ] **Step 1: Add boss pattern constants and mode state**

Near the top of `app/src/audio.js`, after the existing ambiente constants, add:

```js
const PATRON_BOSS_HZ = [110, 110, 130.81, 98, 110, 82.41];
const DURACION_NOTA_BOSS_MS = 420;
const FRECUENCIA_PAD_BOSS_HZ = 55;

let modoMusica = "ambiente"; // "ambiente" | "boss"
```

- [ ] **Step 2: Make the ambient scheduler mode-aware**

Replace `agendarSiguienteNotaAmbiente` so it picks pattern/pad/duration from `modoMusica`:

```js
function agendarSiguienteNotaAmbiente() {
  const patron = modoMusica === "boss" ? PATRON_BOSS_HZ : PATRON_AMBIENTE_HZ;
  const duracionNota = modoMusica === "boss" ? DURACION_NOTA_BOSS_MS : DURACION_NOTA_AMBIENTE_MS;
  const padHz = modoMusica === "boss" ? FRECUENCIA_PAD_BOSS_HZ : FRECUENCIA_PAD_HZ;

  if (indicePatronAmbiente % patron.length === 0) {
    reproducirPadAmbiente(padHz, patron.length * duracionNota);
  }
  reproducirNotaAmbiente(patron[indicePatronAmbiente % patron.length]);
  indicePatronAmbiente += 1;
  setTimeout(agendarSiguienteNotaAmbiente, duracionNota);
}
```

Also update `reproducirNotaAmbiente` callers only via the scheduler (leave the helper signature as-is). For boss mode tension, optionally use a darker oscillator in `reproducirNotaAmbiente` when `modoMusica === "boss"`:

```js
function reproducirNotaAmbiente(frecuenciaHz) {
  const ctx = obtenerContexto();
  const bus = obtenerBusAmbiente();
  const osc = ctx.createOscillator();
  const ganancia = ctx.createGain();
  osc.type = modoMusica === "boss" ? "sawtooth" : "triangle";
  osc.frequency.value = frecuenciaHz;
  const pico = modoMusica === "boss" ? 0.04 : 0.05;
  const duracionMs = modoMusica === "boss" ? DURACION_NOTA_BOSS_MS : DURACION_NOTA_AMBIENTE_MS;
  ganancia.gain.setValueAtTime(0.0001, ctx.currentTime);
  ganancia.gain.exponentialRampToValueAtTime(pico, ctx.currentTime + 0.05);
  ganancia.gain.exponentialRampToValueAtTime(0.0001, ctx.currentTime + duracionMs / 1000);
  osc.connect(ganancia);
  ganancia.connect(bus);
  osc.start();
  osc.stop(ctx.currentTime + duracionMs / 1000);
}
```

- [ ] **Step 3: Export `establecerModoMusica`**

```js
export function establecerModoMusica(modo) {
  if (modo !== "ambiente" && modo !== "boss") return;
  if (modoMusica === modo) return;
  modoMusica = modo;
  indicePatronAmbiente = 0;
}
```

- [ ] **Step 4: Commit**

```bash
git add app/src/audio.js
git commit -m "$(cat <<'EOF'
Add procedural boss music mode switch in the ambient audio loop.

EOF
)"
```

---

### Task 4: JS — panel de arco + sync de modo boss (sin toast aún)

**Files:**
- Modify: `app/src/main.js` (imports, module state, helpers, `renderBandeja`, `actualizarReputacion` / dinero helpers as needed, DOM refs in `DOMContentLoaded`)

- [ ] **Step 1: Extend import and module-level state**

Change the audio import to include `establecerModoMusica`:

```js
import {
  sfxClick,
  sfxTecleo,
  sfxCierreDia,
  sfxTick,
  sfxExito,
  sfxError,
  sfxAscenso,
  iniciarAmbiente,
  alternarMusica,
  alternarEfectos,
  establecerModoMusica,
} from "./audio.js";
```

After `let ticketActivoId = null;` add:

```js
const UMBRAL_ASCENSO_AUXILIAR = 500;

/** @type {{ titulo: string, pass: boolean, deltaDinero: number, deltaRep: number, ascendio: boolean } | null} */
let ultimoFeedback = null;
let modoBossActivo = false;
let bannerBossMostrado = false;
let reputacionActual = 0;
let rangoActual = "Becario";
let ticketActivoMotivo = "";

let panelArcoCaminoEl, panelArcoFillEl, panelArcoRepEl, panelArcoTurnoEl, panelArcoLabelEl;
let ticketToastEl, bossBannerEl;
let dineroHubPopEl, reputacionHubPopEl;
let toastTimer = null;
let bossBannerTimer = null;
```

- [ ] **Step 2: Implement panel + boss sync helpers**

Add these functions (near `renderBandeja` is fine):

```js
function actualizarPanelArco({ empresa, fase, pendientesCount, presupuesto }) {
  const mostrarCamino =
    empresa === "HospitalArcangel" &&
    (rangoActual === "Becario" || fase === "MiniBoss");

  panelArcoCaminoEl.classList.toggle("oculto", !mostrarCamino);

  if (mostrarCamino) {
    const enBoss = fase === "MiniBoss";
    const rep = Math.min(reputacionActual, UMBRAL_ASCENSO_AUXILIAR);
    const pct = enBoss
      ? 100
      : Math.max(0, Math.min(100, (rep / UMBRAL_ASCENSO_AUXILIAR) * 100));
    panelArcoFillEl.style.width = `${pct}%`;
    panelArcoFillEl.classList.toggle("es-completo", enBoss || rep >= UMBRAL_ASCENSO_AUXILIAR);
    panelArcoLabelEl.textContent = enBoss ? "Camino al Auditor — completo" : "Camino al Auditor";
    panelArcoRepEl.textContent = enBoss
      ? `${UMBRAL_ASCENSO_AUXILIAR} / ${UMBRAL_ASCENSO_AUXILIAR} rep`
      : `${rep.toFixed(1)} / ${UMBRAL_ASCENSO_AUXILIAR} rep`;
  }

  panelArcoTurnoEl.textContent = `Bandeja · ${pendientesCount} pendientes · presupuesto ${presupuesto}`;
}

function sincronizarModoBoss(fase) {
  const enBoss = fase === "MiniBoss";
  modoBossActivo = enBoss;
  pantallaHub.classList.toggle("hub-boss", enBoss);
  establecerModoMusica(enBoss ? "boss" : "ambiente");

  if (!enBoss) {
    bannerBossMostrado = false;
    bossBannerEl.classList.add("oculto");
    return;
  }

  if (!bannerBossMostrado) {
    mostrarBannerBoss();
  }
}

function mostrarBannerBoss() {
  bannerBossMostrado = true;
  bossBannerEl.classList.remove("oculto");
  if (bossBannerTimer) clearTimeout(bossBannerTimer);
  bossBannerTimer = setTimeout(() => {
    bossBannerEl.classList.add("oculto");
  }, 2800);
}
```

- [ ] **Step 3: Track reputation/rank and call panel + boss sync from `renderBandeja`**

Update `actualizarReputacion` to also keep the numeric mirror:

```js
function actualizarReputacion(valorFormateado) {
  reputacionEl.textContent = valorFormateado;
  reputacionHubEl.textContent = valorFormateado;
  reputacionActual = Number.parseFloat(valorFormateado) || 0;
}
```

At the end of `renderRango`, set `rangoActual = rango;`.

In `seleccionarTicket`, after setting `ticketActivoId`:

```js
  ticketActivoMotivo = ticket.motivo || ticket.id;
```

At the end of `renderBandeja(estadoTurno)`, before the `ArcoCompletado` agency check, call:

```js
  actualizarPanelArco({
    empresa: estadoTurno.empresa,
    fase: estadoTurno.fase,
    pendientesCount: estadoTurno.pendientes.length,
    presupuesto: estadoTurno.presupuesto_restante,
  });
  sincronizarModoBoss(estadoTurno.fase);
```

- [ ] **Step 4: Wire DOM refs in `DOMContentLoaded`**

After existing hub element queries, add:

```js
  panelArcoCaminoEl = document.querySelector("#panel-arco-camino");
  panelArcoFillEl = document.querySelector("#panel-arco-fill");
  panelArcoRepEl = document.querySelector("#panel-arco-rep");
  panelArcoTurnoEl = document.querySelector("#panel-arco-turno");
  panelArcoLabelEl = document.querySelector("#panel-arco-label");
  ticketToastEl = document.querySelector("#ticket-toast");
  bossBannerEl = document.querySelector("#boss-banner");
  dineroHubPopEl = document.querySelector("#dinero-hub-pop");
  reputacionHubPopEl = document.querySelector("#reputacion-hub-pop");
```

- [ ] **Step 5: Commit**

```bash
git add app/src/main.js
git commit -m "$(cat <<'EOF'
Wire arc progress panel and boss hub/music sync to turno fase.

EOF
)"
```

---

### Task 5: JS — `ultimoFeedback`, toast y pops al cerrar scoring

**Files:**
- Modify: `app/src/main.js` (`submitTicket`, scoring close handler)

- [ ] **Step 1: Add toast/pop helpers**

```js
function mostrarPopBadge(el, texto, esNegativo = false) {
  if (!el) return;
  el.textContent = texto;
  el.classList.toggle("es-negativo", esNegativo);
  el.classList.remove("oculto");
  el.style.animation = "none";
  void el.offsetHeight;
  el.style.animation = "";
  setTimeout(() => el.classList.add("oculto"), 1600);
}

function mostrarToastTicket(feedback) {
  if (!ticketToastEl || !feedback) return;
  const lineaResultado = feedback.pass ? "Resuelto" : "Incorrecto";
  const partes = [`${lineaResultado} · ${feedback.titulo}`];
  if (feedback.pass) {
    partes.push(`+$${feedback.deltaDinero} · +${feedback.deltaRep} rep`);
  }
  ticketToastEl.textContent = partes.join("\n");
  ticketToastEl.classList.toggle("es-fallo", !feedback.pass);
  ticketToastEl.classList.remove("oculto");
  if (toastTimer) clearTimeout(toastTimer);
  toastTimer = setTimeout(() => ticketToastEl.classList.add("oculto"), 3000);
}

function aplicarFeedbackEnHub() {
  if (!ultimoFeedback) return;
  const feedback = ultimoFeedback;
  ultimoFeedback = null;

  mostrarToastTicket(feedback);

  if (feedback.pass && feedback.deltaDinero !== 0) {
    mostrarPopBadge(dineroHubPopEl, `+$${feedback.deltaDinero}`);
  }
  if (feedback.pass && feedback.deltaRep !== 0) {
    mostrarPopBadge(reputacionHubPopEl, `+${feedback.deltaRep}`);
  }

  if (feedback.ascendio) {
    // Force banner even if fase paint races; sincronizarModoBoss will also run from renderBandeja
    bannerBossMostrado = false;
    sincronizarModoBoss("MiniBoss");
  }
}
```

- [ ] **Step 2: Record `ultimoFeedback` on final `submitTicket`**

In `submitTicket`, inside the success path after you know it is final (no `intentos_restantes`), before `mostrarScoring(score)`:

```js
    ultimoFeedback = {
      titulo: ticketActivoMotivo || ticketActivoId || "Ticket",
      pass: score.pass,
      deltaDinero: score.dinero_ganado,
      deltaRep: score.reputacion_ganada,
      ascendio: score.ascendio,
    };
```

Do **not** set `ultimoFeedback` on the retry early-return branch.

- [ ] **Step 3: Fire feedback when scoring closes**

Replace the `#btn-cerrar-scoring` click handler with:

```js
  btnCerrarScoring.addEventListener("click", () => {
    scoringOverlay.classList.add("oculto");
    mostrarPantalla("hub");
    aplicarFeedbackEnHub();
    notificarCierreScoring();
  });
```

Note: `cargarTurno()` already runs inside `submitTicket` before the player closes scoring, so the panel/bandeja should already reflect the new state when they return. `aplicarFeedbackEnHub` only handles toast/pops/banner pulse.

- [ ] **Step 4: Commit**

```bash
git add app/src/main.js
git commit -m "$(cat <<'EOF'
Show toast and badge pops when returning to the hub after scoring.

EOF
)"
```

---

### Task 6: Manual verification pass

**Files:** none (playtest only)

- [ ] **Step 1: Run the app**

From `app/`:

```bash
npm run tauri dev
```

(or the project’s usual Tauri dev command)

- [ ] **Step 2: Checklist (must all pass)**

1. Resolve a ticket OK → after closing scoring: toast + `$`/`rep` pops + arc bar moves.
2. Fail with retries left → only `#status-msg`; no toast/pops.
3. Fail final attempt → toast “Incorrecto”; no positive economy pops.
4. Reach ~500 rep / ascend → boss banner + `.hub-boss` skin + tense music.
5. Stay on hub during MiniBoss after banner fades → skin + music remain; banner does not loop every paint.
6. Finish both Auditor tickets → agency overlay; after confirm / leave MiniBoss → skin off, ambient music back.
7. Mute music during boss → silent; unmute → boss loop while still MiniBoss.
8. In Postafeta → no “Camino al Auditor” bar; no spontaneous boss skin.

- [ ] **Step 3: Commit only if playtest forced tiny fixups**

If fixes were needed, commit them with a clear message, e.g.:

```bash
git add app/src/main.js app/src/styles.css app/src/audio.js
git commit -m "$(cat <<'EOF'
Fix HUD feedback edge cases found in Plan 18 playtest.

EOF
)"
```

If nothing to fix, skip this commit.

---

## Spec coverage (self-review)

| Spec requirement | Task |
|------------------|------|
| Panel arco rep/500 + pendientes/presupuesto | Task 1, 2, 4 |
| Toast + badge pops + bar on final close | Task 1, 2, 5 |
| No celebration on retries | Task 5 (early return unchanged) |
| Boss banner once per entry | Task 1, 2, 4 (`bannerBossMostrado`) |
| Alert skin `.hub-boss` | Task 2, 4 |
| Boss procedural music + mute respect | Task 3, 4 |
| Postafeta without Auditor path | Task 4 `mostrarCamino` |
| No Rust / no catalog rewrite | Global constraints |

**Placeholder scan:** none intentional.  
**Type consistency:** `establecerModoMusica("ambiente"|"boss")`, `ultimoFeedback` shape, `fase === "MiniBoss"` match Plan 8 string serialization already used in `TITULO_FASE`.
