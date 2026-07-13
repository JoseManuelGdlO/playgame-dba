# Fase 0 / Plan 4: Economía — Design

**Estado:** Aprobado (brainstorming)
**Fecha:** 2026-07-12
**Etapa de referencia:** Etapa 12 (Economía), con un recorte de alcance de Etapa 10 (Progresión)

## Contexto

Hoy `submit_ticket` paga un monto fijo (+500 si la respuesta es correcta, 0 si no) hacia un `PerkState { dinero, unlocked }` que solo alimenta el stub de un único perk falso del spike original. No existe reputación, XP por arquetipo, ni ninguna fórmula real. Este plan reemplaza eso por la economía de 3 recursos de la Etapa 12, combinando los 3 puntajes crudos que ya produce el motor de validación (Plan 2) con los pesos de rúbrica que ya trae cada ticket (Plan 3).

## Alcance

- Fórmula completa de recompensa (Etapa 12): `puntaje_base` (pesos por ticket) → `puntaje_final` (× multiplicador de perks) → `dinero_ganado`/`reputacion_ganada`/`xp_ganado`.
- `multiplicador_perks_activos` fijo en `1.0` — no existe todavía el sistema RPG/perks real (Etapa 13); el plan que lo construya conecta el multiplicador real sin tocar esta fórmula.
- Reputación y XP por arquetipo trackeados de verdad, acumulados en el estado del jugador.
- Umbral de ascenso Becario→Auxiliar de Sistemas definido y expuesto como una señal (`puede_ascender: bool`) — **sin** disparar el ascenso real, que depende de superar el mini-boss (Etapa 10), todavía no construido.
- El stub de un solo perk (`unlock_perk`, costo fijo 300) se mantiene funcionalmente idéntico, solo migrado al nuevo estado de jugador — el sistema RPG real (Etapa 13) lo reemplaza en un plan posterior.
- Fuera de alcance: el sistema de perks real y sus multiplicadores reales (Etapa 13), el evento de ascenso de rango / mini-boss / transición de empresa (Etapa 10/11-G), penalizaciones por tickets escalados/ignorados (necesitan el sistema de turnos, Etapa 11-A, no construido).

## Valores de partida (ajustables en playtesting, per Etapa 12)

**`valor_base`/`factor_reputacion` por plantilla** (Plan 3), reflejando que "valor_base sube con prioridad/complejidad del ticket":

| Plantilla | `valor_base` | `factor_reputacion` |
|---|---|---|
| `plantilla_reporte_simple` | 100 | 0.5 |
| `plantilla_reporte_agregado` | 150 | 0.7 |
| `plantilla_reporte_join` | 150 | 0.7 |
| `plantilla_reporte_join_agregado` | 200 | 1.0 |
| `plantilla_depuracion` | 250 | 1.2 |

**XP base por arquetipo** (escala con la dificultad del concepto SQL, Etapa 10): `Select: 10, Join: 20, Agregacion: 25`.

**Umbral de ascenso** Becario→Auxiliar de Sistemas (Hospital Arcángel): `500.0` de reputación.

## Fórmula (Etapa 12, literal)

```
puntaje_base = correctitud × peso_correctitud + velocidad × peso_velocidad + practicas × peso_practicas
puntaje_final = puntaje_base × multiplicador_perks_activos   (= 1.0 en este plan)

dinero_ganado = puntaje_final × valor_base / 100        (solo si el ticket es correcto; si no, todo en 0)
reputacion_ganada = puntaje_final × factor_reputacion / 100
xp_ganado[arquetipo] = xp_base(arquetipo) × puntaje_final / 100, por cada arquetipo del ticket
```

Los tickets incorrectos no otorgan dinero ni reputación ni XP (no hay penalización de reputación en este plan — eso requiere detectar tickets "escalados/ignorados", que depende del sistema de turnos aún no construido).

## Arquitectura

- Extender `Ticket` (`app/src-tauri/src/tickets/mod.rs`) con `valor_base: i64` y `factor_reputacion: f64`, hardcodeados dentro de cada una de las 5 funciones-plantilla (no como parámetros nuevos) — los 12 tickets concretos de los catálogos de empresa no cambian.
- Nuevo módulo `app/src-tauri/src/economia/mod.rs`: función pura `calcular(evaluacion: &validation::Evaluacion, ticket: &tickets::Ticket, multiplicador_perks: f64) -> Resultado`, tabla de XP base por arquetipo, y `EstadoJugador` (dinero, reputación, XP acumulado por arquetipo, y el bool del stub de perk) con `aplicar_resultado()` y `puede_ascender()`.
- `lib.rs`: `Perk`/`PerkState` se renombran/reforman a `EstadoJugador`; `submit_ticket` usa `economia::calcular` en vez de la recompensa fija; `ScoreResult` expone el desglose completo.

## Testing

`economia::calcular` y `EstadoJugador` son 100% puros (sin base de datos) — tests unitarios directos, sin necesidad de Postgres embebido. Casos a cubrir: ticket correcto (dinero/reputación/XP > 0, proporcional a los pesos del ticket), ticket incorrecto (todo en 0), reparto de XP entre múltiples arquetipos de un mismo ticket, `puede_ascender` cruzando el umbral. Regresión completa (suite existente) + smoke test de la app real al final, como en los planes anteriores.

## Auto-revisión del spec

- **Placeholders:** ninguno — `multiplicador_perks_activos = 1.0` y el alcance recortado de "rango" son decisiones explícitas, no placeholders olvidados.
- **Consistencia interna:** la fórmula, los valores por plantilla y el estado del jugador usan los mismos nombres (`puntaje_base`, `puntaje_final`, `dinero_ganado`, `reputacion_ganada`, `xp_ganado`) de principio a fin.
- **Alcance:** un solo subsistema (economía), con un recorte explícito y justificado de "progresión de rango" hacia el plan del mini-boss — cabe en un plan de implementación.
- **Ambigüedad:** ninguna — la fórmula es literal de la Etapa 12, y todos los valores numéricos de partida quedan fijados arriba.
