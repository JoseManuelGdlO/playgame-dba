# Etapa 2: Core Gameplay Loop

**Estado:** Aprobado
**Fecha:** 2026-07-11

## Decisiones de estructura

- **Organización temporal:** bandeja de entrada / cola de tickets con prioridad, urgencia y plazo — no un ticket lineal a la vez, no días con fases rígidas.
- **Presión de tiempo:** sin cronómetro real mientras se escribe la query. La presión de tiempo existe solo a nivel de "tiempo de juego" (SLA/plazos que avanzan según acciones del jugador, no segundos reales). Protege el aprendizaje genuino de SQL frente al estrés de escribir bajo un reloj real.

## Loop Micro (segundos-minutos): resolver un ticket

1. **Leer el ticket** — solicitud en lenguaje natural de un departamento ("Necesitamos los 10 clientes que más compraron este mes"), con contexto narrativo (quién lo pide, por qué, con qué tono).
2. **Explorar el esquema** — el jugador puede inspeccionar tablas/columnas disponibles (como abrir un cliente de DB real).
3. **Escribir y probar libremente** — consola SQL real, sin reloj corriendo. El jugador puede ejecutar la query cuantas veces quiera para iterar antes de decidir que está lista.
4. **Enviar (submit)** — se califica en varias dimensiones a la vez: correctitud, "velocidad" (medida en costo de ejecución/plan, no en segundos reales del jugador), buenas prácticas, uso de índices. Esto alimenta la fórmula tipo Balatro (puntaje base × multiplicadores de perks activos) definida en la Etapa 1.
5. **Feedback inmediato** — resultado narrativo + recompensas (XP por arquetipo de query usado, dinero, reputación).

## Loop Meso (10-30 min): un "turno" de trabajo

- **Bandeja de entrada**: varios tickets llegan con prioridad/urgencia/plazo (medido en tiempo de juego). El jugador elige el orden; ignorar o llegar tarde a un ticket urgente tiene consecuencias (reputación, jefe molesto, escalamiento).
- **Interrupciones**: eventos aleatorios se insertan en la cola (incidente de producción, un compañero pide ayuda, política de oficina) — rompen el ritmo de "solo SQL" (riesgo #2 de la Etapa 1).
- **Cierre de turno**: resumen de tickets resueltos/perdidos/escalados, XP por arquetipo, dinero ganado, cambio de reputación/relación con el jefe.

## Loop Macro (horas): un puesto en una empresa

- Entre turnos, el jugador visita el **árbol de habilidades / L&D** para invertir dinero/reputación en nuevos perks o re-equipar su loadout — momento "tienda" equivalente al de Balatro entre rounds.
- Reuniones periódicas con el jefe/manager (narrativas, a veces con decisiones que afectan stats).
- Al cumplir umbrales de reputación/XP/tickets resueltos → ascenso o cambio de empresa, con transición tipo "entrevista/onboarding" que resetea el contexto narrativo y el esquema de datos, pero conserva el árbol de habilidades y el progreso de carrera.

## Por qué esta estructura de 3 niveles funciona

- El loop micro sin reloj real protege el aprendizaje genuino de SQL.
- El loop meso con cola de tickets y tiempo de juego crea tensión de gestión (priorización) sin castigar la mecanografía bajo presión.
- El loop macro con loadout entre turnos conecta directamente con el sistema de Perks/Balatro de la Etapa 1: cada turno nuevo es oportunidad de repensar la build, dando ritmo de "run a run" dentro de una carrera persistente.

## Pendiente para etapas siguientes

- Duración exacta de un turno, cantidad de tickets por turno, catálogo de tipos de interrupciones/eventos → Etapa 11 (Mecánicas) y Etapa 14 (Sistema de misiones).
- Fórmula exacta de puntaje (correctitud/velocidad/buenas prácticas × multiplicadores de perk) → Etapa 13 (Sistema RPG) y Etapa 17 (Sistema de validación SQL).
