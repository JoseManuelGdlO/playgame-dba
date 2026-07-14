# Ascenso de Rango (Becario → Auxiliar de Sistemas) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implementar el ascenso automático de Becario a Auxiliar de Sistemas: tipo de rango persistido en `EstadoJugador`, gating de tickets por rango en la bandeja, +1 slot de perk como premio, y la señal correspondiente en el frontend (badge de rango + anuncio en la pantalla de scoring).

**Architecture:** El rango vive en `economia::EstadoJugador` (nuevo campo `rango: tickets::Rango`) y el ascenso se dispara dentro de `EstadoJugador::aplicar_resultado` al cruzar el umbral de reputación ya existente (`UMBRAL_ASCENSO_AUXILIAR = 500.0`) — sin acción manual del jugador. El gating de tickets reutiliza la taxonomía `Arquetipo` que ya existe (`tickets::rango_requerido`, Enfoque A) y se aplica al armar cada turno nuevo en `lib.rs`, sin tocar la lógica de rotación de `turno::EstadoTurno`. El frontend solo consume campos ya serializados (`ScoreResult.rango_actual`/`ascendio`, comando nuevo `rango_actual`) sin lógica de negocio nueva del lado del cliente.

**Tech Stack:** Rust (backend Tauri), vanilla JS/HTML (frontend), Postgres embebido para validar las queries doradas de los tickets nuevos — sin dependencias nuevas.

## Global Constraints

- Umbral de ascenso: `reputacion >= 500.0` (constante ya existente `UMBRAL_ASCENSO_AUXILIAR` en `economia/mod.rs`) — no se modifica.
- Solo la condición de reputación dispara el ascenso en este plan; el mini-boss de empresa (Etapa 11-G) queda fuera de alcance y es un plan aparte.
- El ascenso es automático, sin botón ni confirmación del jugador.
- Slots de perk equipados: 2 para Becario, 3 para Auxiliar de Sistemas (Etapa 13).
- Gating de tickets (Enfoque A): un ticket requiere Auxiliar de Sistemas si `arquetipos` contiene `Join` o `Agregacion`; si solo tiene `Select`, es de Becario.
- Tamaño de lote por turno sigue fijo en 3 (`TAMANO_LOTE`, `turno/mod.rs`) — sin cambios.
- Spec de referencia: `docs/superpowers/specs/2026-07-13-fase0-07-ascenso-rango-design.md`.

---

### Task 1: Contenido — 2 tickets Select-only para que Becario tenga una bandeja real

El catálogo actual de Hospital Arcángel (6 tickets) solo tiene 1 ticket con `arquetipos: [Select]`. Con el gating de la Tarea 2, un Becario se quedaría con un solo ticket elegible — la bandeja de 3 (`TAMANO_LOTE`) colapsaría a un ticket repetido. Esta tarea agrega 2 tickets Select-only nuevos para que Becario tenga 3 tickets reales.

**Files:**
- Modify: `app/src-tauri/src/tickets/hospital_arcangel.rs`
- Modify: `app/src-tauri/src/tickets/mod.rs` (test de conteo por empresa)

**Interfaces:**
- Consumes: `plantilla_reporte_simple` (ya existe, sin cambios de firma).
- Produces: 2 tickets nuevos en el catálogo de `Company::HospitalArcangel`: `hospital_reporte_habitaciones_libres`, `hospital_reporte_pacientes_sin_alta` (ambos `arquetipos: vec![Arquetipo::Select]`). El catálogo de Hospital Arcángel pasa de 6 a 8 tickets; Postafeta no cambia (sigue en 6).

- [ ] **Step 1: Agregar los 2 tickets nuevos en `app/src-tauri/src/tickets/hospital_arcangel.rs`**

Localizar:

```rust
    vec![
        plantilla_reporte_simple(
            "hospital_reporte_pacientes_cardiologia",
            "Contabilidad",
            "Contabilidad quiere saber quién ha pisado Cardiología últimamente.",
            "Lista los pacientes admitidos en Cardiología (nombre, fecha de ingreso y diagnóstico), del más reciente al más antiguo.",
            "SELECT nombre, fecha_ingreso, diagnostico FROM pacientes WHERE departamento_id = 1 ORDER BY fecha_ingreso DESC",
            10,
        ),
        plantilla_reporte_agregado(
```

Reemplazar:

```rust
    vec![
        plantilla_reporte_simple(
            "hospital_reporte_pacientes_cardiologia",
            "Contabilidad",
            "Contabilidad quiere saber quién ha pisado Cardiología últimamente.",
            "Lista los pacientes admitidos en Cardiología (nombre, fecha de ingreso y diagnóstico), del más reciente al más antiguo.",
            "SELECT nombre, fecha_ingreso, diagnostico FROM pacientes WHERE departamento_id = 1 ORDER BY fecha_ingreso DESC",
            10,
        ),
        plantilla_reporte_simple(
            "hospital_reporte_habitaciones_libres",
            "Enfermería",
            "Enfermería necesita saber qué camas están libres antes de que llegue el siguiente turno de admisiones.",
            "Lista el número y el tipo de cada habitación que esté libre (no ocupada), ordenadas por número.",
            "SELECT numero, tipo FROM habitaciones WHERE ocupada = false ORDER BY numero",
            10,
        ),
        plantilla_reporte_simple(
            "hospital_reporte_pacientes_sin_alta",
            "Auditoría de Calidad",
            "Auditoría de Calidad necesita confirmar cuántos pacientes siguen internados para su reporte semanal de ocupación.",
            "Lista el nombre y la fecha de ingreso de los pacientes que todavía no tienen fecha de alta, del ingreso más antiguo al más reciente.",
            "SELECT nombre, fecha_ingreso FROM pacientes WHERE fecha_alta IS NULL ORDER BY fecha_ingreso, nombre",
            10,
        ),
        plantilla_reporte_agregado(
```

