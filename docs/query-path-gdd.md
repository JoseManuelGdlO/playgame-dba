# Query Path — Game Design Document

**Versión:** 1.0 (consolidado a partir de las 22 etapas de diseño)
**Fecha:** 2026-07-11
**Documentos fuente:** `docs/superpowers/specs/2026-07-11-01-*` a `2026-07-11-22-*` (detalle y proceso de cada decisión)

---

## Índice

1. [Visión del juego](#1-visión-del-juego)
2. [Core Gameplay Loop](#2-core-gameplay-loop)
3. [Público objetivo](#3-público-objetivo)
4. [Pilares del juego](#4-pilares-del-juego)
5. [Fantasía del jugador](#5-fantasía-del-jugador)
6. [Propuesta de valor](#6-propuesta-de-valor)
7. [Dirección creativa](#7-dirección-creativa)
8. [Dirección artística](#8-dirección-artística)
9. [Mundo y ambientación](#9-mundo-y-ambientación)
10. [Progresión](#10-progresión)
11. [Mecánicas](#11-mecánicas)
12. [Economía](#12-economía)
13. [Sistema RPG](#13-sistema-rpg)
14. [Sistema de misiones](#14-sistema-de-misiones)
15. [Diseño de empresas](#15-diseño-de-empresas)
16. [Diseño de bases de datos](#16-diseño-de-bases-de-datos)
17. [Sistema de validación SQL](#17-sistema-de-validación-sql)
18. [Arquitectura técnica](#18-arquitectura-técnica)
19. [MVP](#19-mvp)
20. [Roadmap](#20-roadmap)
21. [Backlog](#21-backlog)
22. [Plan de producción](#22-plan-de-producción)

---

## 1. Visión del juego

**Nombre de trabajo:** *Query Path*. **Mercado:** consumidor general en Steam. **Equipo:** originalmente concebido como solo dev + IA; en la Etapa 22 se ajustó a un equipo de 2 personas part-time + IA. **Tono:** satírico / humor corporativo.

**Elevator pitch:** un simulador de carrera profesional donde el jugador empieza como becario de sistemas y asciende hasta Chief Data Officer resolviendo problemas reales de negocio escribiendo SQL de verdad, contra bases de datos de verdad. *Game Dev Tycoon* si en vez de videojuegos hicieras consultas, con el humor de *Office Space* y la satisfacción de *TIS-100*.

**Frase de Steam:** "Empieza como becario. Termina siendo el DBA que salvó a la empresa del Black Friday. Aprende SQL de verdad sin darte cuenta."

**Promesa central:** no es un curso disfrazado; cada consulta importa narrativamente; la progresión es diegética; el humor corporativo es la capa de diversión barata de producir.

**Enfoque elegido:** Career-Sim First (sobre Puzzle First y Tycoon Híbrido) — más fiel al brief y más viable para un equipo pequeño.

**Riesgo #1 (repetición) resuelto con el sistema "Query Loadout" inspirado en Balatro:**

| Balatro | Query Path |
|---|---|
| Jugar una mano de cartas | Escribir y ejecutar una query |
| Chips × Mult | Correctitud + Velocidad + Buenas prácticas |
| Jokers | Perks/Habilidades |
| Slots limitados de joker | Loadout limitado (árbol persistente, slots activos limitados) |
| Nivel de mano sube con uso | Maestría por arquetipo de query |
| Combos entre jokers | Combos entre perks |
| Tienda entre rounds | Invertir en habilidades entre turnos |

Decisión estructural: **árbol persistente + loadout limitado** (nunca se pierde progreso, pero solo N perks activos a la vez) — evita tanto la pérdida de tensión de un árbol sin límites como la ruptura de la fantasía de carrera continua de un sistema de resets tipo roguelike.

---

## 2. Core Gameplay Loop

**Estructura temporal:** bandeja de entrada / cola de tickets con prioridad y plazo (no un ticket lineal, no días con fases rígidas). **Presión de tiempo:** sin cronómetro real mientras se escribe la query — la presión es de "tiempo de juego" (SLA que avanza con las acciones del jugador), protegiendo el aprendizaje genuino de SQL.

**Loop Micro** (segundos-minutos): leer ticket → explorar esquema → escribir/probar libremente (sin reloj) → enviar (dispara scoring multidimensional) → feedback inmediato.

**Loop Meso** (10-30 min, un turno): bandeja de tickets con prioridad/plazo en tiempo de juego; interrupciones aleatorias rompen el ritmo de "solo SQL"; cierre de turno con resumen de XP/dinero/reputación.

**Loop Macro** (horas, un puesto): entre turnos se visita el árbol de habilidades para invertir dinero/reputación; reuniones periódicas con el jefe; al cruzar umbrales se dispara ascenso/cambio de empresa vía transición tipo "entrevista".

---

## 3. Público objetivo

- **Principal — "El Fan de Tycoon/Sim":** compradores de Game Dev Tycoon/Software Inc/Two Point Hospital. No buscan aprender SQL; el juego debe funcionar con cero conocimiento previo.
- **Secundario — "El Career-Changer/Autodidacta":** ya motivado a aprender SQL; motor del word-of-mouth orgánico.
- **Terciario (bonus) — "El Completista de Puzzles":** fans de TIS-100/Human Resource Machine, servidos naturalmente por el scoring multidimensional.
- **Anti-persona/validador público — "El DBA/Ingeniero Senior Real":** no es comprador, pero su opinión pública sobre si el SQL "se siente real" afecta la credibilidad frente al comprador secundario.

**Implicación clave:** el marketing y la primera hora deben venderse 100% como sim de carrera con humor, nunca como app educativa (analogía Kerbal Space Program).

---

## 4. Pilares del juego

1. **SQL real, consecuencias reales** — toda query se ejecuta contra un motor real; nunca se simula.
2. **La build es tuya (no la query)** — la repetición se compensa con profundidad de build-crafting, no con variedad forzada.
3. **Carrera, no examen** — todo se presenta vía fantasía diegética; nada debe sonar a lección/curso.
4. **Presión de gestión, no de mecanografía** — el estrés viene de priorizar, nunca de escribir bajo cronómetro real.
5. **Sistemas generativos, no contenido artesanal** *(pilar de producción)* — todo el contenido se diseña como plantillas parametrizables, no piezas escritas a mano una por una.

---

## 5. Fantasía del jugador

> "Soy la persona que escribe la query tan limpia, tan rápida y tan correcta que otros ingenieros se detienen a decir 'espera, muéstrame eso'."

Fantasía principal: **El Artesano/Maestro Técnico** (dominio técnico creciente), sobre las fantasías secundarias de Estatus/Ascenso y Heroísmo/Crisis, que existen como envoltura pero no como motor emocional central.

**Consecuencias de diseño:** retroalimentación nunca binaria; mecánica de **"El Mentor"** (tras enviar, un personaje muestra una solución alternativa más elegante con comentario breve, nunca antes de enviar); el fracaso se siente como "funciona, pero un maestro no lo haría así", no como examen reprobado.

---

## 6. Propuesta de valor

> "El único juego que te hace querer ser bueno en SQL de verdad, porque ser bueno en SQL de verdad es literalmente cómo mejoras en el juego."

Se diferencia de: plataformas de aprendizaje SQL (sin contexto/consecuencia ni build persistente), juegos de puzzles de programación (lenguaje ficticio vs. SQL real transferible), y juegos de tycoon/gestión (su "trabajo" es abstracto; aquí es una habilidad real ejecutada de verdad).

**Riesgo a vigilar:** prometer "SQL real" es también la mayor exposición — si el contenido técnico se siente falso, se pierde la diferenciación central.

---

## 7. Dirección creativa

**Estructura narrativa:** viñetas episódicas por empresa (sin trama/antagonista compartido entre empresas — mantiene el alcance viable), cada una con un arco de 4 beats: Onboarding → Escalada → Mini-boss → Resolución.

**Continuidad sin trama:** "El Mentor" no pertenece a ninguna empresa — acompaña toda la carrera, dando continuidad emocional sin requerir mantenimiento de trama compartida.

**Voz:** humor corporativo satírico — la comedia vive en el envoltorio, las apuestas son reales; tickets entregados vía chat corporativo ficticio; chiste recurrente transversal: RRHH es la misma entidad absurda en todas las empresas.

**Arquetipos reutilizables:** El Jefe local (pool de tipos), El Mini-boss (única pieza única por empresa), Compañeros de oficina, RRHH.

**Revelación del esquema:** ocurre en el beat de Onboarding, enmarcada diegéticamente ("acceso al wiki de datos"); esquema completo desde el día 1 (nunca se ocultan tablas artificialmente); dificultad viene del diseño de tickets, no de ocultar el esquema; panel de "Tablas relevantes" como asistencia contextual sin spoilear.

---

## 8. Dirección artística

**Estética central:** "Realismo de Software Corporativo" — el juego se presenta como software real de oficina.

**Consola SQL "Terminal Moderno":** fondo oscuro tipo GNOME Terminal/iTerm2, paleta *Catppuccin Mocha* (`#1e1e2e` / `#313244` / `#cdd6f4` / acentos verde `#a6e3a1`, azul `#89b4fa`, morado `#cba6f7`, durazno `#fab387`). Editor multi-query (varias sentencias separadas por `;`, selector "ejecutar selección/todas"), botón **▶ Play** (ejecuta de verdad, sin puntuar), resultados en pestañas de grid real estilo DBeaver/SQL Workbench, **"✓ Enviar ticket"** como acción separada que dispara el scoring.

**Superficies secundarias:** chat corporativo y visor ERD, mismo lenguaje visual/paleta.

**Personajes (actualizado en Etapa 18):** retratos **pixel art de baja resolución estilo *Papers, Please*** (reemplaza la idea inicial de avatares de iniciales) — paleta apagada/burocrática, animación mínima. Dos capas de arte coexisten: herramientas (terminal moderno) vs. mundo/personajes (pixel art).

**Tipografía:** monoespaciada para código/consola; sans-serif limpia para UI/chat/narrativa.

---

## 9. Mundo y ambientación

**Regla de oro:** absurdismo heightened (tipo *Severance*) en el **envoltorio** (políticas, jerarquías, rituales, personajes) — nunca en los **datos/lógica de negocio**, que deben ser siempre técnicamente honestos (protege el Pilar 1 y la Propuesta de Valor).

**Geografía:** universo satírico globalizado, sin ubicación real específica.

**La Agencia ("Grupo Ómega RH"):** el jugador nunca es contratado directamente — siempre lo reasigna esta agencia de staffing omnipresente y ligeramente siniestra entre industrias completamente distintas. Explica diegéticamente los saltos entre empresas, es la fuente del chiste recurrente de RRHH, y da textura absurdista sin requerir una trama que mantener.

---

## 10. Progresión

**Principio rector #1:** el SQL nunca se restringe artificialmente — solo los tickets y los perks están gateados por rango.

**Principio rector #2 (no negociable):** el juego es SIEMPRE sobre escribir queries, en todos los rangos, incluyendo Chief Data Officer — nunca se reemplaza el gameplay central por otro tipo de mecánica.

**Escalera de rangos → conceptos SQL:**

| Rango | Conceptos dominantes |
|---|---|
| Becario | `SELECT`, `WHERE`, `ORDER BY`, `LIMIT` |
| Auxiliar de Sistemas | `JOIN` inner, agregación, `GROUP BY` |
| Analista de Datos | `LEFT/RIGHT JOIN`, `HAVING`, subconsultas simples |
| DBA Junior | subconsultas correlacionadas, CTEs simples, window functions básicas |
| DBA | índices, execution plans, CTEs recursivos |
| Senior DBA | window functions avanzadas, transacciones/deadlocks |
| Arquitecto de Datos | diseño de esquema, normalización, particionado |
| Lead DBA | procedimientos, triggers, seguridad/permisos, backups |
| Data Engineer | ETL, pipelines |
| Chief Data Officer | capstone: sigue siendo SQL, mayor alcance narrativo |

**Rango ↔ empresa:** cada rango ofrece un pool de 2-3 empresas elegibles vía la Agencia — el jugador elige, dando rejugabilidad barata sin contenido adicional.

---

## 11. Mecánicas

- **A. Bandeja de entrada:** tickets con prioridad/plazo en tiempo de turno; costo de tiempo por metadata del ticket, no por velocidad real de tecleo; tickets no atendidos escalan (afectan reputación).
- **B. Consola SQL:** ver Etapa 8.
- **C. Visor de esquema/ERD:** siempre accesible, buscador, panel de "Tablas relevantes".
- **D. Loadout de habilidades:** resolver un ticket paga dinero; el dinero desbloquea perks permanentemente (único sink); equipar/desequipar perks ya desbloqueados es gratis, solo limitado por slots, y se hace entre turnos.
- **E. El Mentor:** aparece tras enviar, no en cada ticket — solo cuando hay brecha significativa u ocasión clave.
- **F. Interrupciones:** Incidentes técnicos, Política de oficina, Eventos del Mentor.
- **G. Transición de empresa:** pantalla de "reasignación" de la Agencia con 2-3 opciones.
- **H. Pantalla de resultado:** desglose visual estilo Balatro (puntaje base × multiplicadores), incluye comentario del Mentor cuando aplica.

---

## 12. Economía

**Tres recursos con roles distintos:**

| Recurso | Rol | Se gasta en |
|---|---|---|
| Dinero | Recompensa transaccional | Desbloquear perks (único sink) |
| Reputación | Gate de confianza | No se gasta — gate de ascensos Y de compra de perks |
| XP por arquetipo | Maestría técnica | Desbloquea tiers de perks de ese arquetipo |

**Fórmula:** `puntaje_final = (correctitud·peso + velocidad_plan·peso + buenas_prácticas·peso) × multiplicador_perks`; de ahí se derivan dinero, reputación y XP ganados por ticket.

**Reputación:** local por empresa (reinicia a una base al cambiar de empresa, escalada levemente por el rango del jugador); solo baja por tickets escalados/ignorados, nunca se gasta.

**Perks:** costo doble — dinero (se gasta) + reputación mínima (se verifica, no se gasta) — evita "farmear" poder sin desempeño real.

**Único sink de dinero:** perks. Sin cosméticos ni economías paralelas (Pilar 5). Penalización: solo reputación, nunca dinero.

---

## 13. Sistema RPG

**Formato:** colección filtrable tipo tienda de Balatro, no árbol ramificado literal.

**Slots de loadout:** empieza en 2, +1 en 5 hitos de rango, máximo 7.

**Principio de nomenclatura (corrección crítica):** los perks deben nombrarse y describirse en **lenguaje de jugador**, nunca en jerga técnica de SQL (viola el Pilar 3 si se rompe). La maestría por arquetipo sigue existiendo como mecanismo interno de desbloqueo, invisible en el nombre del perk.

**Catálogo por categorías (lenguaje de jugador):**
- **Detective** (ayudan a encontrar cosas): "Instinto", "Rayos X", "Olfato de Reportero".
- **Manos Rápidas** (facilitan escribir la query): "Piloto Automático", "Plantilla en el Bolsillo", "Red de Seguridad".
- **Billetera y Fama** (aceleran dinero/reputación): "Buena Fama", "Bono Bajo la Mesa", "Currículum Brillante".
- **Ritmo** (ganan tiempo de turno): "Café Cargado", "Modo Turbo".

**Maestría interna:** 6 arquetipos SQL (JOIN, Agregación, Subconsultas, CTE, Window Functions, Optimización), 5 tiers cada uno; cada tier requiere uso acumulado + dinero + reputación mínima simultáneamente.

**Combos:** emergen solos al tener ciertos pares de perks activos, sin desbloqueo aparte (ej. "Instinto" + "Piloto Automático" → bono "Racha Perfecta").

---

## 14. Sistema de misiones

**Alcance de escritura:** solo lectura (`SELECT`) hasta DBA; escritura (`UPDATE/INSERT/DELETE`/DDL/procedimientos) desde DBA/Arquitecto en adelante — narrativamente coherente (un becario no tiene permisos de producción) y técnicamente más simple (evita aislamiento de estado desde el día 1).

**Anatomía de un ticket:** solicitante, **Mensaje = Motivo + Solicitud** (siempre en voz satírica, nunca en tono neutro de examen), prioridad, plazo, costo de tiempo, rango mínimo, arquetipo(s) SQL esperado(s), rúbrica de scoring propia.

> Ejemplo: *Motivo:* "Marketing decidió que 'menos es más' tras un retiro corporativo en Cancún." *Solicitud:* "Encuentra las películas con menos reproducciones en [mes]."

**Tipología:** Reporte/Análisis, Investigación/Depuración, Corrección de datos, Cambios de estructura, Automatización, Crisis/Incidente, Mini-boss.

**Generación por plantillas:** una plantilla paramétrica de solicitud + un pool pequeño (3-5) de motivos intercambiables por plantilla, cruzados con el esquema de cada empresa — genera cientos de variantes sin escribir cada ticket a mano.

---

## 15. Diseño de empresas

8 empresas, parodias reconocibles (guiño directo a marcas reales, sabor mexicano), conectadas por **Grupo Ómega RH**:

| Empresa | Guiño a | Franja de rango | Mini-boss |
|---|---|---|---|
| Hospital Arcángel | Hospital Ángeles | Becario → Analista | Auditor de Cumplimiento |
| Postafeta | Estafeta | Becario → Auxiliar | Auditoría de pérdidas |
| AeroMex | Aeroméxico | Auxiliar → Analista | Cascada de cancelaciones |
| Banamix | Banamex | Analista → DBA Junior | Auditor regulador (AML) |
| Vox | Vix | DBA Junior → DBA | Crisis de licencias/tráfico viral |
| Amazonia | Amazon | DBA → Senior DBA | El Black Friday |
| Casino Candente | Casino Caliente | Senior DBA → Arquitecto | Investigador de fraude |
| Gobierno del Estado de Miramar | *(genérico, sin equivalente real — riesgo legal)* | Lead DBA → CDO | Colapso de sistema legado |

**Excepción Gobierno:** se mantiene genérico deliberadamente — satirizar un gobierno real específico implica riesgo legal/reputacional distinto al de una marca comercial.

**Escalado de complejidad estructural** (no solo conceptual) por franja: de ~5-8 tablas/1-2 joins (Hospital/Postafeta) a ~25+ tablas/joins profundos con inconsistencias deliberadas tipo legado (Gobierno de Miramar).

---

## 16. Diseño de bases de datos

**Decisión:** datos fijos, generados una vez en desarrollo (no aleatorios por partida) — prioriza confiabilidad de que cada ticket tenga respuesta limpia y verificable sobre la rejugabilidad de datos distintos.

**Metodología:** definir entidades reales → diseñar FKs según la guía de tamaño de la Etapa 15 → poblar con datos sintéticos (tipo Faker) → validar a mano que las plantillas de tickets produzcan respuestas limpias → congelar el dataset.

**Ejemplo (Hospital Arcángel, ~6 tablas):** `pacientes`, `departamentos`, `empleados`, `tratamientos`, `seguros`, `habitaciones`.

**Realismo escalado:** empresas tempranas con datos limpios; empresas tardías con NULLs deliberados, columnas duplicadas/deprecadas, nombres inconsistentes — simula deuda técnica real.

**Requisito técnico para la Etapa 18:** el motor debe soportar CTEs (incl. recursivos), window functions, `EXPLAIN` real, y procedimientos/triggers.

---

## 17. Sistema de validación SQL

- **A. Correctitud:** comparación de **resultados** (conjunto de filas, tolerante a alias/orden salvo que se pida explícitamente), nunca de texto de la query — permite múltiples soluciones válidas de forma natural.
- **B. Velocidad:** costo del **plan de ejecución** (`EXPLAIN`), no reloj real — determinista y reproducible en cualquier hardware.
- **C. Buenas prácticas:** linter estático de reglas sobre el AST de la query (evita `SELECT *` injustificado, subconsultas innecesarias, cartesianos, etc.) — genera puntaje, no pase/falla binario.
- **D. El Mentor sin IA en tiempo real:** cada ticket trae precargada (escrita en producción) una solución alternativa + un banco pequeño de comentarios atados a reglas específicas del linter/plan — se siente reactivo sin depender de IA en vivo (costo, confiabilidad, offline-first).

---

## 18. Arquitectura técnica

**Stack elegido:** **Tauri/Electron** (frontend HTML/CSS/JS) + **PostgreSQL embebido** localmente (sin necesidad de internet) — cumple todos los requisitos técnicos (EXPLAIN real, triggers, procedimientos, CTEs) que SQLite no puede cumplir. Publicable en Steam vía Steamworks SDK (patrón probado en juegos indie).

**Componentes:** frontend web-tech (consola, chat, ERD, loadout, capa pixel art), motor Postgres embebido, parser SQL a AST para el linter, capa de aislamiento por snapshot para tickets de escritura, persistencia local + Steam Cloud opcional.

**Sin backend propio:** juego 100% single-player offline-first — evita costo/mantenimiento de infraestructura de servidor para un equipo pequeño.

---

## 19. MVP

**Hipótesis a validar:** ¿es genuinamente divertido escribir SQL envuelto en progresión de carrera + build-crafting de perks, incluso con poco contenido?

| Sistema | Alcance MVP |
|---|---|
| Empresas | 2: Hospital Arcángel, Postafeta |
| Rangos | 2: Becario → Auxiliar de Sistemas |
| Tickets | Solo Reporte/Análisis + Investigación básica |
| Perks | 6-8 perks, 2 slots de loadout |
| Economía | Dinero + reputación completos |
| Mentor | Acotado (3-4 reglas de linter) |
| Mini-boss | Uno (Auditor de Cumplimiento) |
| Eventos/interrupciones | Fuera de alcance |
| Arte | Placeholder/mínimo |
| Stack | Completo (Tauri + Postgres) |
| Steam | Fuera de alcance |

---

## 20. Roadmap

**Decisión estratégica:** Early Access tras la Fase 1 (validación temprana de mercado, común en sims/tycoon indie).

- **Fase 0 — MVP interno** (ver Etapa 19).
- **Fase 1 — Contenido + pulido para EA:** 4 empresas (+ AeroMex, Banamix), ~5 rangos (hasta DBA), catálogo de perks completo, eventos, Steamworks, arte pixel. → **Lanzamiento Early Access.**
- **Fase 2 — Iteración con datos reales:** +2 empresas (Vox, Amazonia), +2 rangos (Senior DBA, Arquitecto), balance ajustado.
- **Fase 3 — Contenido de rango alto:** aislamiento por snapshot, tickets de escritura, últimos rangos/empresas (Casino Candente, Gobierno de Miramar) — roster completo (8 empresas, 10 rangos).
- **Fase 4 — Lanzamiento 1.0:** feature-complete, salida de Early Access, plan de soporte post-lanzamiento.

---

## 21. Backlog

**P0 (bloqueante Fase 1/EA):** catálogo de perks/combos para rangos tempranos, plantillas+motivos para 4 empresas, catálogo de eventos, logros de Steam, datasets validados.

**P1 (Fase 2-3):** contenido de Vox/Amazonia/Casino Candente/Gobierno de Miramar, tickets de escritura, datasets "sucios" de rango alto, balance con datos reales.

**P2 (post-1.0):** leaderboard de "query más elegante", localización a inglés, New Game+/modo consultor freelance, cosméticos de escritorio, contenido vivo estacional.

---

## 22. Plan de producción

**Equipo:** 2 personas, ~20-30h/semana combinadas, sin roles fijos, apoyadas en IA (Pilar 5 en la práctica: contenido/arte/código asistidos por IA, siempre curados/validados a mano).

**Primer hito:** spike técnico (Tauri + Postgres embebido funcionando end-to-end) antes de cualquier contenido.

**Estimación:** spike 2-4 semanas, MVP +2-3 meses, Fase 1 +4-6 meses → **~7-10 meses hasta Early Access**.

**Riesgos de producción:** técnico (patrón Postgres-en-Tauri poco probado), scope creep (alcance rico vs. disciplina de MVP), coordinación (sin roles fijos), playtesting limitado (reclutar 5-10 externos antes de EA), desgaste (hitos pequeños y celebrables).

---

*Fin del documento. Para el detalle completo del proceso de diseño y las alternativas descartadas en cada etapa, ver los documentos individuales en `docs/superpowers/specs/`.*
