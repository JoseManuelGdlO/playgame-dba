# Fase 0 / Plan 7: Ascenso de rango (Becario → Auxiliar de Sistemas) — Design

**Estado:** Aprobado (brainstorming)
**Fecha:** 2026-07-13
**Etapa de referencia:** Etapa 10 (Progresión) y Etapa 13 (Sistema RPG/Perks, hito de slot en Auxiliar de Sistemas) — dentro del alcance de Etapa 19 (MVP)

## Contexto

El Plan 6 dejó el loop core jugable (bandeja/consola/scoring), pero el "ascenso" de rango que la Etapa 19 exige como parte del MVP ("un ascenso, pero completo de principio a fin") no está implementado: hoy existe únicamente `UMBRAL_ASCENSO_AUXILIAR = 500.0` y una función pura `puede_ascender()` en `economia/mod.rs` que compara la reputación contra ese umbral sin ningún efecto — el propio código lo documenta como responsabilidad de "un plan posterior". Ese booleano ya viaja al frontend en cada `ScoreResult` pero se ignora ahí. No existe ningún tipo `Rango` en Rust, ni persistencia de rango en `EstadoJugador`, ni gating de tickets o de slots de perks por rango, ni UI que muestre el rango actual. Este plan cierra ese hueco para el único ascenso que cubre el MVP: Becario → Auxiliar de Sistemas.

## Alcance

- **Modelo de rango:** `pub enum Rango { Becario, AuxiliarDeSistemas }` (Default → `Becario`) y campo `pub rango: Rango` en `EstadoJugador`.
- **Disparo automático:** dentro de `resolver_ticket`, tras `aplicar_resultado`, si `estado.rango == Becario && estado.puede_ascender()` se asigna `estado.rango = Rango::AuxiliarDeSistemas` de inmediato — sin acción manual del jugador, y sin re-dispararse en tickets posteriores (el ascenso ya ocurrió).
- **Condición de ascenso = solo reputación por ahora.** El spec de Progresión (Etapa 10) describe el ascenso real como "superar el mini-boss de la empresa Y cruzar el umbral de reputación", pero el mini-boss de Hospital Arcángel (Auditor de Cumplimiento) no existe todavía y es un plan aparte. Este plan usa únicamente el umbral de reputación ya existente; cuando el mini-boss se construya, se puede añadir como condición adicional (`&&`) sin romper esta lógica.
- **Comunicación al frontend:** `ScoreResult` gana `ascendio: bool` (true solo en el turno exacto del cambio) y `rango_actual: Rango` (serializado, siempre presente). `turno::EstadoTurno` (o el comando `turno_actual` existente) también expone el rango vigente, para que el badge se pinte aunque el jugador no haya resuelto ningún ticket en la sesión actual.
- **Gating de tickets (Enfoque A — reutiliza el arquetipo SQL existente):** nueva función `rango_requerido(ticket: &Ticket) -> Rango` en `tickets/mod.rs` — devuelve `AuxiliarDeSistemas` si `ticket.arquetipos` contiene `Join` o `Agregacion`, si no `Becario`. En los call-sites que hoy llaman `tickets::catalogo(company)` (setup inicial y `cerrar_dia`), el catálogo se filtra por `rango_requerido(t) <= estado.rango` antes de pasarlo a `turno::EstadoTurno::nuevo`; `turno` no cambia su lógica de rotación, solo recibe un slice ya filtrado.
- **Slots de perk escalables:** `MAX_SLOTS_EQUIPADOS` deja de ser una constante fija; se vuelve `estado.max_slots() -> u8` → 2 para Becario, 3 para Auxiliar de Sistemas (hito explícito de la Etapa 13). `desbloquear_perk`/`equipar_perk` usan este método.
- **Frontend:** badge de rango junto a 💰/⭐ en el header, actualizado desde `rango_actual`. Cuando `ascendio == true`, la pantalla de scoring (Plan 6) agrega un bloque destacado: "¡Ascendiste a Auxiliar de Sistemas! +1 slot de perk. Nuevos tickets disponibles." — reutilizando el componente existente, sin pantalla nueva.

