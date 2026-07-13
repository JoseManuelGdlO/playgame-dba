# Fase 0 / Plan 6: Núcleo del Loop de Juego — Design

**Estado:** Aprobado (brainstorming)
**Fecha:** 2026-07-12
**Etapa de referencia:** Etapa 11 (Mecánicas), secciones A (bandeja/turno), B (consola), H (scoring) — dentro del alcance de Etapa 19 (MVP)

## Contexto

Hoy el juego funciona (esquema/datos, validación SQL, tickets, economía, perks — Planes 1-5), pero la experiencia real de jugarlo sigue siendo la del spike original: un ticket "actual" que avanza en round-robin al resolverse, un textarea plano, y el resultado del ticket mostrado como un simple mensaje de texto. Este plan construye el núcleo real del loop descrito en la Etapa 11: bandeja de tickets con presupuesto de tiempo por turno, una consola que ejecuta selección de texto, y una pantalla de resultado con el desglose de puntaje.

## Alcance

- **Bandeja/turno (Etapa 11-A):** lote de 3 tickets por turno (rotación determinista del catálogo, sin aleatoriedad), presupuesto de tiempo fijo de 100 unidades por turno, consumido por `costo_tiempo` de cada ticket al resolverse (correcto o no). Al agotarse el presupuesto o cerrar el día manualmente, los tickets no atendidos escalan (penalización de reputación = `factor_reputacion × 2.0`) y empieza un turno nuevo.
- **Consola real (Etapa 11-B):** ejecución consciente de selección de texto — si hay texto seleccionado en el editor, "▶ Play" corre solo eso; si no, corre todo el contenido (comportamiento actual, preservado como fallback).
- **Pantalla de scoring (Etapa 11-H):** tras enviar un ticket, un panel muestra el desglose de puntaje (correctitud/velocidad/prácticas, dinero/reputación ganados) con animación de conteo, más el comentario de El Mentor cuando aplica. 100% frontend — todos los datos ya existen en `ScoreResult`.

## Fuera de alcance

- Multi-statement/múltiples pestañas de resultado simultáneas (Etapa 11-B completa) — el backend solo ejecuta una query a la vez; esto queda para un plan de UI posterior.
- Visor ERD (Etapa 11-C) y chat de El Mentor como componente dedicado (Etapa 11-E) — el comentario ya se muestra en la pantalla de scoring, pero no como un componente de chat separado.
- Interrupciones/eventos aleatorios (Etapa 11-F) — explícitamente fuera del MVP (Etapa 19).
- Transición de empresa/Agencia y mini-boss (Etapa 11-G) — plan aparte.
- Verificación visual: esta app corre en una ventana nativa de Tauri, no en navegador — no hay herramienta de captura visual en este entorno. La verificación de este plan se apoya en `cargo check`/`cargo test` + revisión cuidadosa del HTML/CSS/JS, no en confirmación visual directa.

## Arquitectura

- `app/src-tauri/src/turno/mod.rs` (nuevo): `EstadoTurno { presupuesto_restante: u32, pendientes: Vec<Ticket> }`, lógica de sorteo del lote (rotación determinista), consumo de tiempo al resolver, escalamiento (penalización de reputación) al agotar presupuesto o cerrar el día.
- `lib.rs`: reemplaza el estado `Tickets{catalogo, indice_actual}` (Plan 3) por el nuevo `Turno` gestionado por Tauri; nuevos comandos `turno_actual()`, `resolver_ticket(id, sql)` (reemplaza `submit_ticket`), `cerrar_dia()`.
- `app/src/main.js`/`index.html`: la bandeja se renderiza como una lista de tickets pendientes (no un ticket único); el editor gana lógica de selección de texto para "▶ Play"; nueva pantalla/panel de scoring con animación de conteo tras enviar un ticket.

## Fórmula de penalización por escalamiento (Etapa 12, valor de partida)

```
penalizacion_reputacion = ticket.factor_reputacion × 2.0
```
Aplicada una vez por cada ticket que queda pendiente al cerrar/agotar el turno. Sujeta a ajuste en playtesting, como el resto de los valores económicos.

## Testing

El sorteo de lote, el consumo de presupuesto, y el escalamiento son lógica pura (sin base de datos) — tests unitarios directos. La integración completa (`resolver_ticket` llamando a `validation`/`economia` reales) sigue el mismo patrón de tests de integración contra Postgres embebido ya usado en planes anteriores. La consola/scoring (frontend) se verifica por revisión de código, no visualmente, según lo indicado arriba.

## Auto-revisión del spec

- **Placeholders:** ninguno — la fórmula de penalización, el tamaño del lote, y el presupuesto de turno están fijados arriba, con su razón.
- **Consistencia interna:** el modelo de datos (`EstadoTurno`), los comandos nuevos, y el frontend usan los mismos nombres (`presupuesto_restante`, `pendientes`, `resolver_ticket`) de principio a fin.
- **Alcance:** cubre 3 secciones relacionadas de la Etapa 11 (A, B, H) que comparten el mismo ciclo (bandeja → consola → scoring) — con recortes explícitos de las partes que no comparten ese ciclo (ERD, chat de Mentor, interrupciones, transición de empresa).
- **Ambigüedad:** ninguna — se reconoce explícitamente la limitación de verificación visual en este entorno, no es un olvido.
