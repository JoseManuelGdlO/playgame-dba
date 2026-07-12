# Fase 0 / Plan 3: Generación de Tickets por Plantillas — Design

**Estado:** Aprobado (brainstorming)
**Fecha:** 2026-07-12
**Etapa de referencia:** Etapa 14 (Sistema de Misiones), dentro del alcance de Etapa 19 (MVP)

## Contexto

Hoy (tras el Plan 1: esquema/datos, y el Plan 2: motor de validación SQL) el juego tiene exactamente **un** ticket, hardcodeado como dos constantes de texto (`TICKET_ENUNCIADO`/`TICKET_SOLUCION`) en `db/hospital_arcangel.rs`. Este plan reemplaza eso por un catálogo de tickets generado por plantillas paramétricas, para las 2 empresas del MVP (Hospital Arcángel, Postafeta), cubriendo los 2 tipos de ticket que la Etapa 14 define para el rango Becario/Auxiliar de Sistemas: **Reporte/Análisis** y **Investigación/Depuración**.

## Alcance

- **Tipos de ticket:** ambos — Reporte/Análisis (plantilla paramétrica de pregunta de negocio) e Investigación/Depuración (se entrega una query ya escrita para arreglar/optimizar).
- **Empresas:** ambas — Hospital Arcángel y Postafeta, cada una con su propio catálogo concreto generado por las mismas funciones-plantilla genéricas.
- **Profundidad del catálogo:** 4 plantillas de Reporte/Análisis + 2 de Investigación/Depuración por empresa (12 tickets concretos en total).
- **Fuera de alcance (para planes posteriores):**
  - Combinar los 3 puntajes crudos (Plan 2) con los pesos de la rúbrica de cada ticket en un `puntaje_base` — este plan solo *almacena* los pesos como metadata del ticket; el *cálculo* es del plan de economía (Etapa 12).
  - Bandeja de entrada, tiempo de turno, escalamiento de tickets no atendidos (Etapa 11-A) — selección de "ticket actual" en este plan es un round-robin simple, sin turnos.
  - XP por arquetipo — el campo `arquetipos` en cada ticket es metadata capturada ahora, consumida después.
  - Panel de "tablas relevantes" (Etapa 11-C) — no se deriva ni se almacena en este plan (no hay nada que lo consuma todavía).
  - Escritura de datos/DDL — sigue fuera de alcance para este rango (Etapa 14).

## Decisión de diseño: tickets de Investigación/Depuración son "lentos", no "rotos"

La Etapa 14 permite que la query entregada al jugador esté "rota o lenta". Este plan usa **solo el caso "lenta pero con el mismo resultado correcto"** (nunca una query que produzca un resultado de negocio distinto/incorrecto), por dos razones:
1. Encaja de forma directa con el motor de velocidad ya construido en el Plan 2 (comparación de costo de `EXPLAIN`), sin necesitar lógica nueva de validación.
2. Evita la ambigüedad de diseño de "qué tan rota" debe estar una query con resultado incorrecto — "lenta pero correcta" es una propiedad verificable de forma determinista (se comprueba con `correctness::son_equivalentes`, ya construido).

## Arquitectura

Mismo patrón modular que `db/` (Plan 1): un archivo por empresa, sin una empresa nueva sin tocar el motor genérico.

- `app/src-tauri/src/tickets/mod.rs` — modelo de datos (`Ticket`, `TipoTicket`, `Prioridad`, `Arquetipo`) + `pub fn catalogo(company: db::Company) -> Vec<Ticket>` (dispatcher).
- `app/src-tauri/src/tickets/hospital_arcangel.rs` — funciones-plantilla (una función Rust parametrizada por plantilla, no un motor genérico de placeholders) + la lista concreta de 6 tickets de Hospital Arcángel.
- `app/src-tauri/src/tickets/postafeta.rs` — mismo patrón para Postafeta.

**Por qué funciones Rust parametrizadas y no un motor genérico de sustitución de texto:** con ~12 plantillas totales, un motor genérico de placeholders (`{top_n}`, `{entidad}`, etc.) para sincronizar texto narrativo y SQL es más aparato del que este alcance necesita, y es frágil (nada garantiza en tiempo de compilación que el placeholder de la narrativa y el de la columna SQL sigan alineados). Una función Rust que arma texto + SQL juntos, en el mismo lugar, con los tipos del compilador de por medio, sigue siendo "una plantilla, múltiples instancias" (llamada varias veces con distintos argumentos saca variedad) sin ese riesgo.

