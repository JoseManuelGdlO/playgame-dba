# Fase 0 / Plan 17: Sistema de Intentos + Perk "Segunda Opinión" Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let a player retry a ticket up to 3 times (5 with the new "Segunda Opinión" perk) before it's permanently lost — a failed attempt with retries left costs no time and pays nothing, and the ticket stays open in the console; only a correct submission or exhausting all attempts closes the ticket for real.

**Architecture:** `EstadoTurno` (`turno/mod.rs`) gains a private, unserialized attempt counter per ticket id plus a `reintentar` method that refunds a ticket's time cost and reinserts it into `pendientes` — the exact inverse of the existing `resolver` (which stays untouched). `perks/mod.rs` gains a new `Efecto::BonoIntentos(u32)` variant and a new perk; `economia/mod.rs` gains an `EstadoJugador::intentos_extra` method mirroring the existing `multiplicador_dinero`/`multiplicador_reputacion` pattern. `lib.rs`'s `resolver_ticket` command keeps its existing atomic "remove-then-validate" anti-double-submit guarantee unchanged, and adds one new early-return branch: an incorrect submission with retries left calls `reintentar` and returns immediately with `intentos_restantes: Some(n)`, never touching the economy at all. `app/src/main.js`'s `submitTicket()` treats any truthy `intentos_restantes` as "not final" — short status message only, no scoring overlay, no `notificarClicEnviar()` (so the Plan 15/16 tutorial's wait-for-scoring-close state is never armed by a non-final submission).

**Tech Stack:** Rust/sqlx/Tauri (Tasks 1-2, most of Task 3), vanilla JS (the frontend half of Task 3) — same stack as every prior Fase 0 plan, no new dependencies.

## Global Constraints

- 3 attempts base per ticket (`INTENTOS_BASE = 3`), 5 with "Segunda Opinión" equipped (+2).
- A failed attempt with retries remaining costs no time and pays no money/reputation/XP — only a final result (correct, or attempts exhausted) consumes the ticket's `costo_tiempo` and runs the economy calculation, exactly as today.
- Succeeding on attempt 2 or 3 pays the full reward for that submission's scores — no penalty for needing more than one try.
- The existing atomic "remove ticket, then validate" anti-double-submit guarantee in `resolver_ticket` (a prior code-review finding) must not be weakened — `EstadoTurno::resolver` itself is not modified.
- The new perk ("Segunda Opinión") uses fun, plain-language copy — no SQL jargon in its name or description, matching every other perk in the catalog.
- No frontend test runner exists in this project — correctness for `main.js` comes from careful diff self-review plus manual verification. Rust changes use `cargo test` (TDD) via the full path `"$HOME/.cargo/bin/cargo.exe"` (cargo is not on PATH in this environment).

---

### Task 1: Attempt tracking + `reintentar` on `EstadoTurno`

**Files:**
- Modify: `app/src-tauri/src/turno/mod.rs:1-22` (imports + struct field), `mod.rs:64-72` (new methods after `resolver`)
- Modify: `app/src-tauri/src/lib.rs:582-591` (the `cargar_partida` reconstruction site — needs the new field initialized)

**Interfaces:**
- Produces (consumed by Task 3): `EstadoTurno::registrar_intento(&mut self, id: &str) -> u32` (increments and returns the new attempt count for that ticket id, starting at 1 on first call), `EstadoTurno::limpiar_intentos(&mut self, id: &str)` (removes the counter entry — call when a ticket is finally resolved, win or lose), `EstadoTurno::reintentar(&mut self, ticket: tickets::Ticket)` (refunds `ticket.costo_tiempo` into `presupuesto_restante` and pushes `ticket` back into `pendientes`).

- [ ] **Step 1: Write the failing tests for the 3 new methods**

In `app/src-tauri/src/turno/mod.rs`, in the existing `#[cfg(test)] mod tests` block, right after the existing `resolver_devuelve_none_si_el_id_no_esta_pendiente` test (currently ends around line 179), add:

```rust
    #[test]
    fn reintentar_reembolsa_el_tiempo_y_reinserta_el_ticket() {
        let catalogo = catalogo_de_prueba();
        let (mut turno, _) = EstadoTurno::nuevo(&catalogo, 0);

        let resuelto = turno.resolver("t2").expect("t2 debe estar pendiente");
        assert_eq!(turno.presupuesto_restante, 70, "resolver ya descontó el costo de t2 (30)");

        turno.reintentar(resuelto);

        assert_eq!(turno.presupuesto_restante, 100, "reintentar debe reembolsar el costo de tiempo");
        assert_eq!(
            turno.pendientes.iter().map(|t| t.id).collect::<Vec<_>>(),
            vec!["t1", "t3", "t2"],
            "el ticket reintentado se reinserta al final de pendientes"
        );
    }

    #[test]
    fn registrar_intento_incrementa_y_devuelve_el_conteo_por_ticket() {
        let catalogo = catalogo_de_prueba();
        let (mut turno, _) = EstadoTurno::nuevo(&catalogo, 0);

        assert_eq!(turno.registrar_intento("t1"), 1);
        assert_eq!(turno.registrar_intento("t1"), 2);
        assert_eq!(turno.registrar_intento("t2"), 1, "el conteo es independiente por ticket");
    }

    #[test]
    fn limpiar_intentos_borra_el_conteo_de_ese_ticket() {
        let catalogo = catalogo_de_prueba();
        let (mut turno, _) = EstadoTurno::nuevo(&catalogo, 0);

        turno.registrar_intento("t1");
        turno.registrar_intento("t1");
        turno.limpiar_intentos("t1");

        assert_eq!(
            turno.registrar_intento("t1"),
            1,
            "tras limpiar, el siguiente registro debe volver a empezar en 1"
        );
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd app/src-tauri && "$HOME/.cargo/bin/cargo.exe" test --quiet --lib turno::tests::reintentar_reembolsa_el_tiempo_y_reinserta_el_ticket turno::tests::registrar_intento_incrementa_y_devuelve_el_conteo_por_ticket turno::tests::limpiar_intentos_borra_el_conteo_de_ese_ticket`
Expected: compile errors — `reintentar`, `registrar_intento`, and `limpiar_intentos` don't exist yet on `EstadoTurno` (`error[E0599]: no method named ... found`).

- [ ] **Step 3: Add the `intentos_usados` field**

In `app/src-tauri/src/turno/mod.rs`, change the `EstadoTurno` struct (lines 17-21):

```rust
#[derive(Debug, Clone, serde::Serialize)]
pub struct EstadoTurno {
    pub presupuesto_restante: u32,
    pub pendientes: Vec<Ticket>,
}
```

to:

```rust
#[derive(Debug, Clone, serde::Serialize)]
pub struct EstadoTurno {
    pub presupuesto_restante: u32,
    pub pendientes: Vec<Ticket>,
    /// Cuántos intentos incorrectos lleva cada ticket pendiente en este turno
    /// (Plan 17) — nunca se manda al frontend (el conteo de intentos
    /// restantes que ve el jugador viene en la respuesta de `resolver_ticket`,
    /// no de aquí). Se limpia cuando el ticket se resuelve de verdad (acierto
    /// o intentos agotados) para que un id de ticket que reaparezca en un
    /// turno futuro empiece en cero.
    #[serde(skip)]
    pub intentos_usados: std::collections::HashMap<String, u32>,
}
```

- [ ] **Step 4: Initialize the new field at both `EstadoTurno` construction sites**

In `app/src-tauri/src/turno/mod.rs`, `EstadoTurno::nuevo` (around line 50-56), change:

```rust
        (
            EstadoTurno {
                presupuesto_restante: PRESUPUESTO_POR_TURNO,
                pendientes,
            },
            indice,
        )
```

to:

```rust
        (
            EstadoTurno {
                presupuesto_restante: PRESUPUESTO_POR_TURNO,
                pendientes,
                intentos_usados: std::collections::HashMap::new(),
            },
            indice,
        )
```

In `app/src-tauri/src/lib.rs`, the `cargar_partida` reconstruction (around line 585-588), change:

```rust
            actual: turno::EstadoTurno {
                presupuesto_restante: partida.presupuesto_restante,
                pendientes,
            },
```

to:

```rust
            actual: turno::EstadoTurno {
                presupuesto_restante: partida.presupuesto_restante,
                pendientes,
                intentos_usados: std::collections::HashMap::new(),
            },
```

A loaded save always starts every pending ticket's attempt count at zero — attempt progress made before a save is not persisted. This is an intentional simplification; nothing in the spec requires attempt counts to survive a save/load cycle.

- [ ] **Step 5: Add the 3 new methods**

In `app/src-tauri/src/turno/mod.rs`, right after the existing `resolver` method (ends around line 72, right before the `turno_agotado` doc comment), add:

```rust
    /// Cuenta un intento incorrecto de `id` y devuelve el nuevo total
    /// (Plan 17) — 1 en el primer registro de ese ticket en este turno.
    pub fn registrar_intento(&mut self, id: &str) -> u32 {
        let contador = self.intentos_usados.entry(id.to_string()).or_insert(0);
        *contador += 1;
        *contador
    }

    /// Olvida el conteo de intentos de `id` (Plan 17) — se llama cuando el
    /// ticket se resuelve de verdad (acierto o intentos agotados), para que
    /// no quede un conteo obsoleto si ese id reapareciera en un turno futuro.
    pub fn limpiar_intentos(&mut self, id: &str) {
        self.intentos_usados.remove(id);
    }

    /// Deshace un `resolver()` (Plan 17): reembolsa el costo de tiempo del
    /// ticket al presupuesto y lo reinserta en `pendientes` — usado cuando un
    /// intento falla pero todavía quedan reintentos disponibles, para que el
    /// ticket siga abierto sin haber costado nada de tiempo ni de pago.
    pub fn reintentar(&mut self, ticket: Ticket) {
        self.presupuesto_restante = self.presupuesto_restante.saturating_add(ticket.costo_tiempo);
        self.pendientes.push(ticket);
    }
```

- [ ] **Step 6: Run the tests to verify they pass**

Run: `cd app/src-tauri && "$HOME/.cargo/bin/cargo.exe" test --quiet --lib turno::`
Expected: all tests in the `turno` module pass, including the 3 new ones and every pre-existing test (unaffected by the new field since they all go through `EstadoTurno::nuevo`, which now initializes it).

- [ ] **Step 7: Confirm the whole project still compiles**

Run: `cd app/src-tauri && "$HOME/.cargo/bin/cargo.exe" check --quiet`
Expected: no output, exit code 0 — this catches the `cargar_partida` construction site in `lib.rs`, which would otherwise fail to compile with a missing-field error.

- [ ] **Step 8: Self-review**

Read the diff. Confirm `#[serde(skip)]` is on the new field (not `#[serde(skip_serializing)]` — `skip` also skips deserialization, which is correct here since nothing should ever construct this field from untrusted wire data). Confirm `reintentar` takes `ticket: Ticket` by value (not `&Ticket`) since the caller in Task 3 will be moving an owned `Ticket` it already has from `resolver()`'s return value. Confirm `EstadoTurno::resolver` itself (the existing method) is completely untouched by this diff.

- [ ] **Step 9: Commit**

```bash
git add app/src-tauri/src/turno/mod.rs app/src-tauri/src/lib.rs
git commit -m "Add per-ticket attempt tracking and a reintentar() refund-and-reinsert method to EstadoTurno"
```

---

### Task 2: "Segunda Opinión" perk + `intentos_extra`

**Files:**
- Modify: `app/src-tauri/src/perks/mod.rs:32-37` (`Efecto` enum), `mod.rs:39-128` (`CATALOGO`), `mod.rs:149-159` (tests)
- Modify: `app/src-tauri/src/economia/mod.rs:216-228` (add `intentos_extra` right after `multiplicador_dinero`)

**Interfaces:**
- Consumes: nothing from Task 1.
- Produces (consumed by Task 3): `perks::Efecto::BonoIntentos(u32)` variant; perk id `"segunda_opinion"` in `perks::catalogo()`; `EstadoJugador::intentos_extra(&self, catalogo: &[Perk]) -> u32` (same signature shape as `multiplicador_dinero`/`multiplicador_reputacion` — takes a `&[Perk]` slice, not `Vec<Perk>`, matching how `perks::catalogo()` is actually typed and called elsewhere in this codebase).

- [ ] **Step 1: Write the failing tests**

In `app/src-tauri/src/perks/mod.rs`, change the `catalogo_tiene_8_perks` test to reflect the new count, and change `catalogo_tiene_2_perks_por_categoria` to allow `ManosRapidas` to have one more than the rest. Find (lines 148-159):

```rust
    #[test]
    fn catalogo_tiene_8_perks() {
        assert_eq!(catalogo().len(), 8);
    }

    #[test]
    fn catalogo_tiene_2_perks_por_categoria() {
        for categoria in [Categoria::Detective, Categoria::ManosRapidas, Categoria::BilleteraYFama, Categoria::Ritmo] {
            let cantidad = catalogo().iter().filter(|p| p.categoria == categoria).count();
            assert_eq!(cantidad, 2, "{categoria:?} debe tener exactamente 2 perks");
        }
    }
```

Replace with:

```rust
    #[test]
    fn catalogo_tiene_9_perks() {
        assert_eq!(catalogo().len(), 9, "8 originales + Segunda Opinion (Plan 17)");
    }

    #[test]
    fn catalogo_tiene_2_perks_por_categoria_salvo_manos_rapidas_con_3() {
        for categoria in [Categoria::Detective, Categoria::BilleteraYFama, Categoria::Ritmo] {
            let cantidad = catalogo().iter().filter(|p| p.categoria == categoria).count();
            assert_eq!(cantidad, 2, "{categoria:?} debe tener exactamente 2 perks");
        }
        let manos_rapidas = catalogo().iter().filter(|p| p.categoria == Categoria::ManosRapidas).count();
        assert_eq!(manos_rapidas, 3, "ManosRapidas suma Segunda Opinion (Plan 17)");
    }
```