- [ ] **Step 2: Actualizar el test de conteo en el mismo archivo**

Localizar:

```rust
    #[test]
    fn catalogo_tiene_4_reportes_y_2_depuraciones() {
        let tickets = catalogo();
        assert_eq!(tickets.len(), 6);
        let reportes = tickets.iter().filter(|t| t.tipo == TipoTicket::ReporteAnalisis).count();
        let depuraciones = tickets.iter().filter(|t| t.tipo == TipoTicket::InvestigacionDepuracion).count();
        assert_eq!(reportes, 4);
        assert_eq!(depuraciones, 2);
    }
```

Reemplazar:

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

- [ ] **Step 3: Actualizar el test de conteo por empresa en `app/src-tauri/src/tickets/mod.rs`**

Localizar:

```rust
    #[test]
    fn catalogo_devuelve_6_tickets_para_cada_empresa() {
        assert_eq!(catalogo(crate::db::Company::HospitalArcangel).len(), 6);
        assert_eq!(catalogo(crate::db::Company::Postafeta).len(), 6);
    }
```

Reemplazar:

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

- [ ] **Step 4: Correr la suite completa**

Run: `cd app/src-tauri && cargo test --lib -- --nocapture`
Expected: 0 failed. Los tests de integración `todas_las_queries_doradas_ejecutan` (mismo archivo, requiere Postgres embebido) también deben pasar — recorren `catalogo()` genéricamente, así que ya validan que las 2 queries nuevas ejecutan correctamente contra el esquema real.

- [ ] **Step 5: Commit**

```bash
git add app/src-tauri/src/tickets/hospital_arcangel.rs app/src-tauri/src/tickets/mod.rs
git commit -m "Add 2 Select-only tickets to Hospital Arcángel so Becario has a real inbox"
```

---

### Task 2: Modelo de `Rango` y gating de tickets (`tickets/mod.rs`)

**Files:**
- Modify: `app/src-tauri/src/tickets/mod.rs`

**Interfaces:**
- Consumes: `Ticket.arquetipos: Vec<Arquetipo>` (ya existe).
- Produces: `pub enum Rango { Becario, AuxiliarDeSistemas }` (Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize — `Becario` es el default y el menor en el orden), `pub fn rango_requerido(ticket: &Ticket) -> Rango`.

- [ ] **Step 1: Agregar `Rango` y `rango_requerido` después del enum `Arquetipo`**

Localizar:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Arquetipo {
    Select,
    Join,
    Agregacion,
}
```

Reemplazar:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Arquetipo {
    Select,
    Join,
    Agregacion,
}

/// Rango de carrera del jugador (Etapa 10, Plan 7): determina qué tickets
/// del catálogo puede recibir en su bandeja. El orden de declaración importa
/// — el derive de `Ord` decide qué rango "alcanza" a cuál según ese orden.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, serde::Serialize)]
pub enum Rango {
    #[default]
    Becario,
    AuxiliarDeSistemas,
}

/// Etapa 10/Plan 7: un ticket requiere Auxiliar de Sistemas si su solución
/// necesita JOIN o agregación — Becario solo domina SELECT/WHERE/ORDER BY.
pub fn rango_requerido(ticket: &Ticket) -> Rango {
    let necesita_auxiliar = ticket
        .arquetipos
        .iter()
        .any(|a| matches!(a, Arquetipo::Join | Arquetipo::Agregacion));
    if necesita_auxiliar {
        Rango::AuxiliarDeSistemas
    } else {
        Rango::Becario
    }
}
```

- [ ] **Step 2: Agregar tests dentro del `mod tests` ya existente de ese mismo archivo**

Agregar (en cualquier punto antes del cierre `}` del módulo):

```rust
    #[test]
    fn rango_becario_es_menor_que_auxiliar_de_sistemas() {
        assert!(Rango::Becario < Rango::AuxiliarDeSistemas);
    }

    #[test]
    fn rango_requerido_es_becario_para_tickets_solo_select() {
        let ticket = plantilla_reporte_simple("id_becario", "Alguien", "un motivo", "una solicitud", "SELECT 1", 10);
        assert_eq!(rango_requerido(&ticket), Rango::Becario);
    }

    #[test]
    fn rango_requerido_es_auxiliar_si_incluye_join_o_agregacion() {
        let con_join = plantilla_reporte_join("id_join", "Alguien", "un motivo", "una solicitud", "SELECT 1", 10);
        let con_agregacion =
            plantilla_reporte_agregado("id_agg", "Alguien", "un motivo", "una solicitud", "SELECT 1", 10);
        let con_ambos =
            plantilla_reporte_join_agregado("id_both", "Alguien", "un motivo", "una solicitud", "SELECT 1", 10);

        assert_eq!(rango_requerido(&con_join), Rango::AuxiliarDeSistemas);
        assert_eq!(rango_requerido(&con_agregacion), Rango::AuxiliarDeSistemas);
        assert_eq!(rango_requerido(&con_ambos), Rango::AuxiliarDeSistemas);
    }

    #[test]
    fn catalogo_de_hospital_arcangel_tiene_3_tickets_elegibles_para_becario() {
        let elegibles = catalogo(crate::db::Company::HospitalArcangel)
            .into_iter()
            .filter(|t| rango_requerido(t) <= Rango::Becario)
            .count();
        assert_eq!(elegibles, 3, "el ticket original de Select + los 2 agregados en la Tarea 1");
    }
```

