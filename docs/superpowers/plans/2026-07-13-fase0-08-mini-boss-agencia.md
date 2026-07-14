# Mini-boss (Auditor de Cumplimiento) + Transición de Agencia Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implementar el mini-boss de Hospital Arcángel (Auditor de Cumplimiento, un lote dedicado de 2 tickets) y la transición de la Agencia que, al superarlo, cambia la empresa activa a Postafeta — resetando reputación pero manteniendo dinero/XP/perks/rango.

**Architecture:** El ascenso a Auxiliar de Sistemas (señal `ascendio` del Plan 7) dispara de inmediato el lote del mini-boss vía un nuevo método `TurnoManejado::actualizar_fase`, que también detecta cuándo ese lote se vacía para marcar el arco completo. Un nuevo comando `confirmar_transicion_agencia` recarga el pool/catálogo de Postafeta (reutilizando `db::load_company`, hoy sin usar en producción) y reconstruye el turno. `turno::EstadoTurno` no cambia — la fase del arco vive en `lib.rs`, no en el módulo `turno`.

**Tech Stack:** Rust (Tauri backend), vanilla JS/HTML (frontend), Postgres embebido — sin dependencias nuevas.

## Global Constraints

- El mini-boss se dispara en el momento exacto del ascenso (`ascendio == true`), reemplazando de inmediato lo que quedara pendiente del turno normal — sin mezclarse con tickets normales.
- El mini-boss son exactamente 2 tickets fijos: uno con `Arquetipo::Join`, otro con `Arquetipo::Agregacion`.
- `valor_base`/`factor_reputacion` del mini-boss deben ser mayores que el máximo del catálogo normal de Hospital Arcángel (hoy 250/1.2, del ticket de depuración más caro) — se usan 300/1.5 para ambos tickets.
- `costo_tiempo` de cada ticket del mini-boss (25) debe quedar cómodamente bajo el presupuesto de turno (100) para que el lote de 2 nunca se agote a mitad de camino.
- "Superar" al mini-boss = vaciar ese lote de 2 (correctos o no — mismo criterio que el resto del juego: el costo de tiempo no depende de acertar).
- Al confirmar la transición: reputación resetea a `0.0`; dinero, XP por arquetipo, perks (desbloqueados/equipados) y rango se mantienen intactos.
- Postafeta es el único destino — sin selector de 2-3 empresas (eso es Fase 1+).
- Spec de referencia: `docs/superpowers/specs/2026-07-13-fase0-08-mini-boss-agencia-design.md`.

---

### Task 1: Catálogo del mini-boss (Auditor de Cumplimiento)

**Files:**
- Modify: `app/src-tauri/src/tickets/hospital_arcangel.rs`
- Modify: `app/src-tauri/src/tickets/mod.rs`

**Interfaces:**
- Produces: `hospital_arcangel::mini_boss() -> Vec<Ticket>` (privado al módulo `tickets`), expuesto como `tickets::mini_boss_hospital_arcangel() -> Vec<Ticket>` (público, consumido por la Tarea 2).

- [ ] **Step 1: Arreglar los imports de `hospital_arcangel.rs` para poder construir `Ticket` a mano fuera de tests**

Localizar:

```rust
use super::{
    plantilla_depuracion, plantilla_reporte_agregado, plantilla_reporte_join,
    plantilla_reporte_join_agregado, plantilla_reporte_simple, Arquetipo, Ticket,
};
#[cfg(test)]
use super::TipoTicket;
```

Reemplazar:

```rust
use super::{
    plantilla_depuracion, plantilla_reporte_agregado, plantilla_reporte_join,
    plantilla_reporte_join_agregado, plantilla_reporte_simple, Arquetipo, Prioridad, Ticket,
    TipoTicket,
};
```

- [ ] **Step 2: Agregar `mini_boss()` en `hospital_arcangel.rs`, después de `catalogo()`**

Localizar:

```rust
        plantilla_depuracion(
            "hospital_depuracion_costo_por_tipo",
            "Finanzas",
            "Finanzas heredó este reporte de un consultor externo que ya no trabaja aquí.",
            "Este reporte tarda una eternidad en cargar. Encuentra una forma de obtener el mismo resultado sin tanto rodeo.",
            "SELECT DISTINCT tipo, (SELECT SUM(costo) FROM tratamientos t2 WHERE t2.tipo = t1.tipo) AS costo_total FROM tratamientos t1 ORDER BY costo_total DESC",
            "SELECT tipo, SUM(costo) AS costo_total FROM tratamientos GROUP BY tipo ORDER BY costo_total DESC",
            vec![Arquetipo::Agregacion],
            20,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{self, Company};
    use crate::validation;
```

Reemplazar:

```rust
        plantilla_depuracion(
            "hospital_depuracion_costo_por_tipo",
            "Finanzas",
            "Finanzas heredó este reporte de un consultor externo que ya no trabaja aquí.",
            "Este reporte tarda una eternidad en cargar. Encuentra una forma de obtener el mismo resultado sin tanto rodeo.",
            "SELECT DISTINCT tipo, (SELECT SUM(costo) FROM tratamientos t2 WHERE t2.tipo = t1.tipo) AS costo_total FROM tratamientos t1 ORDER BY costo_total DESC",
            "SELECT tipo, SUM(costo) AS costo_total FROM tratamientos GROUP BY tipo ORDER BY costo_total DESC",
            vec![Arquetipo::Agregacion],
            20,
        ),
    ]
}

/// Los 2 tickets del mini-boss de Hospital Arcángel — el Auditor de
/// Cumplimiento (Etapa 7/9/11-G, Plan 8), la única pieza de escritura
/// verdaderamente única de esta empresa. Se construyen a mano, no vía
/// `plantilla_*`: su `valor_base`/`factor_reputacion` son deliberadamente más
/// altos que cualquier ticket normal del catálogo, reflejando el clímax
/// narrativo del arco. `costo_tiempo` (25 cada uno) se mantiene
/// deliberadamente bajo el presupuesto de turno (100) para que el lote de 2
/// nunca se agote a mitad de camino.
pub(crate) fn mini_boss() -> Vec<Ticket> {
    vec![
        Ticket {
            id: "hospital_miniboss_pacientes_sin_seguro",
            tipo: TipoTicket::ReporteAnalisis,
            solicitante: "Auditor de Cumplimiento",
            motivo: "El Auditor de Cumplimiento exige saber exactamente quién no tiene seguro médico registrado, antes de que Finanzas lo descubra primero.".to_string(),
            solicitud: "Lista el nombre de cada paciente sin seguro médico junto con el nombre de su departamento, ordenados por nombre de paciente.".to_string(),
            prioridad: Prioridad::Urgente,
            costo_tiempo: 25,
            arquetipos: vec![Arquetipo::Join],
            sql_dorada: "SELECT p.nombre, d.nombre AS departamento FROM pacientes p JOIN departamentos d ON p.departamento_id = d.id WHERE p.seguro_id = 5 ORDER BY p.nombre".to_string(),
            sql_inicial: None,
            requiere_orden: true,
            peso_correctitud: 0.5,
            peso_velocidad: 0.25,
            peso_practicas: 0.25,
            valor_base: 300,
            factor_reputacion: 1.5,
        },
        Ticket {
            id: "hospital_miniboss_tratamientos_por_tipo",
            tipo: TipoTicket::ReporteAnalisis,
            solicitante: "Auditor de Cumplimiento",
            motivo: "El Auditor de Cumplimiento quiere cruzar cuántos tratamientos se facturaron de cada tipo contra los recibos físicos antes de firmar el cierre trimestral.".to_string(),
            solicitud: "Lista cada tipo de tratamiento junto con cuántas veces se registró, del más frecuente al menos frecuente (alfabético en caso de empate).".to_string(),
            prioridad: Prioridad::Urgente,
            costo_tiempo: 25,
            arquetipos: vec![Arquetipo::Agregacion],
            sql_dorada: "SELECT tipo, COUNT(*) AS total FROM tratamientos GROUP BY tipo ORDER BY total DESC, tipo".to_string(),
            sql_inicial: None,
            requiere_orden: true,
            peso_correctitud: 0.5,
            peso_velocidad: 0.25,
            peso_practicas: 0.25,
            valor_base: 300,
            factor_reputacion: 1.5,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{self, Company};
    use crate::validation;
```

- [ ] **Step 3: Agregar tests de `mini_boss()` dentro del `mod tests` ya existente de ese mismo archivo**

Agregar (en cualquier punto antes del cierre `}` del módulo):

```rust
    #[test]
    fn mini_boss_tiene_2_tickets_uno_con_join_y_otro_con_agregacion() {
        let tickets = mini_boss();
        assert_eq!(tickets.len(), 2);
        assert_eq!(tickets[0].arquetipos, vec![Arquetipo::Join]);
        assert_eq!(tickets[1].arquetipos, vec![Arquetipo::Agregacion]);
        assert!(tickets.iter().all(|t| t.prioridad == Prioridad::Urgente));
    }

    #[tokio::test]
    async fn las_queries_doradas_del_mini_boss_ejecutan() {
        let pg = db::init_embedded_postgres().await.expect("Postgres embebido debe arrancar");
        let pool = db::load_company(&pg, Company::HospitalArcangel).await.expect("Hospital Arcángel debe cargar");

        for ticket in mini_boss() {
            db::run_query(&pool, &ticket.sql_dorada)
                .await
                .unwrap_or_else(|e| panic!("la query dorada del mini-boss '{}' debe ejecutar: {e}", ticket.id));
        }

        pool.close().await;
        pg.stop().await.expect("Postgres debe detenerse limpiamente");
    }
```

- [ ] **Step 4: Quitar el `#[allow(dead_code)]` de `Prioridad::Urgente` en `app/src-tauri/src/tickets/mod.rs`**

El mini-boss ya usa esta prioridad, así que deja de ser código muerto.

Localizar:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Prioridad {
    Baja,
    Media,
    // Ningún ticket de los catálogos actuales (Hospital Arcángel/Postafeta,
    // Tareas 2-3) usa esta prioridad todavía — queda reservada para tickets
    // futuros de mayor urgencia.
    #[allow(dead_code)]
    Urgente,
}
```

Reemplazar:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Prioridad {
    Baja,
    Media,
    Urgente,
}
```

- [ ] **Step 5: Exponer `mini_boss_hospital_arcangel()` en `app/src-tauri/src/tickets/mod.rs`**

Localizar:

```rust
/// Catálogo de tickets de `company` (Etapa 14) — generado por las plantillas
/// paramétricas de este módulo, nunca escrito a mano ticket por ticket.
pub fn catalogo(company: crate::db::Company) -> Vec<Ticket> {
    match company {
        crate::db::Company::HospitalArcangel => hospital_arcangel::catalogo(),
        crate::db::Company::Postafeta => postafeta::catalogo(),
    }
}
```

Reemplazar:

```rust
/// Catálogo de tickets de `company` (Etapa 14) — generado por las plantillas
/// paramétricas de este módulo, nunca escrito a mano ticket por ticket.
pub fn catalogo(company: crate::db::Company) -> Vec<Ticket> {
    match company {
        crate::db::Company::HospitalArcangel => hospital_arcangel::catalogo(),
        crate::db::Company::Postafeta => postafeta::catalogo(),
    }
}

/// Los 2 tickets del mini-boss de Hospital Arcángel — el Auditor de
/// Cumplimiento (Etapa 7/9/11-G, Plan 8), el único mini-boss del MVP.
pub fn mini_boss_hospital_arcangel() -> Vec<Ticket> {
    hospital_arcangel::mini_boss()
}
```

- [ ] **Step 6: Agregar un test de `mini_boss_hospital_arcangel()` dentro del `mod tests` ya existente de ese mismo archivo**

Agregar (en cualquier punto antes del cierre `}` del módulo):