Also add a new test right after `solo_billetera_y_fama_tiene_efecto_mecanico_real` (the last test in the file):

```rust
    #[test]
    fn segunda_opinion_da_2_intentos_extra() {
        let perk = buscar("segunda_opinion").expect("segunda_opinion debe existir");
        assert_eq!(perk.categoria, Categoria::ManosRapidas);
        assert_eq!(perk.efecto, Efecto::BonoIntentos(2));
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cd app/src-tauri && "$HOME/.cargo/bin/cargo.exe" test --quiet --lib perks::`
Expected: `catalogo_tiene_9_perks` fails (`8 != 9`), `catalogo_tiene_2_perks_por_categoria_salvo_manos_rapidas_con_3` fails (`ManosRapidas` has 2, not 3), and `segunda_opinion_da_2_intentos_extra` fails to compile (`buscar("segunda_opinion")` returns `None`, `.expect(...)` panics) or fails to find the perk.

- [ ] **Step 3: Add the `BonoIntentos` variant and the "Segunda Opinión" perk**

In `app/src-tauri/src/perks/mod.rs`, change the `Efecto` enum (lines 32-37):

```rust
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub enum Efecto {
    BonoDinero(f64),
    BonoReputacion(f64),
    SinEfectoMecanico,
}
```

to:

```rust
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub enum Efecto {
    BonoDinero(f64),
    BonoReputacion(f64),
    /// Intentos extra por ticket antes de perderlo (Plan 17).
    BonoIntentos(u32),
    SinEfectoMecanico,
}
```

Then change the catalog size (line 39) from `const CATALOGO: [Perk; 8] = [` to `const CATALOGO: [Perk; 9] = [`, and add a new entry at the end of the array (right after the `modo_turbo` entry, before the closing `];`):

```rust
    Perk {
        id: "segunda_opinion",
        nombre: "Segunda Opinión",
        categoria: Categoria::ManosRapidas,
        descripcion: "Antes de rendirte con un ticket difícil, tienes 2 intentos extra para corregir tu respuesta.",
        costo_dinero: 300,
        reputacion_minima: 5.0,
        arquetipo_requerido: Arquetipo::Select,
        xp_minimo: 30,
        efecto: Efecto::BonoIntentos(2),
    },
```

Finally, update the doc comment on `catalogo()` (currently `/// Catálogo completo de perks (Etapa 13) — 8 perks, 2 por categoría.`) to:

```rust
/// Catálogo completo de perks (Etapa 13) — 9 perks: 2 por categoría, salvo
/// Manos Rápidas que tiene 3 (Plan 17 agrega "Segunda Opinión").
pub fn catalogo() -> &'static [Perk] {
    &CATALOGO
}
```

- [ ] **Step 4: Fix the "only Billetera y Fama has a real effect" test**

