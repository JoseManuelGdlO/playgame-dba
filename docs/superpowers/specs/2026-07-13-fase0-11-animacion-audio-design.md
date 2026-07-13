# Fase 0 / Plan 11: Animación y Audio (vida y sonido) — Design

**Estado:** Aprobado (brainstorming)
**Fecha:** 2026-07-13
**Etapa de referencia:** Etapa 8 (Dirección Artística), Etapa 19 (Arte placeholder/mínimo — sonido generado por código es el equivalente de audio a los retratos SVG placeholder)

## Contexto

El Plan 10 vistió visualmente el escritorio, la consola y los overlays, pero el usuario, al jugarlo, sigue sintiendo la app "plana" y "aburrida": las transiciones entre pantallas son instantáneas, las fichas de papel no reaccionan a nada, el momento de resolver un ticket no tiene tensión, y no existe ningún sonido en el juego (ni efectos ni música). Este plan agrega la capa de "vida" que falta — movimiento y audio — sin tocar lógica de juego, comandos de Tauri, ni Rust.

Además, en esta misma conversación se aplicó por separado (fuera de este plan, por ser un cambio de una línea) que la ventana ahora abre en pantalla completa (`"fullscreen": true` en `tauri.conf.json`, commit `1ff6a28`).

## Alcance

- **Motor de audio (`app/src/audio.js`, nuevo módulo):** 100% Web Audio API, sin archivos de audio externos — mismo principio que los retratos SVG placeholder del Plan 10 (arte/audio mínimo válido para el MVP, reemplazable después sin tocar la lógica que lo dispara). Expone funciones puras de disparo de efectos y control de un loop de ambiente.
- **Ambiente de fondo synthwave:** un loop generado (arpegio + pad, estilo retro-futurista) que arranca en el primer gesto del usuario (política de autoplay de los navegadores/webviews) y se repite indefinidamente vía auto-agendado recursivo.
- **Efectos de sonido puntuales**, cada uno mapeado a un punto exacto ya existente en `main.js`:
  - Click genérico → un solo listener delegado en `document` (`event.target.closest("button")`), no uno por botón.
  - Tecleo mecánico → `keydown` en `#sql-input`, con variación aleatoria de pitch (±10%).
  - Tick de revelado → cada línea del overlay de scoring que aparece (ver abajo).
  - Éxito / Error → al final de la secuencia de reveal del overlay de scoring, según `score.pass`.
  - Cierre de día → click en `#btn-cerrar-dia`.
  - Ascenso de rango → cuando el estado recibido indica que el rango cambió respecto al anterior.
- **Dos controles de mute independientes** (música / efectos): dos íconos de bocina fijos a nivel `<body>` (visibles en las 3 pantallas y overlays, mismo patrón que `#status-msg` del Plan 9), sin persistencia entre sesiones — siempre arrancan con audio activado.
- **Transiciones de pantalla:** `mostrarPantalla()` pasa de un toggle instantáneo de `.oculto` a un cross-fade de ~250ms (opacity transition, con el `display:none` real retrasado hasta que termina el fade).
- **Fichas de papel con vida (Hub):** `renderBandeja` y `renderPerks` agregan una animación de entrada escalonada (~60ms de delay por ítem, opacity+translateY) al construir la lista; `.papel` gana un estado `:hover` que la endereza y levanta con más sombra.
- **Overlay de scoring con reveal progresivo:** `mostrarScoring` pasa de pintar todo de golpe a una secuencia `async`: el overlay entra con fade+scale (~200ms), luego cada línea de stat aparece una por una cada ~350ms (con su sonido de tick), y termina con el ✅/❌ grande acompañado de un pulso de brillo (éxito) o un shake corto (error). El botón "Cerrar" queda deshabilitado hasta que la secuencia completa termina.

## Fuera de alcance

