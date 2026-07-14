# Fase 0 / Plan 4: Economía Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reemplazar la recompensa fija (+500 si es correcto) por la economía de 3 recursos de la Etapa 12 — dinero, reputación y XP por arquetipo — combinando los 3 puntajes crudos del motor de validación (Plan 2) con los pesos de rúbrica que ya trae cada ticket (Plan 3).

**Architecture:** Nuevo módulo `app/src-tauri/src/economia/mod.rs`: una función pura `calcular()` que aplica la fórmula de la Etapa 12, y un `EstadoJugador` que acumula el resultado turno a turno. Ninguno de los dos toca la base de datos — son cálculo puro sobre lo que Plan 2 (`validation::Evaluacion`) y Plan 3 (`tickets::Ticket`) ya producen.

**Tech Stack:** Rust puro, sin dependencias nuevas.

## Global Constraints

- Fórmula literal de la Etapa 12: `puntaje_base = correctitud×peso_correctitud + velocidad×peso_velocidad + practicas×peso_practicas`; `puntaje_final = puntaje_base × multiplicador_perks`; `dinero_ganado`/`reputacion_ganada`/`xp_ganado` se derivan de `puntaje_final`.
- `multiplicador_perks_activos` fijo en `1.0` en este plan — no existe todavía el sistema RPG/perks real (Etapa 13); un plan posterior lo conecta sin tocar esta fórmula.
- Los tickets incorrectos no otorgan dinero, reputación ni XP (no hay penalización de reputación en este plan — depende del sistema de turnos, Etapa 11-A, no construido).
- Se trackea reputación real y se expone `puede_ascender()` (umbral de la Etapa 10) — **sin** disparar el ascenso de rango real, que depende del mini-boss (plan posterior).
- El stub de un solo perk (`unlock_perk`, costo fijo 300) se mantiene funcionalmente idéntico, solo migrado al nuevo `EstadoJugador`.
- Valores de partida (Etapa 12: "sujetos a ajuste en playtesting") — `valor_base`/`factor_reputacion` por plantilla (Plan 3): simple=100/0.5, agregado=150/0.7, join=150/0.7, join_agregado=200/1.0, depuración=250/1.2. XP base por arquetipo: Select=10, Join=20, Agregación=25. Umbral de ascenso Becario→Auxiliar: 500.0 de reputación.

---

## File Structure

- Modify: `app/src-tauri/src/tickets/mod.rs` — agrega `valor_base`/`factor_reputacion` al `Ticket` y sus valores hardcodeados por plantilla (Tarea 1); retira los `#[allow(dead_code)]` que ya no hacen falta una vez que `economia` los consume (Tarea 2)
- Create: `app/src-tauri/src/economia/mod.rs` — `Resultado`, `calcular()`, tabla de XP base (Tarea 2); `EstadoJugador` (Tarea 3)
- Modify: `app/src-tauri/src/lib.rs` — agrega `mod economia;`, reemplaza `Perk`/`PerkState` por `Jugador`/`economia::EstadoJugador`, rewira `submit_ticket`/`unlock_perk` (Tarea 4)

---

### Task 1: Extender `Ticket` con `valor_base` y `factor_reputacion`

**Files:**
- Modify: `app/src-tauri/src/tickets/mod.rs`

**Interfaces:**
- Produces: `Ticket.valor_base: i64`, `Ticket.factor_reputacion: f64` (nuevos campos, hardcodeados dentro de cada función `plantilla_*` — sin cambios en las firmas de esas funciones, así que las 12 llamadas concretas en `hospital_arcangel.rs`/`postafeta.rs` no se tocan)

- [ ] **Step 1: Reemplazar el `struct Ticket` completo**

Localizar (líneas 1-32 del archivo actual):

```rust
/// Un ticket concreto (Etapa 14): la unidad de trabajo que el jugador
/// resuelve escribiendo SQL. Generado por una plantilla paramétrica (las
/// funciones `plantilla_*` de este archivo), nunca escrito a mano ticket por
/// ticket (Pilar 5).
#[derive(Debug, Clone, serde::Serialize)]
pub struct Ticket {
    pub id: &'static str,
    pub tipo: TipoTicket,
    pub solicitante: &'static str,
    pub motivo: String,
    pub solicitud: String,
    pub prioridad: Prioridad,
    pub costo_tiempo: u32,
    pub arquetipos: Vec<Arquetipo>,
    /// Nunca debe llegar al cliente: es la respuesta correcta del puzzle.
    #[serde(skip_serializing)]
    pub sql_dorada: String,
    pub sql_inicial: Option<String>,
    pub requiere_orden: bool,
    // Ya no se serializan (no deben llegar al cliente) y aún no las lee
    // ningún código de producción — solo los tests de este módulo — por lo
    // que el análisis de código muerto las marcaría sin este allow.
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    pub peso_correctitud: f64,
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    pub peso_velocidad: f64,
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    pub peso_practicas: f64,
}
```