```rust
    #[test]
    fn mini_boss_hospital_arcangel_tiene_2_tickets_auxiliar_tier_mas_exigentes_que_el_resto() {
        let mini_boss = mini_boss_hospital_arcangel();
        assert_eq!(mini_boss.len(), 2);
        assert!(
            mini_boss.iter().all(|t| rango_requerido(t) == Rango::AuxiliarDeSistemas),
            "los 2 tickets del mini-boss deben ser Auxiliar-tier"
        );

        let catalogo_normal = catalogo(crate::db::Company::HospitalArcangel);
        let max_valor_base_normal = catalogo_normal.iter().map(|t| t.valor_base).max().unwrap();
        let max_factor_reputacion_normal =
            catalogo_normal.iter().map(|t| t.factor_reputacion).fold(0.0, f64::max);

        assert!(
            mini_boss.iter().all(|t| t.valor_base > max_valor_base_normal),
            "el mini-boss debe pagar más que cualquier ticket normal"
        );
        assert!(
            mini_boss.iter().all(|t| t.factor_reputacion > max_factor_reputacion_normal),
            "el mini-boss debe dar más reputación que cualquier ticket normal"
        );
    }
```

- [ ] **Step 7: Correr la suite completa**

Run: `cd app/src-tauri && cargo test --lib -- --nocapture`
Expected: 0 failed, incluyendo los tests nuevos de este paso (algunos requieren Postgres embebido y tardan unos segundos).

- [ ] **Step 8: Commit**

```bash
git add app/src-tauri/src/tickets/hospital_arcangel.rs app/src-tauri/src/tickets/mod.rs
git commit -m "Add Hospital Arcángel mini-boss ticket batch (Auditor de Cumplimiento)"
```

---

### Task 2: Fase del arco (`FaseArco`) y disparo del mini-boss en `lib.rs`

**Files:**
- Modify: `app/src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: `tickets::mini_boss_hospital_arcangel()` (Tarea 1).
- Produces: `enum FaseArco { TrabajoNormal, MiniBoss, ArcoCompletado }`, `TurnoManejado.fase: FaseArco`, `TurnoManejado::actualizar_fase(&mut self, ascendio: bool, jugador: &mut economia::EstadoJugador)`, `struct EstadoTurnoView { presupuesto_restante, pendientes, fase }` + `impl From<&TurnoManejado> for EstadoTurnoView`. Los comandos `turno_actual`/`cerrar_dia` pasan de devolver `turno::EstadoTurno` a `EstadoTurnoView`.

- [ ] **Step 1: Agregar `FaseArco`, el campo `fase`, `actualizar_fase`, y `EstadoTurnoView`**

Localizar:

```rust
/// El catálogo completo de la empresa activa, el índice de rotación para el
/// próximo turno, y el turno (bandeja) actual (Etapa 11-A) — reemplaza la
/// selección round-robin simple de un solo "ticket actual" (Plan 3).
struct TurnoManejado {
    catalogo: Vec<tickets::Ticket>,
    indice_siguiente: usize,
    actual: turno::EstadoTurno,
}

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
        let elegibles = tickets::tickets_elegibles(&self.catalogo, jugador.rango);
        let (nuevo_turno, siguiente_indice) = turno::EstadoTurno::nuevo(&elegibles, self.indice_siguiente);
        self.actual = nuevo_turno;
        self.indice_siguiente = siguiente_indice;
    }
}

struct Turno(Mutex<TurnoManejado>);
```

Reemplazar:

```rust
/// Fase del arco de la empresa activa (Etapa 7/11-G, Plan 8): trabajo
/// normal, el lote dedicado del mini-boss, o el arco ya completo (esperando
/// que el jugador confirme la transición de la Agencia). Deliberadamente
/// específica del único mini-boss del MVP — no es un sistema genérico para
/// más empresas (Fase 1+).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
enum FaseArco {
    TrabajoNormal,
    MiniBoss,
    ArcoCompletado,
}

/// El catálogo completo de la empresa activa, el índice de rotación para el
/// próximo turno, el turno (bandeja) actual (Etapa 11-A) — reemplaza la
/// selección round-robin simple de un solo "ticket actual" (Plan 3) — y la
/// fase del arco de esa empresa (Etapa 7/11-G, Plan 8).
struct TurnoManejado {
    catalogo: Vec<tickets::Ticket>,
    indice_siguiente: usize,
    actual: turno::EstadoTurno,
    fase: FaseArco,
}

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
        let elegibles = tickets::tickets_elegibles(&self.catalogo, jugador.rango);
        let (nuevo_turno, siguiente_indice) = turno::EstadoTurno::nuevo(&elegibles, self.indice_siguiente);
        self.actual = nuevo_turno;
        self.indice_siguiente = siguiente_indice;
    }

    /// Aplica las consecuencias de haber resuelto un ticket sobre la fase
    /// del arco (Etapa 7/11-G, Plan 8), llamado desde `resolver_ticket`
    /// justo después de `aplicar_resultado`: si `ascendio` es `true`, el lote
    /// normal se reemplaza de inmediato por los 2 tickets del mini-boss (sin
    /// mezclarse con lo que quedaba pendiente); si ya estábamos en el lote
    /// del mini-boss y se acaba de vaciar, el arco queda completo; en
    /// cualquier otro caso, se comporta como antes (`escalar_y_avanzar`
    /// cuando el turno normal se agota o se vacía).
    fn actualizar_fase(&mut self, ascendio: bool, jugador: &mut economia::EstadoJugador) {
        if ascendio {
            let (turno_mini_boss, _) = turno::EstadoTurno::nuevo(&tickets::mini_boss_hospital_arcangel(), 0);
            self.actual = turno_mini_boss;
            self.fase = FaseArco::MiniBoss;
        } else if self.fase == FaseArco::MiniBoss && self.actual.pendientes.is_empty() {
            self.fase = FaseArco::ArcoCompletado;
        } else if self.fase == FaseArco::TrabajoNormal
            && (self.actual.pendientes.is_empty() || self.actual.turno_agotado())
        {
            self.escalar_y_avanzar(jugador);
        }
    }
}