- [ ] **Step 3: Correr la suite completa**

Run: `cd app/src-tauri && cargo test --lib -- --nocapture`
Expected: 0 failed, incluyendo los 4 tests nuevos de este paso.

- [ ] **Step 4: Commit**

```bash
git add app/src-tauri/src/tickets/mod.rs
git commit -m "Add Rango model and rango_requerido ticket gating (Stage 10)"
```

---

### Task 3: Ascenso automático + slots dinámicos (`economia/mod.rs`)

**Files:**
- Modify: `app/src-tauri/src/economia/mod.rs`

**Interfaces:**
- Consumes: `tickets::Rango` (Tarea 2).
- Produces: `EstadoJugador.rango: tickets::Rango`, `EstadoJugador::aplicar_resultado(&mut self, &Resultado) -> bool` (cambia de firma: ahora devuelve `true` solo en la entrega exacta en que ocurre el ascenso), `EstadoJugador::max_slots(&self) -> usize`.

- [ ] **Step 1: Importar `Rango`**

Localizar:

```rust
use crate::perks::{Efecto, Perk};
use crate::tickets::{Arquetipo, Ticket};
use crate::validation::Evaluacion;
```

Reemplazar:

```rust
use crate::perks::{Efecto, Perk};
use crate::tickets::{Arquetipo, Rango, Ticket};
use crate::validation::Evaluacion;
```

- [ ] **Step 2: Agregar el campo `rango`, disparar el ascenso en `aplicar_resultado`, y agregar `max_slots`**

Localizar:

```rust
/// Máximo de perks equipados simultáneamente (Etapa 19, MVP): 2 slots fijos.
/// La escalera de hasta 7 slots por hito de rango (Etapa 13 completa) es de
/// un plan posterior.
const MAX_SLOTS_EQUIPADOS: usize = 2;

/// Estado acumulado del jugador (Etapa 12/13): dinero, reputación, XP por
/// arquetipo, y los perks desbloqueados/equipados.
#[derive(Debug, Clone, Default)]
pub struct EstadoJugador {
    pub dinero: i64,
    pub reputacion: f64,
    pub xp_por_arquetipo: Vec<(Arquetipo, i64)>,
    pub perks_desbloqueados: Vec<&'static str>,
    pub perks_equipados: Vec<&'static str>,
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
```

Reemplazar:

```rust
/// Estado acumulado del jugador (Etapa 12/13): dinero, reputación, XP por
/// arquetipo, rango de carrera, y los perks desbloqueados/equipados.
#[derive(Debug, Clone, Default)]
pub struct EstadoJugador {
    pub dinero: i64,
    pub reputacion: f64,
    pub xp_por_arquetipo: Vec<(Arquetipo, i64)>,
    pub rango: Rango,
    pub perks_desbloqueados: Vec<&'static str>,
    pub perks_equipados: Vec<&'static str>,
}

/// Umbral de reputación para ascender de Becario a Auxiliar de Sistemas en
/// Hospital Arcángel (Etapa 10). El ascenso de este plan usa únicamente esta
/// condición — el mini-boss de la empresa (Etapa 11-G) queda fuera de
/// alcance y se puede añadir como condición adicional en un plan posterior
/// sin romper esta lógica (Plan 7).
const UMBRAL_ASCENSO_AUXILIAR: f64 = 500.0;

impl EstadoJugador {
    /// Aplica el resultado de una entrega (Etapa 12): acumula dinero,
    /// reputación y XP por arquetipo sobre el estado existente, y dispara el
    /// ascenso automático de rango si la reputación acumulada ya cruzó el
    /// umbral (Etapa 10, Plan 7). Devuelve `true` solo en la entrega exacta
    /// en la que el ascenso ocurre, para que el llamador pueda anunciarlo.
    pub fn aplicar_resultado(&mut self, resultado: &Resultado) -> bool {
        self.dinero += resultado.dinero_ganado;
        self.reputacion += resultado.reputacion_ganada;
        for &(arquetipo, xp) in &resultado.xp_ganado {
            match self.xp_por_arquetipo.iter_mut().find(|(a, _)| *a == arquetipo) {
                Some((_, existente)) => *existente += xp,
                None => self.xp_por_arquetipo.push((arquetipo, xp)),
            }
        }
        if self.rango == Rango::Becario && self.puede_ascender() {
            self.rango = Rango::AuxiliarDeSistemas;
            true
        } else {
            false
        }
    }

    /// Etapa 10: señal de que la reputación ya cruzó el umbral de ascenso —
    /// no dispara ningún cambio de estado por sí sola.
    pub fn puede_ascender(&self) -> bool {
        self.reputacion >= UMBRAL_ASCENSO_AUXILIAR
    }

    /// Máximo de perks equipados simultáneamente (Etapa 13, Plan 7): 2 slots
    /// para Becario, 3 para Auxiliar de Sistemas (hito de slot de esa
    /// etapa). La escalera completa de hasta 7 slots por rango es de un plan
    /// posterior, junto con más rangos (Fase 1+).
    pub fn max_slots(&self) -> usize {
        match self.rango {
            Rango::Becario => 2,
            Rango::AuxiliarDeSistemas => 3,
        }
    }
```

- [ ] **Step 3: Usar `max_slots()` en `equipar_perk` en vez de la constante fija**

Localizar:

