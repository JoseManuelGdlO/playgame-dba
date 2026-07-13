# Fase 0 / Plan 10: Vestido Visual (escritorio, terminal, retratos) — Design

**Estado:** Aprobado (brainstorming, con mockups)
**Fecha:** 2026-07-13
**Etapa de referencia:** Etapa 8 (Dirección Artística — "Realismo de Software Corporativo": paleta Catppuccin Mocha para superficies de herramienta, retratos pixel art estilo Papers, Please para superficies de personaje/mundo, ambas ya aprobadas), Etapa 1 (Papers, Please como referencia central del diseño)

## Contexto

El Plan 9 separó la app en 3 pantallas navegables (Menú/Hub/Consola) pero deliberadamente sin tocar el vestido visual — sigue usando el HTML/CSS plano de siempre. Jugando esa build, la app "se siente como una web app", no como un juego: la paleta Catppuccin Mocha ya está bien implementada en `styles.css`, pero falta por completo la capa de personajes/mundo (retratos, escritorio) que la Etapa 8 define como el segundo pilar visual del juego. Este plan cierra esa brecha construyendo la capa que faltaba, sobre las 3 pantallas que el Plan 9 ya dejó listas — sin volver a tocar navegación, guardado, ni ningún comando de Tauri.

Explorado con mockups interactivos en esta sesión de brainstorming (compañero visual): 3 iteraciones — dirección de "chrome" de terminal → capa de escritorio/retratos → aplicación a las 3 pantallas reales del Plan 9, con dos hallazgos de alcance resueltos explícitamente con el usuario durante el proceso (ver Alcance).

## Alcance

- **Menú:** pantalla de título limpia, sin el escritorio (fuera de la ficción, como el menú de cualquier juego) — fondo oscuro tipo degradado radial, título "QUERY_PATH" en monoespaciada, botones con la paleta Catppuccin ya existente.
- **Hub y Consola (dentro de `#app-shell`):** fondo de escritorio apagado y burocrático (degradado oscuro oliva/café), reemplazando el fondo plano Catppuccin solo en estas 2 pantallas — la paleta Catppuccin sigue viva dentro de los elementos de "herramienta real" (consola, resultados), consistente con la separación de capas ya aprobada en la Etapa 8.
- **Bandeja de tickets (Hub):** cada ticket se dibuja como una ficha de papel física sobre el escritorio (color crema, ligera rotación aleatoria, sombra) en vez de una tarjeta plana.
- **Perks (Hub) — cambia de interacción, no solo de estilo:** hoy es un `<select>` con 2 botones compartidos; pasa a ser una lista de fichas de papel individuales ("carpeta de personal"), cada una con su propio botón contextual (Desbloquear / Equipar / Desequipar según su estado) — decisión explícita del usuario tras señalar el desajuste entre el mockup y la interacción actual. Sigue usando los mismos comandos Tauri ya existentes (`catalogo_perks`, `desbloquear_perk`, `equipar_perk`, `desequipar_perk`), solo cambia cómo se renderizan y a qué escuchan los botones.
- **Consola:** el editor/resultado se enmarca en una ventana de terminal (barra de título + semáforo rojo/amarillo/verde, título dinámico con el id del ticket activo) sobre el fondo de escritorio. Se agrega un retrato del solicitante junto al motivo/solicitud del ticket.
- **Retratos — 3 en total, no uno por solicitante:** genérico (para cualquier solicitante que no sea uno de los 2 siguientes), El Mentor, y el Auditor de Cumplimiento (Plan 8) — mapeados por el campo `ticket.solicitante` (ya viaja al frontend, sin cambios de backend). Placeholders construidos como SVG simple estilo pixel art, paleta apagada/burocrática — sin arte real encargado, reemplazables después sin tocar código (Etapa 19: arte mínimo/placeholder es válido para el MVP).
- **Overlays de scoring y Agencia (Planes 6/8):** se incluyen en este plan — el fondo del overlay pasa del rgba plano actual al mismo degradado de escritorio (semi-transparente), y el panel se enmarca con la misma ventana de terminal que la Consola (título "query-path — resultado" / "query-path — agencia").