struct Turno(Mutex<TurnoManejado>);

/// Vista de `EstadoTurno` (módulo `turno`) más la fase del arco (Etapa
/// 7/11-G, Plan 8) — `turno::EstadoTurno` se queda sin saber nada de
/// empresas/mini-boss, así que esta vista combinada vive aquí, no ahí.
#[derive(serde::Serialize)]
struct EstadoTurnoView {
    presupuesto_restante: u32,
    pendientes: Vec<tickets::Ticket>,
    fase: FaseArco,
}

impl From<&TurnoManejado> for EstadoTurnoView {
    fn from(manejado: &TurnoManejado) -> Self {
        EstadoTurnoView {
            presupuesto_restante: manejado.actual.presupuesto_restante,
            pendientes: manejado.actual.pendientes.clone(),
            fase: manejado.fase,
        }
    }
}
```

- [ ] **Step 2: Actualizar `turno_actual` y `cerrar_dia` para devolver `EstadoTurnoView`**

Localizar:

```rust
#[tauri::command]
fn turno_actual(turno: tauri::State<'_, Turno>) -> turno::EstadoTurno {
    turno.0.lock().unwrap().actual.clone()
}
```

Reemplazar:

```rust
#[tauri::command]
fn turno_actual(turno: tauri::State<'_, Turno>) -> EstadoTurnoView {
    EstadoTurnoView::from(&*turno.0.lock().unwrap())
}
```

Localizar:

```rust
#[tauri::command]
fn cerrar_dia(jugador: tauri::State<'_, Jugador>, turno_state: tauri::State<'_, Turno>) -> turno::EstadoTurno {
    let mut estado = jugador.0.lock().unwrap();
    let mut manejado = turno_state.0.lock().unwrap();
    manejado.escalar_y_avanzar(&mut estado);
    manejado.actual.clone()
}
```

Reemplazar:

```rust
#[tauri::command]
fn cerrar_dia(jugador: tauri::State<'_, Jugador>, turno_state: tauri::State<'_, Turno>) -> EstadoTurnoView {
    let mut estado = jugador.0.lock().unwrap();
    let mut manejado = turno_state.0.lock().unwrap();
    // Etapa 7/11-G, Plan 8: cerrar el día no tiene sentido narrativo (ni
    // mecánico) durante el mini-boss o esperando la Agencia — el jugador no
    // puede simplemente saltárselos, así que fuera de `TrabajoNormal` esto
    // no hace nada.
    if manejado.fase == FaseArco::TrabajoNormal {
        manejado.escalar_y_avanzar(&mut estado);
    }
    EstadoTurnoView::from(&*manejado)
}
```

- [ ] **Step 3: Usar `actualizar_fase` dentro de `resolver_ticket`**

Localizar:

```rust
    let mut manejado = turno_state.0.lock().unwrap();
    if manejado.actual.pendientes.is_empty() || manejado.actual.turno_agotado() {
        manejado.escalar_y_avanzar(&mut estado);
    }
```

Reemplazar:

```rust
    let mut manejado = turno_state.0.lock().unwrap();
    manejado.actualizar_fase(ascendio, &mut estado);
```

- [ ] **Step 4: Agregar `fase` a la construcción de `TurnoManejado` en `setup()`**

Localizar:

```rust
                handle.manage(Turno(Mutex::new(TurnoManejado {
                    catalogo,
                    indice_siguiente,
                    actual: turno_inicial,
                })));
```

Reemplazar:

```rust
                handle.manage(Turno(Mutex::new(TurnoManejado {
                    catalogo,
                    indice_siguiente,
                    actual: turno_inicial,
                    fase: FaseArco::TrabajoNormal,
                })));
```

- [ ] **Step 5: Agregar `fase` a la construcción de `TurnoManejado` en el test ya existente**

Localizar:

```rust
        let mut manejado = TurnoManejado {
            catalogo,
            indice_siguiente,
            actual: turno_inicial,
        };
```

Reemplazar:

```rust
        let mut manejado = TurnoManejado {
            catalogo,
            indice_siguiente,
            actual: turno_inicial,
            fase: FaseArco::TrabajoNormal,
        };
```

- [ ] **Step 6: Agregar tests de `actualizar_fase` dentro del `mod tests` ya existente de `lib.rs`**

Agregar (en cualquier punto antes del cierre `}` del módulo):

```rust
    #[test]
    fn actualizar_fase_dispara_el_lote_del_mini_boss_al_ascender() {
        let catalogo = tickets::catalogo(db::Company::HospitalArcangel);
        let elegibles_becario = tickets::tickets_elegibles(&catalogo, Rango::Becario);
        let (turno_inicial, indice_siguiente) = turno::EstadoTurno::nuevo(&elegibles_becario, 0);
        let mut manejado = TurnoManejado {
            catalogo,
            indice_siguiente,
            actual: turno_inicial,
            fase: FaseArco::TrabajoNormal,
        };
        let mut jugador = economia::EstadoJugador::default();

        manejado.actualizar_fase(true, &mut jugador);

        assert_eq!(manejado.fase, FaseArco::MiniBoss);
        assert_eq!(manejado.actual.pendientes.len(), 2);
        assert!(
            manejado
                .actual
                .pendientes
                .iter()
                .all(|t| tickets::rango_requerido(t) == Rango::AuxiliarDeSistemas),
            "los 2 tickets del mini-boss son Auxiliar-tier"
        );
    }

    #[test]
    fn actualizar_fase_completa_el_arco_al_vaciar_el_lote_del_mini_boss() {
        let catalogo = tickets::catalogo(db::Company::HospitalArcangel);
        let (turno_vacio, _) = turno::EstadoTurno::nuevo(&[], 0);
        let mut manejado = TurnoManejado {
            catalogo,
            indice_siguiente: 0,
            actual: turno_vacio,
            fase: FaseArco::MiniBoss,
        };
        let mut jugador = economia::EstadoJugador::default();

        manejado.actualizar_fase(false, &mut jugador);

        assert_eq!(manejado.fase, FaseArco::ArcoCompletado);
        assert!(manejado.actual.pendientes.is_empty(), "no debe dibujarse un turno normal");
    }

    #[test]
    fn actualizar_fase_avanza_el_turno_normal_cuando_no_hay_ascenso_ni_mini_boss() {
        let catalogo = tickets::catalogo(db::Company::HospitalArcangel);
        let (turno_vacio, indice_siguiente) = turno::EstadoTurno::nuevo(&[], 0);
        let mut manejado = TurnoManejado {
            catalogo,
            indice_siguiente,
            actual: turno_vacio,
            fase: FaseArco::TrabajoNormal,
        };
        let mut jugador = economia::EstadoJugador::default();

        manejado.actualizar_fase(false, &mut jugador);

        assert_eq!(manejado.fase, FaseArco::TrabajoNormal);
        assert!(!manejado.actual.pendientes.is_empty(), "debe dibujar un turno normal nuevo");
    }

    #[test]
    fn actualizar_fase_no_hace_nada_en_arco_completado() {
        let catalogo = tickets::catalogo(db::Company::HospitalArcangel);
        let (turno_vacio, _) = turno::EstadoTurno::nuevo(&[], 0);
        let mut manejado = TurnoManejado {
            catalogo,
            indice_siguiente: 0,
            actual: turno_vacio,
            fase: FaseArco::ArcoCompletado,
        };
        let mut jugador = economia::EstadoJugador::default();

        manejado.actualizar_fase(false, &mut jugador);

        assert_eq!(manejado.fase, FaseArco::ArcoCompletado);
        assert!(manejado.actual.pendientes.is_empty());
    }
