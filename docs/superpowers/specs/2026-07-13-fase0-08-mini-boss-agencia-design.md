# Fase 0 / Plan 8: Mini-boss (Auditor de Cumplimiento) + Transición de Agencia — Design

**Estado:** Aprobado (brainstorming)
**Fecha:** 2026-07-13
**Etapa de referencia:** Etapa 7 (Dirección creativa, estructura de arco por empresa), Etapa 9 (Agencia/Grupo Ómega RH), Etapa 11-G (transición de empresa), Etapa 12 (reset de reputación al cambiar de empresa) — dentro del alcance de Etapa 19 (MVP)

## Contexto

El Plan 7 implementó el ascenso Becario → Auxiliar de Sistemas usando solo el umbral de reputación existente, dejando explícitamente fuera el mini-boss ("Auditor de Cumplimiento") y la transición de empresa vía la Agencia — ambos documentados como "plan aparte" en los Planes 6 y 7. El spec original (Etapa 10/11-G) describe el mini-boss y la transición como un solo evento que dispara el ascenso de rango; este plan los trata como un hito **separado y posterior** al ascenso (ya construido y probado en Plan 7) para no reabrir esa lógica. Los specs de diseño nunca definieron el mini-boss mecánicamente más allá de "combina 2+ tipos de ticket anteriores en una secuencia narrativa de alto riesgo" con `factor_reputacion` más alto — este plan resuelve esa ambigüedad de la forma más simple que reutiliza mecánica ya existente. Hoy no existe ninguna plomería para cambiar de empresa en tiempo de ejecución: `Company::Postafeta` está marcado `#[allow(dead_code)]` porque nada en producción lo construye todavía.

## Alcance

- **Disparo:** al ascender a Auxiliar de Sistemas (reutilizando la señal `ascendio: bool` que `EstadoJugador::aplicar_resultado` ya emite desde el Plan 7), la bandeja se reemplaza de inmediato por un lote dedicado de 2 tickets del Auditor de Cumplimiento — uno con `Arquetipo::Join`, otro con `Arquetipo::Agregacion`, ambos con `factor_reputacion`/`valor_base` más altos que el promedio del catálogo normal — sin mezclarse con tickets normales de Hospital Arcángel.
- **Catálogo del mini-boss:** nueva función `tickets::mini_boss_hospital_arcangel() -> Vec<Ticket>` (2 tickets fijos, construidos con las plantillas ya existentes `plantilla_reporte_join`/`plantilla_reporte_agregado` o `plantilla_depuracion`). Ningún campo nuevo en `Ticket` — lo que distingue a estos tickets es cómo se entregan (un lote dedicado), no un atributo propio.
- **Fase del arco:** `TurnoManejado` gana `fase: FaseArco` con variantes `TrabajoNormal`, `MiniBoss`, `ArcoCompletado`. En `resolver_ticket`: si `aplicar_resultado` devuelve `ascendio == true`, se reemplaza `actual` por `EstadoTurno::nuevo(&mini_boss_hospital_arcangel(), 0)` y `fase = MiniBoss`, saltando el chequeo normal de "pendientes vacíos/turno agotado → turno nuevo" para esa misma llamada. Si `fase == MiniBoss` y `pendientes` queda vacío tras resolver el segundo ticket, `fase = ArcoCompletado` (no se dibuja un turno nuevo).
- **Resolución de los tickets del mini-boss:** pasan por el mismo `resolver_ticket`/pantalla de scoring de siempre — correctos o no, se retiran del lote y consumen su `costo_tiempo` igual que cualquier ticket (sin mecánica de "fallo especial"). "Superar" al mini-boss significa simplemente vaciar ese lote de 2, sin exigir que ambos hayan sido correctos — consistente con que el resto del juego nunca bloquea el avance por errores, solo penaliza con el puntaje/economía ya existentes.
- **Transición de Agencia:** cuando `fase == ArcoCompletado`, el frontend muestra un overlay ("reasignación de la Agencia") con el mismo patrón visual que el overlay de scoring: Postafeta como única opción + una línea de sabor absurdista, y un botón para confirmar. Al confirmar, se invoca un nuevo comando async `confirmar_transicion_agencia()`.
- **Qué hace `confirmar_transicion_agencia()`:** carga el pool de Postafeta con `db::load_company` (ya existe y es paramétrico, hoy solo usado en tests), reconstruye `TurnoManejado` con el catálogo de Postafeta filtrado por el rango actual del jugador (reutilizando `tickets::tickets_elegibles`, ya extraído en el Plan 7), resetea `EstadoJugador.reputacion` a `0.0`, y deja `fase = TrabajoNormal`. Dinero, XP por arquetipo, perks (desbloqueados/equipados) y rango **se mantienen** — solo la reputación resetea (Etapa 12: "eres el nuevo" en la empresa nueva).
- **`AppState` pasa de un `PgPool` suelto a `Mutex<EstadoConexion { pool, empresa }>`** para permitir el swap en caliente y para que el invariante "pool y catálogo deben ser siempre de la misma empresa" (hoy documentado como riesgo en un comentario de `lib.rs`, no protegido por ningún tipo) quede resguardado por el mismo lock en vez de vivir en dos lugares separados que podrían divergir.
- **Frontend:** la bandeja durante `FaseArco::MiniBoss` reutiliza el componente de lista de tickets existente, con un encabezado distinto ("El Auditor de Cumplimiento quiere verte"). El overlay de Agencia reutiliza el patrón de `#scoring-overlay` (nueva sección oculta por defecto). Ningún componente nuevo de UI.