```rust
    /// Equipa un perk ya desbloqueado (Etapa 11-D: equipar es gratis).
    /// Falla si no está desbloqueado, o si ya se ocuparon los 2 slots.
    /// Idempotente si ya estaba equipado.
    pub fn equipar_perk(&mut self, id: &str) -> Result<(), String> {
        if !self.perks_desbloqueados.contains(&id) {
            return Err(format!("'{id}' no está desbloqueado todavía."));
        }
        if self.perks_equipados.contains(&id) {
            return Ok(());
        }
        if self.perks_equipados.len() >= MAX_SLOTS_EQUIPADOS {
            return Err(format!(
                "Ya tienes {MAX_SLOTS_EQUIPADOS} perks equipados — desequipa uno primero."
            ));
        }
```

Reemplazar:

```rust
    /// Equipa un perk ya desbloqueado (Etapa 11-D: equipar es gratis).
    /// Falla si no está desbloqueado, o si ya se ocuparon los slots
    /// disponibles para el rango actual (Etapa 13, Plan 7). Idempotente si
    /// ya estaba equipado.
    pub fn equipar_perk(&mut self, id: &str) -> Result<(), String> {
        if !self.perks_desbloqueados.contains(&id) {
            return Err(format!("'{id}' no está desbloqueado todavía."));
        }
        if self.perks_equipados.contains(&id) {
            return Ok(());
        }
        let max_slots = self.max_slots();
        if self.perks_equipados.len() >= max_slots {
            return Err(format!("Ya tienes {max_slots} perks equipados — desequipa uno primero."));
        }
```

- [ ] **Step 4: Agregar tests dentro del `mod tests` ya existente de ese mismo archivo**

Agregar (en cualquier punto antes del cierre `}` del módulo):

```rust
    #[test]
    fn aplicar_resultado_asciende_a_auxiliar_al_cruzar_el_umbral_una_sola_vez() {
        let mut estado = EstadoJugador::default();
        assert_eq!(estado.rango, Rango::Becario);
        let resultado = Resultado {
            puntaje_base: 100.0,
            puntaje_final: 100.0,
            dinero_ganado: 100,
            reputacion_ganada: 500.0,
            xp_ganado: vec![],
        };

        let ascendio = estado.aplicar_resultado(&resultado);

        assert!(ascendio, "debe ascender en la entrega exacta que cruza el umbral");
        assert_eq!(estado.rango, Rango::AuxiliarDeSistemas);

        let resultado_siguiente = Resultado {
            puntaje_base: 100.0,
            puntaje_final: 100.0,
            dinero_ganado: 50,
            reputacion_ganada: 10.0,
            xp_ganado: vec![],
        };
        let ascendio_de_nuevo = estado.aplicar_resultado(&resultado_siguiente);
        assert!(!ascendio_de_nuevo, "ya ascendido, no debe volver a dispararse");
    }

    #[test]
    fn max_slots_es_2_para_becario_y_3_para_auxiliar_de_sistemas() {
        let mut estado = EstadoJugador::default();
        assert_eq!(estado.max_slots(), 2);

        estado.rango = Rango::AuxiliarDeSistemas;
        assert_eq!(estado.max_slots(), 3);
    }

    #[test]
    fn equipar_perk_permite_un_tercer_slot_para_auxiliar_de_sistemas() {
        let mut estado = EstadoJugador::default();
        estado.rango = Rango::AuxiliarDeSistemas;
        estado.perks_desbloqueados = vec!["instinto", "rayos_x", "piloto_automatico"];

        estado.equipar_perk("instinto").unwrap();
        estado.equipar_perk("rayos_x").unwrap();
        estado.equipar_perk("piloto_automatico").unwrap();

        assert_eq!(estado.perks_equipados, vec!["instinto", "rayos_x", "piloto_automatico"]);
    }
```

- [ ] **Step 5: Correr la suite completa**

Run: `cd app/src-tauri && cargo test --lib -- --nocapture`
Expected: 0 failed. El test ya existente `equipar_perk_respeta_el_limite_de_2_slots` debe seguir pasando sin cambios (Becario sigue en 2 por default).

- [ ] **Step 6: Commit**

```bash
git add app/src-tauri/src/economia/mod.rs
git commit -m "Add automatic rank ascension and rank-scaled perk slots (Stage 10/13)"
```

---

### Task 4: `turno::EstadoTurno::nuevo` tolera catálogos filtrados de tamaño variable

A partir de la Tarea 5, `EstadoTurno::nuevo` recibirá slices filtrados por rango en vez del catálogo completo siempre del mismo tamaño. `indice_inicial` puede quedar fuera de rango si el tamaño del slice cambió desde el turno anterior (por ejemplo, justo después de un ascenso). Esta tarea es independiente y se puede probar sin tocar `lib.rs`.

**Files:**
- Modify: `app/src-tauri/src/turno/mod.rs`

**Interfaces:**
- Produces: `EstadoTurno::nuevo` ya no puede entrar en pánico por índice fuera de rango ni por un catálogo vacío (antes asumía implícitamente `indice_inicial < catalogo.len()`).

- [ ] **Step 1: Normalizar `indice_inicial` con módulo y cubrir el catálogo vacío**

Localizar:

```rust
    pub fn nuevo(catalogo: &[Ticket], indice_inicial: usize) -> (Self, usize) {
        let tamano = TAMANO_LOTE.min(catalogo.len());
        let mut pendientes = Vec::with_capacity(tamano);
        let mut indice = indice_inicial;
        for _ in 0..tamano {
            pendientes.push(catalogo[indice].clone());
            indice = (indice + 1) % catalogo.len();
        }
        (
            EstadoTurno {
                presupuesto_restante: PRESUPUESTO_POR_TURNO,
                pendientes,
            },
            indice,
        )
    }
```

Reemplazar:

```rust
    pub fn nuevo(catalogo: &[Ticket], indice_inicial: usize) -> (Self, usize) {
        let tamano = TAMANO_LOTE.min(catalogo.len());
        let mut pendientes = Vec::with_capacity(tamano);
        // Plan 7: `catalogo` ya no es siempre el catálogo completo de la
        // empresa — el gating por rango (`lib.rs`) puede pasar un slice
        // filtrado de tamaño distinto entre turnos. Normalizar con módulo
        // evita un panic por índice fuera de rango; un catálogo vacío no
        // debe darse en la práctica (Becario siempre tiene tickets
        // elegibles), pero se cubre explícitamente para no dejar un panic
        // latente.
        let mut indice = if catalogo.is_empty() { 0 } else { indice_inicial % catalogo.len() };
        for _ in 0..tamano {
            pendientes.push(catalogo[indice].clone());
            indice = (indice + 1) % catalogo.len();
        }
        (
            EstadoTurno {
                presupuesto_restante: PRESUPUESTO_POR_TURNO,
                pendientes,
            },
            indice,
        )
    }
```

- [ ] **Step 2: Agregar tests dentro del `mod tests` ya existente de ese mismo archivo**

Agregar (en cualquier punto antes del cierre `}` del módulo):

```rust
    #[test]
    fn nuevo_normaliza_un_indice_inicial_fuera_de_rango_con_modulo() {
        let catalogo = catalogo_de_prueba();
        let (turno, _) = EstadoTurno::nuevo(&catalogo, 7);
        assert_eq!(
            turno.pendientes.iter().map(|t| t.id).collect::<Vec<_>>(),
            vec!["t3", "t4", "t5"],
            "7 % 5 == 2, debe empezar en t3"
        );
    }

    #[test]
    fn nuevo_no_entra_en_panico_con_catalogo_vacio() {
        let catalogo: Vec<Ticket> = vec![];
        let (turno, siguiente_indice) = EstadoTurno::nuevo(&catalogo, 0);
        assert!(turno.pendientes.is_empty());
        assert_eq!(siguiente_indice, 0);
    }
```

- [ ] **Step 3: Correr la suite completa**

Run: `cd app/src-tauri && cargo test --lib -- --nocapture`
Expected: 0 failed, incluyendo los 2 tests nuevos.

- [ ] **Step 4: Commit**

```bash
git add app/src-tauri/src/turno/mod.rs
git commit -m "Make EstadoTurno::nuevo tolerate variably-sized filtered catalogs"
```

---

### Task 5: Wiring en `lib.rs` — gating por rango, comando `rango_actual`, `ScoreResult`

**Files:**
- Modify: `app/src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: `tickets::Rango`, `tickets::rango_requerido` (Tarea 2), `EstadoJugador.rango`, `EstadoJugador::aplicar_resultado -> bool` (Tarea 3), `EstadoTurno::nuevo` tolerante a tamaño variable (Tarea 4).
- Produces: comando `rango_actual() -> tickets::Rango`; `ScoreResult.ascendio: bool` y `ScoreResult.rango_actual: tickets::Rango` nuevos.

- [ ] **Step 1: Filtrar por rango en `TurnoManejado::escalar_y_avanzar`**

Localizar:

```rust
impl TurnoManejado {
    /// Escala los tickets pendientes del turno actual (penaliza reputación)
    /// y arranca el turno siguiente — usado tanto cuando el presupuesto se
    /// agota como cuando el jugador cierra el día manualmente (Etapa 11-A).
    fn escalar_y_avanzar(&mut self, jugador: &mut economia::EstadoJugador) {
        for escalamiento in self.actual.escalar_pendientes() {
            jugador.aplicar_penalizacion(escalamiento.reputacion_perdida);
        }
        let (nuevo_turno, siguiente_indice) = turno::EstadoTurno::nuevo(&self.catalogo, self.indice_siguiente);
        self.actual = nuevo_turno;
        self.indice_siguiente = siguiente_indice;
    }
}
```

Reemplazar:

```rust
impl TurnoManejado {
    /// Escala los tickets pendientes del turno actual (penaliza reputación)
    /// y arranca el turno siguiente — usado tanto cuando el presupuesto se
    /// agota como cuando el jugador cierra el día manualmente (Etapa 11-A).
    /// El lote nuevo se filtra por el rango actual del jugador (Etapa 10,
    /// Plan 7): un ascenso a mitad de turno no reordena la bandeja ya
    /// mostrada, pero el turno siguiente ya refleja el catálogo desbloqueado.
    fn escalar_y_avanzar(&mut self, jugador: &mut economia::EstadoJugador) {
        for escalamiento in self.actual.escalar_pendientes() {
            jugador.aplicar_penalizacion(escalamiento.reputacion_perdida);
        }
        let elegibles: Vec<tickets::Ticket> = self
            .catalogo
            .iter()
            .filter(|t| tickets::rango_requerido(t) <= jugador.rango)
            .cloned()
            .collect();
        let (nuevo_turno, siguiente_indice) = turno::EstadoTurno::nuevo(&elegibles, self.indice_siguiente);
        self.actual = nuevo_turno;
        self.indice_siguiente = siguiente_indice;
    }
}
```

- [ ] **Step 2: Agregar los campos nuevos a `ScoreResult`**

Localizar:

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
    mensaje: String,
}
```

Reemplazar:

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

- [ ] **Step 3: Capturar `ascendio` y poblar los campos nuevos en `resolver_ticket`**

Localizar:

```rust
    let mut estado = jugador.0.lock().unwrap();
    let multiplicador_dinero = estado.multiplicador_dinero(perks::catalogo());
    let multiplicador_reputacion = estado.multiplicador_reputacion(perks::catalogo());
    let resultado = economia::calcular(&evaluacion, &ticket, multiplicador_dinero, multiplicador_reputacion);
    estado.aplicar_resultado(&resultado);

    let mut manejado = turno_state.0.lock().unwrap();
    if manejado.actual.pendientes.is_empty() || manejado.actual.turno_agotado() {
        manejado.escalar_y_avanzar(&mut estado);
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
        puede_ascender: estado.puede_ascender(),
        mensaje: if evaluacion.correcta {
            "Ticket resuelto. Contabilidad procesará tu pago... eventualmente.".to_string()
        } else {
            "El resultado no coincide con lo que pidió la solicitud. Revisa tu consulta.".to_string()
        },
    })
}
```

Reemplazar:

```rust
    let mut estado = jugador.0.lock().unwrap();
    let multiplicador_dinero = estado.multiplicador_dinero(perks::catalogo());
    let multiplicador_reputacion = estado.multiplicador_reputacion(perks::catalogo());
    let resultado = economia::calcular(&evaluacion, &ticket, multiplicador_dinero, multiplicador_reputacion);
    let ascendio = estado.aplicar_resultado(&resultado);

    let mut manejado = turno_state.0.lock().unwrap();
    if manejado.actual.pendientes.is_empty() || manejado.actual.turno_agotado() {
        manejado.escalar_y_avanzar(&mut estado);
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
        puede_ascender: estado.puede_ascender(),
        ascendio,
        rango_actual: estado.rango,
        mensaje: if evaluacion.correcta {
            "Ticket resuelto. Contabilidad procesará tu pago... eventualmente.".to_string()
        } else {
            "El resultado no coincide con lo que pidió la solicitud. Revisa tu consulta.".to_string()
        },
    })
}
```

- [ ] **Step 4: Agregar el comando `rango_actual` y registrarlo**

Localizar:

```rust
#[tauri::command]
fn turno_actual(turno: tauri::State<'_, Turno>) -> turno::EstadoTurno {
    turno.0.lock().unwrap().actual.clone()
}

#[tauri::command]
async fn run_query(state: tauri::State<'_, AppState>, sql: String) -> Result<db::QueryResult, String> {
```

Reemplazar:

```rust
#[tauri::command]
fn turno_actual(turno: tauri::State<'_, Turno>) -> turno::EstadoTurno {
    turno.0.lock().unwrap().actual.clone()
}

/// Etapa 10, Plan 7: expone el rango vigente para que el frontend pinte el
/// badge apenas carga, sin depender de haber resuelto un ticket primero.
#[tauri::command]
fn rango_actual(jugador: tauri::State<'_, Jugador>) -> tickets::Rango {
    jugador.0.lock().unwrap().rango
}

#[tauri::command]
async fn run_query(state: tauri::State<'_, AppState>, sql: String) -> Result<db::QueryResult, String> {
```

Localizar:

```rust
        .invoke_handler(tauri::generate_handler![
            turno_actual,
            run_query,
            resolver_ticket,
            cerrar_dia,
            catalogo_perks,
            desbloquear_perk,
            equipar_perk,
            desequipar_perk
        ])
```

Reemplazar:

```rust
        .invoke_handler(tauri::generate_handler![
            turno_actual,
            rango_actual,
            run_query,
            resolver_ticket,
            cerrar_dia,
            catalogo_perks,
            desbloquear_perk,
            equipar_perk,
            desequipar_perk
        ])
```

- [ ] **Step 5: Filtrar por rango el turno inicial en `setup()`**

Localizar:

```rust
                let catalogo = tickets::catalogo(db::Company::HospitalArcangel);
                let (turno_inicial, indice_siguiente) = turno::EstadoTurno::nuevo(&catalogo, 0);
                handle.manage(AppState { pool });
                handle.manage(Jugador(Mutex::new(economia::EstadoJugador::default())));
                handle.manage(Turno(Mutex::new(TurnoManejado {
                    catalogo,
                    indice_siguiente,
                    actual: turno_inicial,
                })));
```

Reemplazar:

```rust
                let catalogo = tickets::catalogo(db::Company::HospitalArcangel);
                let jugador_inicial = economia::EstadoJugador::default();
                // Etapa 10, Plan 7: el turno inicial ya se filtra por rango —
                // un Becario recién llegado no debe ver tickets de
                // Join/Agregación en su primera bandeja.
                let elegibles: Vec<tickets::Ticket> = catalogo
                    .iter()
                    .filter(|t| tickets::rango_requerido(t) <= jugador_inicial.rango)
                    .cloned()
                    .collect();
                let (turno_inicial, indice_siguiente) = turno::EstadoTurno::nuevo(&elegibles, 0);
                handle.manage(AppState { pool });
                handle.manage(Jugador(Mutex::new(jugador_inicial)));
                handle.manage(Turno(Mutex::new(TurnoManejado {
                    catalogo,
                    indice_siguiente,
                    actual: turno_inicial,
                })));
```

- [ ] **Step 6: Compilar y correr la suite completa**

Run: `cd app/src-tauri && cargo build && cargo test --lib -- --nocapture`
Expected: build sin errores, 0 tests failed. `lib.rs` no tiene su propio `mod tests` — esta tarea se valida por compilación exitosa más los tests de `economia`/`tickets`/`turno` de las tareas anteriores; la verificación end-to-end manual queda para después de la Tarea 6, igual que se hizo con el Plan 6.

- [ ] **Step 7: Commit**

```bash
git add app/src-tauri/src/lib.rs
git commit -m "Wire rank gating, ascension signal, and rango_actual command into lib.rs"
```

---

### Task 6: Frontend — badge de rango y anuncio de ascenso

