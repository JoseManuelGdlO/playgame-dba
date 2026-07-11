# Etapa 1: Visión del Juego

**Estado:** Aprobado
**Fecha:** 2026-07-11
**Nombre de trabajo del proyecto:** *Query Path* (por definir en Dirección Creativa)

## Decisiones de contexto (condicionan todo el diseño posterior)

- **Mercado objetivo:** consumidor general en Steam (no B2B educativo, no bundle institucional). Debe competir por diversión primero, aprendizaje como consecuencia.
- **Equipo:** solo developer apoyado en herramientas/IA, presupuesto mínimo. El diseño debe favorecer sistemas generativos/templados sobre contenido artesanal, y arte minimalista/reutilizable sobre producción costosa.
- **Tono:** satírico / humor corporativo (tipo *Office Space*, *Game Dev Tycoon*). El desafío SQL en sí es serio y real; el envoltorio narrativo es cómico.

## Elevator Pitch

*Query Path* es un simulador de carrera profesional donde el jugador empieza como becario de sistemas y asciende hasta Chief Data Officer resolviendo problemas reales de negocio escribiendo SQL de verdad, contra bases de datos de verdad. Es *Game Dev Tycoon* si en vez de hacer videojuegos, hicieras consultas — con el humor corporativo de *Office Space* y la satisfacción de resolver un puzzle bien hecho de *TIS-100*.

## Frase de una línea (Steam)

> "Empieza como becario. Termina siendo el DBA que salvó a la empresa del Black Friday. Aprende SQL de verdad sin darte cuenta."

## Promesa central al jugador

- **No es un curso disfrazado de juego.** Es un juego de gestión de carrera cuya única "arma" es SQL — como en *Papers, Please* el sello es la herramienta, aquí es la query.
- **Cada consulta importa narrativamente.** No se resuelven ejercicios abstractos: se resuelve *"el hospital necesita saber qué pacientes llevan más de 40 días internados antes de que llegue la auditoría."*
- **La progresión es diegética.** Subir de rango, cambiar de empresa, la relación con el jefe, el salario — todo eso *es* el sistema de aprendizaje, disfrazado de RPG de oficina.
- **El humor corporativo es la capa de "diversión barata".** Reuniones inútiles, jefes tóxicos, Slack corporativo ficticio, memes de oficina: contenido narrativo reutilizable y económico de producir para un equipo de un solo dev.

## Enfoque de visión elegido: Career-Sim First

Se evaluaron tres enfoques posibles:

| Enfoque | Descripción | Veredicto |
|---|---|---|
| **A. Career-Sim First** | Progresión de carrera y fantasía de rol como gancho principal; tickets SQL reutilizables entre empresas; RPG/economía como envoltura ligera y barata de producir. | **Elegido.** Más fiel al brief original y más viable para un solo dev: el contenido (tickets) se genera/templa por sistema, y la "carne" cara de producir (arte, economía profunda) se mantiene ligera. |
| B. Puzzle First | La elegancia de la solución SQL es el corazón; la carrera es temática/decorativa (tipo TIS-100). | Descartado: público más nicho, menos gancho comercial amplio. |
| C. Tycoon Híbrido | Gestión de equipo/departamento además de SQL individual (tipo Software Inc). | Descartado: alto riesgo de scope inviable para un solo desarrollador. |

## Por qué esto puede funcionar comercialmente

| Referencia | Qué toma prestado | Qué lo distingue |
|---|---|---|
| Game Dev Tycoon | Progresión de carrera + humor + producto que "creces" | El "producto" es la propia carrera del jugador, no una empresa externa |
| TIS-100 / Human Resource Machine | Satisfacción de resolver un puzzle de lógica elegante | Usa SQL real, transferible al mundo laboral real — no un lenguaje esotérico inventado |
| Papers, Please | Una única herramienta que se vuelve el verbo de todo el juego | El "sello" aquí es la query SQL, con infinitas variantes de dificultad reales |
| Software Inc / Cities: Skylines | Fantasía de simulación profesional seria | Deliberadamente NO se copia su complejidad de sistemas económicos (riesgo de scope) |

