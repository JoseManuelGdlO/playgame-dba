# Etapa 10: Progresión

**Estado:** Aprobado
**Fecha:** 2026-07-11

## Principio rector: el SQL nunca se restringe artificialmente — solo los tickets y los perks

El motor SQL es real (Pilar 1): el jugador podría escribir una window function en el ticket #1 si ya la conoce, y funcionaría. Lo que sí está gateado por rango es (a) qué tan complejos/exigentes son los tickets disponibles, y (b) qué perks/bonos del árbol de habilidades (Etapa 1/13) se pueden desbloquear. La dificultad percibida viene de la demanda del trabajo, no de una restricción artificial del lenguaje.

## Principio rector #2 (confirmado explícitamente por el diseñador): el juego es SIEMPRE sobre escribir queries

En ningún rango — ni siquiera en el más alto (Chief Data Officer) — el gameplay central se reemplaza por otro tipo de mecánica (minijuegos de gestión, simulación abstracta de equipo, etc.). Todo ascenso de rango debe expresarse a través de más contexto/alcance narrativo alrededor del ticket, nunca sustituyendo la acción de escribir SQL como el verbo principal del juego. Este es un principio no negociable para todas las etapas siguientes (Mecánicas, Sistema de misiones, Diseño de empresas).

## Escalera de rangos, mapeada a conceptos SQL dominantes

| Rango | Conceptos SQL que definen ese nivel |
|---|---|
| Becario | `SELECT`, `WHERE`, `ORDER BY`, `LIMIT`, alias básicos |
| Auxiliar de Sistemas | `JOIN` (inner), `COUNT/SUM/AVG`, `GROUP BY` |
| Analista de Datos | `LEFT/RIGHT JOIN`, `HAVING`, subconsultas simples, funciones de fecha/texto |
| DBA Junior | subconsultas correlacionadas, CTEs simples, funciones de ventana básicas (`ROW_NUMBER`, `RANK`) |
| DBA | índices, lectura de execution plans, optimización básica, CTEs recursivos |
| Senior DBA | window functions avanzadas (`LAG/LEAD`, particiones complejas), optimización de queries pesadas, transacciones/deadlocks |
| Arquitecto de Datos | diseño de esquema, normalización/desnormalización, particionado, vistas materializadas |
| Lead DBA | procedimientos almacenados, triggers, seguridad/permisos, backups/restore |
| Data Engineer | ETL, pipelines de datos, integración entre sistemas |
| Chief Data Officer | capstone: sigue escribiendo SQL como mecánica central; sus tickets tienen mayor alcance narrativo (decisiones que afectan a toda la empresa), pero la fantasía de "gestión" se expresa en el *contexto* del ticket, nunca reemplazando la mecánica principal |

## Cómo se combinan rango + empresa

- Cada rango tiene un pool de 2-3 empresas elegibles (etiquetadas por rango en su diseño, Etapa 15), y al ascender la Agencia (Etapa 9) ofrece esas opciones — el jugador elige su siguiente destino.
- Esto da rejugabilidad barata: mismas empresas, orden distinto entre partidas/jugadores, sin necesitar contenido adicional (Pilar 5).
- El ascenso de rango ocurre al completar el arco de una empresa (mini-boss superado, Etapa 7) y cruzar un umbral de reputación/XP (números exactos se definen en Etapa 12: Economía).

## Pendiente para etapas siguientes

- Umbrales exactos de XP/reputación por rango → Etapa 12 (Economía).
- Etiquetado de empresas por rango-elegible → Etapa 15 (Diseño de empresas).
