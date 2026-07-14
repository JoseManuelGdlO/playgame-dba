# Fase 0 / Plan 15: Tutorial de Onboarding con El Mentor

**Estado:** Aprobado
**Fecha:** 2026-07-14

## Problema

El MVP no tiene ningún camino de entrada para jugadores que nunca han usado una base de datos. Alguien sin experiencia previa con SQL llega directo a la bandeja de tickets sin ninguna guía sobre qué es una tabla, una columna, o cómo se escribe un `SELECT`.

## Personaje: reusar a El Mentor

El GDD ya define a **El Mentor** (Etapa 5/11): un personaje narrativo que acompaña al jugador toda su carrera, con retrato pixel-art placeholder ya existente (`RETRATOS["El Mentor"]` en `main.js`) y una caja de comentario que hoy aparece tras cada ticket (`#scoring-mentor`). El tutorial reusa este mismo personaje en vez de introducir uno nuevo — da continuidad narrativa gratis y encaja con el diseño ya aprobado de Dirección Creativa (Etapa 7): el onboarding debe sentirse diegético ("IT/el Mentor te da acceso al wiki de datos"), no una pantalla de tutorial fuera de ficción.

## Alcance: enseñar SQL desde cero, no solo los controles

El tutorial no asume conocimiento previo de SQL ni de bases de datos. Guía al jugador, paso a paso, a resolver su primer ticket real de principio a fin — escribiendo la query él mismo (Pilar 1: SQL real siempre, nunca una versión simplificada o un minijuego sustituto). Usa el primer ticket real del catálogo de Hospital Arcángel:

> `hospital_reporte_pacientes_cardiologia` — "Contabilidad quiere saber quién ha pisado Cardiología últimamente." → `SELECT nombre, fecha_ingreso, diagnostico FROM pacientes WHERE departamento_id = 1 ORDER BY fecha_ingreso DESC`

## Disparo y salida

- Se dispara automáticamente al terminar `iniciar_partida()` (partida nueva), antes de pintar la bandeja por primera vez. **Nunca** se dispara al `cargar_partida()`.
- Un botón **"Ya sé SQL, saltar"** está visible en todo momento durante el tutorial (no solo al inicio). Al presionarlo, termina el tutorial al instante, quita cualquier bloqueo/spotlight activo, dispara un comentario de cierre corto de El Mentor, y deja al jugador en el punto normal del juego (mismo ticket abierto, sin resolver, bandeja y demás controles ya utilizables).
- No hay persistencia de progreso del tutorial: vive como una bandera en memoria (`tutorialActivo`) en `main.js`. Si el jugador cierra la app a medias y luego carga esa misma partida, el tutorial no se repite — simplemente continúa donde el autoguardado lo dejó, sin más guía. Si en cambio inicia otra partida nueva, se repite desde el principio salvo que lo salte.
- Mientras el tutorial está activo, **Esc no abre el menú de pausa** — la única forma de interrumpir el flujo guiado es el botón "Saltar". Esto es intencional: el tutorial debe ser invasivo, el jugador no puede hacer nada más que seguirlo o saltarlo.

## Arquitectura: sistema de diálogo reusable (`app/src/dialogo.js`)

Nuevo módulo frontend puro, mismo patrón que `audio.js` (Plan 11): sin comandos de Tauri nuevos, sin cambios en Rust, funciones puras exportadas, sin estado que limpiar entre usos.

- **`mostrarDialogo(personaje, lineas, opciones)`**: crea una tarjeta flotante centrada (retrato + nombre + texto, mismo lenguaje visual que `.scoring-panel`) y revela cada línea de texto progresivamente, carácter por carácter (~30-40ms por carácter). Cada 2-3 caracteres dispara `sfxBlip()` (nueva función en `audio.js`, mismo patrón que `sfxTecleo`/`sfxTick`): un blip corto con pitch ligeramente aleatorio — el "gruñido" hablado estilo *Papers, Please* — respetando el mute de efectos ya existente.
  - Un click mientras el texto se revela lo completa instantáneamente. Un click con el texto ya completo avanza a la siguiente línea, o cierra el diálogo si era la última.