Reemplazar por (agrega `valor_base`/`factor_reputacion`, con el mismo tratamiento temporal de `#[allow(dead_code)]` que ya tienen los pesos — todavía no tienen consumidor de producción, solo tests, hasta la Tarea 2):

```rust
/// Un ticket concreto (Etapa 14): la unidad de trabajo que el jugador
/// resuelve escribiendo SQL. Generado por una plantilla paramétrica (las
/// funciones `plantilla_*` de este archivo), nunca escrito a mano ticket por
/// ticket (Pilar 5).
#[derive(Debug, Clone, serde::Serialize)]
pub struct Ticket {
    pub id: &'static str,
    pub tipo: TipoTicket,
    pub solicitante: &'static str,
    pub motivo: String,
    pub solicitud: String,
    pub prioridad: Prioridad,
    pub costo_tiempo: u32,
    pub arquetipos: Vec<Arquetipo>,
    /// Nunca debe llegar al cliente: es la respuesta correcta del puzzle.
    #[serde(skip_serializing)]
    pub sql_dorada: String,
    pub sql_inicial: Option<String>,
    pub requiere_orden: bool,
    // Ya no se serializan (no deben llegar al cliente) y aún no las lee
    // ningún código de producción — solo los tests de este módulo — por lo
    // que el análisis de código muerto las marcaría sin este allow.
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    pub peso_correctitud: f64,
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    pub peso_velocidad: f64,
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    pub peso_practicas: f64,
    /// Valor base de dinero (Etapa 12) — sube con prioridad/complejidad del
    /// ticket. Dato interno de la fórmula de economía, sin uso del lado del
    /// cliente.
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    pub valor_base: i64,
    /// Factor de reputación (Etapa 12) — mayor en tickets de mayor exigencia.
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    pub factor_reputacion: f64,
}
```

- [ ] **Step 2: Agregar los 2 campos nuevos al final de cada uno de los 5 `Ticket { ... }` construidos por las plantillas**

En `plantilla_reporte_simple`, localizar el final del `Ticket { ... }`:

```rust
        peso_correctitud: 0.6,
        peso_velocidad: 0.2,
        peso_practicas: 0.2,
    }
}
```

Reemplazar por:

```rust
        peso_correctitud: 0.6,
        peso_velocidad: 0.2,
        peso_practicas: 0.2,
        valor_base: 100,
        factor_reputacion: 0.5,
    }
}
```

En `plantilla_reporte_agregado`, localizar:

```rust
        peso_correctitud: 0.5,
        peso_velocidad: 0.2,
        peso_practicas: 0.3,
    }
}

/// Plantilla "reporte con JOIN"
```

Reemplazar por (cuidado: este mismo bloque de pesos 0.5/0.2/0.3 se repite en `plantilla_reporte_join` — usar el comentario `/// Plantilla "reporte con JOIN"` que sigue inmediatamente para ubicar la ocurrencia correcta, la de `plantilla_reporte_agregado`):

```rust
        peso_correctitud: 0.5,
        peso_velocidad: 0.2,
        peso_practicas: 0.3,
        valor_base: 150,
        factor_reputacion: 0.7,
    }
}

/// Plantilla "reporte con JOIN"
```

En `plantilla_reporte_join`, localizar el final de su propio `Ticket { ... }` (mismos pesos 0.5/0.2/0.3, pero seguido del comentario de `plantilla_reporte_join_agregado`):

```rust
        peso_correctitud: 0.5,
        peso_velocidad: 0.2,
        peso_practicas: 0.3,
    }
}

/// Plantilla "reporte con JOIN + agregación"
```

Reemplazar por:

```rust
        peso_correctitud: 0.5,
        peso_velocidad: 0.2,
        peso_practicas: 0.3,
        valor_base: 150,
        factor_reputacion: 0.7,
    }
}

/// Plantilla "reporte con JOIN + agregación"
```

En `plantilla_reporte_join_agregado`, localizar:

```rust
        peso_correctitud: 0.4,
        peso_velocidad: 0.3,
        peso_practicas: 0.3,
    }
}
```

Reemplazar por:

```rust
        peso_correctitud: 0.4,
        peso_velocidad: 0.3,
        peso_practicas: 0.3,
        valor_base: 200,
        factor_reputacion: 1.0,
    }
}
```

En `plantilla_depuracion`, localizar:

```rust
        peso_correctitud: 0.3,
        peso_velocidad: 0.5,
        peso_practicas: 0.2,
    }
}
```

Reemplazar por:

```rust
        peso_correctitud: 0.3,
        peso_velocidad: 0.5,
        peso_practicas: 0.2,
        valor_base: 250,
        factor_reputacion: 1.2,
    }
}
```