## Riesgo de diseño #1: repetición — resuelto con "Query Loadout" (inspirado en Balatro)

La acción base (escribir una query) se repite todo el juego, igual que en Balatro "jugar una mano" se repite todo el juego. Lo que evita la monotonía no es cambiar la acción base, sino **cómo se modifica**, vía un sistema de build-crafting inspirado directamente en el diseño de jokers de Balatro.

### Paralelismo de diseño

| Balatro | Query Path |
|---|---|
| Jugar una mano de cartas | Escribir y ejecutar una query |
| Chips × Mult (puntaje base) | Correctitud + Velocidad + Buenas prácticas (puntaje base del ticket) |
| Jokers (modificadores pasivos) | **Perks/Habilidades**: modificadores pasivos desbloqueados en un árbol de carrera |
| Slots limitados de joker (obliga a elegir build) | **Loadout limitado**: las habilidades se desbloquean para siempre (progreso persistente), pero solo N están "activas"/equipadas a la vez |
| Nivel de mano (Flush, Full House suben de nivel con uso) | **Maestría por arquetipo de query**: usar JOINs, Window Functions, CTEs, Subqueries repetidamente sube su nivel de maestría por separado, desbloqueando perks propios de ese arquetipo |
| Combos entre jokers (sinergias) | **Combos entre perks**: ej. tener "Ojo de Índice" + "Lector de Execution Plan" activos y resolver una query que usa índice correctamente Y evita table scan dispara un bono de combo |
| Comprar jokers con las monedas del round | **Invertir en habilidades** con el dinero/reputación que paga la empresa entre asignaciones |

### Decisión estructural: árbol persistente + loadout limitado

Se evaluaron tres estructuras posibles:

1. **Árbol persistente + Loadout limitado (elegido).** Las habilidades se desbloquean para siempre (nunca se pierden), pero solo N pueden estar activas/equipadas a la vez. El jugador debe elegir su build según el tipo de empresa/tickets que enfrenta, y puede re-equipar entre turnos. Crea variedad de juego a juego sin sacrificar progreso permanente.
2. Árbol persistente sin límite de slots. Descartado: pierde la tensión de elegir build; eventualmente el jugador tiene todos los bonos y la elección deja de importar.
3. Runs con reset tipo roguelike. Descartado: contradice la fantasía de "carrera profesional continua" ya establecida como pilar.

### Cómo se siente jugarlo

Antes de una tanda de tickets (o al cambiar de departamento/empresa), el jugador revisa qué tipo de trabajo se avecina y **arma su loadout** de habilidades activas — igual que en Balatro se revisa la mano de jokers antes del siguiente blind. ¿Vienen muchos tickets de reportes con agregaciones pesadas? Se equipan perks de GROUP BY/Window Functions. ¿Viene una crisis de rendimiento? Se equipan perks de índices y optimización. Elegir mal el loadout penaliza; encontrar la combinación correcta para el contexto es la parte "de estrategia" que evita que el juego sea solo "escribir SQL una y otra vez".

Este sistema se especificará por completo en la **Etapa 13 (Sistema RPG)**, pero queda fijado ya como **pilar de diseño**: *la repetición de la acción base se compensa con profundidad de build-crafting, no con variedad forzada de la acción en sí.*

## Riesgo de diseño #2 (pendiente de resolver en etapas siguientes)

Más allá del build-crafting de habilidades, sigue pendiente: necesitamos variedad real en el *tipo* de desafío (no solo SELECT con distinto WHERE), momentos no-SQL que rompan el ritmo (incidentes de producción, política de oficina, decisiones de gestión), y una razón emocional para que cada ticket importe (un jefe específico, una consecuencia narrativa, una audiencia dramática tipo Black Friday). Se resuelve en **Etapa 2 (Core Gameplay Loop)** y **Etapa 14 (Sistema de misiones)**.
