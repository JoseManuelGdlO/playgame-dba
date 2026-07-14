# Fase 0 / Plan 16: Escalón fácil + Tutorial extendido de El Mentor

**Estado:** Aprobado
**Fecha:** 2026-07-14

## Problema

Tras jugar el tutorial de Plan 15, el jugador reportó dos cosas: (1) incluso el nivel más fácil de tickets hoy disponible (SELECT + WHERE + ORDER BY sobre una sola tabla) ya se siente pesado como punto de partida — hace falta un escalón todavía más fácil antes de esos 3; (2) el tutorial explica bien los pasos ("escribe esto"), pero no profundiza en los conceptos (qué es una tabla, un FROM, un WHERE, una comparación) ni cubre para qué sirven el dinero, la reputación, los perks, o cómo funciona el ascenso de rango.

## Alcance: solo Hospital Arcángel

Este plan solo toca Hospital Arcángel (la primera empresa, donde vive el tutorial y donde arranca todo jugador nuevo). Postafeta (la segunda empresa del arco) queda fuera de alcance — se alcanza más adelante, con más experiencia acumulada, y puede revisarse por separado si hace falta.

## Escalón fácil: 3 tickets nuevos, antes de los 3 actuales

Un nivel de dificultad por debajo de "SELECT + WHERE + ORDER BY": solo elegir columnas de una tabla completa, sin filtrar ni ordenar. Se agregan al catálogo de `tickets/hospital_arcangel.rs`, todos con `arquetipos: vec![Arquetipo::Select]` (mismo rango Becario que ya tienen los tickets simples — no cambia `rango_requerido`), colocados **antes** de los 3 tickets simples existentes en el orden de declaración del catálogo (los tickets se sirven en orden de catálogo, sin aleatoriedad — Etapa/Plan 7).

1. **`hospital_reporte_departamentos`** (este es el que enseña el tutorial — Tramo A) — *"Lista el nombre y el piso de cada departamento."* → `SELECT nombre, piso FROM departamentos`
2. **`hospital_reporte_empleados_directorio`** — *"Lista el nombre y el puesto de cada empleado."* → `SELECT nombre, puesto FROM empleados`
3. **`hospital_reporte_habitaciones_inventario`** — *"Lista el número y tipo de cada habitación."* → `SELECT numero, tipo FROM habitaciones`

Los tickets 2 y 3 son práctica libre — el tutorial no los guía, aparecen para el jugador después de terminar el tutorial (o para quien lo saltó), dando más repetición en este nivel antes de subir a WHERE/ORDER BY.

Consecuencia directa de este reordenamiento: el primer ticket que recibe cualquier jugador nuevo deja de ser `hospital_reporte_pacientes_cardiologia` y pasa a ser `hospital_reporte_departamentos` — de ahí que el tutorial deba extenderse (ver abajo), y no solo su contenido.

## El tutorial pasa a 3 tramos continuos, sin fricción entre ellos

Mismo sistema de diálogo (`app/src/dialogo.js`, sin cambios) y mismo patrón de beats que Plan 15 (hablar / esperar-acción / escribir-cláusula), solo con más contenido y dos tickets guiados en vez de uno.

### Tramo A — `hospital_reporte_departamentos` (nuevo primer ticket)

1. Bienvenida (igual que hoy).
2. Bandeja (spotlight al primer ticket, igual que hoy).
3. **Nuevo — concepto de tabla** (sin tocar la consola todavía): "Una tabla es como una hoja de cálculo: cada fila es un departamento, cada columna es un dato de ese departamento — su nombre, en qué piso está." Sienta las bases antes de pedirle escribir nada.
4. Leer el ticket (spotlight al panel de info): traduce la solicitud a "necesitamos el nombre y el piso de cada departamento."
5. Escribir `SELECT nombre, piso` (spotlight al editor).
6. Escribir `FROM departamentos`.
7. Play.
8. Enviar — pasa por el motor de scoring real de siempre.
9. Al cerrar el scoring, en vez del cierre final de Plan 15, El Mentor continúa sin pausa: "Bien, ya viste tu primer reporte. El siguiente te va a pedir además un filtro." y abre directo el segundo ticket (mismo mecanismo que hoy usa `notificarClicPrimerTicket`, ahora aplicado también a este segundo ticket).

### Tramo B — `hospital_reporte_pacientes_cardiologia` (el ticket que ya enseña Plan 15)