- [ ] **Step 3: Extender las 5 aserciones de pesos ya existentes en `mod tests` para cubrir los 2 campos nuevos**

En `plantilla_reporte_simple_arma_un_ticket_de_reporte_sin_join`, localizar:

```rust
        assert_eq!((ticket.peso_correctitud, ticket.peso_velocidad, ticket.peso_practicas), (0.6, 0.2, 0.2));
    }
```

Reemplazar por:

```rust
        assert_eq!((ticket.peso_correctitud, ticket.peso_velocidad, ticket.peso_practicas), (0.6, 0.2, 0.2));
        assert_eq!((ticket.valor_base, ticket.factor_reputacion), (100, 0.5));
    }
```

En `plantilla_reporte_agregado_arma_un_ticket_de_agregacion`, localizar:

```rust
        assert_eq!((ticket.peso_correctitud, ticket.peso_velocidad, ticket.peso_practicas), (0.5, 0.2, 0.3));
    }

    #[test]
    fn plantilla_reporte_join_arma_un_ticket_con_join() {
```

Reemplazar por:

```rust
        assert_eq!((ticket.peso_correctitud, ticket.peso_velocidad, ticket.peso_practicas), (0.5, 0.2, 0.3));
        assert_eq!((ticket.valor_base, ticket.factor_reputacion), (150, 0.7));
    }

    #[test]
    fn plantilla_reporte_join_arma_un_ticket_con_join() {
```

En `plantilla_reporte_join_arma_un_ticket_con_join`, localizar:

```rust
        assert_eq!((ticket.peso_correctitud, ticket.peso_velocidad, ticket.peso_practicas), (0.5, 0.2, 0.3));
    }

    #[test]
    fn plantilla_reporte_join_agregado_arma_un_ticket_con_join_y_agregacion() {
```

Reemplazar por:

```rust
        assert_eq!((ticket.peso_correctitud, ticket.peso_velocidad, ticket.peso_practicas), (0.5, 0.2, 0.3));
        assert_eq!((ticket.valor_base, ticket.factor_reputacion), (150, 0.7));
    }

    #[test]
    fn plantilla_reporte_join_agregado_arma_un_ticket_con_join_y_agregacion() {
```

En `plantilla_reporte_join_agregado_arma_un_ticket_con_join_y_agregacion`, localizar:

```rust
        assert_eq!((ticket.peso_correctitud, ticket.peso_velocidad, ticket.peso_practicas), (0.4, 0.3, 0.3));
    }
```

Reemplazar por:

```rust
        assert_eq!((ticket.peso_correctitud, ticket.peso_velocidad, ticket.peso_practicas), (0.4, 0.3, 0.3));
        assert_eq!((ticket.valor_base, ticket.factor_reputacion), (200, 1.0));
    }
```

En `plantilla_depuracion_arma_un_ticket_con_sql_inicial`, localizar:

```rust
        assert_eq!((ticket.peso_correctitud, ticket.peso_velocidad, ticket.peso_practicas), (0.3, 0.5, 0.2));
    }
```

Reemplazar por:

```rust
        assert_eq!((ticket.peso_correctitud, ticket.peso_velocidad, ticket.peso_practicas), (0.3, 0.5, 0.2));
        assert_eq!((ticket.valor_base, ticket.factor_reputacion), (250, 1.2));
    }
```

- [ ] **Step 4: Correr la suite completa**

Run: `cd app/src-tauri && cargo test --lib -- --nocapture`
Expected: todos los tests en verde, 0 failed (mismos tests de antes, con aserciones extendidas — ninguno nuevo).

- [ ] **Step 5: Commit**

```bash
git add app/src-tauri/src/tickets/mod.rs
git commit -m "Add valor_base and factor_reputacion to Ticket templates (Stage 12)"
```

---

### Task 2: Fórmula de economía (`economia::calcular`)

**Files:**
- Create: `app/src-tauri/src/economia/mod.rs`
- Modify: `app/src-tauri/src/tickets/mod.rs` (retira los `#[allow(dead_code)]` de `peso_correctitud`/`peso_velocidad`/`peso_practicas`/`valor_base`/`factor_reputacion` — ya tienen consumidor real)
- Modify: `app/src-tauri/src/lib.rs` (agrega `mod economia;`)

**Interfaces:**
- Consumes: `crate::validation::Evaluacion` (Plan 2), `crate::tickets::{Ticket, Arquetipo}` (Plan 3/Tarea 1)
- Produces: `economia::Resultado { puntaje_base, puntaje_final, dinero_ganado, reputacion_ganada, xp_ganado: Vec<(Arquetipo, i64)> }`, `economia::calcular(evaluacion: &Evaluacion, ticket: &Ticket, multiplicador_perks: f64) -> Resultado`