- **`opciones.bloquear = true`** (el caso por defecto en el tutorial): un click-catcher invisible a pantalla completa intercepta todo click excepto la tarjeta de diálogo y, si se especifica, `opciones.elementoPermitido` (selector CSS) — ese elemento queda interactuable e iluminado con un recorte tipo spotlight (`box-shadow: 0 0 0 2000px rgba(10,10,8,0.72)` alrededor de sus límites, con borde `#f9e2af`), igual que en el mockup validado durante el brainstorming.
- **`ocultarDialogo()`**: destruye la tarjeta, el click-catcher y el recorte, sin dejar listeners colgando.

Este sistema queda disponible para reusarse más adelante en el comentario post-ticket de El Mentor (`#scoring-mentor`, hoy texto estático) — no se toca ahora, es fuera de alcance de este plan, pero la arquitectura no lo bloquea.

## Guion: beats del tutorial

1. **Bienvenida** (modal centrado, sin spotlight): El Mentor recibe al jugador su primer día, explica que va a resolver pedidos reales escribiendo SQL de verdad contra la base de datos de la empresa. Aparece el botón "Ya sé SQL, saltar".
2. **La bandeja** (spotlight sobre la tarjeta del primer ticket en la bandeja): "Ahí llegan tus pendientes. Dale click a ese para abrir el primero." — el jugador da el click él mismo para avanzar; no hay botón "continuar" del Mentor en este paso.
3. **Leer el ticket** (spotlight sobre el panel de info del ticket, ya en la consola): explica qué pide Contabilidad y traduce "Cardiología" → `departamento_id = 1`.
4. **Enseñar por cláusula** (spotlight sobre el editor, resto de la pantalla bloqueado), una cláusula a la vez — `SELECT columnas`, `FROM tabla`, `WHERE filtro`, `ORDER BY orden` — cada una con una metáfora simple (tabla = hoja de cálculo, columna = dato, fila = paciente). El jugador escribe cada cláusula él mismo en el editor real.
5. **Probarlo** (spotlight sobre ▶ Play): "Dale Play para probarlo contra la base de datos real."
6. **Enviarlo** (spotlight sobre ✓ Enviar ticket): pasa por el motor de scoring real de siempre — sin tratamiento especial, así se resuelve todo en el juego.
7. **Cierre**: tras el scoring normal, un comentario especial de "primer día" de El Mentor (distinto del comentario post-ticket estándar), termina el tutorial, se oculta el botón de saltar, el juego continúa normal.

### Validación por cláusula

Cada paso de "escribe X" no duplica el motor de validación SQL real (`validation/`) — solo revisa, en el frontend, que el fragmento de texto esperado ya esté presente en el editor (comparación floja: sin distinguir mayúsculas/espacios) para dejar avanzar al siguiente beat. El veredicto real de correcto/incorrecto sigue viniendo únicamente de "Enviar ticket" → motor de validación existente, igual que cualquier otro ticket del juego.

Si el jugador ya sabe SQL y escribe la query completa de un jirón (sin esperar cada beat), el tutorial detecta que ya cumplió varias cláusulas de golpe y avanza los pasos correspondientes sin obligarlo a esperar cada blip uno por uno.

## Fuera de alcance

- No se toca el comentario post-ticket existente de El Mentor (`#scoring-mentor`) — sigue siendo texto estático por ahora, aunque el nuevo sistema de diálogo queda listo para adoptarlo después.
- No se añade ningún comando nuevo de Tauri ni cambio en Rust — 100% frontend, igual que el patrón ya establecido en los Planes 6-11.
- No se persiste progreso parcial del tutorial entre sesiones — reiniciar la app a medias de un tutorial no lo retoma; solo se dispara de nuevo con una partida nueva.

## Testing

Mismo patrón que Planes 6-11 (sin runner de frontend en este proyecto): corrección por revisión cuidadosa del diff más verificación manual guiada en la app real — confirmar que una partida nueva dispara el tutorial, que cada beat bloquea correctamente lo que debe bloquear, que "Saltar" funciona desde cualquier punto, que Esc no abre pausa mientras el tutorial está activo, que cargar una partida existente no lo dispara, y que el ticket real se resuelve y puntúa igual que cualquier otro al terminar.