```

- [ ] **Step 7: Correr la suite completa**

Run: `cd app/src-tauri && cargo build && cargo test --lib -- --nocapture`
Expected: build sin errores, 0 tests failed, incluyendo los 4 tests nuevos de este paso y el test de la Tarea 1 (`mini_boss`).

- [ ] **Step 8: Commit**

```bash
git add app/src-tauri/src/lib.rs
git commit -m "Add FaseArco state machine and trigger the mini-boss batch on ascension"
```

---

### Task 3: Comando `confirmar_transicion_agencia` y cambio de empresa

**Files:**
- Modify: `app/src-tauri/src/lib.rs`
- Modify: `app/src-tauri/src/db/mod.rs`

**Interfaces:**
- Consumes: `FaseArco`, `TurnoManejado`, `EstadoTurnoView` (Tarea 2); `db::load_company`, `tickets::catalogo`, `tickets::tickets_elegibles` (ya existentes).
- Produces: comando `confirmar_transicion_agencia() -> Result<EstadoTurnoView, String>`; `AppState` pasa de `{ pool: PgPool }` a `(Mutex<PgPool>)`.

- [ ] **Step 1: `AppState` pasa a `Mutex<PgPool>`**

Localizar:

```rust
/// Pool de conexión al Postgres embebido, gestionado por Tauri.
struct AppState {
    pool: sqlx::PgPool,
}
```

Reemplazar:

```rust
/// Pool de conexión al Postgres embebido, gestionado por Tauri. `Mutex` en
/// vez de un campo suelto (Etapa 11-G, Plan 8) porque
/// `confirmar_transicion_agencia` necesita poder reemplazarlo en caliente al
/// cambiar de empresa; `PgPool` es barato de clonar (handle basado en Arc
/// internamente), así que los comandos que lo usan clonan la copia vigente
/// en vez de tener que mantener el lock abierto durante un `.await`.
struct AppState(Mutex<sqlx::PgPool>);
```

- [ ] **Step 2: Actualizar `run_query` y `resolver_ticket` para clonar el pool desde el Mutex**

Localizar:

```rust
#[tauri::command]
async fn run_query(state: tauri::State<'_, AppState>, sql: String) -> Result<db::QueryResult, String> {
    db::run_query(&state.pool, &sql).await.map_err(|e| e.to_string())
}
```

Reemplazar:

```rust
#[tauri::command]
async fn run_query(state: tauri::State<'_, AppState>, sql: String) -> Result<db::QueryResult, String> {
    let pool = state.0.lock().unwrap().clone();
    db::run_query(&pool, &sql).await.map_err(|e| e.to_string())
}
```

Localizar:

```rust
    let evaluacion = validation::evaluar_entrega(&state.pool, &sql, &ticket.sql_dorada, ticket.requiere_orden)
        .await
        .map_err(|e| e.to_string())?;
```

Reemplazar:

```rust
    let pool = state.0.lock().unwrap().clone();
    let evaluacion = validation::evaluar_entrega(&pool, &sql, &ticket.sql_dorada, ticket.requiere_orden)
        .await
        .map_err(|e| e.to_string())?;
```

- [ ] **Step 3: Actualizar la construcción de `AppState` en `setup()`**

Localizar:

```rust
                handle.manage(AppState { pool });
```

Reemplazar:

```rust
                handle.manage(AppState(Mutex::new(pool)));
```

- [ ] **Step 4: Agregar `transicionar_a_empresa` y el comando `confirmar_transicion_agencia`, después de `cerrar_dia`**

Localizar:

```rust
#[tauri::command]
fn cerrar_dia(jugador: tauri::State<'_, Jugador>, turno_state: tauri::State<'_, Turno>) -> EstadoTurnoView {
    let mut estado = jugador.0.lock().unwrap();
    let mut manejado = turno_state.0.lock().unwrap();
    // Etapa 7/11-G, Plan 8: cerrar el día no tiene sentido narrativo (ni
    // mecánico) durante el mini-boss o esperando la Agencia — el jugador no
    // puede simplemente saltárselos, así que fuera de `TrabajoNormal` esto
    // no hace nada.
    if manejado.fase == FaseArco::TrabajoNormal {
        manejado.escalar_y_avanzar(&mut estado);
    }
    EstadoTurnoView::from(&*manejado)
}
```

Reemplazar:

```rust
#[tauri::command]
fn cerrar_dia(jugador: tauri::State<'_, Jugador>, turno_state: tauri::State<'_, Turno>) -> EstadoTurnoView {
    let mut estado = jugador.0.lock().unwrap();
    let mut manejado = turno_state.0.lock().unwrap();
    // Etapa 7/11-G, Plan 8: cerrar el día no tiene sentido narrativo (ni
    // mecánico) durante el mini-boss o esperando la Agencia — el jugador no
    // puede simplemente saltárselos, así que fuera de `TrabajoNormal` esto
    // no hace nada.
    if manejado.fase == FaseArco::TrabajoNormal {
        manejado.escalar_y_avanzar(&mut estado);
    }
    EstadoTurnoView::from(&*manejado)
}

