# Etapa 14: Sistema de Misiones

**Estado:** Aprobado
**Fecha:** 2026-07-11

## Decisión de alcance: solo lectura al inicio, escritura en rangos altos

Becario→DBA: solo `SELECT` (reportes/análisis). A partir de DBA/Arquitecto se introducen `UPDATE/INSERT/DELETE`/DDL/procedimientos. Más simple técnicamente (los tickets de solo lectura no necesitan aislar el estado de la BD) y con sentido narrativo (un becario no debería tener permisos de escritura en producción).

## Anatomía de un ticket

- **Solicitante** (persona/departamento, con su tono propio — Etapa 7).
- **Mensaje**, dividido siempre en dos piezas obligatorias:
  - **Motivo** — por qué esto importa *ahora*, con consecuencia implícita (se va a quitar contenido, viene una auditoría, hay una reunión con la directiva, un cliente importante se quejó). Escrito siempre en la voz satírica corporativa de la Etapa 7 — nunca en tono neutro de examen.
  - **Solicitud** — la pregunta de negocio en sí, ya conectada al motivo, también en esa voz.
- **Prioridad** (baja/media/urgente) y **plazo** en tiempo de turno (Etapa 11-A).
- **Costo de tiempo estimado** y **rango mínimo** requerido.
- **Arquetipo(s) SQL esperado(s)** (metadata interna, alimenta XP y el panel de "tablas relevantes", Etapa 11-C).
- **Rúbrica de scoring** propia (pesos de correctitud/velocidad/buenas prácticas — algunos tickets valoran más la velocidad del plan, otros las buenas prácticas).

### Ejemplo de referencia (Netflix-like)

> **Motivo:** "Marketing decidió que 'menos es más' después de un retiro corporativo de tres días en Cancún. Alguien tiene que pagar ese retiro quitando películas del catálogo."
> **Solicitud:** "Encuentra las películas con menos reproducciones en [mes] antes de que alguien de Marketing intente ayudar con SQL de nuevo."

## Tipología de tickets (variedad real, resuelve el riesgo #2 de la Etapa 1)

| Tipo | Rango | Descripción |
|---|---|---|
| Reporte/Análisis | Todos (default) | Preguntas de negocio resueltas con `SELECT` — el grueso del contenido |
| Investigación/Depuración | Todos, sube en complejidad | Se entrega una query ya escrita (rota o lenta) para arreglar/optimizar — conecta con la fantasía de maestría (Etapa 5) y el Mentor |
| Corrección de datos (`UPDATE/DELETE/INSERT`) | DBA+ | Requiere snapshot aislado por ticket (implicación técnica → Etapa 18) |
| Cambios de estructura (DDL: `CREATE/ALTER`, índices) | Arquitecto de Datos+ | Mismo requisito de aislamiento |
| Automatización (procedimientos/triggers) | Lead DBA+ | El ticket pide "que esto se resuelva solo la próxima vez" |
| Crisis/Incidente | Todos (escala con rango) | Ligado a las interrupciones (Etapa 11-F), mayor urgencia |
| Mini-boss | Fin de cada empresa | Combina 2+ tipos anteriores en una secuencia narrativa de alto riesgo (Etapa 7) |

## Generación por plantillas (Pilar 5 — contenido sistémico, no artesanal)

Cada tipo se define como una plantilla paramétrica de solicitud, ejemplo:

> "¿Cuáles son los [top N] [entidad] que [más/menos] [métrica] en [periodo/condición]?"

Cruzada con el esquema específico de cada empresa (Etapa 16), esta única plantilla genera cientos de variantes reales sin escribir cada ticket a mano.

Cada plantilla de solicitud se empareja además con un **pool pequeño de motivos intercambiables** (3-5 por plantilla) propios de esa empresa/departamento, siempre en voz satírica. Al generar un ticket se combina `[motivo aleatorio del pool] + [plantilla de solicitud con sus parámetros]` — multiplica la sensación de variedad narrativa sin multiplicar el trabajo de escritura, y protege la regla de "nunca sonar a examen" en cada instancia generada.

## Pendiente para etapas siguientes

- Diseño técnico del aislamiento por snapshot para tickets de escritura → Etapa 18 (Arquitectura técnica).
- Esquemas concretos por empresa que alimentan las plantillas → Etapa 16 (Diseño de bases de datos).
- Catálogo real de plantillas y pools de motivos por empresa → Etapa 15 (Diseño de empresas) y Etapa 21 (Backlog).
