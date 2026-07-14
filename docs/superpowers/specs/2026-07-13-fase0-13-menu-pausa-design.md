# Fase 0 / Plan 13: Menú de Pausa (ESC) — Design

**Estado:** Aprobado (brainstorming)
**Fecha:** 2026-07-13
**Etapa de referencia:** Etapa 8 (Dirección Artística) — reutiliza el estilo de ventana de terminal ya establecido para overlays de sistema (Plan 10).

## Contexto

Hoy no hay forma de pausar el juego, guardar manualmente (aunque el juego ya autoguarda tras cada acción importante), volver al Menú principal sin cerrar la app, o controlar el audio salvo por dos botones flotantes siempre visibles en la esquina superior derecha (Plan 11). El usuario pidió una pantalla de pausa accesible con ESC que consolide estas tres cosas: Guardar, Opciones (audio), Salir.

## Alcance

- **Apertura/cierre:** ESC abre un nuevo overlay de pausa (`#pausa-overlay`) desde el Hub o la Consola. Si el overlay de resultado (`#scoring-overlay`) o de agencia (`#agencia-overlay`) está visible, ESC no hace nada — evita overlays encimados. ESC de nuevo, o un botón "Continuar" dentro del overlay, lo cierra.
- **Estilo:** mismo `.ventana-terminal` (semáforo + barra de título) que ya usan los overlays de resultado/agencia — título `query-path — pausa`. Consistente porque es un menú de sistema, no parte visual del Hub ni de la Consola específicamente.
- **Guardar:** un botón que confirma "Partida guardada." vía `setStatus`. El juego ya autoguarda automáticamente tras cada acción importante (`resolver_ticket`, `cerrar_dia`, transiciones de empresa) — este botón es tranquilidad visual, no dispara ningún comando nuevo de Tauri.
- **Opciones:** los dos toggles de música/efectos (`#btn-mute-musica`/`#btn-mute-efectos`, Plan 11) se mudan de su posición flotante actual (esquina superior derecha, siempre visibles) hacia dentro de este overlay — mismos ids, misma lógica JS (`alternarMusica`/`alternarEfectos`), solo cambia su ubicación en el HTML. Dejan de estar visibles fuera del menú de pausa.
- **Salir:** un botón que regresa al Menú principal (`mostrarPantalla("menu")`, función ya existente) — no cierra la aplicación. Como el progreso ya está autoguardado, no se necesita una confirmación de "¿seguro que quieres salir?".

## Fuera de alcance

- **Guardado manual real (nuevo comando de Tauri):** decisión explícita del usuario — el botón "Guardar" es solo confirmación visual, el autoguardado existente ya cubre la necesidad real.
- **Cerrar la aplicación por completo:** "Salir" regresa al Menú, no mata el proceso.
- **Cualquier cambio en Rust/Tauri:** este plan es estrictamente frontend — reutiliza `mostrarPantalla`, `setStatus`, `alternarMusica`/`alternarEfectos`, todos ya existentes.
- **Pausar algún reloj/temporizador de juego:** el presupuesto de tiempo del turno es un contador de turno, no un reloj en tiempo real — no hay nada que "pausar" mecánicamente, el overlay solo bloquea la interacción visualmente mientras está abierto (mismo patrón que los overlays existentes).

## Arquitectura

- **`app/src/index.html`:** nuevo `<div id="pausa-overlay" class="scoring-overlay oculto">` (reutiliza la clase compartida `.scoring-overlay` para el fondo/backdrop, igual que agencia) envolviendo un `.ventana-terminal` con título "query-path — pausa" y 3 secciones: botón Guardar, sección Opciones (con los 2 toggles de audio movidos aquí desde su posición flotante actual), botón Salir, y un botón "Continuar" para cerrar.
- **`app/src/main.js`:** un listener `keydown` en `document` para la tecla `Escape` que alterna `#pausa-overlay` — solo abre si ni `#scoring-overlay` ni `#agencia-overlay` están visibles (chequeo de sus clases `oculto`). Botón "Guardar" → `setStatus("Partida guardada.", "ok")`. Botón "Salir" → cierra el overlay y llama `mostrarPantalla("menu")`. Botón "Continuar" → cierra el overlay. Los listeners de `btnMuteMusica`/`btnMuteEfectos` (ya existentes) seguirán funcionando sin cambios, solo sus elementos HTML cambian de posición.
- **`app/src/styles.css`:** ninguna clase nueva necesaria — reutiliza `.scoring-overlay`, `.ventana-terminal`, `.ventana-terminal-barra`, `.ventana-terminal-cuerpo`, `.scoring-panel`, `.control-audio` (o una versión sin `position:fixed` ya que ahora viven dentro del overlay, no flotando).

## Testing

Frontend puro (HTML/CSS/JS) — sin comandos de Tauri nuevos, sin cambios en Rust. Sin runner de tests de frontend en este proyecto (mismo patrón que Planes 6-12): corrección por revisión cuidadosa del diff (que ESC no abra el menú de pausa si un overlay de resultado/agencia ya está visible, que los ids de los botones de mute se conserven exactamente, que "Salir" efectivamente regrese al Menú) más verificación manual guiada en la app real al final del plan.

## Auto-revisión del spec

- **Placeholders:** ninguno — las 3 secciones (Guardar/Opciones/Salir), su comportamiento exacto, y el estilo visual están fijados con su razón.
- **Consistencia interna:** reutiliza `mostrarPantalla`, `setStatus`, `alternarMusica`/`alternarEfectos`, `.scoring-overlay`/`.ventana-terminal` — todos ya existentes, sin inventar mecánicas nuevas.
- **Alcance:** un solo plan, estrictamente frontend, que no reabre Rust ni el sistema de guardado real (Plan 9) — solo agrega una capa de UI de pausa sobre lo que ya existe.