10. Leer el segundo ticket (spotlight al panel de info): traduce "Cardiología" → `departamento_id = 1`.
11. Escribir `SELECT nombre, fecha_ingreso, diagnostico`.
12. Escribir `FROM pacientes`.
13. **Ampliado — comparaciones**: antes de pedir el WHERE, explica qué es un filtro y qué significa `=` ("igual a"), mencionando que también existen `>`, `<`, `>=`, `<=` para comparar números y fechas, aunque este ticket solo necesite `=`.
14. Escribir `WHERE departamento_id = 1`.
15. Escribir `ORDER BY fecha_ingreso DESC`.
16. Play.
17. Enviar — pasa por el motor de scoring real.
18. Al cerrar el scoring, en vez de terminar el tutorial aquí, El Mentor continúa hacia el Tramo C.

### Tramo C — Tour del Hub (nuevo)

Ya de vuelta en el Hub, sin abrir ningún ticket más — solo diálogo con spotlight sobre partes reales del Hub que ya existen:

19. Spotlight en las insignias 💰/⭐ (`.hub-badge` de dinero y reputación): "El dinero lo usas para desbloquear perks. La reputación además determina qué perks y qué rango puedes alcanzar."
20. Spotlight en la columna de Perks (`.hub-columna-perks`): "Los perks son bonos permanentes — algunos te dan más dinero o reputación por ticket resuelto. Cada uno cuesta dinero y pide una reputación mínima para desbloquearse."
21. Spotlight en la tarjeta de Progreso de carrera (`.tarjeta-progreso-carrera`): "Cada ticket bien resuelto suma reputación. Al llegar al umbral necesario subes de rango — de Becario a Auxiliar de Sistemas, por ejemplo — lo que desbloquea tickets nuevos y un slot más de perk."
22. Cierre final (el mismo mensaje de "ahí te dejo, el resto de tu bandeja funciona igual" de Plan 15, movido a este punto).

**El botón de saltar sigue disponible durante los 3 tramos completos** — saltar en cualquier punto termina todo el tutorial de golpe, sin importar en qué tramo esté.

## Cambios técnicos

- **`app/src-tauri/src/tickets/hospital_arcangel.rs`**: 3 nuevas entradas en `catalogo()`, antes de las 3 actuales de `plantilla_reporte_simple`. El test que cuenta el total de tickets del catálogo (`reportes: 6, ...`, Plan 7) se actualiza al nuevo total.
- **`app/src/main.js`**: el guard añadido en el fix de revisión final de Plan 15 (`TICKET_TUTORIAL_ID`, que compara `pendientes[0].id`) se reemplaza por dos constantes, `TICKET_TUTORIAL_ID_PASO1`/`TICKET_TUTORIAL_ID_PASO2`, verificando que `pendientes[0].id` y `pendientes[1].id` sean exactamente esos dos ids, en ese orden, antes de arrancar el tutorial. Si no coinciden, el tutorial simplemente no arranca (mismo comportamiento defensivo que ya existe).
- **`app/src/tutorial.js`**: se extiende con los beats del Tramo A y Tramo C, y se amplía el texto del Tramo B (comparaciones) — mismo patrón de funciones (`pasoN...`), mismas primitivas de `dialogo.js` (`mostrarDialogo`/`ocultarDialogo`/`permitirSiempre`), sin cambios a `dialogo.js` mismo.
- **Sin cambios en el motor de validación/economía** — los 3 tickets nuevos usan la misma plantilla (`plantilla_reporte_simple`) y el mismo pipeline de scoring que ya existe.

## Fuera de alcance

- Postafeta no se toca en este plan.
- No se añade ningún concepto de "primera vez que aparece LIKE/JOIN" (eso es un plan aparte, todavía por diseñar).
- No cambia el sistema de intentos/reintentos (otro plan aparte, todavía por diseñar).

## Testing

Mismo patrón manual que Plan 15 (sin runner de frontend): confirmar que una partida nueva sirve `hospital_reporte_departamentos` como primer ticket, que el tutorial recorre los 3 tramos sin fricción entre tickets, que el tour del Hub resalta los elementos reales correctos, que saltar funciona en cualquier tramo, y que los 2 tickets nuevos de práctica libre (empleados, habitaciones) aparecen después con normalidad. En Rust, `cargo test` cubre el nuevo conteo de catálogo y que los 3 tickets nuevos ejecutan correctamente contra el esquema real (mismo patrón que los tickets simples existentes).
