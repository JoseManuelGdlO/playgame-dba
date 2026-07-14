# Fase 0 / Plan 3: Generación de Tickets por Plantillas Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reemplazar el único ticket hardcodeado del spike por un catálogo de tickets generado por plantillas paramétricas (Etapa 14), cubriendo los 2 tipos de ticket del rango Becario/Auxiliar (Reporte/Análisis e Investigación/Depuración) para las 2 empresas del MVP (Hospital Arcángel, Postafeta).

**Architecture:** `app/src-tauri/src/tickets/mod.rs` define el modelo de datos (`Ticket` + enums) y 5 funciones-plantilla parametrizadas (Rust puro, no un motor genérico de placeholders) reutilizables entre empresas; `tickets/hospital_arcangel.rs` y `tickets/postafeta.rs` llaman a esas plantillas con datos concretos de cada empresa para construir su catálogo. `lib.rs` sirve el catálogo con selección round-robin simple.

**Tech Stack:** Rust puro — sin dependencias nuevas. Reutiliza `db::run_query`/`load_company` (Plan 1) y `validation::evaluar_entrega` (Plan 2) tal cual.

## Global Constraints

- Cobertura: ambos tipos de ticket (Reporte/Análisis e Investigación/Depuración) para ambas empresas del MVP (Hospital Arcángel, Postafeta).
- Profundidad: 4 plantillas de Reporte/Análisis + 2 de Investigación/Depuración por empresa (12 tickets concretos en total).
- Los tickets de Investigación/Depuración son **lentos pero correctos** (mismo resultado que la query dorada, plan peor) — nunca "rotos" (resultado de negocio incorrecto).
- Plantillas = funciones Rust parametrizadas, reutilizables entre empresas — no un motor genérico de sustitución de placeholders.
- Este plan NO combina los 3 puntajes crudos (Plan 2) con los pesos de la rúbrica en un `puntaje_base`, ni calcula dinero/reputación/XP más allá del stub ya existente — solo almacena los pesos como metadata del ticket (`peso_correctitud`/`peso_velocidad`/`peso_practicas`). Esa combinación es responsabilidad de un plan de economía posterior (Etapa 12).
- Sin bandeja de entrada ni tiempo de turno todavía (Etapa 11-A) — selección de "ticket actual" es un round-robin simple sobre el catálogo de la empresa activa.
- `arquetipos` (metadata SQL por ticket) se captura pero no se consume todavía (futuro sistema de XP, Etapa 13) — sin panel de "tablas relevantes" (Etapa 11-C), nada lo consume aún.
- Alcance de lectura únicamente — nada de escritura de datos/DDL (Etapa 14).

---

## File Structure

- Create: `app/src-tauri/src/tickets/mod.rs` — modelo de datos (`Ticket`, `TipoTicket`, `Prioridad`, `Arquetipo`) + 5 funciones-plantilla + (desde la Tarea 3) el dispatcher `pub fn catalogo(company) -> Vec<Ticket>`
- Create: `app/src-tauri/src/tickets/hospital_arcangel.rs` — catálogo concreto de Hospital Arcángel (6 tickets) + tests
- Create: `app/src-tauri/src/tickets/postafeta.rs` — catálogo concreto de Postafeta (6 tickets) + tests
- Modify: `app/src-tauri/src/lib.rs` — agrega `mod tickets;`, un nuevo estado `Tickets` (catálogo + índice round-robin), y rewira `ticket_actual`/`submit_ticket`
- Modify: `app/src-tauri/src/db/mod.rs` y `app/src-tauri/src/db/hospital_arcangel.rs` — retira el ticket único obsoleto (`TICKET_ENUNCIADO`/`TICKET_SOLUCION`), ya completamente reemplazado por el catálogo
- Modify: `app/src/main.js` — ajuste mínimo de compatibilidad: `ticket_actual` ahora devuelve un objeto, no un string; precarga `sql_inicial` en el editor cuando aplica

---

### Task 1: Modelo de datos y plantillas paramétricas

**Files:**
- Create: `app/src-tauri/src/tickets/mod.rs`
- Modify: `app/src-tauri/src/lib.rs` (agrega `mod tickets;`)

**Interfaces:**
- Produces: `tickets::Ticket` (struct pública, `Serialize + Clone + Debug`), `tickets::{TipoTicket, Prioridad, Arquetipo}` (enums públicos), y 5 funciones de plantilla (privadas al módulo `tickets`, visibles desde sus submódulos hijos): `plantilla_reporte_simple`, `plantilla_reporte_agregado`, `plantilla_reporte_join`, `plantilla_reporte_join_agregado`, `plantilla_depuracion`.

