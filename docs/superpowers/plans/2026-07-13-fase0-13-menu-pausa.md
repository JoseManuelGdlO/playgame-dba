# Fase 0 / Plan 13: Menú de Pausa (ESC) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an ESC-triggered pause overlay (Guardar / Opciones / Salir) that consolidates a save confirmation, the existing music/effects mute toggles (moved out of their floating corner position), and a return-to-Menu action.

**Architecture:** A new `#pausa-overlay` reuses the existing `.scoring-overlay`/`.ventana-terminal` styling (same visual family as the result/agencia overlays). A single `keydown` listener toggles it on `Escape`, gated on being inside `#app-shell` (Hub or Consola) and no other overlay being open. The mute buttons keep their exact ids and JS wiring — only their HTML position changes.

**Tech Stack:** Vanilla JS (ES modules), no new dependencies, no backend changes.

## Global Constraints

- Strictly frontend: only `app/src/index.html`, `app/src/main.js`, `app/src/styles.css` change. No Rust, no new Tauri commands — this plan reuses `mostrarPantalla`, `setStatus`, `alternarMusica`, `alternarEfectos`, `mostrarMenu`, all already implemented.
- ESC only opens the pause overlay when `#app-shell` is visible (Hub or Consola active) AND neither `#scoring-overlay` nor `#agencia-overlay` is visible. ESC again (or clicking "Continuar") closes it.
- "Guardar" is a confirmation message only (`setStatus("Partida guardada.", "ok")`) — no new save command. The game already autosaves after every meaningful action.
- "Salir" returns to the Menú (via the existing `mostrarMenu()` function, which also re-checks whether a save exists to correctly enable/disable "Cargar partida") — it must NOT close the application process.
- The mute buttons (`#btn-mute-musica`/`#btn-mute-efectos`) keep their exact ids — only their HTML location and CSS positioning change. Their existing click listeners in `DOMContentLoaded` need zero changes.

---

### Task 1: Pause overlay — markup, styling, and wiring

**Files:**
- Modify: `app/src/index.html:12-15` (remove the floating mute buttons), `app/src/index.html` (add `#pausa-overlay` near the end, alongside the existing `#scoring-overlay`/`#agencia-overlay`)
- Modify: `app/src/styles.css` (`.control-audio` loses its floating/fixed positioning; new `.pausa-opciones`/`.pausa-opciones-audio` rules)
- Modify: `app/src/main.js:9` (new `pausaOverlay` module var), `main.js` (`DOMContentLoaded`: query it, wire its 3 buttons, add the `keydown` Escape listener)

**Interfaces:**
- Consumes: `mostrarMenu()`, `setStatus(text, kind)`, `alternarMusica()`/`alternarEfectos()` (already imported from `./audio.js`), `scoringOverlay`/`agenciaOverlay`/`appShell` module vars — all already defined earlier in `main.js`, unchanged.
- Produces: nothing consumed by later work — this is the only task in the plan.

- [ ] **Step 1: Remove the floating mute buttons and add the pause overlay markup**

In `app/src/index.html`, remove these two lines (currently right after `<p id="status-msg"></p>`):

```html
    <button id="btn-mute-musica" class="control-audio" title="Silenciar música">🔊</button>
    <button id="btn-mute-efectos" class="control-audio" title="Silenciar efectos">🔊</button>
```

so that block becomes just:

```html
    <p id="status-msg"></p>
    <div id="tooltip-global" class="tooltip-global oculto"></div>
```

Then, right after the closing `</div>` of `#agencia-overlay` (the last overlay in the file, immediately before `</body>`), add:

```html
    <div id="pausa-overlay" class="scoring-overlay oculto">
      <div class="ventana-terminal">
        <div class="ventana-terminal-barra">
          <span class="ventana-terminal-punto rojo"></span>
          <span class="ventana-terminal-punto amarillo"></span>
          <span class="ventana-terminal-punto verde"></span>
          <span class="ventana-terminal-titulo">query-path — pausa</span>
        </div>
        <div class="ventana-terminal-cuerpo scoring-panel">
          <h2>Pausa</h2>
          <button id="btn-guardar-pausa">Guardar</button>

          <div class="pausa-opciones">
            <p class="pausa-opciones-titulo">Opciones</p>
            <div class="pausa-opciones-audio">
              <button id="btn-mute-musica" class="control-audio" title="Silenciar música">🔊</button>
              <button id="btn-mute-efectos" class="control-audio" title="Silenciar efectos">🔊</button>
            </div>
          </div>

          <button id="btn-salir-pausa">Salir al Menú</button>
          <button id="btn-continuar-pausa">Continuar</button>
        </div>
      </div>
    </div>
```

The mute buttons keep their exact ids (`#btn-mute-musica`/`#btn-mute-efectos`) — they are being relocated, not recreated, so the existing `DOMContentLoaded` code that queries and wires them needs no changes.

- [ ] **Step 2: Update `.control-audio` — it no longer floats**

In `app/src/styles.css`, find this existing rule:

```css
.control-audio {
  position: fixed;
  top: 0.75rem;
  z-index: 10;
  padding: 0.3rem 0.5rem;
  font-size: 1rem;
  line-height: 1;
}

#btn-mute-musica {
  right: 0.75rem;
}

#btn-mute-efectos {
  right: 3.25rem;
}
```

Replace it with:

```css
.control-audio {
  padding: 0.3rem 0.5rem;
  font-size: 1rem;
  line-height: 1;
}
```

The `#btn-mute-musica`/`#btn-mute-efectos` corner-offset rules are removed entirely — their layout is now handled by `.pausa-opciones-audio` (Step 3), not by fixed positioning.

- [ ] **Step 3: Add the pause-menu CSS**

In `app/src/styles.css`, append at the end of the file:

```css
.pausa-opciones {
  margin: 1rem 0;
}

.pausa-opciones-titulo {
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: #9399b2;
  margin: 0 0 0.5rem;
}

.pausa-opciones-audio {
  display: flex;
  gap: 0.5rem;
  justify-content: center;
}

#pausa-overlay .scoring-panel button {
  display: block;
  width: 100%;
  margin-top: 0.6rem;
}
```

- [ ] **Step 4: Add the `pausaOverlay` module variable**

In `app/src/main.js`, change line 9 from:

```js
let pantallaMenu, appShell, pantallaHub, pantallaConsola, btnCargarPartida;
```

to:

```js
let pantallaMenu, appShell, pantallaHub, pantallaConsola, btnCargarPartida;
let pausaOverlay;
```

- [ ] **Step 5: Wire the pause overlay in `DOMContentLoaded`**

In `app/src/main.js`'s `DOMContentLoaded` handler, add this line right after the existing `agenciaOverlay = document.querySelector("#agencia-overlay");` assignment:

```js
  pausaOverlay = document.querySelector("#pausa-overlay");
```

Then, after the existing `document.querySelector("#tab-logros").addEventListener(...)` block, add:

```js
  document.querySelector("#btn-guardar-pausa").addEventListener("click", () => {
    setStatus("Partida guardada.", "ok");
  });

  document.querySelector("#btn-salir-pausa").addEventListener("click", async () => {
    pausaOverlay.classList.add("oculto");
    await mostrarMenu();
  });

  document.querySelector("#btn-continuar-pausa").addEventListener("click", () => {
    pausaOverlay.classList.add("oculto");
  });

  document.addEventListener("keydown", (evento) => {
    if (evento.key !== "Escape") return;
    if (appShell.classList.contains("oculto")) return;
    const hayOverlayResultado = !scoringOverlay.classList.contains("oculto");
    const hayOverlayAgencia = !agenciaOverlay.classList.contains("oculto");
    if (hayOverlayResultado || hayOverlayAgencia) return;
    pausaOverlay.classList.toggle("oculto");
  });
```

The `keydown` guard order matters: `appShell.classList.contains("oculto")` is checked first (ESC does nothing from the Menú screen, since `#app-shell` is hidden there), then the two overlay checks (ESC does nothing if the result or agencia overlay is already open) — only if both pass does it toggle `pausaOverlay`, which both opens it (removes `oculto`) and closes it again on a second ESC press (adds `oculto` back), since `classList.toggle` flips the class each call.

- [ ] **Step 6: Self-review**

Read the diff. Confirm the mute buttons appear exactly once in the final `index.html` (inside `#pausa-overlay`, not also left in their old floating position). Confirm `#btn-mute-musica`/`#btn-mute-efectos`'s existing `addEventListener` wiring in `DOMContentLoaded` (already present before this task, unchanged) still finds these elements correctly via `document.querySelector` regardless of their new location in the DOM — `querySelector` matches by id anywhere in the document, so relocating the elements requires no change to that existing code. Confirm the `keydown` listener references `scoringOverlay`/`agenciaOverlay`/`appShell`, which are all assigned earlier in the same `DOMContentLoaded` handler (before this new code runs), not `pausaOverlay` itself in the guard conditions. Confirm `mostrarMenu()` is called (not raw `mostrarPantalla("menu")`) for "Salir", since `mostrarMenu()` additionally re-checks `existe_partida_guardada` to keep "Cargar partida"'s disabled state correct.

- [ ] **Step 7: Commit**

```bash
git add app/src/index.html app/src/styles.css app/src/main.js
git commit -m "Add an ESC pause menu: Guardar confirmation, Opciones (audio), Salir to Menú"
```

---

## Manual Verification (after the task)

Same pattern as prior plans — guided verification in the real running app via `screencapture`, purely visual/interactive (no audio behavior changes, though the mute buttons' own toggling should still work from their new location). Cover:
- No floating mute buttons visible anywhere outside the pause menu.
- ESC from the Hub opens the pause overlay; ESC again closes it; "Continuar" also closes it.
- ESC from the Consola (with or without an active ticket) also opens it.
- ESC while the Menú screen is showing does nothing.
- ESC while the scoring/agencia overlay is open does nothing (no pause overlay stacking on top).
- "Guardar" shows the "Partida guardada." status message.
- "Salir al Menú" returns to the Menú screen; "Cargar partida" is enabled/disabled correctly there.
- The mute buttons inside the pause menu still toggle music/effects correctly.