**Files:**
- Modify: `app/src/index.html`
- Modify: `app/src/main.js`

**Interfaces:**
- Consumes: comando `rango_actual` (Tarea 5), `ScoreResult.rango_actual`/`ascendio` (Tarea 5).
- Produces: badge de rango visible en el header; bloque de anuncio en la pantalla de scoring cuando `ascendio == true`.

- [ ] **Step 1: Agregar el badge de rango al header en `app/src/index.html`**

Localizar:

```html
        <div class="stats">
          <span>💰 <span id="dinero">0</span></span>
          <span>⭐ <span id="reputacion">0</span></span>
        </div>
```

Reemplazar:

```html
        <div class="stats">
          <span>💰 <span id="dinero">0</span></span>
          <span>⭐ <span id="reputacion">0</span></span>
          <span>🎓 <span id="rango">Becario</span></span>
        </div>
```

- [ ] **Step 2: Agregar el párrafo de anuncio de ascenso en la pantalla de scoring, mismo archivo**

Localizar:

```html
        <p>⭐ +<span id="scoring-reputacion">0</span></p>
        <p id="scoring-mentor"></p>
        <button id="btn-cerrar-scoring">Cerrar</button>
```

Reemplazar:

```html
        <p>⭐ +<span id="scoring-reputacion">0</span></p>
        <p id="scoring-mentor"></p>
        <p id="scoring-ascenso"></p>
        <button id="btn-cerrar-scoring">Cerrar</button>
```

- [ ] **Step 3: Declarar las variables nuevas y el mapeo de nombres en `app/src/main.js`**

Localizar:

```js
let sqlInput, statusMsg, resultTable, dineroEl, reputacionEl;
let perksSelect, perksEquipadosMsg;
let presupuestoEl, listaTickets, ticketActivoInfo;
let scoringOverlay;
```

Reemplazar:

```js
let sqlInput, statusMsg, resultTable, dineroEl, reputacionEl, rangoEl;
let perksSelect, perksEquipadosMsg;
let presupuestoEl, listaTickets, ticketActivoInfo;
let scoringOverlay, scoringAscenso;

const NOMBRE_RANGO = {
  Becario: "Becario",
  AuxiliarDeSistemas: "Auxiliar de Sistemas",
};

function renderRango(rango) {
  rangoEl.textContent = NOMBRE_RANGO[rango] || rango;
}
```

- [ ] **Step 4: Agregar `cargarRango`, mismo archivo**

Localizar:

```js
async function cargarTurno() {
  const estadoTurno = await invoke("turno_actual");
  renderBandeja(estadoTurno);
}
```

Reemplazar:

```js
async function cargarTurno() {
  const estadoTurno = await invoke("turno_actual");
  renderBandeja(estadoTurno);
}

async function cargarRango() {
  const rango = await invoke("rango_actual");
  renderRango(rango);
}
```

- [ ] **Step 5: Mostrar el anuncio de ascenso en `mostrarScoring`, mismo archivo**

Localizar:

```js
function mostrarScoring(score) {
  document.querySelector("#scoring-titulo").textContent = score.pass ? "✅ Resuelto" : "❌ Incorrecto";
  animarNumero(document.querySelector("#scoring-correctitud"), score.puntaje_correctitud, 0);
  animarNumero(document.querySelector("#scoring-velocidad"), score.puntaje_velocidad, 0);
  animarNumero(document.querySelector("#scoring-practicas"), score.puntaje_practicas, 0);
  animarNumero(document.querySelector("#scoring-dinero"), score.dinero_ganado, 0);
  animarNumero(document.querySelector("#scoring-reputacion"), score.reputacion_ganada, 1);
  document.querySelector("#scoring-mentor").textContent = score.comentario_mentor || "";
  scoringOverlay.classList.remove("oculto");
}
```

Reemplazar:

```js
function mostrarScoring(score) {
  document.querySelector("#scoring-titulo").textContent = score.pass ? "✅ Resuelto" : "❌ Incorrecto";
  animarNumero(document.querySelector("#scoring-correctitud"), score.puntaje_correctitud, 0);
  animarNumero(document.querySelector("#scoring-velocidad"), score.puntaje_velocidad, 0);
  animarNumero(document.querySelector("#scoring-practicas"), score.puntaje_practicas, 0);
  animarNumero(document.querySelector("#scoring-dinero"), score.dinero_ganado, 0);
  animarNumero(document.querySelector("#scoring-reputacion"), score.reputacion_ganada, 1);
  document.querySelector("#scoring-mentor").textContent = score.comentario_mentor || "";
  scoringAscenso.textContent = score.ascendio
    ? `¡Ascendiste a ${NOMBRE_RANGO[score.rango_actual] || score.rango_actual}! +1 slot de perk. Nuevos tickets disponibles.`
    : "";
  scoringOverlay.classList.remove("oculto");
}
```

- [ ] **Step 6: Actualizar el badge tras cada envío en `submitTicket`, mismo archivo**

Localizar:

```js
async function submitTicket() {
  if (!ticketActivoId) {
    setStatus("Elige un ticket de la bandeja primero.", "error");
    return;
  }
  setStatus("Enviando ticket...", "");
  try {
    const score = await invoke("resolver_ticket", { id: ticketActivoId, sql: sqlInput.value });
    dineroEl.textContent = score.dinero_total;
    reputacionEl.textContent = score.reputacion_total.toFixed(1);
    mostrarScoring(score);
    setStatus(score.mensaje, score.pass ? "ok" : "error");
    await cargarTurno();
  } catch (err) {
    setStatus(String(err), "error");
  }
}
```

Reemplazar:

