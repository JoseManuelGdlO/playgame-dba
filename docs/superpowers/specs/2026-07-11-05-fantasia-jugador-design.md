# Etapa 5: Fantasía del Jugador

**Estado:** Aprobado
**Fecha:** 2026-07-11

## Declaración de fantasía

> "Soy la persona que escribe la query tan limpia, tan rápida y tan correcta que otros ingenieros se detienen a decir 'espera, muéstrame eso'."

La fantasía principal es de **dominio técnico creciente (El Artesano/Maestro Técnico)** — no la fantasía heroica ("sobreviví a la crisis") ni la de estatus ("llegué a la cima"), aunque ambas están presentes como envoltura narrativa (Career-Sim First, Etapa 1). Cada solución un poco más elegante que la anterior, con menos esfuerzo, es la fuente real de satisfacción.

## Consecuencias directas para el diseño

- **La retroalimentación no puede ser binaria** (correcto/incorrecto). El scoring multi-dimensional (correctitud + velocidad + buenas prácticas, Etapa 1) se eleva a requisito central: el jugador necesita *ver* por qué su query es buena o mediocre — visualización de plan de ejecución, comparación de alternativas, no solo un número.
- **Mecánica: "El Mentor".** Un personaje narrativo (DBA senior de la empresa, o una IA interna tipo "code review bot" con personalidad) que, tras cada submit, puede mostrar una solución alternativa más elegante con un comentario breve ("yo hubiera usado una window function aquí, te ahorrabas el subquery"). Refuerza la fantasía de maestría sin romper la inmersión de oficina, y es una vía de enseñanza pasiva sin sonar a curso (protege el Pilar 3 de la Etapa 4).
- **El fracaso no es examen reprobado**, sino "funciona, pero un maestro no lo haría así" — gradientes de calidad en vez de pass/fail.
- **Momentos de reconocimiento técnico** (no solo ascensos): un compañero cita tu query en una reunión, tu solución se vuelve "el estándar del equipo", el Mentor deja de tener comentarios porque ya no puede mejorar lo que escribiste. Estos son los verdaderos "high fives" del juego, más que la ceremonia de ascenso en sí.

## Relación con las fantasías secundarias

- **Estatus/ascenso** sigue siendo el vehículo de progresión de carrera (Etapas 1-2), pero se enmarca como *consecuencia* de la maestría técnica, no como el objetivo en sí. Se asciende porque el trabajo es objetivamente bueno, no por política de oficina (la política puede aparecer como fricción satírica, pero no como vía de progreso).
- **Heroísmo/crisis** (Black Friday, incidentes de producción) sigue siendo el clímax narrativo de la Etapa 2, pero se gana porque la solución técnica fue la correcta — reforzando de nuevo la maestría como fuente real de la victoria.

## Pendiente para etapas siguientes

- Diseño detallado de "El Mentor" (frecuencia de aparición, cómo se muestra la comparación de queries, tono del personaje) → Etapa 11 (Mecánicas) y Etapa 17 (Sistema de validación SQL).