- [ ] **Step 1: Escribir `app/src-tauri/src/economia/mod.rs`**

```rust
use crate::tickets::{Arquetipo, Ticket};
use crate::validation::Evaluacion;

/// Puntos de XP que otorga usar cada arquetipo SQL una vez, antes de escalar
/// por el puntaje final (Etapa 10: la dificultad del concepto define el XP
/// base — Join vale más que Select, Agregación más que Join).
fn xp_base_por_arquetipo(arquetipo: Arquetipo) -> i64 {
    match arquetipo {
        Arquetipo::Select => 10,
        Arquetipo::Join => 20,
        Arquetipo::Agregacion => 25,
    }
}

/// Resultado de aplicar la fórmula de economía (Etapa 12) a una entrega ya
/// evaluada (Plan 2) contra un ticket (Plan 3).
#[derive(Debug, Clone, PartialEq)]
pub struct Resultado {
    pub puntaje_base: f64,
    pub puntaje_final: f64,
    pub dinero_ganado: i64,
    pub reputacion_ganada: f64,
    pub xp_ganado: Vec<(Arquetipo, i64)>,
}

/// Calcula dinero/reputación/XP ganados por una entrega, siguiendo la
/// fórmula literal de la Etapa 12. `multiplicador_perks` representa el
/// efecto de los perks activos del jugador — fijo en 1.0 hasta que exista el
/// sistema RPG real (Etapa 13, plan posterior). Si la entrega es incorrecta,
/// no se otorga dinero/reputación/XP (la penalización por tickets escalados
/// es solo de reputación y depende del sistema de turnos, Etapa 11-A, no
/// construido — este cálculo no la implementa).
pub fn calcular(evaluacion: &Evaluacion, ticket: &Ticket, multiplicador_perks: f64) -> Resultado {
    let puntaje_base = evaluacion.puntaje_correctitud * ticket.peso_correctitud
        + evaluacion.puntaje_velocidad * ticket.peso_velocidad
        + evaluacion.puntaje_practicas * ticket.peso_practicas;
    let puntaje_final = puntaje_base * multiplicador_perks;

    if !evaluacion.correcta {
        return Resultado {
            puntaje_base,
            puntaje_final,
            dinero_ganado: 0,
            reputacion_ganada: 0.0,
            xp_ganado: Vec::new(),
        };
    }

    let dinero_ganado = (puntaje_final * ticket.valor_base as f64 / 100.0).round() as i64;
    let reputacion_ganada = puntaje_final * ticket.factor_reputacion / 100.0;
    let xp_ganado = ticket
        .arquetipos
        .iter()
        .map(|&arquetipo| {
            let xp = (xp_base_por_arquetipo(arquetipo) as f64 * puntaje_final / 100.0).round() as i64;
            (arquetipo, xp)
        })
        .collect();

    Resultado {
        puntaje_base,
        puntaje_final,
        dinero_ganado,
        reputacion_ganada,
        xp_ganado,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tickets::{Prioridad, TipoTicket};

    fn ticket_de_prueba(arquetipos: Vec<Arquetipo>) -> Ticket {
        Ticket {
            id: "ticket_de_prueba",
            tipo: TipoTicket::ReporteAnalisis,
            solicitante: "Alguien",
            motivo: "un motivo".to_string(),
            solicitud: "una solicitud".to_string(),
            prioridad: Prioridad::Media,
            costo_tiempo: 10,
            arquetipos,
            sql_dorada: "SELECT 1".to_string(),
            sql_inicial: None,
            requiere_orden: true,
            peso_correctitud: 0.6,
            peso_velocidad: 0.2,
            peso_practicas: 0.2,
            valor_base: 100,
            factor_reputacion: 0.5,
        }
    }

    fn evaluacion_perfecta() -> Evaluacion {
        Evaluacion {
            correcta: true,
            puntaje_correctitud: 100.0,
            puntaje_velocidad: 100.0,
            puntaje_practicas: 100.0,
            comentario_mentor: None,
        }
    }

    #[test]
    fn calcular_ticket_correcto_otorga_recompensa_proporcional() {
        let ticket = ticket_de_prueba(vec![Arquetipo::Select]);
        let evaluacion = evaluacion_perfecta();

        let resultado = calcular(&evaluacion, &ticket, 1.0);

        assert_eq!(resultado.puntaje_base, 100.0);
        assert_eq!(resultado.puntaje_final, 100.0);
        assert_eq!(resultado.dinero_ganado, 100);
        assert_eq!(resultado.reputacion_ganada, 0.5);
        assert_eq!(resultado.xp_ganado, vec![(Arquetipo::Select, 10)]);
    }

    #[test]
    fn calcular_ticket_incorrecto_no_otorga_dinero_ni_reputacion_ni_xp() {
        let ticket = ticket_de_prueba(vec![Arquetipo::Select]);
        let mut evaluacion = evaluacion_perfecta();
        evaluacion.correcta = false;

        let resultado = calcular(&evaluacion, &ticket, 1.0);

        assert_eq!(resultado.dinero_ganado, 0);
        assert_eq!(resultado.reputacion_ganada, 0.0);
        assert!(resultado.xp_ganado.is_empty());
        assert_eq!(
            resultado.puntaje_base, 100.0,
            "el puntaje de calidad se calcula aunque el resultado sea incorrecto"
        );
    }

    #[test]
    fn calcular_reparte_xp_entre_varios_arquetipos() {
        let mut ticket = ticket_de_prueba(vec![Arquetipo::Join, Arquetipo::Agregacion]);
        ticket.peso_correctitud = 0.4;
        ticket.peso_velocidad = 0.3;
        ticket.peso_practicas = 0.3;
        let evaluacion = evaluacion_perfecta();

        let resultado = calcular(&evaluacion, &ticket, 1.0);

        assert_eq!(
            resultado.xp_ganado,
            vec![(Arquetipo::Join, 20), (Arquetipo::Agregacion, 25)]
        );
    }

    #[test]
    fn calcular_aplica_el_multiplicador_de_perks() {
        let ticket = ticket_de_prueba(vec![Arquetipo::Select]);
        let evaluacion = evaluacion_perfecta();

        let resultado = calcular(&evaluacion, &ticket, 2.0);

        assert_eq!(resultado.puntaje_final, 200.0);
        assert_eq!(resultado.dinero_ganado, 200);
    }
}
```