The existing test `solo_billetera_y_fama_tiene_efecto_mecanico_real` (near the end of the file) asserts that every perk with a non-`SinEfectoMecanico` effect belongs to `Categoria::BilleteraYFama` — "Segunda Opinión" now breaks that invariant on purpose (it's a real effect outside that category). Find:

```rust
    #[test]
    fn solo_billetera_y_fama_tiene_efecto_mecanico_real() {
        for perk in catalogo() {
            let tiene_efecto_real = !matches!(perk.efecto, Efecto::SinEfectoMecanico);
            assert_eq!(
                tiene_efecto_real,
                perk.categoria == Categoria::BilleteraYFama,
                "'{}' (categoría {:?}) no debe tener un efecto real fuera de Billetera y Fama",
                perk.nombre,
                perk.categoria
            );
        }
    }
```

Replace with:

```rust
    #[test]
    fn solo_billetera_y_fama_y_segunda_opinion_tienen_efecto_mecanico_real() {
        for perk in catalogo() {
            let tiene_efecto_real = !matches!(perk.efecto, Efecto::SinEfectoMecanico);
            let debe_tener_efecto_real = perk.categoria == Categoria::BilleteraYFama || perk.id == "segunda_opinion";
            assert_eq!(
                tiene_efecto_real,
                debe_tener_efecto_real,
                "'{}' (categoría {:?}) tiene un efecto real inesperado, o le falta uno (Plan 17: Segunda Opinion es la única excepción fuera de Billetera y Fama)",
                perk.nombre,
                perk.categoria
            );
        }
    }
```

- [ ] **Step 5: Run the perks tests to verify they pass**

Run: `cd app/src-tauri && "$HOME/.cargo/bin/cargo.exe" test --quiet --lib perks::`
Expected: all tests pass.

- [ ] **Step 6: Write the failing test for `intentos_extra`**

In `app/src-tauri/src/economia/mod.rs`, in the test module, right after `multiplicador_reputacion_solo_cuenta_perks_equipados_no_solo_desbloqueados` (which ends around line 581), add:

```rust
    #[test]
    fn intentos_extra_es_0_sin_el_perk_y_2_con_el_equipado() {
        let catalogo = perks::catalogo();
        let mut estado = EstadoJugador::default();
        estado.perks_desbloqueados.push("segunda_opinion");

        assert_eq!(
            estado.intentos_extra(catalogo),
            0,
            "desbloqueado pero no equipado no debe aplicar"
        );

        estado.equipar_perk("segunda_opinion").unwrap();
        assert_eq!(estado.intentos_extra(catalogo), 2, "equipado, +2 intentos");
    }
```

- [ ] **Step 7: Run the test to verify it fails**

Run: `cd app/src-tauri && "$HOME/.cargo/bin/cargo.exe" test --quiet --lib economia::tests::intentos_extra_es_0_sin_el_perk_y_2_con_el_equipado`
Expected: compile error — `EstadoJugador` has no method `intentos_extra`.

- [ ] **Step 8: Implement `intentos_extra`**

In `app/src-tauri/src/economia/mod.rs`, right after the existing `multiplicador_reputacion` method (ends around line 242, right before the closing `}` of the `impl EstadoJugador` block), add:

```rust
    /// Intentos extra por ticket (Plan 17) por los perks equipados que dan
    /// `BonoIntentos` — 0 si ninguno está activo. Se suma, no se multiplica
    /// (a diferencia de dinero/reputación): dos perks de este tipo, si
    /// alguna vez existieran, sumarían sus bonos en vez de componerse.
    pub fn intentos_extra(&self, catalogo: &[Perk]) -> u32 {
        let mut extra = 0;
        for &id in &self.perks_equipados {
            if let Some(perk) = catalogo.iter().find(|p| p.id == id) {
                if let Efecto::BonoIntentos(bono) = perk.efecto {
                    extra += bono;
                }
            }
        }
        extra
    }
```

- [ ] **Step 9: Run the test to verify it passes, then the full economia suite**

Run: `cd app/src-tauri && "$HOME/.cargo/bin/cargo.exe" test --quiet --lib economia::`
Expected: all tests pass, including the new one.

- [ ] **Step 10: Self-review**

Read the diff. Confirm `intentos_extra` takes `catalogo: &[Perk]` (a slice), matching `multiplicador_dinero`/`multiplicador_reputacion`'s existing signature shape — not `Vec<Perk>`, which would force callers to clone/collect the catalog unnecessarily. Confirm the perk catalog array size annotation (`[Perk; 9]`) matches the actual number of entries (9) — a mismatch here is a compile error, so this should already be caught by Step 5, but double-check by counting the entries in the diff. Confirm "Segunda Opinión"'s description contains no SQL jargon (no mention of queries, SELECT, WHERE, etc.) — matching the project's established player-facing-copy rule.

- [ ] **Step 11: Commit**

```bash
git add app/src-tauri/src/perks/mod.rs app/src-tauri/src/economia/mod.rs
git commit -m 'Add the "Segunda Opinion" perk (+2 attempts) and EstadoJugador::intentos_extra'
```

---

### Task 3: Wire retries into `resolver_ticket` + the frontend submit flow

**Files:**
- Modify: `app/src-tauri/src/lib.rs:231-251` (`ScoreResult` struct), `lib.rs:319-387` (`resolver_ticket`)
- Modify: `app/src/main.js:498-517` (`submitTicket`)

**Interfaces:**
- Consumes: `EstadoTurno::registrar_intento`/`limpiar_intentos`/`reintentar` (Task 1), `EstadoJugador::intentos_extra` (Task 2).
- Produces: `ScoreResult` gains `intentos_restantes: Option<u32>` — serialized to the frontend as `intentos_restantes` (`null` or a positive number, never `0`: a final result, win or exhausted, always serializes as `null`). `submitTicket()`'s existing return contract (`true`/`false`, already consumed by the `#btn-submit` listener at `main.js:711-716`, unchanged by this task) now also returns `false` for a non-final retry-available result, in addition to its existing `false` cases (no ticket selected, backend error).

- [ ] **Step 1: Add `intentos_restantes` to `ScoreResult`**

In `app/src-tauri/src/lib.rs`, change the `ScoreResult` struct (lines 231-251):

```rust
#[derive(serde::Serialize)]
struct ScoreResult {
    pass: bool,
    puntaje_correctitud: f64,
    puntaje_velocidad: f64,
    puntaje_practicas: f64,
    puntaje_base: f64,
    puntaje_final: f64,
    comentario_mentor: Option<String>,
    dinero_ganado: i64,
    dinero_total: i64,
    reputacion_ganada: f64,
    reputacion_total: f64,
    xp_ganado: Vec<(tickets::Arquetipo, i64)>,
    puede_ascender: bool,
    /// Etapa 10, Plan 7: `true` solo en la entrega exacta en la que el
    /// ascenso de rango ocurrió, para que el frontend muestre el anuncio.
    ascendio: bool,
    rango_actual: tickets::Rango,
    mensaje: String,
}
```

to:

```rust
#[derive(serde::Serialize)]
struct ScoreResult {
    pass: bool,
    puntaje_correctitud: f64,
    puntaje_velocidad: f64,
    puntaje_practicas: f64,
    puntaje_base: f64,
    puntaje_final: f64,
    comentario_mentor: Option<String>,
    dinero_ganado: i64,
    dinero_total: i64,
    reputacion_ganada: f64,
    reputacion_total: f64,
    xp_ganado: Vec<(tickets::Arquetipo, i64)>,
    puede_ascender: bool,
    /// Etapa 10, Plan 7: `true` solo en la entrega exacta en la que el
    /// ascenso de rango ocurrió, para que el frontend muestre el anuncio.
    ascendio: bool,
    rango_actual: tickets::Rango,
    mensaje: String,
    /// Plan 17: `None` en cualquier resultado final (acierto, o intentos
    /// agotados) — el frontend muestra el overlay de puntaje de siempre.
    /// `Some(n)` con `n > 0` cuando la entrega falló pero quedan `n`
    /// reintentos — el ticket sigue pendiente, no se cobra tiempo ni se paga
    /// nada, y el frontend solo muestra un mensaje corto.
    intentos_restantes: Option<u32>,
}
```

- [ ] **Step 2: Rewrite `resolver_ticket`'s body**

In `app/src-tauri/src/lib.rs`, right before the `resolver_ticket` function (around line 318), add the new constant:

```rust
/// Cuántas veces puede fallar un jugador un ticket antes de perderlo del
/// todo (Plan 17) — antes de cualquier perk. "Segunda Opinión" suma 2 más
/// (`EstadoJugador::intentos_extra`).
const INTENTOS_BASE: u32 = 3;
```

Then replace the whole body of `resolver_ticket` (currently lines 326-387, from the `async fn resolver_ticket(` signature through its closing `}`):

```rust
#[tauri::command]
async fn resolver_ticket(
    state: tauri::State<'_, AppState>,
    jugador: tauri::State<'_, Jugador>,
    turno_state: tauri::State<'_, Turno>,
    dir: tauri::State<'_, DirectorioGuardado>,
    id: String,
    sql: String,
) -> Result<ScoreResult, String> {
    // Se retira el ticket de `pendientes` ANTES de validar/premiar (en vez de
    // solo consultarlo con `buscar_pendiente`) para que un doble envío
    // concurrente del mismo ticket (p. ej. doble clic en "✓ Enviar ticket"
    // antes de que resuelva la primera promesa) sea imposible de premiar dos
    // veces: `resolver` es la operación atómica de "check-and-remove" bajo
    // este mismo lock, así que solo la primera llamada puede obtener
    // `Some(ticket)` — la segunda ve `None` y falla aquí, antes de tocar
    // `Jugador` o correr validación/economía.
    let ticket = {
        let mut manejado = turno_state.0.lock().unwrap();
        manejado
            .actual
            .resolver(&id)
            .ok_or_else(|| format!("'{id}' ya fue resuelto o ya no está pendiente."))?
    };

    let pool = state.0.lock().unwrap().clone();
    let evaluacion = validation::evaluar_entrega(&pool, &sql, &ticket.sql_dorada, ticket.requiere_orden)
        .await
        .map_err(|e| e.to_string())?;

    // Plan 17: una entrega incorrecta con reintentos disponibles no cuenta
    // como resuelta — se reembolsa el tiempo y el ticket vuelve a la bandeja
    // sin cobrar nada, en vez de caer en el camino final de abajo.
    if !evaluacion.correcta {
        let estado = jugador.0.lock().unwrap();
        let limite = INTENTOS_BASE + estado.intentos_extra(perks::catalogo());
        let mut manejado = turno_state.0.lock().unwrap();
        let usados = manejado.actual.registrar_intento(&id);
        if usados < limite {
            manejado.actual.reintentar(ticket);
            return Ok(ScoreResult {
                pass: false,
                puntaje_correctitud: evaluacion.puntaje_correctitud,
                puntaje_velocidad: evaluacion.puntaje_velocidad,
                puntaje_practicas: evaluacion.puntaje_practicas,
                puntaje_base: 0.0,
                puntaje_final: 0.0,
                comentario_mentor: evaluacion.comentario_mentor.map(str::to_string),
                dinero_ganado: 0,
                dinero_total: estado.dinero,
                reputacion_ganada: 0.0,
                reputacion_total: estado.reputacion,
                xp_ganado: Vec::new(),
                puede_ascender: estado.puede_ascender(),
                ascendio: false,
                rango_actual: estado.rango,
                mensaje: format!("No es correcto todavía. Te quedan {} intento(s).", limite - usados),
                intentos_restantes: Some(limite - usados),
            });
        }
    }

    let mut estado = jugador.0.lock().unwrap();
    let multiplicador_dinero = estado.multiplicador_dinero(perks::catalogo());
    let multiplicador_reputacion = estado.multiplicador_reputacion(perks::catalogo());
    let resultado = economia::calcular(&evaluacion, &ticket, multiplicador_dinero, multiplicador_reputacion);
    let ascendio = estado.aplicar_resultado(&resultado);

    let mut manejado = turno_state.0.lock().unwrap();
    manejado.actual.limpiar_intentos(&id);
    manejado.actualizar_fase(ascendio, &mut estado);
    autoguardar(&dir.0, &estado, &manejado);

    Ok(ScoreResult {
        pass: evaluacion.correcta,
        puntaje_correctitud: evaluacion.puntaje_correctitud,
        puntaje_velocidad: evaluacion.puntaje_velocidad,
        puntaje_practicas: evaluacion.puntaje_practicas,
        puntaje_base: resultado.puntaje_base,
        puntaje_final: resultado.puntaje_final,
        comentario_mentor: evaluacion.comentario_mentor.map(str::to_string),
        dinero_ganado: resultado.dinero_ganado,
        dinero_total: estado.dinero,
        reputacion_ganada: resultado.reputacion_ganada,
        reputacion_total: estado.reputacion,
        xp_ganado: resultado.xp_ganado,
        puede_ascender: estado.puede_ascender(),
        ascendio,
        rango_actual: estado.rango,
        mensaje: if evaluacion.correcta {
            "Ticket resuelto. Contabilidad procesará tu pago... eventualmente.".to_string()
        } else {
            "El resultado no coincide con lo que pidió la solicitud. Revisa tu consulta.".to_string()
        },
        intentos_restantes: None,
    })
}
```

Design notes for the reviewer:
- The `if !evaluacion.correcta { ... }` block only ever early-`return`s when `usados < limite` (retries left) — when attempts are exhausted (`usados >= limite`), execution falls through to the same final-path code that already existed for both correct and incorrect submissions, completely unchanged from before this plan. This is why `manejado.actual.limpiar_intentos(&id);` is called unconditionally in that shared final path: it correctly no-ops (removing a key that's never been inserted) for a submission that was correct on the very first try, and correctly cleans up for a submission that used some retries before succeeding or being exhausted.
- `ticket` is moved (not cloned) into `manejado.actual.reintentar(ticket)` in the retry branch, since that branch returns immediately afterward. On the "falls through" path (attempts exhausted), `ticket` was never moved, so it's still available for `economia::calcular(&evaluacion, &ticket, ...)` below — Rust's flow-sensitive move checker allows this because the move only happens on a path that returns before reaching the later use.
- The `estado` (jugador) lock taken inside the `if !evaluacion.correcta` block is a separate, shorter-lived borrow from the `estado` taken again below it — by the time execution reaches the shared final path, that first lock has already gone out of scope (the `if` block ended), so re-acquiring `jugador.0.lock()` does not deadlock.

- [ ] **Step 3: Run the Rust checks**

Run: `cd app/src-tauri && "$HOME/.cargo/bin/cargo.exe" check --quiet`
Expected: no output, exit code 0.

Run: `cd app/src-tauri && "$HOME/.cargo/bin/cargo.exe" test --quiet --lib`
Expected: the full existing test suite still passes — this task doesn't add new Rust unit tests of its own (the underlying pieces it composes were already tested in Tasks 1-2), but every pre-existing test across `lib.rs`, `turno`, `perks`, `economia`, `tickets`, and `validation` must remain green, since `resolver_ticket` itself has no dedicated unit tests in this codebase (it's a Tauri command, exercised indirectly).

- [ ] **Step 4: Update `submitTicket()` in the frontend**

In `app/src/main.js`, change `submitTicket` (currently lines 498-517):

```js
async function submitTicket() {
  if (!ticketActivoId) {
    setStatus("Elige un ticket de la bandeja primero.", "error");
    return false;
  }
  setStatus("Enviando ticket...", "");
  try {
    const score = await invoke("resolver_ticket", { id: ticketActivoId, sql: sqlInput.value });
    actualizarDinero(score.dinero_total);
    actualizarReputacion(score.reputacion_total.toFixed(1));
    renderRango(score.rango_actual);
    mostrarScoring(score);
    setStatus(score.mensaje, score.pass ? "ok" : "error");
    await cargarTurno();
    return true;
  } catch (err) {
    setStatus(String(err), "error");
    return false;
  }
}
```

to:

```js
async function submitTicket() {
  if (!ticketActivoId) {
    setStatus("Elige un ticket de la bandeja primero.", "error");
    return false;
  }
  setStatus("Enviando ticket...", "");
  try {
    const score = await invoke("resolver_ticket", { id: ticketActivoId, sql: sqlInput.value });
    if (score.intentos_restantes) {
      setStatus(score.mensaje, "error");
      return false;
    }
    actualizarDinero(score.dinero_total);
    actualizarReputacion(score.reputacion_total.toFixed(1));
    renderRango(score.rango_actual);
    mostrarScoring(score);
    setStatus(score.mensaje, score.pass ? "ok" : "error");
    await cargarTurno();
    return true;
  } catch (err) {
    setStatus(String(err), "error");
    return false;
  }
}
```

`if (score.intentos_restantes)` is correctly falsy for both `null` (a final result — Tauri/serde serializes Rust's `None` as JSON `null`) and `undefined`, and truthy only for a positive number — the backend never sends `0` (see Task 3 Step 1's design note: the final path always sends `None`, never `Some(0)`). No `cargarTurno()` call happens on this new early-return path, so the bandeja isn't refreshed and the console stays exactly as the player left it — the same `ticketActivoId` is still selected, and `sqlInput.value` still holds whatever the player typed, ready to correct and resubmit.

Existing callers of `submitTicket()` need no changes: `main.js:711-716`'s `#btn-submit` listener already does `const exito = await submitTicket(); if (exito) { notificarClicEnviar(); }` — a retry-available result now correctly returns `false` from this new early-return branch, so `notificarClicEnviar()` (which arms the Plan 15/16 tutorial's wait-for-scoring-close state) is skipped exactly as it should be.