- [ ] **Step 1: Escribir `app/src-tauri/src/tickets/mod.rs`**

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
    pub sql_dorada: String,
    pub sql_inicial: Option<String>,
    pub requiere_orden: bool,
    pub peso_correctitud: f64,
    pub peso_velocidad: f64,
    pub peso_practicas: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum TipoTicket {
    ReporteAnalisis,
    InvestigacionDepuracion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Prioridad {
    Baja,
    Media,
    Urgente,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Arquetipo {
    Select,
    Join,
    Agregacion,
}

/// Plantilla "reporte simple": filtra y ordena una tabla por una columna,
/// sin JOIN ni agregación (Becario: SELECT/WHERE/ORDER BY, Etapa 10).
fn plantilla_reporte_simple(
    id: &'static str,
    solicitante: &'static str,
    motivo: impl Into<String>,
    solicitud: impl Into<String>,
    sql_dorada: impl Into<String>,
    costo_tiempo: u32,
) -> Ticket {
    Ticket {
        id,
        tipo: TipoTicket::ReporteAnalisis,
        solicitante,
        motivo: motivo.into(),
        solicitud: solicitud.into(),
        prioridad: Prioridad::Media,
        costo_tiempo,
        arquetipos: vec![Arquetipo::Select],
        sql_dorada: sql_dorada.into(),
        sql_inicial: None,
        requiere_orden: true,
        peso_correctitud: 0.6,
        peso_velocidad: 0.2,
        peso_practicas: 0.2,
    }
}

/// Plantilla "reporte agregado": agrupa una tabla por una columna y calcula
/// una métrica (Auxiliar: GROUP BY + COUNT/SUM, Etapa 10).
fn plantilla_reporte_agregado(
    id: &'static str,
    solicitante: &'static str,
    motivo: impl Into<String>,
    solicitud: impl Into<String>,
    sql_dorada: impl Into<String>,
    costo_tiempo: u32,
) -> Ticket {
    Ticket {
        id,
        tipo: TipoTicket::ReporteAnalisis,
        solicitante,
        motivo: motivo.into(),
        solicitud: solicitud.into(),
        prioridad: Prioridad::Baja,
        costo_tiempo,
        arquetipos: vec![Arquetipo::Agregacion],
        sql_dorada: sql_dorada.into(),
        sql_inicial: None,
        requiere_orden: true,
        peso_correctitud: 0.5,
        peso_velocidad: 0.2,
        peso_practicas: 0.3,
    }
}

/// Plantilla "reporte con JOIN": combina 2 tablas y lista resultados
/// (Auxiliar: JOIN inner, Etapa 10).
fn plantilla_reporte_join(
    id: &'static str,
    solicitante: &'static str,
    motivo: impl Into<String>,
    solicitud: impl Into<String>,
    sql_dorada: impl Into<String>,
    costo_tiempo: u32,
) -> Ticket {
    Ticket {
        id,
        tipo: TipoTicket::ReporteAnalisis,
        solicitante,
        motivo: motivo.into(),
        solicitud: solicitud.into(),
        prioridad: Prioridad::Media,
        costo_tiempo,
        arquetipos: vec![Arquetipo::Join],
        sql_dorada: sql_dorada.into(),
        sql_inicial: None,
        requiere_orden: true,
        peso_correctitud: 0.5,
        peso_velocidad: 0.2,
        peso_practicas: 0.3,
    }
}

/// Plantilla "reporte con JOIN + agregación": combina 2 tablas, agrupa y
/// calcula una métrica (Auxiliar: JOIN + GROUP BY + COUNT, Etapa 10).
fn plantilla_reporte_join_agregado(
    id: &'static str,
    solicitante: &'static str,
    motivo: impl Into<String>,
    solicitud: impl Into<String>,
    sql_dorada: impl Into<String>,
    costo_tiempo: u32,
) -> Ticket {
    Ticket {
        id,
        tipo: TipoTicket::ReporteAnalisis,
        solicitante,
        motivo: motivo.into(),
        solicitud: solicitud.into(),
        prioridad: Prioridad::Media,
        costo_tiempo,
        arquetipos: vec![Arquetipo::Join, Arquetipo::Agregacion],
        sql_dorada: sql_dorada.into(),
        sql_inicial: None,
        requiere_orden: true,
        peso_correctitud: 0.4,
        peso_velocidad: 0.3,
        peso_practicas: 0.3,
    }
}

/// Plantilla "Investigación/Depuración" (Etapa 14): se entrega una query ya
/// escrita — lenta pero con el mismo resultado correcto, nunca con un
/// resultado de negocio distinto — que el jugador debe optimizar. Conecta
/// con la fantasía de maestría y El Mentor (Etapa 5).
fn plantilla_depuracion(
    id: &'static str,
    solicitante: &'static str,
    motivo: impl Into<String>,
    solicitud: impl Into<String>,
    sql_inicial: impl Into<String>,
    sql_dorada: impl Into<String>,
    arquetipos: Vec<Arquetipo>,
    costo_tiempo: u32,
) -> Ticket {
    Ticket {
        id,
        tipo: TipoTicket::InvestigacionDepuracion,
        solicitante,
        motivo: motivo.into(),
        solicitud: solicitud.into(),
        prioridad: Prioridad::Baja,
        costo_tiempo,
        arquetipos,
        sql_dorada: sql_dorada.into(),
        sql_inicial: Some(sql_inicial.into()),
        requiere_orden: true,
        peso_correctitud: 0.3,
        peso_velocidad: 0.5,
        peso_practicas: 0.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plantilla_reporte_simple_arma_un_ticket_de_reporte_sin_join() {
        let ticket = plantilla_reporte_simple("id1", "Alguien", "un motivo", "una solicitud", "SELECT 1", 10);
        assert_eq!(ticket.tipo, TipoTicket::ReporteAnalisis);
        assert_eq!(ticket.arquetipos, vec![Arquetipo::Select]);
        assert!(ticket.sql_inicial.is_none());
        assert!(ticket.requiere_orden);
        assert_eq!((ticket.peso_correctitud, ticket.peso_velocidad, ticket.peso_practicas), (0.6, 0.2, 0.2));
    }

    #[test]
    fn plantilla_reporte_agregado_arma_un_ticket_de_agregacion() {
        let ticket = plantilla_reporte_agregado("id2", "Alguien", "un motivo", "una solicitud", "SELECT 1", 15);
        assert_eq!(ticket.tipo, TipoTicket::ReporteAnalisis);
        assert_eq!(ticket.arquetipos, vec![Arquetipo::Agregacion]);
        assert_eq!(ticket.prioridad, Prioridad::Baja);
        assert_eq!((ticket.peso_correctitud, ticket.peso_velocidad, ticket.peso_practicas), (0.5, 0.2, 0.3));
    }

    #[test]
    fn plantilla_reporte_join_arma_un_ticket_con_join() {
        let ticket = plantilla_reporte_join("id3", "Alguien", "un motivo", "una solicitud", "SELECT 1", 15);
        assert_eq!(ticket.tipo, TipoTicket::ReporteAnalisis);
        assert_eq!(ticket.arquetipos, vec![Arquetipo::Join]);
        assert_eq!((ticket.peso_correctitud, ticket.peso_velocidad, ticket.peso_practicas), (0.5, 0.2, 0.3));
    }

    #[test]
    fn plantilla_reporte_join_agregado_arma_un_ticket_con_join_y_agregacion() {
        let ticket = plantilla_reporte_join_agregado("id4", "Alguien", "un motivo", "una solicitud", "SELECT 1", 20);
        assert_eq!(ticket.tipo, TipoTicket::ReporteAnalisis);
        assert_eq!(ticket.arquetipos, vec![Arquetipo::Join, Arquetipo::Agregacion]);
        assert_eq!((ticket.peso_correctitud, ticket.peso_velocidad, ticket.peso_practicas), (0.4, 0.3, 0.3));
    }

    #[test]
    fn plantilla_depuracion_arma_un_ticket_con_sql_inicial() {
        let ticket = plantilla_depuracion(
            "id5",
            "Alguien",
            "un motivo",
            "una solicitud",
            "SELECT lenta",
            "SELECT rapida",
            vec![Arquetipo::Join],
            20,
        );
        assert_eq!(ticket.tipo, TipoTicket::InvestigacionDepuracion);
        assert_eq!(ticket.sql_inicial, Some("SELECT lenta".to_string()));
        assert_eq!(ticket.sql_dorada, "SELECT rapida");
        assert_eq!(ticket.arquetipos, vec![Arquetipo::Join]);
        assert_eq!((ticket.peso_correctitud, ticket.peso_velocidad, ticket.peso_practicas), (0.3, 0.5, 0.2));
    }
}
```

- [ ] **Step 2: Registrar el módulo en `app/src-tauri/src/lib.rs`**

Localizar el inicio del archivo:

```rust
mod db;
mod validation;
```

Reemplazar por (orden alfabético):

```rust
mod db;
mod tickets;
mod validation;
```

- [ ] **Step 3: Correr la suite completa**

Run: `cd app/src-tauri && cargo test --lib -- --nocapture`
Expected: todos los tests en verde, 0 failed — más los 5 nuevos de `tickets::tests`.

- [ ] **Step 4: Commit**

```bash
git add app/src-tauri/src/tickets/mod.rs app/src-tauri/src/lib.rs
git commit -m "Add ticket data model and parametrized template functions (Stage 14)"
```

---

### Task 2: Catálogo de Hospital Arcángel

**Files:**
- Create: `app/src-tauri/src/tickets/hospital_arcangel.rs`
- Modify: `app/src-tauri/src/tickets/mod.rs` (agrega `mod hospital_arcangel;`)

**Interfaces:**
- Consumes: `super::{plantilla_reporte_simple, plantilla_reporte_agregado, plantilla_reporte_join, plantilla_reporte_join_agregado, plantilla_depuracion, Arquetipo, Ticket, TipoTicket}` (Tarea 1), `crate::db::{self, Company}`, `crate::validation` (Plan 1/2, sin cambios)
- Produces: `tickets::hospital_arcangel::catalogo() -> Vec<Ticket>` (visibilidad `pub(crate)`, 6 tickets: 4 Reporte/Análisis + 2 Investigación/Depuración)

- [ ] **Step 1: Escribir `app/src-tauri/src/tickets/hospital_arcangel.rs`**

```rust
use super::{
    plantilla_depuracion, plantilla_reporte_agregado, plantilla_reporte_join,
    plantilla_reporte_join_agregado, plantilla_reporte_simple, Arquetipo, Ticket, TipoTicket,
};

pub(crate) fn catalogo() -> Vec<Ticket> {
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
            "hospital_reporte_costo_por_tipo",
            "Dirección General",
            "El CEO quiere un número para su próxima galleta de la fortuna motivacional sobre gastos médicos.",
            "¿Cuánto hemos gastado en total por cada tipo de tratamiento? Ordena de mayor a menor gasto.",
            "SELECT tipo, SUM(costo) AS costo_total FROM tratamientos GROUP BY tipo ORDER BY costo_total DESC",
            15,
        ),
        plantilla_reporte_join(
            "hospital_reporte_empleados_departamento",
            "Recursos Humanos",
            "RH necesita el directorio actualizado antes de la auditoría de nómina de este trimestre.",
            "Lista el nombre de cada empleado junto con el nombre de su departamento, ordenados por nombre de empleado.",
            "SELECT e.nombre AS empleado, d.nombre AS departamento FROM empleados e JOIN departamentos d ON e.departamento_id = d.id ORDER BY e.nombre",
            15,
        ),
        plantilla_reporte_join_agregado(
            "hospital_reporte_habitaciones_ocupadas",
            "Administración de Instalaciones",
            "Mantenimiento quiere saber cuántas camas siguen ocupadas antes de programar la fumigación trimestral.",
            "¿Cuántas habitaciones ocupadas hay en cada departamento? Ordena por nombre de departamento.",
            "SELECT d.nombre AS departamento, COUNT(*) AS habitaciones_ocupadas FROM habitaciones h JOIN departamentos d ON h.departamento_id = d.id WHERE h.ocupada = true GROUP BY d.nombre ORDER BY d.nombre",
            20,
        ),
        plantilla_depuracion(
            "hospital_depuracion_pacientes_departamento",
            "El Mentor",
            "Un becario anterior escribió este reporte de altas y funciona, pero el Mentor sospecha que hay una forma más rápida.",
            "Optimiza esta consulta para que no tenga que preguntarle a la base de datos una vez por cada paciente.",
            "SELECT nombre, (SELECT d.nombre FROM departamentos d WHERE d.id = pacientes.departamento_id) AS departamento FROM pacientes ORDER BY nombre",
            "SELECT p.nombre, d.nombre AS departamento FROM pacientes p JOIN departamentos d ON p.departamento_id = d.id ORDER BY p.nombre",
            vec![Arquetipo::Join],
            20,
        ),
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

    #[test]
    fn catalogo_tiene_4_reportes_y_2_depuraciones() {
        let tickets = catalogo();
        assert_eq!(tickets.len(), 6);
        let reportes = tickets.iter().filter(|t| t.tipo == TipoTicket::ReporteAnalisis).count();
        let depuraciones = tickets.iter().filter(|t| t.tipo == TipoTicket::InvestigacionDepuracion).count();
        assert_eq!(reportes, 4);
        assert_eq!(depuraciones, 2);
    }

    #[tokio::test]
    async fn todas_las_queries_doradas_ejecutan() {
        let pg = db::init_embedded_postgres().await.expect("Postgres embebido debe arrancar");
        let pool = db::load_company(&pg, Company::HospitalArcangel).await.expect("Hospital Arcángel debe cargar");

        for ticket in catalogo() {
            db::run_query(&pool, &ticket.sql_dorada)
                .await
                .unwrap_or_else(|e| panic!("la query dorada de '{}' debe ejecutar: {e}", ticket.id));
        }

        pool.close().await;
        pg.stop().await.expect("Postgres debe detenerse limpiamente");
    }

    #[tokio::test]
    async fn los_tickets_de_depuracion_son_correctos_pero_lentos() {
        let pg = db::init_embedded_postgres().await.expect("Postgres embebido debe arrancar");
        let pool = db::load_company(&pg, Company::HospitalArcangel).await.expect("Hospital Arcángel debe cargar");

        for ticket in catalogo().into_iter().filter(|t| t.sql_inicial.is_some()) {
            let sql_inicial = ticket.sql_inicial.clone().unwrap();
            let evaluacion = validation::evaluar_entrega(&pool, &sql_inicial, &ticket.sql_dorada, ticket.requiere_orden)
                .await
                .unwrap_or_else(|e| panic!("la evaluación de '{}' debe ejecutar: {e}", ticket.id));

            assert!(evaluacion.correcta, "'{}': sql_inicial debe dar el mismo resultado que sql_dorada", ticket.id);
            assert!(
                evaluacion.puntaje_velocidad < 100.0,
                "'{}': sql_inicial debe costar más que sql_dorada (puntaje_velocidad={})",
                ticket.id,
                evaluacion.puntaje_velocidad
            );
        }

        pool.close().await;
        pg.stop().await.expect("Postgres debe detenerse limpiamente");
    }
}
```

- [ ] **Step 2: Actualizar `app/src-tauri/src/tickets/mod.rs`**

Agregar al final del archivo (o justo antes de la sección `#[cfg(test)]`):

```rust
mod hospital_arcangel;
```

- [ ] **Step 3: Correr la suite completa**

Run: `cd app/src-tauri && cargo test --lib -- --nocapture`
Expected: todos los tests en verde, 0 failed — más los 3 nuevos de `tickets::hospital_arcangel::tests` (1 conteo + 2 integración contra Postgres real).

- [ ] **Step 4: Commit**

```bash
git add app/src-tauri/src/tickets/hospital_arcangel.rs app/src-tauri/src/tickets/mod.rs
git commit -m "Add Hospital Arcángel ticket catalog (4 reports + 2 debugging)"
```

---

### Task 3: Catálogo de Postafeta + dispatcher por empresa

**Files:**
- Create: `app/src-tauri/src/tickets/postafeta.rs`
- Modify: `app/src-tauri/src/tickets/mod.rs` (agrega `mod postafeta;` y el dispatcher `catalogo(company)`)

**Interfaces:**
- Consumes: mismas plantillas de la Tarea 1, `crate::db::{self, Company}`, `crate::validation`
- Produces: `tickets::postafeta::catalogo() -> Vec<Ticket>` (`pub(crate)`, 6 tickets), `tickets::catalogo(company: crate::db::Company) -> Vec<Ticket>` (público, dispatcher — primera interfaz de este módulo que consumirá `lib.rs` en la Tarea 4)

- [ ] **Step 1: Escribir `app/src-tauri/src/tickets/postafeta.rs`**

```rust
use super::{
    plantilla_depuracion, plantilla_reporte_agregado, plantilla_reporte_join_agregado,
    plantilla_reporte_simple, Arquetipo, Ticket, TipoTicket,
};

pub(crate) fn catalogo() -> Vec<Ticket> {
    vec![
        plantilla_reporte_simple(
            "postafeta_reporte_paquetes_centro",
            "Gerencia de Operaciones",
            "Kevin reporta que la gerencia quiere ver el estado de los envíos antes de la junta semanal.",
            "Lista todos los paquetes enviados desde la sucursal Centro (id 1), con su estado, del más reciente al más antiguo.",
            "SELECT id, estado, fecha_envio FROM paquetes WHERE sucursal_origen_id = 1 ORDER BY fecha_envio DESC",
            10,
        ),
        plantilla_reporte_agregado(
            "postafeta_reporte_incidencias_por_tipo",
            "Kevin",
            "Kevin necesita un resumen antes de firmarlo todo con \"- Kevin\" y mandarlo a la matriz.",
            "¿Cuántas incidencias hay de cada tipo? Ordena de mayor a menor.",
            "SELECT tipo, COUNT(*) AS total FROM incidencias GROUP BY tipo ORDER BY total DESC",
            15,
        ),
        plantilla_reporte_join_agregado(
            "postafeta_reporte_repartidor_top",
            "Recursos Humanos",
            "RH quiere reconocer (o interrogar) al repartidor más ocupado del mes.",
            "Lista cada repartidor junto con cuántos paquetes ha entregado, ordenado de mayor a menor.",
            "SELECT e.nombre AS repartidor, COUNT(p.id) AS total_paquetes FROM empleados e JOIN paquetes p ON p.repartidor_id = e.id GROUP BY e.nombre ORDER BY total_paquetes DESC",
            20,
        ),
        plantilla_reporte_agregado(
            "postafeta_reporte_clientes_por_ciudad",
            "Marketing",
            "Marketing quiere saber en qué ciudades concentrar la próxima campaña.",
            "¿Cuántos clientes tenemos registrados en cada ciudad? Ordena de mayor a menor.",
            "SELECT ciudad, COUNT(*) AS total_clientes FROM clientes GROUP BY ciudad ORDER BY total_clientes DESC",
            15,
        ),
        plantilla_depuracion(
            "postafeta_depuracion_paquetes_cliente",
            "Kevin",
            "Kevin heredó este reporte de otro becario que ya no está (nadie sabe quién).",
            "Esta consulta tarda demasiado. Encuentra una forma más directa de obtener lo mismo.",
            "SELECT id, (SELECT c.nombre FROM clientes c WHERE c.id = paquetes.cliente_id) AS cliente, estado FROM paquetes ORDER BY id",
            "SELECT p.id, c.nombre AS cliente, p.estado FROM paquetes p JOIN clientes c ON p.cliente_id = c.id ORDER BY p.id",
            vec![Arquetipo::Join],
            20,
        ),
        plantilla_depuracion(
            "postafeta_depuracion_incidencias_por_tipo",
            "Finanzas",
            "Finanzas quiere este reporte, pero Kevin dice que \"últimamente se cuelga\".",
            "Optimiza este reporte para que no repita el mismo conteo una y otra vez.",
            "SELECT DISTINCT tipo, (SELECT COUNT(*) FROM incidencias i2 WHERE i2.tipo = i1.tipo) AS total FROM incidencias i1 ORDER BY total DESC",
            "SELECT tipo, COUNT(*) AS total FROM incidencias GROUP BY tipo ORDER BY total DESC",
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

    #[test]
    fn catalogo_tiene_4_reportes_y_2_depuraciones() {
        let tickets = catalogo();
        assert_eq!(tickets.len(), 6);
        let reportes = tickets.iter().filter(|t| t.tipo == TipoTicket::ReporteAnalisis).count();
        let depuraciones = tickets.iter().filter(|t| t.tipo == TipoTicket::InvestigacionDepuracion).count();
        assert_eq!(reportes, 4);
        assert_eq!(depuraciones, 2);
    }

    #[tokio::test]
    async fn todas_las_queries_doradas_ejecutan() {
        let pg = db::init_embedded_postgres().await.expect("Postgres embebido debe arrancar");
        let pool = db::load_company(&pg, Company::Postafeta).await.expect("Postafeta debe cargar");

        for ticket in catalogo() {
            db::run_query(&pool, &ticket.sql_dorada)
                .await
                .unwrap_or_else(|e| panic!("la query dorada de '{}' debe ejecutar: {e}", ticket.id));
        }

        pool.close().await;
        pg.stop().await.expect("Postgres debe detenerse limpiamente");
    }

    #[tokio::test]
    async fn los_tickets_de_depuracion_son_correctos_pero_lentos() {
        let pg = db::init_embedded_postgres().await.expect("Postgres embebido debe arrancar");
        let pool = db::load_company(&pg, Company::Postafeta).await.expect("Postafeta debe cargar");

        for ticket in catalogo().into_iter().filter(|t| t.sql_inicial.is_some()) {
            let sql_inicial = ticket.sql_inicial.clone().unwrap();
            let evaluacion = validation::evaluar_entrega(&pool, &sql_inicial, &ticket.sql_dorada, ticket.requiere_orden)
                .await
                .unwrap_or_else(|e| panic!("la evaluación de '{}' debe ejecutar: {e}", ticket.id));

            assert!(evaluacion.correcta, "'{}': sql_inicial debe dar el mismo resultado que sql_dorada", ticket.id);
            assert!(
                evaluacion.puntaje_velocidad < 100.0,
                "'{}': sql_inicial debe costar más que sql_dorada (puntaje_velocidad={})",
                ticket.id,
                evaluacion.puntaje_velocidad
            );
        }

        pool.close().await;
        pg.stop().await.expect("Postgres debe detenerse limpiamente");
    }
}
```

- [ ] **Step 2: Reemplazar `app/src-tauri/src/tickets/mod.rs` para agregar `postafeta` y el dispatcher**

Localizar la línea agregada en la Tarea 2:

```rust
mod hospital_arcangel;
```

Reemplazar por:

```rust
mod hospital_arcangel;
mod postafeta;

/// Catálogo de tickets de `company` (Etapa 14) — generado por las plantillas
/// paramétricas de este módulo, nunca escrito a mano ticket por ticket.
pub fn catalogo(company: crate::db::Company) -> Vec<Ticket> {
    match company {
        crate::db::Company::HospitalArcangel => hospital_arcangel::catalogo(),
        crate::db::Company::Postafeta => postafeta::catalogo(),
    }
}
```

- [ ] **Step 3: Agregar un test del dispatcher en `tickets/mod.rs`**

Dentro del `#[cfg(test)] mod tests` ya existente (de la Tarea 1), agregar:

```rust
    #[test]
    fn catalogo_devuelve_6_tickets_para_cada_empresa() {
        assert_eq!(catalogo(crate::db::Company::HospitalArcangel).len(), 6);
        assert_eq!(catalogo(crate::db::Company::Postafeta).len(), 6);
    }
```

- [ ] **Step 4: Correr la suite completa**

Run: `cd app/src-tauri && cargo test --lib -- --nocapture`
Expected: todos los tests en verde, 0 failed — más los 4 nuevos (3 de `tickets::postafeta::tests` + 1 de `tickets::tests::catalogo_devuelve_6_tickets_para_cada_empresa`).

- [ ] **Step 5: Commit**

```bash
git add app/src-tauri/src/tickets/postafeta.rs app/src-tauri/src/tickets/mod.rs
git commit -m "Add Postafeta ticket catalog and per-company dispatcher"
```

---

### Task 4: Conectar el catálogo a la app y retirar el ticket único obsoleto

**Files:**
- Modify: `app/src-tauri/src/lib.rs`
- Modify: `app/src-tauri/src/db/mod.rs`
- Modify: `app/src-tauri/src/db/hospital_arcangel.rs`
- Modify: `app/src/main.js`

**Interfaces:**
- Consumes: `tickets::catalogo(db::Company) -> Vec<tickets::Ticket>` (Tarea 3), `validation::evaluar_entrega` (sin cambios)
- Produces: comando Tauri `ticket_actual` ahora devuelve `tickets::Ticket` (serializado, no más `&'static str`); `submit_ticket` usa el ticket actual del catálogo en vez de las constantes fijas; el catálogo obsoleto de un solo ticket (`db::TICKET_ENUNCIADO`/`TICKET_SOLUCION`/su re-export) se elimina por completo — ya no queda ningún consumidor.

Este plan explícitamente **no** construye bandeja de entrada, tiempo de turno, ni auto-avance de la UI tras enviar un ticket (Etapa 11-A, plan de UI/loop posterior) — el único cambio de frontend es el mínimo necesario para no romper la muestra del ticket actual con los datos nuevos.

- [ ] **Step 1: Leer el estado actual de `app/src-tauri/src/lib.rs` y confirmar la forma exacta antes de editar**

Las tareas anteriores de este proyecto han encontrado pequeñas diferencias entre lo que un brief asume y el archivo real — leer primero con la herramienta `Read` y confirmar que el `struct EmbeddedPostgres`, `ticket_actual`, `submit_ticket`, y el bloque `.setup(...)` coinciden en esencia con lo descrito abajo antes de aplicar los cambios.

- [ ] **Step 2: Agregar el nuevo estado `Tickets` (justo después de `struct EmbeddedPostgres { ... }`)**

```rust
/// Catálogo de tickets de la empresa activa + índice del ticket actual
/// (Etapa 14). Selección round-robin simple — sin bandeja de entrada ni
/// tiempo de turno todavía (Etapa 11-A, plan de UI/loop posterior).
struct Tickets {
    catalogo: Vec<tickets::Ticket>,
    indice_actual: Mutex<usize>,
}
```

- [ ] **Step 3: Reemplazar el comando `ticket_actual`**

Localizar:

```rust
#[tauri::command]
fn ticket_actual() -> &'static str {
    db::TICKET_ENUNCIADO
}
```

Reemplazar por:

```rust
#[tauri::command]
fn ticket_actual(tickets: tauri::State<'_, Tickets>) -> tickets::Ticket {
    let indice = *tickets.indice_actual.lock().unwrap();
    tickets.catalogo[indice].clone()
}
```

- [ ] **Step 4: Reemplazar el comando `submit_ticket`**

Localizar el cuerpo actual (que usa `db::TICKET_SOLUCION_ACTUAL` y `true` fijos) y reemplazar por:

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

(Nota: el `mensaje` de error cambia de "Revisa tu WHERE/ORDER BY" — específico del ticket único viejo — a un texto genérico "Revisa tu consulta", porque ahora los tickets varían en qué conceptos SQL necesitan.)

- [ ] **Step 5: Actualizar el bloque `.setup(...)` para construir y registrar `Tickets`**

Localizar dentro de `tauri::async_runtime::block_on(async move { ... })`:

```rust
                let pool = db::load_company(&pg, db::Company::HospitalArcangel)
                    .await
                    .expect("no se pudo cargar Hospital Arcángel");
                handle.manage(AppState { pool });
                handle.manage(Perk(Mutex::new(PerkState::default())));
                handle.manage(EmbeddedPostgres(Mutex::new(Some(pg))));
```

Reemplazar por:

```rust
                let pool = db::load_company(&pg, db::Company::HospitalArcangel)
                    .await
                    .expect("no se pudo cargar Hospital Arcángel");
                let catalogo = tickets::catalogo(db::Company::HospitalArcangel);
                handle.manage(AppState { pool });
                handle.manage(Perk(Mutex::new(PerkState::default())));
                handle.manage(Tickets { catalogo, indice_actual: Mutex::new(0) });
                handle.manage(EmbeddedPostgres(Mutex::new(Some(pg))));
```

- [ ] **Step 6: Retirar el ticket único obsoleto de `db/mod.rs`**

Localizar y **borrar por completo** la línea:

```rust
pub(crate) use hospital_arcangel::{TICKET_ENUNCIADO, TICKET_SOLUCION as TICKET_SOLUCION_ACTUAL};
```

(Tras los Steps 3-5, `lib.rs` ya no usa `db::TICKET_ENUNCIADO` ni `db::TICKET_SOLUCION_ACTUAL` — dejar el re-export produciría una advertencia de código muerto, el mismo tipo de problema que la revisión final del Plan 2 ya encontró y corrigió una vez.)

- [ ] **Step 7: Retirar las constantes obsoletas de `db/hospital_arcangel.rs` y ajustar su único test**

Leer el archivo actual con la herramienta `Read` para confirmar la ubicación exacta antes de editar. Borrar por completo las constantes `pub const TICKET_ENUNCIADO` y `pub(crate) const TICKET_SOLUCION` (con sus comentarios de documentación). Dentro de `mod tests`, localizar el `use super::TICKET_SOLUCION;` (agregado en el Plan 2) y borrarlo también — ya no hace falta.

En el test `hospital_arcangel_end_to_end`, localizar la comprobación de auto-consistencia que usaba la constante:

```rust
        let jugador = run_query(&pool, TICKET_SOLUCION).await.unwrap();
        let esperado = run_query(&pool, TICKET_SOLUCION).await.unwrap();
        assert_eq!(jugador.rows, esperado.rows, "la solución del ticket debe pasar contra sí misma");
```

Reemplazar por la misma comprobación, con la query como literal en línea (ya no depende de una constante — el propósito del test es solo confirmar que `run_query` es determinista al llamarlo dos veces con el mismo SQL, no probar "el" ticket):

```rust
        let query_de_referencia =
            "SELECT nombre, fecha_ingreso, diagnostico FROM pacientes WHERE departamento_id = 1 ORDER BY fecha_ingreso DESC";
        let jugador = run_query(&pool, query_de_referencia).await.unwrap();
        let esperado = run_query(&pool, query_de_referencia).await.unwrap();
        assert_eq!(jugador.rows, esperado.rows, "la misma query debe dar el mismo resultado ejecutada dos veces");
```

- [ ] **Step 8: Ajuste mínimo de compatibilidad en `app/src/main.js`**

Localizar dentro de `window.addEventListener("DOMContentLoaded", ...)`:

```js
  ticketEnunciado.textContent = await invoke("ticket_actual");
  sqlInput.value = "SELECT * FROM pacientes;";
```

Reemplazar por:

```js
  const ticket = await invoke("ticket_actual");
  ticketEnunciado.textContent = `Motivo: ${ticket.motivo}\nSolicitud: ${ticket.solicitud}`;
  sqlInput.value = ticket.sql_inicial || "SELECT * FROM pacientes;";
```

(`ticket_actual` ahora devuelve el objeto `Ticket` completo en vez de un string — este es el único ajuste necesario para que la consola siga mostrando texto legible y para precargar la query a optimizar en los tickets de Investigación/Depuración. No se toca nada más de la UI.)

- [ ] **Step 9: Verificar que compila**

Run: `cd app/src-tauri && cargo check`
Expected: `Finished` sin errores **ni warnings** (en particular, sin `dead_code` sobre las constantes retiradas en el Step 6/7).

- [ ] **Step 10: Correr la suite completa**

Run: `cd app/src-tauri && cargo test --lib -- --nocapture`
Expected: todos los tests en verde, 0 failed (sin tests nuevos en este paso — es wiring + retiro de código obsoleto).

- [ ] **Step 11: Smoke test de la app real**

Run: `cd app && npm run tauri dev` en segundo plano; esperar a que compile y arranque (buscar la línea `Running \`target/debug/app\`` en el log, sin pánico, y confirmar con `ps aux` que el proceso vive). Verificar visualmente que el ticket mostrado ya no es "[object Object]" sino el texto de motivo/solicitud. Detener limpiamente el proceso (`npm`/`tauri dev`/`target/debug/app`, y cualquier Postgres embebido que la app haya arrancado) antes de terminar.

- [ ] **Step 12: Commit**

```bash
git add app/src-tauri/src/lib.rs app/src-tauri/src/db/mod.rs app/src-tauri/src/db/hospital_arcangel.rs app/src/main.js
git commit -m "Wire ticket catalog into the app, retire the single-ticket spike constants"
```

---

## Self-Review Notes

- **Cobertura del spec:** Etapa 14 (ambos tipos de ticket ✓, generación por plantillas paramétricas cruzadas con el esquema de cada empresa ✓, rúbrica propia por ticket almacenada como pesos ✓) — alcance MVP (Etapa 19: 2 empresas ✓).
- **Decisión "lenta no rota" verificada antes de escribir este plan:** las 4 queries de Investigación/Depuración (2 por empresa) ya se corrieron contra Postgres embebido real durante el brainstorming, confirmando `correcta = true` y `puntaje_velocidad < 100` en las 4 — no queda como incógnita para el implementador.
- **Fuera de alcance deliberado (para planes posteriores):** combinar los 3 puntajes con los pesos de la rúbrica en un `puntaje_base` y calcular dinero/reputación/XP real (Etapa 12, economía) — este plan solo guarda los pesos; bandeja de entrada/tiempo de turno y auto-avance de UI tras enviar (Etapa 11-A) — la Tarea 4 explícitamente no los construye; panel de tablas relevantes (Etapa 11-C) — no se deriva ni almacena.
- **Lección aplicada de la revisión final del Plan 2:** la Tarea 4 retira de forma proactiva el ticket único obsoleto (`TICKET_ENUNCIADO`/`TICKET_SOLUCION`/su re-export) en el mismo plan que los vuelve innecesarios, en vez de dejarlos como código muerto para que una revisión final posterior los descubra.
- **Consistencia de tipos:** `Ticket`/`TipoTicket`/`Prioridad`/`Arquetipo` (Tarea 1) se usan con los mismos nombres de campo en `hospital_arcangel.rs`/`postafeta.rs` (Tareas 2-3) y en `lib.rs` (Tarea 4) — sin conversiones sorpresa.

---

## Execution Handoff

Plan completo y guardado en `docs/superpowers/plans/2026-07-12-fase0-03-generacion-tickets.md`. Dos opciones de ejecución:

1. **Subagent-Driven (recomendado)** — despacho un subagente fresco por tarea, reviso el resultado entre cada una antes de seguir
2. **Ejecución inline** — ejecuto las tareas en esta sesión con executing-plans, ejecución por lotes con checkpoints

¿Cuál prefieres?
