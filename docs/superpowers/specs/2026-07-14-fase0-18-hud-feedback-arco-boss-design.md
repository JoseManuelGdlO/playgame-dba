# Fase 0 / Plan 18: HUD — feedback de ticket, progreso de arco y señal de boss — Design

**Estado:** Aprobado (brainstorming)
**Fecha:** 2026-07-14
**Etapa de referencia:** Etapa 10 (progresión / mini-boss), Plan 8 (mini-boss + agencia), Plan 11 (animación/audio), Plan 12 (hub carpeta)

## Contexto

El hub y el loop de tickets ya están jugables (bandeja, scoring, ascenso a Auxiliar, mini-boss Auditor, agencia → Postafeta). Al cerrar un ticket el jugador vuelve al hub sin una señal clara de “qué gané”; no ve cuánto le falta al climax de la empresa; y el modo Auditor se siente como un cambio silencioso de bandeja, sin presión de boss (ni música dedicada). La progresión real no es un catálogo finito que se agota: el boss se dispara por reputación (~500, señal `ascendio`) y la bandeja rota tickets. Este plan no reescribe esa regla — la hace legible y la celebra en el HUD.

## Decisiones de brainstorming

- Prioridad: feedback al cerrar ticket + pack de progreso (no solo una pieza aislada).
- Modelo de progreso: **híbrido** — barra reputación→Auditor + contador de turno (pendientes / presupuesto).
- Feedback de ticket: **toast + pops en badges + barra** (no solo uno de ellos).
- Señal de boss: **banner de transición** + que se sienta pelea (no solo título de bandeja).
- Atmósfera de boss: **música de presión + skin de alerta** en el hub (sin overlay/diálogo largo).
- Enfoque técnico: **híbrido mínimo frontend-first** — sin rehacer el arco en Rust; usar estado/scoring ya existentes.

## Alcance

- **Panel de arco en el hub** (cerca de empresa/bandeja):
  - En Hospital Arcángel / Becario camino a Auxiliar: barra `reputacion / 500` con etiqueta “Camino al Auditor” (o equivalente corto).
  - Siempre en turno activo: texto `pendientes` + `presupuesto_restante`.
  - En `fase === MiniBoss`: barra llena / estado completo + se mantiene el título de bandeja ya existente (“El Auditor de Cumplimiento quiere verte”).
  - En Postafeta (sin mini-boss): panel muestra solo datos de turno; **no** muestra “Camino al Auditor” ni techo 500 mentiroso.
- **Feedback al cerrar un ticket (solo cierre final, no reintentos):**
  - Al cerrar el overlay de scoring y pintar el hub: toast corto (~3s) con resultado + título del ticket + `+$` / `+rep` si aplica.
  - Pops temporales junto a badges de dinero/reputación del hub.
  - Actualización animada de la barra de arco cuando cambia la reputación.
  - Si quedan `intentos_restantes`: comportamiento actual (`#status-msg` únicamente) — sin toast/pops de celebración.
- **Modo boss (al entrar en MiniBoss / `ascendio`):**
  - Banner de transición una vez por entrada (“Ascenso · El Auditor te espera” o copy equivalente).
  - Clase CSS en `#pantalla-hub` (skin de alerta: acento de borde/título; papeles del lote mini-boss con estilo distinto si aplica).
  - Cambio de loop de música a patrón procedural más tenso/grave via `audio.js`; al salir de MiniBoss (`ArcoCompletado` / post-agencia en TrabajoNormal) volver al ambiente normal.
  - Respetar mute de música existente; no forzar audio si está muteado; al desmutear, el loop correcto debe sonar según actualización de fase.
- **Estado UI solo en JS** (`ultimoFeedback`, flags `modoBossActivo` / `bannerBossMostrado`) alimentado por `score` de `resolver_ticket` y `fase` de turno — sin nuevos comandos Tauri ni campos Rust.

### Archivos tocados (previsto)

- `app/src/index.html` — markup del panel de arco, toast, banner de boss.
- `app/src/main.js` — triggers al cerrar scoring / `pintarHub` / `renderBandeja`; estado `ultimoFeedback` y flags de boss.
- `app/src/styles.css` — toast, pops, barra, skin `.hub-boss` (o nombre equivalente), banner.
- `app/src/audio.js` — modo ambiente vs boss (mismo motor Web Audio, sin archivos externos).

## Fuera de alcance

- Catálogo finito “X de Y tickets únicos de la empresa” (cambiaría la progresión; no es el modelo híbrido acordado).
- Cutscene / overlay de diálogo largo del Auditor / retrato amenazante permanente.
- Archivos de audio externos / tracks licenciados (se mantiene procedural como Plan 11; un track real podría sustituirse después sin cambiar los triggers).
- Cambios a reglas de `FaseArco`, umbral de reputación, o economía en Rust.
- Generalizar “Camino al boss” para Postafeta u otras empresas (Postafeta hoy no tiene mini-boss).
- Verificación visual/audio automatizada (mismo patrón Fase 0: checklist manual).

## Arquitectura

```
resolver_ticket → score
       │
       ├─ intentos_restantes? → status only
       │
       └─ cierre final → mostrarScoring(score)
                              │
                              ▼
                     (cerrar scoring) → pintar hub
                              │
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
         toast + pops    panel arco      si ascendio / MiniBoss
                                           → banner + skin + música boss
```

- **Contrato `ultimoFeedback`:** `{ titulo, pass, deltaDinero, deltaRep, ascendio } | null` — se consume al mostrar toast/pops y se limpia tras disparar (o al expirar el toast).
- **Triggers de modo boss:** `score.ascendio === true` al cerrar scoring, o primer paint del hub con `fase === "MiniBoss"` si el banner aún no se mostró en esa entrada.
- **Salida de modo boss:** fase deja de ser `MiniBoss` (incluida transición de agencia) → quitar clase CSS, restaurar ambiente, resetear flags de banner/modo.

## Testing

Frontend puro — sin runner de tests UI. Verificación manual guiada:

1. Resolver ticket OK → toast + pops `+$`/`+rep` + barra se mueve.
2. Fallar con reintentos → solo `#status-msg`; sin toast/pops.
3. Fallo final → toast “Incorrecto”; sin pops positivos de economía.
4. Subir a ~500 rep / `ascendio` → banner + skin alerta + música de boss.
5. Recargar/reentrar hub mid-MiniBoss → skin + música on; banner no se spamea si ya se mostró.
6. Completar lote Auditor → sale modo boss / overlay agencia; música vuelve a ambiente.
7. Mute música durante boss → silencio; unmute → loop boss (si sigue MiniBoss).
8. En Postafeta → sin “Camino al Auditor” ni skin/banner de boss espontáneo.

## Auto-revisión del spec

- **Placeholders:** ninguno — umbral 500, fase `MiniBoss`, sources de scoring y archivos concretos están fijados.
- **Consistencia interna:** no contradice Plan 8 (boss por `ascendio`/lote) ni Plan 11 (audio procedural + mutes); el “progreso de empresa” se comunica vía rep→boss, no vía checklist de catálogo.
- **Alcance:** un solo plan frontend; no reabre transición de agencia, validación SQL, ni catálogo de tickets.
- **Ambigüedad cerrada:** reintentos no celebran; toast al volver al hub (no encima del scoring); Postafeta sin barra de Auditor; música boss procedural con skin B (alerta), no solo banner suave.