## Modelo de datos

```rust
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

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub enum TipoTicket {
    ReporteAnalisis,
    InvestigacionDepuracion,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
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
```

`sql_inicial` es `Some(...)` solo para `TipoTicket::InvestigacionDepuracion` (la query a arreglar, precargada en el editor); `None` para `ReporteAnalisis` (el jugador escribe desde cero).

## Catálogo por empresa (nivel conceptual — el texto/SQL exacto se redacta y verifica en el plan de implementación)

**Hospital Arcángel** (6 tablas: departamentos, empleados, seguros, pacientes, tratamientos, habitaciones):
- Reporte 1: pacientes de un departamento ordenados por fecha de ingreso (Becario: `SELECT/WHERE/ORDER BY`) — evolución del ticket único actual del spike.
- Reporte 2: costo total de tratamientos agrupado por tipo (Auxiliar: `GROUP BY/SUM`).
- Reporte 3: empleados por departamento con su jefe directo (Auxiliar: `JOIN`).
- Reporte 4: habitaciones ocupadas por departamento (Auxiliar: `JOIN/GROUP BY/COUNT`).
- Depuración 1 y 2: dos queries lentas-pero-correctas (p. ej. subconsulta correlacionada innecesaria en vez de `JOIN`, o un scan sin aprovechar un predicado directo) — la query dorada de cada una es la versión bien escrita equivalente.

**Postafeta** (5 tablas: sucursales, empleados, clientes, paquetes, incidencias): mismo patrón — reportes sobre paquetes por sucursal/estado, incidencias por tipo, repartidor con más entregas, etc., más 2 queries lentas-pero-correctas.

## Integración con la app

- `AppState` (en `lib.rs`) gana `tickets: Vec<tickets::Ticket>` (el catálogo de la empresa activa, cargado al arrancar) más un índice de "ticket actual" (`Mutex<usize>`).
- `ticket_actual` (comando Tauri) deja de devolver `&'static str` y pasa a devolver el `Ticket` completo serializado a JSON (el frontend, en un plan de UI posterior, decide qué mostrar de motivo/solicitud/sql_inicial).
- Selección **round-robin**: al enviar un ticket con éxito, el índice avanza al siguiente (`(indice + 1) % tickets.len()`). Sin bandeja de entrada ni tiempo de turno todavía — eso es explícitamente un plan de UI/loop posterior (Etapa 11-A).
- `submit_ticket` usa `ticket.sql_dorada` y `ticket.requiere_orden` del ticket actual (no más las constantes `TICKET_SOLUCION_ACTUAL` fijas) al llamar a `validation::evaluar_entrega`.

## Testing

- Por cada ticket generado: test de integración (Postgres embebido real, reutilizando `db::load_company`) que confirma que `sql_dorada` ejecuta sin error.
- Para los tickets de Investigación/Depuración: test adicional que confirma `sql_inicial` y `sql_dorada` producen resultados **equivalentes** (reutilizando `validation::correctness::son_equivalentes` del Plan 2) — verificación automática de que "lenta" no se volvió "rota" por accidente.
- Tests de conteo: el catálogo de cada empresa tiene exactamente 4 tickets de Reporte/Análisis + 2 de Investigación/Depuración.

## Auto-revisión del spec

- **Placeholders:** ninguno — cada decisión de alcance tiene una respuesta concreta (tipos incluidos, empresas incluidas, profundidad, "lenta no rota").
- **Consistencia interna:** el modelo de datos, el catálogo conceptual y la integración con `lib.rs` usan los mismos nombres de campos (`sql_dorada`, `requiere_orden`, `sql_inicial`) de principio a fin.
- **Alcance:** un solo subsistema (generación de tickets), sin mezclar economía/RPG/UI — cabe en un plan de implementación.
- **Ambigüedad:** el texto narrativo y el SQL literal de cada plantilla se dejan para el plan de implementación (donde se prototipan y verifican contra Postgres real, igual que el Plan 2), no para este spec — explícitamente señalado arriba, no es un placeholder olvidado.