/// Carga `company` (Etapa 11-G, Plan 8) y reconstruye el turno/catálogo para
/// ella — aislado de `confirmar_transicion_agencia` para poder probarse
/// contra Postgres embebido real sin pasar por el estado de Tauri. No toca
/// `EstadoJugador` directamente (solo lee `rango`, por valor) para que el
/// llamador nunca necesite mantener el lock de `Jugador` abierto durante el
/// `.await`.
async fn transicionar_a_empresa(
    pg: &postgresql_embedded::PostgreSQL,
    company: db::Company,
    rango: tickets::Rango,
) -> anyhow::Result<(sqlx::PgPool, TurnoManejado)> {
    let pool = db::load_company(pg, company).await?;
    let catalogo = tickets::catalogo(company);
    let elegibles = tickets::tickets_elegibles(&catalogo, rango);
    let (actual, indice_siguiente) = turno::EstadoTurno::nuevo(&elegibles, 0);
    Ok((
        pool,
        TurnoManejado {
            catalogo,
            indice_siguiente,
            actual,
            fase: FaseArco::TrabajoNormal,
        },
    ))
}

/// Confirma la transición de la Agencia (Etapa 9/11-G, Plan 8): solo el
/// único destino del MVP, Postafeta. Falla si el arco todavía no está
/// completo. Resetea la reputación a 0 (Etapa 12: "eres el nuevo") — dinero,
/// XP por arquetipo, perks y rango se mantienen intactos.
#[tauri::command]
async fn confirmar_transicion_agencia(
    state: tauri::State<'_, AppState>,
    jugador: tauri::State<'_, Jugador>,
    turno_state: tauri::State<'_, Turno>,
    embedded: tauri::State<'_, EmbeddedPostgres>,
) -> Result<EstadoTurnoView, String> {
    {
        let manejado = turno_state.0.lock().unwrap();
        if manejado.fase != FaseArco::ArcoCompletado {
            return Err("El arco de la empresa todavía no está completo.".to_string());
        }
    }

    let rango = jugador.0.lock().unwrap().rango;

    let pg = embedded
        .0
        .lock()
        .unwrap()
        .take()
        .ok_or_else(|| "Postgres embebido no está disponible.".to_string())?;
    let resultado = transicionar_a_empresa(&pg, db::Company::Postafeta, rango).await;
    embedded.0.lock().unwrap().replace(pg);
    let (pool, nuevo_manejado) = resultado.map_err(|e| e.to_string())?;

    jugador.0.lock().unwrap().reputacion = 0.0;
    *state.0.lock().unwrap() = pool;
    *turno_state.0.lock().unwrap() = nuevo_manejado;

    Ok(EstadoTurnoView::from(&*turno_state.0.lock().unwrap()))
}
```

- [ ] **Step 5: Registrar el comando nuevo**

Localizar:

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

Reemplazar:

```rust
        .invoke_handler(tauri::generate_handler![
            turno_actual,
            rango_actual,
            run_query,
            resolver_ticket,
            cerrar_dia,
            confirmar_transicion_agencia,
            catalogo_perks,
            desbloquear_perk,
            equipar_perk,
            desequipar_perk
        ])
```

- [ ] **Step 6: Quitar el `#[allow(dead_code)]` de `Company::Postafeta` en `app/src-tauri/src/db/mod.rs`**

Ya se construye en producción (`transicionar_a_empresa`), deja de ser código muerto.

Localizar:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Company {
    HospitalArcangel,
    // Se construye desde lib.rs cuando el cambio de empresa (Etapa 11-G) esté
    // implementado; por ahora solo se usa en tests.
    #[allow(dead_code)]
    Postafeta,
}
```

Reemplazar:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Company {
    HospitalArcangel,
    Postafeta,
}
```

- [ ] **Step 7: Agregar el test de integración de `transicionar_a_empresa` dentro del `mod tests` ya existente de `lib.rs`**

Agregar (en cualquier punto antes del cierre `}` del módulo):

```rust
    #[tokio::test]
    async fn transicionar_a_empresa_resetea_el_turno_y_arma_el_catalogo_de_la_nueva_empresa() {
        let pg = db::init_embedded_postgres().await.expect("Postgres embebido debe arrancar");

        let (pool, manejado) = transicionar_a_empresa(&pg, db::Company::Postafeta, Rango::AuxiliarDeSistemas)
            .await
            .expect("la transición a Postafeta debe completarse");

        assert_eq!(manejado.fase, FaseArco::TrabajoNormal);
        assert_eq!(manejado.catalogo.len(), 6, "catálogo completo de Postafeta");
        assert!(!manejado.actual.pendientes.is_empty(), "debe armar un turno inicial en la empresa nueva");

        let paquetes = db::run_query(&pool, "SELECT * FROM paquetes").await.expect("Postafeta debe responder queries");
        assert_eq!(paquetes.rows.len(), 30, "el pool devuelto debe apuntar a la base de datos de Postafeta");

        pool.close().await;
        pg.stop().await.expect("Postgres debe detenerse limpiamente");
    }
```

- [ ] **Step 8: Compilar y correr la suite completa**

Run: `cd app/src-tauri && cargo build && cargo test --lib -- --nocapture`
Expected: build sin errores, 0 tests failed.

- [ ] **Step 9: Commit**