## Fuera de alcance

- **Elección real entre 2-3 empresas:** el spec completo (Etapa 11-G) describe una pantalla con 2-3 opciones de empresa siguiente. El MVP solo tiene Postafeta como destino — se muestra como una única opción a confirmar (mantiene el "ritual" de la Agencia) en vez de construir un selector genérico de N empresas que hoy no tendría para qué elegir.
- **Reputación base escalada por rango general:** Etapa 12 dice que el valor base al llegar a una empresa nueva "escala levemente con el rango general del jugador" — para este plan (solo hay un rango de transición posible: Becario→Auxiliar) se simplifica a resetear siempre a `0.0`. La fórmula de escalado se construye cuando haya más de un rango de transición real (Fase 1+).
- **Retroceder o repetir el arco de Hospital Arcángel:** una vez confirmada la transición, Hospital Arcángel queda atrás por completo (mismo criterio que el spec: "el esquema y elenco cambian por completo") — no se construye ningún mecanismo de volver.
- **Arte/portadas pixel art del Auditor de Cumplimiento:** Etapa 8 describe portadas dedicadas para mini-bosses; fuera de alcance del MVP (mismo criterio ya aplicado en Planes 6/7 — arte mínimo/placeholder).
- **Generalización a más empresas/mini-bosses:** `FaseArco` y `mini_boss_hospital_arcangel()` están nombrados y construidos específicamente para esta única transición del MVP, no como un sistema genérico de mini-bosses por empresa — eso es contenido de Fase 1+ (6 empresas y mini-bosses restantes).
- **Verificación visual automatizada:** mismo patrón ya usado en Planes 6/7 — esta app corre en ventana nativa de Tauri; la verificación end-to-end se hace guiada, no por un harness.

## Arquitectura

- `app/src-tauri/src/tickets/hospital_arcangel.rs`: nueva función `pub(crate) fn mini_boss() -> Vec<Ticket>` (o equivalente, expuesta vía `tickets::mini_boss_hospital_arcangel()` en `tickets/mod.rs`) — 2 tickets nuevos, ambos Auxiliar-tier por construcción (contienen `Join`/`Agregacion`).
- `app/src-tauri/src/turno/mod.rs` o `lib.rs` (a decidir en el plan de implementación): `enum FaseArco { TrabajoNormal, MiniBoss, ArcoCompletado }`, campo `fase` en `TurnoManejado`.
- `app/src-tauri/src/lib.rs`: `resolver_ticket` gana la rama de "si `ascendio`, reemplazar turno por el del mini-boss"; nuevo comando `confirmar_transicion_agencia()`; `AppState` se refactoriza a `Mutex<EstadoConexion { pool, empresa }>`.
- `app/src-tauri/src/db/mod.rs`: `Company::Postafeta` deja de ser `#[allow(dead_code)]` — primer uso real en producción.
- `app/src/index.html` / `main.js`: encabezado condicional de bandeja durante `FaseArco::MiniBoss`; nuevo overlay de Agencia (mismo patrón que scoring).

## Testing

- Rust: tests para `mini_boss_hospital_arcangel()` (2 tickets, ambos Auxiliar-tier vía `rango_requerido`), para la transición `TrabajoNormal → MiniBoss` dentro de `resolver_ticket` cuando `ascendio == true`, y para `MiniBoss → ArcoCompletado` al vaciar el lote de 2. La transición de empresa (`confirmar_transicion_agencia`) se prueba contra Postgres embebido real, siguiendo el mismo patrón de integración ya usado en `db::tests` (`ambas_empresas_conviven_en_el_mismo_servidor` ya existe y confirma que ambas bases de datos coexisten en el mismo servidor embebido).
- Verificación manual guiada en la app real (mismo patrón que Planes 6/7): ascender a Auxiliar → ver la bandeja del Auditor → resolver ambos tickets → ver el overlay de Agencia → confirmar → ver Postafeta activo con reputación en 0 y dinero/perks/rango intactos.

## Auto-revisión del spec

- **Placeholders:** ninguno — el disparo (`ascendio`), la fase (`FaseArco`), el reset de reputación (a `0.0`), y el destino único (Postafeta) están fijados arriba con su razón.
- **Consistencia interna:** el mini-boss no introduce ningún campo nuevo en `Ticket` ni en `EstadoJugador` más allá de lo estrictamente necesario (`fase` en `TurnoManejado`) — reutiliza `ascendio`, `tickets_elegibles`, y `db::load_company`, todos ya existentes de planes anteriores.
- **Alcance:** un solo mini-boss, una sola transición, explícitamente no generalizado — evita mezclar este plan con el roster completo de 8 empresas (Fase 1+).
- **Ambigüedad:** la forma del mini-boss (secuencia de 2 tickets, no un ticket único ni una UI nueva) y el momento de disparo (bandeja dedicada inmediata al ascender, no mezclado) fueron decisiones explícitas confirmadas con el usuario antes de escribir este documento, no supuestos silenciosos.