```js
async function submitTicket() {
  if (!ticketActivoId) {
    setStatus("Elige un ticket de la bandeja primero.", "error");
    return;
  }
  setStatus("Enviando ticket...", "");
  try {
    const score = await invoke("resolver_ticket", { id: ticketActivoId, sql: sqlInput.value });
    dineroEl.textContent = score.dinero_total;
    reputacionEl.textContent = score.reputacion_total.toFixed(1);
    renderRango(score.rango_actual);
    mostrarScoring(score);
    setStatus(score.mensaje, score.pass ? "ok" : "error");
    await cargarTurno();
  } catch (err) {
    setStatus(String(err), "error");
  }
}
```

- [ ] **Step 7: Enlazar los elementos nuevos y cargar el rango al iniciar, en `DOMContentLoaded`**

Localizar:

```js
window.addEventListener("DOMContentLoaded", async () => {
  sqlInput = document.querySelector("#sql-input");
  statusMsg = document.querySelector("#status-msg");
  resultTable = document.querySelector("#result-table");
  dineroEl = document.querySelector("#dinero");
  reputacionEl = document.querySelector("#reputacion");
  perksSelect = document.querySelector("#perks-select");
  perksEquipadosMsg = document.querySelector("#perks-equipados-msg");
  presupuestoEl = document.querySelector("#presupuesto");
  listaTickets = document.querySelector("#lista-tickets");
  ticketActivoInfo = document.querySelector("#ticket-activo-info");
  scoringOverlay = document.querySelector("#scoring-overlay");

  await cargarTurno();
  await cargarPerks();
```

Reemplazar:

```js
window.addEventListener("DOMContentLoaded", async () => {
  sqlInput = document.querySelector("#sql-input");
  statusMsg = document.querySelector("#status-msg");
  resultTable = document.querySelector("#result-table");
  dineroEl = document.querySelector("#dinero");
  reputacionEl = document.querySelector("#reputacion");
  rangoEl = document.querySelector("#rango");
  perksSelect = document.querySelector("#perks-select");
  perksEquipadosMsg = document.querySelector("#perks-equipados-msg");
  presupuestoEl = document.querySelector("#presupuesto");
  listaTickets = document.querySelector("#lista-tickets");
  ticketActivoInfo = document.querySelector("#ticket-activo-info");
  scoringOverlay = document.querySelector("#scoring-overlay");
  scoringAscenso = document.querySelector("#scoring-ascenso");

  await cargarTurno();
  await cargarRango();
  await cargarPerks();
```

- [ ] **Step 8: Verificación**

Este proyecto no tiene runner de tests de frontend (sin framework de JS testing en `package.json`), y la app corre en una ventana nativa de Tauri sin herramienta de captura automatizada en este entorno — la misma limitación reconocida en el Plan 6. No hay un paso de "correr tests" aquí; la corrección de este paso se cubre con una revisión cuidadosa del diff (nombres de variables e ids consistentes entre HTML/JS) más la verificación manual guiada de la Tarea 7 abajo.

- [ ] **Step 9: Commit**

```bash
git add app/src/index.html app/src/main.js
git commit -m "Add rank badge and ascension announcement to the frontend"
```

---

## Self-Review Notes

- **Cobertura del spec:** modelo de `Rango` + campo en `EstadoJugador` ✓ (Tareas 2-3), disparo automático al cruzar el umbral existente ✓ (Tarea 3), condición de ascenso solo por reputación (mini-boss fuera de alcance, comentario actualizado) ✓ (Tarea 3), gating de tickets Enfoque A ✓ (Tareas 1-2, 5), +1 slot de perk en Auxiliar de Sistemas ✓ (Tarea 3), comunicación al frontend vía `ScoreResult`/comando dedicado ✓ (Tarea 5), badge + anuncio de ascenso en frontend ✓ (Tarea 6).
- **Hueco de contenido detectado durante la planeación (no estaba en el spec original):** el catálogo real de Hospital Arcángel solo tenía 1 ticket Select-only de 6 — gating puro habría colapsado la bandeja de Becario a un ticket repetido. Se agregó la Tarea 1 (2 tickets nuevos) para que el spec sea jugable de verdad; confirmado con el usuario antes de escribir este plan.
- **Placeholders:** ninguno — cada Step tiene código completo, comandos exactos, y salida esperada.
- **Consistencia de tipos:** `Rango` se define una sola vez (`tickets::mod.rs`) y se usa con el mismo nombre en `economia::EstadoJugador.rango`, `ScoreResult.rango_actual`, el comando `rango_actual`, y el frontend (`score.rango_actual`, `NOMBRE_RANGO`) — sin conversiones ni renombres a mitad de camino. `aplicar_resultado` cambia de `()` a `bool` en la Tarea 3; se verificó que ningún llamador existente (los tests de `economia::mod.rs`) dependía del tipo de retorno anterior.
- **Alcance:** 6 tareas, cada una con su propio ciclo de test y commit; ninguna depende de una tarea posterior (Tareas 1-2 son contenido/modelo puro, 3 es economía pura, 4 es turno puro, 5 conecta todo en `lib.rs`, 6 es frontend). Mini-boss, transición de empresa, y escalera completa de rangos quedan fuera, como ya declaraba el spec.

## Execution Handoff

Plan completo y guardado en `docs/superpowers/plans/2026-07-13-fase0-07-ascenso-rango.md`. Dos opciones de ejecución:

1. **Subagent-Driven (recomendado)** — despacho un subagente fresco por tarea, reviso el resultado entre cada una antes de seguir
2. **Ejecución inline** — ejecuto las tareas en esta sesión con executing-plans, ejecución por lotes con checkpoints

¿Cuál prefieres?