```bash
git add app/src-tauri/src/lib.rs app/src-tauri/src/db/mod.rs
git commit -m "Add confirmar_transicion_agencia command and swap to a hot-reloadable pool"
```

---

### Task 4: Frontend — bandeja del mini-boss y overlay de la Agencia

**Files:**
- Modify: `app/src/index.html`
- Modify: `app/src/main.js`

**Interfaces:**
- Consumes: comando `confirmar_transicion_agencia` (Tarea 3), campo `fase` de `EstadoTurnoView` (Tarea 2, ya devuelto por `turno_actual`/`cerrar_dia`).
- Produces: encabezado de bandeja dinámico; overlay de Agencia.

- [ ] **Step 1: Agregar `id` al encabezado de la bandeja en `app/src/index.html`**

Localizar:

```html
      <section class="bandeja">
        <h2>Bandeja — turno actual</h2>
        <p>⏱️ Presupuesto de tiempo: <span id="presupuesto">0</span></p>
```

Reemplazar:

```html
      <section class="bandeja">
        <h2 id="bandeja-titulo">Bandeja — turno actual</h2>
        <p>⏱️ Presupuesto de tiempo: <span id="presupuesto">0</span></p>
```

- [ ] **Step 2: Agregar el overlay de la Agencia, mismo archivo**

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
  </body>
</html>
```

Reemplazar:

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

    <div id="agencia-overlay" class="scoring-overlay oculto">
      <div class="scoring-panel">
        <h2>Grupo Ómega RH — Reasignación</h2>
        <p>Has superado al Auditor de Cumplimiento. Tu siguiente asignación:</p>
        <p><strong>Postafeta</strong> — todo el Slack de la empresa lo administra un becario invisible llamado Kevin; todo viene firmado "- Kevin".</p>
        <button id="btn-confirmar-agencia">Aceptar reasignación</button>
      </div>
    </div>
  </body>
</html>
```

- [ ] **Step 3: Declarar las variables nuevas y el mapeo de título por fase en `app/src/main.js`**

Localizar:

```js
let sqlInput, statusMsg, resultTable, dineroEl, reputacionEl, rangoEl;
let perksSelect, perksEquipadosMsg;
let presupuestoEl, listaTickets, ticketActivoInfo;
let scoringOverlay, scoringAscenso;
```

Reemplazar:

```js
let sqlInput, statusMsg, resultTable, dineroEl, reputacionEl, rangoEl;
let perksSelect, perksEquipadosMsg;
let presupuestoEl, listaTickets, ticketActivoInfo, bandejaTitulo;
let scoringOverlay, scoringAscenso, agenciaOverlay;

const TITULO_FASE = {
  MiniBoss: "El Auditor de Cumplimiento quiere verte",
};
```

- [ ] **Step 4: Actualizar `renderBandeja` para pintar el título por fase y mostrar el overlay de la Agencia**

Localizar:

```js
function renderBandeja(estadoTurno) {
  presupuestoEl.textContent = estadoTurno.presupuesto_restante;
  listaTickets.innerHTML = "";
  for (const ticket of estadoTurno.pendientes) {
    const li = document.createElement("li");
    const info = document.createElement("span");
    info.textContent = `[⏱️ ${ticket.costo_tiempo}] ${ticket.motivo}`;
    const boton = document.createElement("button");
    boton.textContent = ticket.id === ticketActivoId ? "En curso" : "Trabajar en este";
    boton.addEventListener("click", () => seleccionarTicket(ticket));
    li.appendChild(info);
    li.appendChild(boton);
    listaTickets.appendChild(li);
  }
  if (!estadoTurno.pendientes.some((t) => t.id === ticketActivoId)) {
    ticketActivoId = null;
    ticketActivoInfo.textContent = "Elige un ticket de la bandeja para empezar.";
  }
}
```

Reemplazar:

```js
function renderBandeja(estadoTurno) {
  presupuestoEl.textContent = estadoTurno.presupuesto_restante;
  bandejaTitulo.textContent = TITULO_FASE[estadoTurno.fase] || "Bandeja — turno actual";
  listaTickets.innerHTML = "";
  for (const ticket of estadoTurno.pendientes) {
    const li = document.createElement("li");
    const info = document.createElement("span");
    info.textContent = `[⏱️ ${ticket.costo_tiempo}] ${ticket.motivo}`;
    const boton = document.createElement("button");
    boton.textContent = ticket.id === ticketActivoId ? "En curso" : "Trabajar en este";
    boton.addEventListener("click", () => seleccionarTicket(ticket));
    li.appendChild(info);
    li.appendChild(boton);
    listaTickets.appendChild(li);
  }
  if (!estadoTurno.pendientes.some((t) => t.id === ticketActivoId)) {
    ticketActivoId = null;
    ticketActivoInfo.textContent = "Elige un ticket de la bandeja para empezar.";
  }
  if (estadoTurno.fase === "ArcoCompletado") {
    agenciaOverlay.classList.remove("oculto");
  }
}
```

- [ ] **Step 5: Agregar `confirmarTransicionAgencia`, después de `cerrarDia`**

Localizar:

```js
async function cerrarDia() {
  const estadoTurno = await invoke("cerrar_dia");
  ticketActivoId = null;
  renderBandeja(estadoTurno);
  setStatus("Día cerrado. Turno nuevo.", "ok");
}
```

Reemplazar:

```js
async function cerrarDia() {
  const estadoTurno = await invoke("cerrar_dia");
  ticketActivoId = null;
  renderBandeja(estadoTurno);
  setStatus("Día cerrado. Turno nuevo.", "ok");
}

async function confirmarTransicionAgencia() {
  try {
    const estadoTurno = await invoke("confirmar_transicion_agencia");
    reputacionEl.textContent = "0.0";
    agenciaOverlay.classList.add("oculto");
    ticketActivoId = null;
    renderBandeja(estadoTurno);
    setStatus("Bienvenido a Postafeta.", "ok");
  } catch (err) {
    setStatus(String(err), "error");
  }
}
```

