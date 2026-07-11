# Etapa 22: Plan de Producción

**Estado:** Aprobado
**Fecha:** 2026-07-11

## Equipo y ritmo

2 personas, ~20-30 horas/semana combinadas (menos de 1 tiempo completo: una persona a medio tiempo ~15-20h/semana, la otra en tardes/noches/fines de semana ~5-10h/semana), sin roles fijos — ambos usan asistencia de IA para acelerar diseño de contenido, arte y código (Pilar 5, Etapa 4). Nota: el equipo creció de "solo dev" (asunción original de la Etapa 1/4) a 2 personas durante esta etapa; el espíritu del Pilar 5 (sistemas antes que contenido artesanal) se mantiene igualmente, dado que el total de horas combinadas sigue siendo limitado.

## Primer hito crítico: spike técnico antes que contenido

Antes de construir ningún contenido de juego, se valida la asunción técnica más riesgosa de todo el proyecto (Etapa 18): que Tauri + PostgreSQL embebido funcionen juntos de forma confiable, sin depender de internet, empaquetados para Windows. Este "walking skeleton" (una sola empresa, un solo ticket, un solo perk, de punta a punta) se construye primero — si esta combinación de stack falla, es mucho más barato descubrirlo antes de invertir meses en contenido sobre un stack que no funciona.

## Estimación de tiempo (rangos conservadores dado el ritmo part-time combinado)

| Hito | Estimado |
|---|---|
| Spike técnico (Tauri + Postgres embebido funcionando) | 2-4 semanas |
| Fase 0 — MVP completo (Etapa 19) | 2-3 meses adicionales |
| Fase 1 — Contenido + pulido para Early Access (Etapa 20) | 4-6 meses adicionales |
| Total estimado hasta Early Access | ~7-10 meses a este ritmo |

Las Fases 2-4 dependen de feedback real de Early Access — no se estiman en detalle ahora (el Roadmap, Etapa 20, ya señala que esas fases se ajustan con datos reales).

## Riesgos de producción (distintos a los riesgos de diseño ya cubiertos en etapas anteriores)

- **Riesgo técnico:** Postgres embebido en Tauri es un patrón menos probado que SQLite → mitigado por el spike técnico como primer paso obligatorio.
- **Riesgo de scope creep:** el diseño completo (8 empresas, 10 rangos, decenas de perks) es rico — el riesgo real es querer construirlo todo antes de validar. Mitigación: respetar estrictamente el alcance de MVP (Etapa 19) antes de tocar cualquier ítem del Backlog (Etapa 21).
- **Riesgo de coordinación:** sin roles fijos entre 2 personas, riesgo de duplicar esfuerzo o dejar huecos. Mitigación: checkpoint corto semanal de sincronización, aunque sea informal.
- **Riesgo de playtesting limitado:** un equipo de 2 no detecta solo todos los problemas de balance/diversión. Mitigación: reclutar un grupo externo pequeño de playtesters (5-10 personas) antes de Early Access, no después.
- **Riesgo de desgaste:** proyecto part-time de largo aliento. Mitigación: cada fase del Roadmap es un hito pequeño y celebrable, evitar plantear todo como un solo esfuerzo monolítico hasta Early Access.

## Uso de IA en el flujo de producción (Pilar 5 en la práctica)

- **Contenido** (tickets, motivos narrativos, datos sintéticos): generado con asistencia de IA, siempre revisado a mano por humor y corrección técnica (Etapa 14/16).
- **Arte pixel** (Etapa 8/18): generado/asistido por IA, curado a mano para mantener consistencia de estilo entre personajes.
- **Código:** asistido por herramientas de IA para acelerar boilerplate de UI y la integración con Postgres/Tauri, siempre validado con pruebas reales antes de darlo por bueno.