- [ ] **Step 2: Retirar los `#[allow(dead_code)]` que ya no hacen falta en `tickets/mod.rs`**

Localizar el `struct Ticket` (con los 5 `#[allow(dead_code)]` agregados/mantenidos en la Tarea 1):

```rust
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    pub peso_correctitud: f64,
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    pub peso_velocidad: f64,
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    pub peso_practicas: f64,
    /// Valor base de dinero (Etapa 12) — sube con prioridad/complejidad del
    /// ticket. Dato interno de la fórmula de economía, sin uso del lado del
    /// cliente.
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    pub valor_base: i64,
    /// Factor de reputación (Etapa 12) — mayor en tickets de mayor exigencia.
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    pub factor_reputacion: f64,
```

Reemplazar por (ya tienen un consumidor real: `economia::calcular`, así que el `#[allow(dead_code)]` ya no aplica — se mantiene `#[serde(skip_serializing)]`, que es una decisión de diseño aparte, no un parche de código muerto):

```rust
    #[serde(skip_serializing)]
    pub peso_correctitud: f64,
    #[serde(skip_serializing)]
    pub peso_velocidad: f64,
    #[serde(skip_serializing)]
    pub peso_practicas: f64,
    /// Valor base de dinero (Etapa 12) — sube con prioridad/complejidad del
    /// ticket. Dato interno de la fórmula de economía, sin uso del lado del
    /// cliente.
    #[serde(skip_serializing)]
    pub valor_base: i64,
    /// Factor de reputación (Etapa 12) — mayor en tickets de mayor exigencia.
    #[serde(skip_serializing)]
    pub factor_reputacion: f64,
```

- [ ] **Step 3: Registrar el módulo en `app/src-tauri/src/lib.rs`**

Localizar:

```rust
mod db;
mod tickets;
mod validation;
```

Reemplazar por (orden alfabético):

```rust
mod db;
mod economia;
mod tickets;
mod validation;
```

- [ ] **Step 4: Correr la suite completa**

Run: `cd app/src-tauri && cargo test --lib -- --nocapture`
Expected: todos los tests en verde, 0 failed, más los 4 nuevos de `economia::tests` — y **sin ninguna advertencia** de `cargo build` (los `#[allow(dead_code)]` retirados en el Step 2 ya no deben hacer falta; si aparece alguna advertencia nueva, no debe quedar sin resolver).

- [ ] **Step 5: Commit**

```bash
git add app/src-tauri/src/economia/mod.rs app/src-tauri/src/tickets/mod.rs app/src-tauri/src/lib.rs
git commit -m "Add economia::calcular implementing the Stage 12 reward formula"
```

---

### Task 3: Estado acumulado del jugador (`EstadoJugador`)

**Files:**
- Modify: `app/src-tauri/src/economia/mod.rs` (agrega `EstadoJugador` y sus tests al `mod tests` ya existente)

**Interfaces:**
- Consumes: `Resultado` (Tarea 2)
- Produces: `economia::EstadoJugador { dinero: i64, reputacion: f64, xp_por_arquetipo: Vec<(Arquetipo, i64)>, perk_desbloqueado: bool }` (deriva `Default`), `EstadoJugador::aplicar_resultado(&mut self, resultado: &Resultado)`, `EstadoJugador::puede_ascender(&self) -> bool`