- [ ] **Step 6: Enlazar los elementos nuevos y el botón de la Agencia, en `DOMContentLoaded`**

Localizar:

```js
  presupuestoEl = document.querySelector("#presupuesto");
  listaTickets = document.querySelector("#lista-tickets");
  ticketActivoInfo = document.querySelector("#ticket-activo-info");
  scoringOverlay = document.querySelector("#scoring-overlay");
  scoringAscenso = document.querySelector("#scoring-ascenso");

  await cargarTurno();
  await cargarRango();
  await cargarPerks();

  document.querySelector("#btn-play").addEventListener("click", runQuery);
  document.querySelector("#btn-submit").addEventListener("click", submitTicket);
  document.querySelector("#btn-cerrar-dia").addEventListener("click", cerrarDia);
  document.querySelector("#btn-cerrar-scoring").addEventListener("click", () => scoringOverlay.classList.add("oculto"));
  document.querySelector("#btn-unlock-perk").addEventListener("click", desbloquearPerkSeleccionado);
  document.querySelector("#btn-equip-perk").addEventListener("click", equiparODesequiparPerkSeleccionado);
});
```

Reemplazar:

```js
  presupuestoEl = document.querySelector("#presupuesto");
  listaTickets = document.querySelector("#lista-tickets");
  ticketActivoInfo = document.querySelector("#ticket-activo-info");
  bandejaTitulo = document.querySelector("#bandeja-titulo");
  scoringOverlay = document.querySelector("#scoring-overlay");
  scoringAscenso = document.querySelector("#scoring-ascenso");
  agenciaOverlay = document.querySelector("#agencia-overlay");

  await cargarTurno();
  await cargarRango();
  await cargarPerks();

  document.querySelector("#btn-play").addEventListener("click", runQuery);
  document.querySelector("#btn-submit").addEventListener("click", submitTicket);
  document.querySelector("#btn-cerrar-dia").addEventListener("click", cerrarDia);
  document.querySelector("#btn-cerrar-scoring").addEventListener("click", () => scoringOverlay.classList.add("oculto"));
  document.querySelector("#btn-unlock-perk").addEventListener("click", desbloquearPerkSeleccionado);
  document.querySelector("#btn-equip-perk").addEventListener("click", equiparODesequiparPerkSeleccionado);
  document.querySelector("#btn-confirmar-agencia").addEventListener("click", confirmarTransicionAgencia);
});
```

- [ ] **Step 7: Verificación**

Este proyecto no tiene runner de tests de frontend, y la app corre en una ventana nativa de Tauri sin herramienta de captura automatizada en este entorno (mismo patrón ya usado en Planes 6/7). No hay paso de "correr tests" aquí; la corrección se cubre con revisión cuidadosa del diff (ids consistentes entre HTML/JS) más la verificación manual guiada del final del plan.

- [ ] **Step 8: Commit**

```bash
git add app/src/index.html app/src/main.js
git commit -m "Add mini-boss inbox heading and Agencia transition overlay to the frontend"
```

---

## Self-Review Notes

- **Cobertura del spec:** disparo inmediato del mini-boss al ascender ✓ (Tarea 2), 2 tickets fijos Join/Agregación con `valor_base`/`factor_reputacion` por encima del máximo normal ✓ (Tarea 1), arco completo al vaciar el lote ✓ (Tarea 2), overlay de Agencia con Postafeta como única opción ✓ (Tarea 4), transición real de empresa (pool + catálogo) con reputación reseteada y dinero/XP/perks/rango intactos ✓ (Tarea 3), `cerrar_dia` no puede saltarse el mini-boss/Agencia ✓ (Tarea 2).
- **Simplificación consciente respecto al spec completo:** el diseño original consideraba etiquetar la empresa activa junto al pool bajo el mismo lock (`EstadoConexion { pool, empresa }`) para blindar el invariante "pool y catálogo son de la misma empresa". Se simplificó a `Mutex<PgPool>` sin campo de empresa porque nada en este plan necesita leer la empresa activa como dato — `confirmar_transicion_agencia`/`transicionar_a_empresa` ya construyen `pool` y `catalogo` a partir del mismo valor `Company` en la misma función, así que el invariante se sostiene por construcción, no por un campo adicional que el compilador habría marcado como no leído.
- **Placeholders:** ninguno — SQL/valores/costos del mini-boss, y el flujo completo de `confirmar_transicion_agencia`, están fijados con código completo y su razón.
- **Consistencia de tipos:** `FaseArco`/`EstadoTurnoView` se definen una sola vez y se usan con el mismo nombre en los 3 comandos que los tocan (`turno_actual`, `cerrar_dia`, `confirmar_transicion_agencia`) y en el frontend (`estadoTurno.fase`). `AppState` cambia de struct-con-campo a tupla-con-Mutex; se verificó que los 3 call sites que leían `state.pool` (Tareas 3) quedan actualizados a `state.0.lock().unwrap().clone()`.
- **Concurrencia:** se verificó explícitamente que ningún `MutexGuard` (`std::sync::Mutex`) queda retenido a través de un `.await` en `resolver_ticket` (ya existente) ni en el nuevo `confirmar_transicion_agencia`/`transicionar_a_empresa` — cada lock se toma y suelta en un bloque síncrono antes de cualquier punto de espera async, siguiendo el mismo patrón ya usado por `resolver_ticket` para el lock de `Turno`.
- **Alcance:** 4 tareas con su propio ciclo de test y commit; Postafeta como único destino, sin selector genérico de empresas — eso y la escalera completa de mini-bosses quedan para Fase 1+, como ya declaraba el spec.

## Execution Handoff

Plan completo y guardado en `docs/superpowers/plans/2026-07-13-fase0-08-mini-boss-agencia.md`. Dos opciones de ejecución:

1. **Subagent-Driven (recomendado)** — despacho un subagente fresco por tarea, reviso el resultado entre cada una antes de seguir
2. **Ejecución inline** — ejecuto las tareas en esta sesión con executing-plans, ejecución por lotes con checkpoints

¿Cuál prefieres?