## Fuera de alcance

- **Mini-boss / transición de empresa (Etapa 11-G, Agencia):** el ascenso de este plan es puramente numérico (reputación); la validación narrativa completa ("clímax de empresa") y el paso a Postafeta quedan para un plan aparte, ya identificado en el roadmap.
- **Escalera completa de rangos/slots:** solo se modela Becario→Auxiliar (2 rangos, 2→3 slots). Los otros hitos de slot de la Etapa 13 (hasta 7 slots) y el resto de la escalera de 10 rangos de la Etapa 10 son contenido de Fase 1+.
- **Campo `rango_minimo` explícito por ticket (Enfoque B):** descartado para este plan — con solo 2 rangos, reutilizar la taxonomía de `arquetipos` ya existente cubre la necesidad sin mantenimiento duplicado. Si en Fase 1 aparecen necesidades narrativas que no correlacionen con el arquetipo SQL, se puede introducir entonces.
- **Persistencia de `EstadoJugador` en base de datos:** sigue en memoria (`Mutex`) como hoy; no es un hueco introducido por este plan, es el patrón ya existente en toda la app.
- **Verificación visual automatizada:** igual que en el Plan 6, esta app corre en ventana nativa de Tauri sin herramienta de captura automatizada en este entorno; la verificación end-to-end se hace guiada (como en la revisión del Plan 6), no por un harness.

## Arquitectura

- `app/src-tauri/src/economia/mod.rs`: nuevo `enum Rango`, campo `rango` en `EstadoJugador`, método `max_slots()`, lógica de ascenso automático (probablemente dentro de `aplicar_resultado` o inmediatamente después, en el mismo lugar donde hoy se computa `puede_ascender()` para `ScoreResult`).
- `app/src-tauri/src/tickets/mod.rs`: nueva función `rango_requerido(&Ticket) -> Rango`.
- `app/src-tauri/src/lib.rs`: los call-sites de `tickets::catalogo(company)` (setup y `cerrar_dia`) filtran por rango antes de construir/renovar el `Turno`; `ScoreResult` gana `ascendio` y `rango_actual`.
- `app/src/main.js` / `index.html`: badge de rango en el header; bloque de anuncio de ascenso en la pantalla de scoring cuando `ascendio == true`.

## Testing

- Rust: tests de `puede_ascender` ya existen y se mantienen. Se agregan tests para: el cambio de rango ocurre exactamente una vez al cruzar el umbral (no se re-dispara en tickets posteriores ya ascendidos), `max_slots()` devuelve 2/3 según rango, y `rango_requerido` clasifica correctamente tickets con distintas combinaciones de `arquetipos` (incluyendo un ticket con `Select` + `Join` → `AuxiliarDeSistemas`).
- Verificación manual en la app real (mismo patrón que la revisión del Plan 6): subir reputación hasta cruzar 500 → confirmar banner de ascenso en scoring → confirmar bandeja con tickets de Join/Agregación disponibles → confirmar 3er slot de perk habilitado en el catálogo de Perks.

## Auto-revisión del spec

- **Placeholders:** ninguno — el umbral (500.0) ya existe en código, la regla de gating (Select→Becario, Join/Agregación→Auxiliar) y el cambio de slots (2→3) están fijados arriba con su razón.
- **Consistencia interna:** el modelo (`Rango`, `rango_requerido`, `max_slots`) usa los mismos nombres desde el backend hasta los campos de `ScoreResult` consumidos por el frontend; no hay conversión sorpresa de tipos.
- **Alcance:** un solo ascenso, ya acotado explícitamente contra el mini-boss (fuera de alcance) y contra la escalera completa de rangos (Fase 1+) — evita mezclar este plan con la transición de empresa.
- **Ambigüedad:** ninguna decisión quedó abierta — condición de ascenso (solo reputación), disparo (automático), gating de tickets (sí, Enfoque A) y slots (sí, 2→3) fueron confirmados explícitamente antes de escribir este documento.
