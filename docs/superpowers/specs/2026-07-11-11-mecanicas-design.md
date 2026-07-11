# Etapa 11: Mecánicas

**Estado:** Aprobado
**Fecha:** 2026-07-11

## A. Bandeja de entrada y tiempo de turno

- Cada turno el jugador recibe un lote de tickets con prioridad (baja/media/urgente) y un plazo medido en unidades de tiempo de turno (no segundos reales — Etapa 2).
- Cada ticket tiene un costo de tiempo estimado (definido por metadata del ticket, no por cuánto tarda el jugador en escribir — protege el Pilar "presión de gestión, no de mecanografía", Etapa 4): un ticket simple cuesta poco tiempo de turno, uno complejo cuesta más, independientemente de la velocidad real de tecleo.
- El turno termina cuando se agota el presupuesto de tiempo o el jugador decide cerrar el día. Tickets no atendidos a tiempo se escalan (afectan reputación, pueden generar un evento de consecuencia) en vez de simplemente desaparecer.

## B. Consola SQL (definida vía mockups, Etapa 8 — resumen operativo)

Editor multi-query con selección de "ejecutar selección/todas" vía ▶, resultados en pestañas de grid real (estilo DBeaver/SQL Workbench), y "✓ Enviar ticket" como acción separada que dispara el scoring.

## C. Visor de esquema/ERD (Etapa 7 — reglas operativas)

- Accesible en cualquier momento desde una pestaña fija, nunca bloqueado.
- Buscador de tablas/columnas por nombre.
- Panel de "Tablas relevantes" que se auto-resalta al abrir un ticket (heurística basada en metadata del ticket, no en la solución) — guía sin spoilear.

## D. Loadout de habilidades (equipar/cambiar perks) — con circuito económico

- **Resolver un ticket paga dinero** (la empresa paga por el trabajo entregado, escalado según la calidad del scoring — Etapa 1/5: una solución mediocre paga menos que una elegante).
- Ese dinero se gasta en el árbol de habilidades para **desbloquear permanentemente** nuevos perks — único lugar donde el dinero se consume.
- **Equipar/desequipar** perks ya desbloqueados en el loadout activo (dentro del límite de slots) es **gratis** y se hace en los puntos de transición entre turnos — la única restricción es de slots, no de dinero.
- Separación clara de decisiones: "¿en qué invierto mi dinero a largo plazo?" (desbloquear) vs. "¿qué llevo activo hoy?" (equipar) — ambas con peso estratégico propio.
- Montos exactos (cuánto paga cada ticket, cuánto cuesta cada perk) se calibran en la Etapa 12 (Economía).

## E. El Mentor (mecánica operativa, Etapa 5)

- Aparece después de enviar un ticket, no antes (nunca da la respuesta, solo comenta sobre lo ya entregado).
- Frecuencia: no en cada ticket (se volvería ruido) — aparece cuando hay una brecha significativa entre la solución del jugador y una alternativa más óptima, o en momentos clave (primer uso de un concepto nuevo, mini-boss).
- Presentado como un mensaje corto en el chat corporativo (mismo componente visual que los tickets, Etapa 8), no una pantalla dedicada.

## F. Interrupciones y eventos aleatorios (resuelve el riesgo #2 de la Etapa 1)

Tres categorías reutilizables (plantillas, Pilar 5):
1. **Incidentes técnicos** — un ticket urgente inyectado a mitad de turno (ej. caída de reporte, query en producción que se cuelga).
2. **Política de oficina** — evento de solo-lectura/decisión narrativa breve, sin SQL, que afecta reputación/relaciones (ej. un compañero pide "arreglar" un dato de forma poco ética — decidir tiene consecuencias).
3. **Eventos del Mentor** — check-ins narrativos que no cuestan tiempo de turno, refuerzan la relación de carrera larga.

## G. Transición de empresa (la Agencia, Etapa 9)

- Al completar el arco de una empresa (mini-boss + umbral, Etapa 10), se presenta una pantalla de "reasignación" de la Agencia con 2-3 opciones de empresa siguiente (con una línea de sabor absurdista por opción) — el jugador elige.
- El árbol de habilidades y el progreso de rango se mantienen; el esquema y elenco cambian por completo.

## H. Pantalla de resultado/scoring

Tras "Enviar ticket", se muestra un desglose visual estilo Balatro (Etapa 1): puntaje base (correctitud/velocidad/buenas prácticas) × multiplicadores de perks activos, con animación de números que se acumulan — el momento de mayor "dopamina" del loop micro. Incluye el comentario del Mentor cuando aplica (sección E).

## Pendiente para etapas siguientes

- Montos exactos de dinero/costos de perks → Etapa 12 (Economía).
- Catálogo completo de perks y slots de loadout → Etapa 13 (Sistema RPG).
