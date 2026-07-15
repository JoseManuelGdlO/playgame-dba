# Fase 0 / Plan 19: Consola y scoring — presencia de presentación — Design

**Estado:** Aprobado (brainstorming)
**Fecha:** 2026-07-15
**Etapa de referencia:** Plan 11 (animación/audio), Plan 18 (HUD feedback ticket/arco/boss), UI loop consola

## Contexto

Tras ~4 queries el loop se siente aburrido. Diagnóstico de playtest: lo peor es **variedad de puzzles** (mismos SELECT), pero el jugador también marca la **consola SQL (B)** y el **overlay de scoring / toast (C)** como pantallas planas que refuerzan ese vacío. Se eligió atacar solo **presentación** (enfoque 2): dramatizar B y C sin tickets nuevos ni rebalanceo de economía/progresión. Este pass no cura la repetición de SQL; evita que B/C empeoren el sentir.

Hoy la consola es un terminal estático (textarea + tabla). El scoring ya revela líneas en cascada con `animarNumero` / `sfxTick`, pero el ritual es siempre el mismo (título binario pass/fail al final, sin skins, rewards mezclados con métricas).

## Decisiones de brainstorming

- Prioridad: **presentación B + C** (no híbrido contenido+UI, no solo tickets nuevos).
- Alcance: **frontend-first** — sin comandos Tauri ni campos Rust nuevos; reutilizar `ScoreResult` y estado de turno/intentos ya expuestos.
- Consola: estados visuales vivos + Play con presencia + retrato reactivo + alerta suave por presión (intentos/presupuesto).
- Scoring: ritual más claro (cascada → título/skin por tier → pops de $ / rep → mentor/ascenso) + toast más expresivo.
- Umbral “excelente”: regla local en JS sobre puntajes existentes (ajustable por constante).

## Alcance

### Consola / editor (B)

- **Estados de ventana** en `#pantalla-consola` / `.ventana-terminal` (o contenedor equivalente) vía clases CSS:
  - `consola-idle` — ticket abierto, sin run reciente.
  - `consola-querying` — al pulsar Play / Ejecutar todas (hasta pintar resultado).
  - `consola-ok` — último preview sin error.
  - `consola-error` — último preview con error o fallo de parseo/ejecución visible en `#result-table`.
  - `consola-alerta` — presión: `intentos_restantes ≤ 1` en el ticket activo, **o** `presupuesto_restante ≤ 20` (presupuesto de turno base = 100). Combinable con ok/error (alerta es acento, no reemplaza ok/error).
- **Título de barra** (`#consola-titulo` o span dedicado): refleja el estado (`query-path`, `· ejecutando…`, `· ok`, `· error`) sin cambiar el layout.
- **Play con presencia:**
  - Flash breve / estado “running…” en el panel Resultado al iniciar ejecución.
  - Filas de resultado con stagger al pintar éxito (solo presentación; mismos datos).
  - Error: shake corto del bloque de resultado + mensaje de error más visible (clase existente `resultado-error` reforzada).
- **Retrato del ticket** (`#ticket-retrato`): micro-reacción al run ok (pulso) / error (shake o ceño vía clase). Sin sprites nuevos obligatorios; CSS sobre el nodo actual.
- **Submit:** clase `consola-querying` o equivalente breve mientras `resolver_ticket` está en vuelo; al volver al hub el estado se limpia.
- **Fuera de B:** syntax highlight completo, redesign de layout a 2 columnas, minijuegos en el editor, tickets/variantes SQL nuevas.

### Feedback post-ticket (C)

- **Ritual de `mostrarScoring` (reescritura del orden, no del contrato de score):**
  1. Abrir overlay; métricas (correctitud, velocidad, prácticas) en cascada con count-up (comportamiento actual, conservar timings cercanos).
  2. Aplicar **skin de tier** al panel/ventana según clasificación (ver Umbrales).
  3. Título + SFX según tier (no solo pass/fail binario).
  4. **Recompensas** ($ y rep) con pop aparte tras las métricas.
  5. `comentario_mentor` y mensaje de ascenso como beat final (después de rewards).
  6. Habilitar Cerrar al terminar la secuencia; **click en overlay / Cerrar durante la secuencia = skip** (saltar a estado final filled + botón enabled).