- **Archivos de audio reales / música con licencia:** decisión explícita del usuario — todo el audio es generado por código, sin riesgo de derechos de autor. Si más adelante se contrata audio real, este plan no lo bloquea (el motor se reemplaza sin tocar los puntos de disparo).
- **Persistencia del estado de mute entre sesiones:** cada sesión arranca con música y efectos activados.
- **Sliders de volumen:** solo mute/unmute binario por canal (música, efectos), no control granular de volumen.
- **Fullscreen:** ya aplicado por separado (commit `1ff6a28`), no es parte de este plan.
- **Cualquier cambio de lógica de juego, comandos de Tauri, o Rust:** este plan es estrictamente frontend (HTML/CSS/JS), igual que el Plan 10.
- **Slide u otras transiciones más elaboradas:** se eligió cross-fade simple por velocidad de percepción; transiciones más vistosas quedan fuera.

## Arquitectura

- **`app/src/audio.js` (nuevo):** un único `AudioContext` compartido, creado perezosamente en el primer gesto del usuario. Exporta funciones puras por efecto (`sfxClick()`, `sfxTecleo()`, `sfxTick()`, `sfxExito()`, `sfxError()`, `sfxCierreDia()`, `sfxAscenso()`), cada una programando 1-3 osciladores con envolvente de volumen (attack/decay) que se desconectan solos al terminar — sin estado que limpiar. Exporta también `iniciarAmbiente()` (arranca el loop synthwave recursivo la primera vez que se llama) y `alternarMusica()`/`alternarEfectos()` (banderas booleanas; música muteada corta la ganancia del bus de ambiente sin detener el scheduler, para poder reanudar sin reiniciar el loop).
- **`app/src/main.js` (modificado):** un listener delegado en `document` para el click genérico; un listener `keydown` en `#sql-input` para el tecleo; llamadas a `sfxCierreDia()`/detección de ascenso en los puntos ya existentes; `renderBandeja`/`renderPerks` agregan la clase de entrada escalonada; `mostrarPantalla()` reescrita para el cross-fade; `mostrarScoring` reescrita como secuencia `async` con el reveal progresivo descrito arriba.
- **`app/src/styles.css` (modificado):** nuevas clases `.papel-entrando` (keyframe de entrada), `.papel:hover` (estado de levantado), clases de transición para el cross-fade de pantallas, clases para el fade+scale del overlay y el pulso/shake del resultado final.
- **`app/src/index.html` (modificado):** dos botones de mute (música/efectos) a nivel `<body>`, fuera de las 3 pantallas — mismo patrón que `#status-msg`.

## Testing

Frontend puro (HTML/CSS/JS) — sin comandos de Tauri nuevos, sin cambios en Rust. Sin runner de tests de frontend en este proyecto (mismo patrón que Planes 6-10): corrección por revisión cuidadosa del diff (temporizadores y secuencias correctas, nodos de audio desconectados tras usarse, ids consistentes entre HTML/JS) más verificación manual guiada en la app real al final del plan — con el añadido de que esta vez hay que **escuchar**, no solo ver: la verificación debe confirmar audio real en la app corriendo, no solo capturas de pantalla.

## Auto-revisión del spec

- **Placeholders:** ninguno — el motor de audio, cada disparador, y el comportamiento exacto de cada animación están fijados con su razón.
- **Consistencia interna:** reutiliza los mismos patrones ya establecidos en Planes 9/10 (elemento persistente a nivel `<body>` para `#status-msg` → mismo patrón para los botones de mute; retratos SVG placeholder → mismo principio de "generado, sin archivos" aplicado a audio).
- **Hueco de alcance detectado y cerrado durante el brainstorming:** el usuario pidió inicialmente "buscar música 80s", lo cual habría implicado archivos con licencia real — se confirmó explícitamente con el usuario que la música es un ambiente synthwave generado por código, no un track real, para mantener la consistencia con la decisión de "sin archivos externos" ya tomada para los efectos de sonido.
- **Alcance:** un solo plan, estrictamente frontend, que no reabre navegación/guardado/Tauri (Plan 9) ni el vestido visual ya construido (Plan 10) — solo agrega movimiento y sonido sobre lo que ya existe.
