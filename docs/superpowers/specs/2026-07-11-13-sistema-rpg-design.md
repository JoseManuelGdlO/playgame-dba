# Etapa 13: Sistema RPG

**Estado:** Aprobado
**Fecha:** 2026-07-11

## Formato: colección filtrable, no árbol ramificado literal

Para mantener el costo de producción bajo (Pilar 5) y la UI coherente con la dirección artística (Etapa 8), el sistema de habilidades es una colección tipo grid/tienda (como la colección de jokers de Balatro), filtrable por categoría — no un diagrama de árbol ramificado con líneas de conexión.

## Slots de loadout: crecen con el rango

- Se empieza con 2 slots activos (Becario).
- Se gana +1 slot en 5 hitos de rango específicos (Auxiliar de Sistemas, DBA Junior, Senior DBA, Lead DBA, Chief Data Officer), llegando a un máximo de 7 slots.
- Cada ascenso da un premio mecánico inmediato y palpable, no solo narrativo (Etapa 10).

## Principio de nomenclatura (corrección importante): lenguaje de jugador, nunca jerga técnica

Primera versión de este catálogo nombraba los perks con términos técnicos de SQL ("Sin Cartesianos", "Legibilidad CTE"), lo cual viola el **Pilar 3 (Etapa 4): "Carrera, no examen"** — sonaba a certificación, no a habilidad de personaje divertida de comprar. Corrección: la maestría por arquetipo SQL sigue existiendo por debajo (para efectos de qué se desbloquea y cuándo — ver sección siguiente), pero el nombre y la descripción de cada perk deben sentirse como una habilidad de personaje, centrada en **lo que el perk hace por el jugador**, nunca en qué concepto SQL "certifica".

## Catálogo de perks (4 categorías, lenguaje de jugador)

### 1. Perks de Detective (ayudan a encontrar cosas)
- **"Instinto"** — al abrir un ticket, resalta automáticamente qué tablas probablemente necesitas.
- **"Rayos X"** — ves las relaciones entre tablas al instante en el visor de esquema.
- **"Olfato de Reportero"** — avisa si hay una columna "sospechosa" (nulos raros, datos legados) antes de que muerda en la query.

### 2. Perks de Manos Rápidas (hacen la query más fácil de escribir)
- **"Piloto Automático"** — autocompletado más inteligente en el editor.
- **"Plantilla en el Bolsillo"** — inserta con un clic un bloque común (JOIN, agrupado, ventana).
- **"Red de Seguridad"** — deshacer ilimitado + aviso antes de ejecutar algo probablemente erróneo.

### 3. Perks de Billetera y Fama (aceleran dinero/reputación)
- **"Buena Fama"** — reputación extra por cada ticket entregado.
- **"Bono Bajo la Mesa"** — dinero extra por cada ticket entregado.
- **"Currículum Brillante"** — empiezas cada empresa nueva con más reputación base (Etapa 12).

### 4. Perks de Ritmo (ganan tiempo de turno)
- **"Café Cargado"** — reduce el costo de tiempo de los tickets.
- **"Modo Turbo"** — más presupuesto de tiempo por turno.

## Maestría por arquetipo (mecanismo interno de desbloqueo, invisible en el nombre del perk)

6 arquetipos SQL (JOIN, Agregación/GROUP BY, Subconsultas, CTE, Window Functions, Optimización/Índices), 5 tiers cada uno. Cada tier de perk se desbloquea al cumplir tres condiciones simultáneas:
1. Uso acumulado suficiente de ese arquetipo (sube automáticamente al usarlo en tickets — análogo al nivel de mano de Balatro).
2. Dinero suficiente.
3. Reputación mínima (Etapa 12).

Esto ata el poder del build a la práctica real del concepto, no solo al dinero — pero el jugador ve el perk con su nombre/efecto de personaje ("Instinto" tier 3), nunca con la etiqueta técnica que lo desbloquea por debajo.

## Combos: emergen solos, en lenguaje de jugador

Ciertos pares de perks activos simultáneamente disparan un bono adicional automático al resolver una query que use ambos conceptos correctamente — no requieren compra extra. Ejemplo: "Instinto" + "Piloto Automático" activos + ticket resuelto sin errores → bono **"Racha Perfecta"** con animación. El jugador descubre combos jugando (a lo Balatro), no leyendo una ficha técnica. El catálogo completo de combos es contenido de producción (Etapa 21: Backlog), no algo a fijar en este documento.

## Pendiente para etapas siguientes

- Catálogo completo y balanceado de los ~30 perks de arquetipo + ~10-15 generales → Etapa 21 (Backlog).
- Lista completa de combos → Etapa 21 (Backlog).