- **Tres skins / copy:**
  - **Excelente** (`pass` + umbral): acento brillante, título tipo “Query limpia” (o equivalente corto), SFX de éxito (opcional variante si ya hay hook fácil; si no, `sfxExito`).
  - **Pass normal:** look pulido del terminal actual; título tipo “Resuelto”.
  - **Fail** (cierre final incorrecto): skin fría; título seco; énfasis visual en la métrica más débil entre correctitud/prácticas (velocidad secundaria). Reintentos con `intentos_restantes` **no** abren este overlay (se mantiene el flujo actual de status en consola).
- **Toast (Plan 18):** al cerrar scoring y pintar hub, el toast existente gana clases/`copy` por tier (`excelente` / `pass` / `fail`). Pops de badges y barra de arco **sin cambio de contrato**; solo coherencia visual si hace falta una clase.
- **Fuera de C:** cutscenes largas, ranking global, replay de la query, nuevos campos en `ScoreResult`, cambios a validación/economía Rust.

### Umbrales (JS)

Constante única (nombre tentativo `UMBRAL_SCORE_EXCELENTE`):

- **Excelente** si `score.pass` y:
  - `(puntaje_correctitud + puntaje_velocidad + puntaje_practicas) / 3 >= 85`, **o**
  - `puntaje_velocidad >= 95`.
- En caso contrario, con `pass` → tier normal; sin `pass` → fail.
- Valores tuneables en un solo sitio; no hardcodear umbrales en CSS.

### Archivos tocados (previsto)

- `app/src/main.js` — estados de consola en Play/submit/resultados; rewrite de orden en `mostrarScoring`; clasificación de tier; toast tipado; skip de cascada.
- `app/src/styles.css` — clases `consola-*`, skins `scoring-excelente` / `scoring-pass` / `scoring-fail` (nombres finales libres), stagger filas, pops de reward, toast por tier.
- `app/src/index.html` — markup mínimo si hace falta envolver filas del scoring o un nodo “running…” en Resultado (preferir clases sobre nodos nuevos).
- `app/src/audio.js` — solo si se añade un SFX reutilizando el motor actual; no tracks externos.

## Fuera de alcance

- Nuevos tickets, variantes SELECT, shuffle de catálogo, o escalado de dificultad.
- Eventos de oficina / rival / narrativa mid-bandeja (salvo micro-reacción de retrato ya en alcance).
- Cambios a `FaseArco`, umbral de Ascenso, economía, perks, o validación SQL en Rust.
- Redesign del hub (A) más allá del toast tipado de Plan 18.
- Pruebas visuales/audio automatizadas (checklist manual Fase 0).

## Arquitectura

```
Play / Ejecutar
    → consola-querying + “running…”
    → renderResultados
         ├─ ok  → consola-ok + stagger + pulso retrato
         └─ err → consola-error + shake + ceño retrato
    → + consola-alerta si presión (intentos/presupuesto)

resolver_ticket (cierre final)
    → clasificarTier(score)  // JS puro
    → mostrarScoring
         métricas cascada → skin+título → $ / rep pop → mentor / ascenso
         (skip → estado final)
    → cerrar overlay → toast tier + pops hub (Plan 18)
```

- **Contrato Rust:** sin cambios. Se consume el `score` actual (`pass`, puntajes, `dinero_ganado`, `reputacion_ganada`, `comentario_mentor`, `ascendio`, etc.).
- **Estado UI:** flags/clases locales en consola; tier derivado al mostrar scoring / armar `ultimoFeedback` (extender `ultimoFeedback` con `tier` opcional para el toast; no persistir a guardado).

## Criterios de éxito (playtest manual)

1. En consola, Play ok vs error se distinguen a simple vista (borde/título/resultado/retrato) sin leer el status.
2. Con 1 intento restante (o presupuesto crítico), la consola muestra alerta sin cambiar reglas.
3. Un clear “excelente”, uno “pass normal” y un fail se sienten distintos en scoring (skin + título + orden rewards) en menos de ~3s de lectura.
4. Skip durante la cascada deja el panel completo y Cerrar usable.
5. Toast al volver al hub refleja el tier.
6. Mute / flujo de reintentos / Ascenso / MiniBoss existentes no se rompen.

## Testing

Frontend puro — checklist manual:

- [ ] Preview ok / error en Hospital Becario
- [ ] Submit pass excelente, pass normal, fail final
- [ ] Reintento con `intentos_restantes` (sin overlay de celebración)
- [ ] Skip mid-cascada
- [ ] Toast + pops al cerrar scoring
- [ ] Ascenso (`ascendio`) sigue mostrando beat final
- [ ] Consola alerta con intento bajo
- [ ] Volver al hub limpia estados de consola