- [ ] **Step 1: Agregar `EstadoJugador` a `app/src-tauri/src/economia/mod.rs`, justo antes de `#[cfg(test)] mod tests`**

```rust
/// Estado acumulado del jugador (Etapa 12): dinero, reputación y XP por
/// arquetipo ganados a lo largo de la partida, más el stub de un solo perk
/// heredado del spike original (Etapa 13 lo reemplaza en un plan posterior).
#[derive(Debug, Clone, Default)]
pub struct EstadoJugador {
    pub dinero: i64,
    pub reputacion: f64,
    pub xp_por_arquetipo: Vec<(Arquetipo, i64)>,
    pub perk_desbloqueado: bool,
}

/// Umbral de reputación para ascender de Becario a Auxiliar de Sistemas en
/// Hospital Arcángel (Etapa 10). El ascenso real (superar el mini-boss,
/// cambiar de rango) es responsabilidad de un plan posterior — esta
/// constante solo define cuándo se cumple la condición de reputación.
const UMBRAL_ASCENSO_AUXILIAR: f64 = 500.0;

impl EstadoJugador {
    /// Aplica el resultado de una entrega (Etapa 12): acumula dinero,
    /// reputación y XP por arquetipo sobre el estado existente.
    pub fn aplicar_resultado(&mut self, resultado: &Resultado) {
        self.dinero += resultado.dinero_ganado;
        self.reputacion += resultado.reputacion_ganada;
        for &(arquetipo, xp) in &resultado.xp_ganado {
            match self.xp_por_arquetipo.iter_mut().find(|(a, _)| *a == arquetipo) {
                Some((_, existente)) => *existente += xp,
                None => self.xp_por_arquetipo.push((arquetipo, xp)),
            }
        }
    }

    /// Etapa 10: señal de que la reputación ya cruzó el umbral de ascenso —
    /// no dispara ningún cambio de estado por sí sola.
    pub fn puede_ascender(&self) -> bool {
        self.reputacion >= UMBRAL_ASCENSO_AUXILIAR
    }
}
```

- [ ] **Step 2: Agregar 3 tests dentro del `mod tests` ya existente (de la Tarea 2), después de `calcular_aplica_el_multiplicador_de_perks`**

```rust
    #[test]
    fn aplicar_resultado_acumula_dinero_reputacion_y_xp() {
        let mut estado = EstadoJugador::default();
        let resultado = Resultado {
            puntaje_base: 100.0,
            puntaje_final: 100.0,
            dinero_ganado: 100,
            reputacion_ganada: 0.5,
            xp_ganado: vec![(Arquetipo::Select, 10)],
        };

        estado.aplicar_resultado(&resultado);

        assert_eq!(estado.dinero, 100);
        assert_eq!(estado.reputacion, 0.5);
        assert_eq!(estado.xp_por_arquetipo, vec![(Arquetipo::Select, 10)]);
    }

    #[test]
    fn aplicar_resultado_suma_xp_al_mismo_arquetipo_en_llamadas_sucesivas() {
        let mut estado = EstadoJugador::default();
        let resultado = Resultado {
            puntaje_base: 100.0,
            puntaje_final: 100.0,
            dinero_ganado: 100,
            reputacion_ganada: 0.5,
            xp_ganado: vec![(Arquetipo::Select, 10)],
        };

        estado.aplicar_resultado(&resultado);
        estado.aplicar_resultado(&resultado);

        assert_eq!(estado.dinero, 200);
        assert_eq!(
            estado.xp_por_arquetipo,
            vec![(Arquetipo::Select, 20)],
            "debe acumular en la misma entrada, no duplicarla"
        );
    }

    #[test]
    fn puede_ascender_es_false_bajo_el_umbral_y_true_al_cruzarlo() {
        let mut estado = EstadoJugador::default();
        assert!(!estado.puede_ascender());

        estado.reputacion = 499.9;
        assert!(!estado.puede_ascender());

        estado.reputacion = 500.0;
        assert!(estado.puede_ascender());
    }
```

- [ ] **Step 3: Correr la suite completa**

Run: `cd app/src-tauri && cargo test --lib -- --nocapture`
Expected: todos los tests en verde, 0 failed, más los 3 nuevos.

- [ ] **Step 4: Commit**

```bash
git add app/src-tauri/src/economia/mod.rs
git commit -m "Add EstadoJugador: accumulated economy state and ascension threshold"
```

---

### Task 4: Conectar la economía real a la app