## Fuera de alcance

- **Arte pixel real** (encargado o generado fuera de esta sesión): los 3 retratos son placeholders SVG — se reemplazan después sin tocar la lógica que los selecciona.
- **Más de 3 retratos:** el resto de solicitantes (Contabilidad, RH, Finanzas, etc.) usan el genérico — one-off portraits por solicitante son Fase 1+, cuando haya más contenido narrativo por empresa.
- **Cualquier cambio de navegación, guardado, o comando de Tauri:** el Plan 9 ya cerró esa capa; este plan no la vuelve a tocar.
- **Restructurar la bandeja de tickets como interacción** (a diferencia de Perks): la bandeja ya es una lista con un botón por ítem — solo cambia su estilo visual (papel), no su interacción, a diferencia de Perks que sí requería el cambio de dropdown a lista.
- **Verificación visual automatizada:** mismo patrón ya usado en Planes 6/7/8/9 — verificación manual guiada en la app real al final del plan.

## Arquitectura

- `app/src/styles.css`: nuevas clases reutilizables — `.escritorio` (fondo de degradado, aplicado a `#app-shell`), `.ventana-terminal` (barra de título + semáforo, envuelve el contenido de la Consola y de ambos overlays), `.papel`/`.papel-perk` (ficha de papel para tickets/perks, con variantes de rotación y borde de color por estado), `.retrato` (caja de retrato de tamaño fijo). El fondo de `.scoring-overlay`/`.agencia-overlay` cambia de `rgba(...)` plano al degradado de escritorio semi-transparente.
- `app/src/index.html`: agrega el contenedor de retrato en la Consola (junto a `#ticket-activo-info`); envuelve el contenido de la Consola y de ambos overlays en el marcado de `.ventana-terminal`; reemplaza `<select id="perks-select">` por `<ul id="lista-perks">`.
- `app/src/main.js`: nueva función `retratoParaSolicitante(solicitante)` (3 casos + default); `renderPerks` reescrito para construir una lista de fichas con botón contextual por perk (mismo patrón ya usado por `renderBandeja`/`seleccionarTicket` para los tickets), en vez de poblar un `<select>`; `seleccionarTicket` pinta el retrato correspondiente al abrir la Consola.

## Testing

Frontend puro (HTML/CSS/JS) — sin comandos de Tauri nuevos, sin cambios en Rust. Sin runner de tests de frontend en este proyecto (mismo patrón ya usado en Planes 6/7/8/9): la corrección se cubre con revisión cuidadosa del diff (ids consistentes entre HTML/JS, la lógica de `renderPerks` reescrita se comporta igual que antes desde la perspectiva de los comandos que invoca) más verificación manual guiada en la app real al final del plan — esta vez sí se puede confirmar visualmente, ya que en esta misma sesión se demostró que `screencapture` funciona en este entorno (usado para verificar los Planes 6-8).

## Auto-revisión del spec

- **Placeholders:** ninguno — los 3 retratos, la regla de mapeo por `solicitante`, y el alcance exacto de Perks (cambio de interacción, no solo estilo) están fijados con su razón.
- **Consistencia interna:** reutiliza el campo `ticket.solicitante` que ya viaja al frontend sin cambios — no se necesita ningún comando ni campo nuevo de Tauri para este plan completo.
- **Hueco de alcance detectado y cerrado durante el brainstorming:** el mockup original de Perks asumía fichas individuales, pero el código real usa un `<select>` — confirmado explícitamente con el usuario que este plan sí incluye ese cambio de interacción (no es "solo CSS"), documentado arriba en vez de descubrirse a mitad de la implementación.
- **Alcance:** un solo plan, estrictamente frontend, sobre pantallas que el Plan 9 ya construyó — sin generalizar a más retratos ni a un sistema de temas por empresa (eso es Fase 1+, cuando existan más empresas/personajes).
