# Etapa 17: Sistema de Validación SQL

**Estado:** Aprobado
**Fecha:** 2026-07-11

## A. Correctitud: comparación de resultados, no de texto de la query

En vez de comparar la query del jugador contra una "query correcta" única (lo cual rompería el requisito de que existan varias soluciones válidas), se compara el resultado que produce contra un resultado de referencia precalculado con una "query dorada" escrita al crear el ticket (Etapa 14/16). Reglas de comparación:
- Se compara como conjunto de filas (sin importar el orden) salvo que el ticket pida explícitamente un orden (ahí se compara como secuencia).
- Se tolera nombres de columna/alias distintos si los valores coinciden en posición y tipo.
- Se tolera redondeo menor en decimales.

Esto permite que cualquier query que llegue al resultado correcto cuente como válida, sin importar cómo esté escrita — la variedad de soluciones emerge sola.

## B. Velocidad: costo del plan de ejecución, no reloj real

Medir segundos reales de ejecución es poco confiable (varía por hardware, y en datasets fijos y pequeños casi todo corre instantáneo). Se usa el costo estimado del plan de ejecución (`EXPLAIN`, Pilar 1: es la métrica que usa un DBA real) — detecta table scans innecesarios, joins mal ordenados, uso o no de índices disponibles. Determinista y reproducible en cualquier máquina.

## C. Buenas prácticas: linter estático de reglas sobre el AST de la query

Un conjunto de reglas deterministas analiza la estructura de la query (no su texto literal): evita `SELECT *` sin razón, prefiere JOIN sobre subconsultas anidadas innecesarias, alias consistentes, evita productos cartesianos accidentales, uso correcto del tipo de JOIN. Genera un puntaje, no un pase/falla binario (coherente con la Etapa 5: gradientes de calidad, no examen aprobado/reprobado).

## Cómo esto habilita múltiples soluciones válidas de forma natural

Como el scoring es 100% sobre resultado + plan + patrones estructurales, nunca sobre el texto exacto de la query, dos jugadores pueden resolver el mismo ticket con SQL completamente distinto y ambos obtener una puntuación alta (o baja) según qué tan buena sea su solución — sin necesidad de que el juego prediga cada variante posible de antemano.

## D. "El Mentor" sin depender de IA en tiempo real (protege Pilar 5 y la confiabilidad del producto)

El comentario del Mentor (Etapa 5/11) no se genera con una llamada a IA en vivo durante el juego — sería costoso, dependiente de internet y poco confiable para un producto comercial offline. Cada ticket tiene, precargados en su contenido (escritos una vez en producción, Pilar 5): una solución de referencia alternativa + un banco pequeño de comentarios pre-escritos, cada uno atado a una regla específica del linter (sección C) o a un patrón del plan (sección B). Cuando el sistema detecta qué regla "falló" en la solución del jugador, el Mentor muestra el comentario pre-escrito correspondiente — se siente reactivo y personalizado, pero es contenido curado, no generado en vivo.

## Requisito técnico impuesto a la Etapa 18 (Arquitectura)

Se necesita: un parser SQL real capaz de producir un AST para el linter (sección C), acceso al plan de ejecución real del motor (`EXPLAIN`, sección B), y una forma de ejecutar/comparar resultados de forma aislada por ticket (sección A).