- [ ] **Step 5: Syntax-check the frontend change**

Run: `node --check app/src/main.js`
Expected: no output, exit code 0.

- [ ] **Step 6: Self-review**

Read the full diff across both files. Confirm the Rust side never sends `Some(0)` — trace both the retry branch (always `Some(limite - usados)` where `usados < limite`, so the value is always `>= 1`) and the final path (always `None`). Confirm `submitTicket`'s new early-return happens *before* `actualizarDinero`/`actualizarReputacion`/`renderRango`/`mostrarScoring`/`cargarTurno` — a retry-available result must not touch any of those. Confirm the tutorial's `notificarClicEnviar()` call site in the `#btn-submit` listener is untouched (this task doesn't need to modify it — the existing `if (exito)` gate already does the right thing once `submitTicket()` returns `false` for a retry).

- [ ] **Step 7: Commit**

```bash
git add app/src-tauri/src/lib.rs app/src/main.js
git commit -m "Wire retry attempts into resolver_ticket and the frontend submit flow"
```

---

## Manual Verification (after all 3 tasks)

Same pattern as every prior Fase 0 plan — guided verification in the real running app:

- Open a ticket, submit a deliberately wrong query: confirm a short status message appears ("No es correcto todavía. Te quedan 2 intento(s).") with **no** scoring overlay, the turn's time budget is unchanged, and the ticket is still open in the console with your (wrong) query still in the editor.
- Submit wrong 2 more times (3 total): on the 3rd wrong submission, confirm the *normal* scoring overlay now appears (❌, $0/0, and the ticket is gone from the tray) — same behavior as before this plan.
- On a fresh ticket, submit wrong once, then submit the correct query: confirm the normal scoring overlay appears with ✅ and the *full* reward (no penalty for it being the 2nd attempt).
- Unlock and equip "Segunda Opinión" (Perks tab): confirm a ticket now allows 5 wrong submissions before the final, unrecoverable failure (instead of 3).
- During the Plan 15/16 tutorial's guided tickets, deliberately mistype a clause before fixing it and submitting: confirm the tutorial's Mentor dialogue and blocking behave normally afterward — a non-final retry must not wedge or skip any tutorial beat.
- Confirm `cerrar_dia` / turn rotation still behaves normally for a ticket that still has an open retry loop when the turn's time budget runs out (this plan doesn't change `turno_agotado`/`escalar_pendientes`, so a ticket mid-retry-loop is scaled/penalized exactly like any other still-pending ticket when the day closes).