**Files:**
- Modify: `app/src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: `economia::{EstadoJugador, calcular}` (Tareas 2-3)
- Produces: `ScoreResult` expone el desglose completo de economía; `submit_ticket`/`unlock_perk` operan sobre `EstadoJugador` en vez del `PerkState` stub — mismos nombres de comando Tauri, mismo comportamiento externo de `unlock_perk`

- [ ] **Step 1: Leer el estado actual de `app/src-tauri/src/lib.rs` y confirmar que coincide con lo descrito abajo antes de editar**

- [ ] **Step 2: Reemplazar `PerkState`/`Perk` por `Jugador`**

Localizar:

```rust
/// Estado stub de economía/loadout (Etapa 12/13) — solo para probar la forma
/// del loop en el walking skeleton, sin persistencia entre sesiones.
#[derive(Default)]
struct PerkState {
    unlocked: bool,
    dinero: i64,
}

struct Perk(Mutex<PerkState>);
```

Reemplazar por:

```rust
/// Estado de economía del jugador (Etapa 12), gestionado por Tauri. El bool
/// de perk es el mismo stub heredado del spike — el sistema RPG real
/// (Etapa 13) lo reemplaza en un plan posterior.
struct Jugador(Mutex<economia::EstadoJugador>);
```

- [ ] **Step 3: Extender `ScoreResult` con el desglose de economía**

Localizar:

```rust
#[derive(serde::Serialize)]
struct ScoreResult {
    pass: bool,
    puntaje_correctitud: f64,
    puntaje_velocidad: f64,
    puntaje_practicas: f64,
    comentario_mentor: Option<String>,
    dinero_ganado: i64,
    dinero_total: i64,
    mensaje: String,
}
```

Reemplazar por:

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
    mensaje: String,
}
```

- [ ] **Step 4: Reemplazar `submit_ticket`**

Localizar el cuerpo actual (usa `perk: tauri::State<'_, Perk>` y la recompensa fija):

```rust
#[tauri::command]
async fn submit_ticket(
    state: tauri::State<'_, AppState>,
    perk: tauri::State<'_, Perk>,
    tickets: tauri::State<'_, Tickets>,
    sql: String,
) -> Result<ScoreResult, String> {
    let indice = *tickets.indice_actual.lock().unwrap();
    let sql_dorada = tickets.catalogo[indice].sql_dorada.clone();
    let requiere_orden = tickets.catalogo[indice].requiere_orden;

    let evaluacion = validation::evaluar_entrega(&state.pool, &sql, &sql_dorada, requiere_orden)
        .await
        .map_err(|e| e.to_string())?;

    let mut perk_state = perk.0.lock().unwrap();
    let dinero_ganado = if evaluacion.correcta { 500 } else { 0 };
    perk_state.dinero += dinero_ganado;

    if evaluacion.correcta {
        let mut indice_mut = tickets.indice_actual.lock().unwrap();
        *indice_mut = (*indice_mut + 1) % tickets.catalogo.len();
    }

    Ok(ScoreResult {
        pass: evaluacion.correcta,
        puntaje_correctitud: evaluacion.puntaje_correctitud,
        puntaje_velocidad: evaluacion.puntaje_velocidad,
        puntaje_practicas: evaluacion.puntaje_practicas,
        comentario_mentor: evaluacion.comentario_mentor.map(str::to_string),
        dinero_ganado,
        dinero_total: perk_state.dinero,
        mensaje: if evaluacion.correcta {
            "Ticket resuelto. Contabilidad procesará tu pago... eventualmente.".to_string()
        } else {
            "El resultado no coincide con lo que pidió la solicitud. Revisa tu consulta.".to_string()
        },
    })
}
```

Reemplazar por:

```rust
#[tauri::command]
async fn submit_ticket(
    state: tauri::State<'_, AppState>,
    jugador: tauri::State<'_, Jugador>,
    tickets: tauri::State<'_, Tickets>,
    sql: String,
) -> Result<ScoreResult, String> {
    let indice = *tickets.indice_actual.lock().unwrap();
    let ticket = tickets.catalogo[indice].clone();

    let evaluacion = validation::evaluar_entrega(&state.pool, &sql, &ticket.sql_dorada, ticket.requiere_orden)
        .await
        .map_err(|e| e.to_string())?;

    let resultado = economia::calcular(&evaluacion, &ticket, 1.0);

    let mut estado = jugador.0.lock().unwrap();
    estado.aplicar_resultado(&resultado);

    if evaluacion.correcta {
        let mut indice_mut = tickets.indice_actual.lock().unwrap();
        *indice_mut = (*indice_mut + 1) % tickets.catalogo.len();
    }

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
        mensaje: if evaluacion.correcta {
            "Ticket resuelto. Contabilidad procesará tu pago... eventualmente.".to_string()
        } else {
            "El resultado no coincide con lo que pidió la solicitud. Revisa tu consulta.".to_string()
        },
    })
}
```

