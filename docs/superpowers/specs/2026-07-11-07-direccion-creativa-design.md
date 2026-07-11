# Etapa 7: Dirección Creativa

**Estado:** Aprobado
**Fecha:** 2026-07-11

## Estructura narrativa: viñetas episódicas + mini-boss por empresa

Cada empresa es autocontenida (su propio elenco, su propio jefe, sus propios conflictos), sin un antagonista o trama que conecte toda la carrera. Esto reduce drásticamente la carga de escritura de un solo dev (Pilar 5, Etapa 4) frente a mantener continuidad de trama a través de 8+ empresas.

### Arco narrativo de una empresa (repetible, plantillado)

1. **Onboarding** — se conoce al jefe local y 1-2 compañeros, tickets fáciles para aprender el esquema. Incluye el beat de "revelación del esquema" (ver sección dedicada abajo).
2. **Escalada** — los tickets suben en complejidad, se introducen conceptos SQL nuevos propios de esa industria.
3. **Mini-boss** — el desafío/crisis narrativo más difícil y memorable de esa empresa, protagonizado por un personaje o evento antagonista propio del tema de la empresa (ej: en el hospital, el auditor de cumplimiento; en el casino, el investigador de fraude; en la aerolínea, la cascada de cancelaciones por tormenta).
4. **Resolución** — superar el mini-boss dispara la transición de ascenso/cambio de empresa (Etapa 2).

## Continuidad sin trama: "El Mentor" como constante de carrera

El Mentor (Etapa 5) no pertenece a ninguna empresa — acompaña al jugador a lo largo de toda la carrera (un ex-jefe de la primera pasantía, o una herramienta interna tipo "code review bot" con personalidad, que se lleva de trabajo en trabajo). Da continuidad emocional real (la relación con el Mentor evoluciona con la maestría del jugador) sin requerir mantenimiento de una trama/antagonista compartida entre empresas.

## Voz de escritura: humor corporativo satírico calibrado

- **La comedia vive en el envoltorio, las apuestas son reales.** Un ticket puede llegar con humor de oficina (jerga corporativa vacía, jefes pasivo-agresivos, memes de Slack), pero la crisis técnica detrás (sistema caído, auditoría, Black Friday) tiene consecuencias reales dentro de la ficción — nunca se resuelve con un chiste, se resuelve con SQL correcto.
- **Formato de entrega:** los tickets llegan a través de una app de mensajería corporativa ficticia dentro del juego (parodia de Slack/Teams/Jira) — barato de producir (una sola UI reutilizada en todas las empresas, Pilar 5) y refuerza la fantasía de "estoy en una oficina de verdad".
- **Chiste recurrente transversal (sin ser trama):** el departamento de RRHH es literalmente la misma entidad genérica y absurda en todas las empresas (mismos emails automáticos, mismas políticas sin sentido) — guiño de continuidad puramente cómico, no narrativo.

## Arquetipos de personajes reutilizables (plantillas, no personajes únicos por empresa)

Para mantener producción viable, cada empresa recicla un set de arquetipos con skin narrativo distinto:
- **El Jefe local** (varía por empresa, sale de un pool reutilizable: Micromanager, Ausente/Delegador, Ex-técnico-ahora-gerente, Trepador corporativo).
- **El Mini-boss** (antagonista o evento específico del tema de esa empresa — la única pieza de escritura verdaderamente única por empresa).
- **Compañeros de oficina** (alivio cómico / rival amistoso / aliado) — reutilizables con pequeñas variaciones de diálogo.
- **RRHH** (chiste recurrente transversal, ver arriba).

## El momento de revelar el esquema (dentro del beat de "Onboarding")

**Dónde ocurre:** el primer beat narrativo de cada empresa nueva (paso 1 del arco, "Onboarding") incluye explícitamente el momento en que el jugador recibe acceso al esquema — enmarcado diegéticamente como "IT/el Mentor te da acceso al wiki de datos de la empresa", no como una pantalla de tutorial fuera de ficción (protege el Pilar 3).

**Qué se muestra y cómo (protegiendo el Pilar 1 "SQL real"):**
- Se entrega el esquema completo de la empresa desde el día 1 (no se ocultan tablas artificialmente — un empleado real tendría acceso a la documentación completa, aunque no la entienda toda todavía).
- Se presenta como un visor ERD navegable (diagrama entidad-relación con líneas de foreign keys, tipos de columna, y pequeños comentarios de sabor tipo `// campo legado, no usar`) — herramienta persistente accesible en cualquier momento desde una pestaña fija de la interfaz, no una cinemática de un solo uso.
- La dificultad no viene de ocultar tablas, sino del diseño de los tickets: los tickets tempranos solo requieren 1-2 tablas obvias; los avanzados combinan muchas. La complejidad real de la empresa se revela orgánicamente sin "desbloquear" tablas de forma artificial.
- Asistencia contextual sin spoilear la solución: al abrir un ticket, un panel de "Tablas relevantes" resalta (sin resolver el problema) qué partes del esquema probablemente se necesitan — reduce la fricción de "no sé ni por dónde empezar" en jugadores sin experiencia previa (Etapa 3), sin ocultar el esquema completo.

## Pendiente para etapas siguientes

- Especificación técnica completa del visor de esquema/ERD (interacción, filtros, búsqueda) → Etapa 11 (Mecánicas).
- Cómo se generan/documentan los esquemas por empresa → Etapa 16 (Diseño de bases de datos).