- [ ] **Step 5: Reemplazar `unlock_perk`**

Localizar:

```rust
#[tauri::command]
fn unlock_perk(perk: tauri::State<'_, Perk>) -> Result<PerkStatus, String> {
    const COSTO: i64 = 300;
    let mut perk_state = perk.0.lock().unwrap();
    if perk_state.unlocked {
        return Ok(PerkStatus { unlocked: true, dinero_total: perk_state.dinero });
    }
    if perk_state.dinero < COSTO {
        return Err(format!(
            "No tienes suficiente dinero para este perk (cuesta {COSTO}, tienes {}).",
            perk_state.dinero
        ));
    }
    perk_state.dinero -= COSTO;
    perk_state.unlocked = true;
    Ok(PerkStatus { unlocked: true, dinero_total: perk_state.dinero })
}
```

Reemplazar por:

```rust
#[tauri::command]
fn unlock_perk(jugador: tauri::State<'_, Jugador>) -> Result<PerkStatus, String> {
    const COSTO: i64 = 300;
    let mut estado = jugador.0.lock().unwrap();
    if estado.perk_desbloqueado {
        return Ok(PerkStatus { unlocked: true, dinero_total: estado.dinero });
    }
    if estado.dinero < COSTO {
        return Err(format!(
            "No tienes suficiente dinero para este perk (cuesta {COSTO}, tienes {}).",
            estado.dinero
        ));
    }
    estado.dinero -= COSTO;
    estado.perk_desbloqueado = true;
    Ok(PerkStatus { unlocked: true, dinero_total: estado.dinero })
}
```

- [ ] **Step 6: Actualizar `.setup(...)` para registrar `Jugador` en vez de `Perk`**

Localizar:

```rust
                handle.manage(Perk(Mutex::new(PerkState::default())));
```

Reemplazar por:

```rust
                handle.manage(Jugador(Mutex::new(economia::EstadoJugador::default())));
```

- [ ] **Step 7: Verificar que compila sin advertencias**

Run: `cd app/src-tauri && cargo check`
Expected: `Finished` sin errores ni warnings.

- [ ] **Step 8: Correr la suite completa**

Run: `cd app/src-tauri && cargo test --lib -- --nocapture`
Expected: todos los tests en verde, 0 failed (sin tests nuevos en este paso — es wiring de `lib.rs`).

- [ ] **Step 9: Smoke test de la app real**

Run: `cd app && npm run tauri dev` en segundo plano; esperar a que compile y arranque (buscar `Running \`target/debug/app\`` en el log, sin pánico ni warnings, y confirmar con `ps aux` que el proceso vive). Detener limpiamente el proceso (`npm`/`tauri dev`/`target/debug/app`, y cualquier Postgres embebido que la app haya arrancado) antes de terminar.

- [ ] **Step 10: Commit**

```bash
git add app/src-tauri/src/lib.rs
git commit -m "Wire the real economy formula into submit_ticket/unlock_perk"
```

---

## Self-Review Notes

- **Cobertura del spec:** fórmula literal de la Etapa 12 (puntaje_base → puntaje_final → dinero/reputación/XP) ✓, multiplicador de perks fijo en 1.0 hasta que exista el sistema real (Etapa 13) ✓, reputación real trackeada + `puede_ascender()` expuesto sin disparar el ascenso (Etapa 10, deliberadamente recortado) ✓, stub de perk preservado sin cambio de comportamiento ✓.
- **Fuera de alcance deliberado (para planes posteriores):** sistema RPG/perks real y su multiplicador real (Etapa 13); evento de ascenso de rango, mini-boss, transición de empresa (Etapa 10/11-G); penalización de reputación por tickets escalados (Etapa 11-A, sistema de turnos).
- **Consistencia de tipos:** `Resultado`/`EstadoJugador` (Tareas 2-3) usan los mismos nombres de campo que `ScoreResult` expone en `lib.rs` (Tarea 4) — `dinero_ganado`, `reputacion_ganada`, `xp_ganado`, sin conversiones sorpresa.
- **Lección aplicada de planes anteriores:** los `#[allow(dead_code)]` que Task 1 agrega temporalmente a `valor_base`/`factor_reputacion` se retiran en la Tarea 2, en el mismo plan que les da un consumidor real — no se dejan para que una revisión final posterior los descubra.

---

## Execution Handoff

Plan completo y guardado en `docs/superpowers/plans/2026-07-12-fase0-04-economia.md`. Dos opciones de ejecución:

1. **Subagent-Driven (recomendado)** — despacho un subagente fresco por tarea, reviso el resultado entre cada una antes de seguir
2. **Ejecución inline** — ejecuto las tareas en esta sesión con executing-plans, ejecución por lotes con checkpoints

¿Cuál prefieres?
