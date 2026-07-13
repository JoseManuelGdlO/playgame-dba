# Fase 0 / Plan 9: Pantallas (Menú/Hub/Consola) + Guardado — Design

**Estado:** Aprobado (brainstorming)
**Fecha:** 2026-07-13
**Etapa de referencia:** Etapa 1 (Papers, Please como referencia de diseño), Etapa 8 (Dirección Artística — separación consola/mundo, aplicada aquí solo a nivel estructural), Etapa 11 (Mecánicas) — extiende el MVP (Etapa 19) más allá de su tabla de alcance original, motivado por la necesidad de que el prototipo se sienta como un juego real antes de playtestear la hipótesis central.

## Contexto

Hoy toda la app vive en una sola pantalla (`index.html`): bandeja, consola SQL, perks y resultado están todos visibles a la vez, sin ninguna pantalla de menú ni forma de guardar/cargar progreso — el estado del jugador vive solo en memoria y se pierde al cerrar la app. Jugando la build actual, se siente "como una web app", no como un juego — un problema real para el propósito del MVP (Etapa 19: validar si el loop se siente genuinamente divertido), porque una presentación plana puede enmascarar si el loop en sí es divertido. Este plan resuelve la parte **estructural**: separar la experiencia en 3 pantallas navegables (Menú, Hub, Consola) y agregar guardado/carga real. El vestido visual (escritorio apagado, retratos, marco de terminal — explorado con mockups en esta misma sesión de brainstorming) es un plan aparte (Plan 10), que se construye encima de la estructura de este plan.

## Alcance

### Pantallas y navegación

- **① Menú** (pantalla inicial de la app): opciones "Cargar partida" (deshabilitada si no existe una partida guardada), "Iniciar partida" (siempre disponible, empieza de cero y sobreescribe el estado en memoria — no borra el archivo de guardado hasta el próximo autosave), y "Multijugador" (deshabilitada, con texto "próximamente" — fuera de alcance de este plan y de toda Fase 0/1).
- **② Hub**: stats (💰/⭐/🎓), bandeja de tickets (igual que hoy, con el encabezado ya dinámico por fase del Plan 8), sección de Perks, botón "Cerrar día". **No incluye el editor SQL ni el área de resultado** — eso vive solo en la Consola.
- **③ Consola**: se entra desde el Hub al hacer clic en "Trabajar en este" sobre un ticket. Muestra motivo/solicitud del ticket activo, el editor SQL, los botones ▶ Play / ✓ Enviar ticket, y el área de resultado — todo lo que hoy ya existe en la sección `.console`/`.output`, movido a su propia vista. Incluye un botón **"‹ Volver"** que regresa al Hub sin resolver el ticket (el ticket sigue pendiente en la bandeja, sin penalización ni consumo de tiempo).
- Al hacer "✓ Enviar ticket" y cerrar la pantalla de scoring, la navegación vuelve automáticamente al Hub (mismo patrón que hoy, solo que ahora es un cambio de pantalla real, no solo un overlay que se cierra).
- El overlay de la Agencia (Plan 8) sigue apareciendo sobre el Hub cuando `fase == ArcoCompletado` — no es una pantalla nueva, es el mismo overlay ya construido.
- Implementación: 3 bloques `<section>` de nivel superior en `index.html` (`#pantalla-menu`, `#pantalla-hub`, `#pantalla-consola`), una función `mostrarPantalla(nombre)` en `main.js` que oculta las otras dos y muestra la solicitada — mismo mecanismo `.oculto`/`classList` ya usado para los overlays existentes, sin introducir un router nuevo.

### Guardado y carga

- **Un solo slot de guardado**, en un archivo JSON dentro del directorio de datos de la app (`app.path().app_data_dir()`, ya disponible vía `tauri::Manager`, importado en `lib.rs`).
- **Autoguardado** (sin acción manual del jugador) después de: cada `resolver_ticket`, cada `cerrar_dia`, y cada `confirmar_transicion_agencia` — nunca se pierde más de una acción si la app se cierra inesperadamente.
- **Qué se persiste:** dinero, reputación, XP por arquetipo, rango, perks desbloqueados/equipados (por id, no el catálogo completo), la empresa activa, la fase del arco, el índice de rotación, el presupuesto restante del turno, y **los ids** de los tickets pendientes (no el `Ticket` completo — al cargar, cada id se resuelve contra el catálogo de la empresa activa, o contra `mini_boss_hospital_arcangel()` si `fase != TrabajoNormal`, reconstruyendo el `Ticket` completo). Esto evita tener que serializar/deserializar el `Ticket` completo (que hoy tiene campos `#[serde(skip_serializing)]` como `sql_dorada` que nunca deben llegar al cliente, pero sí deben persistir en el archivo de guardado local).
- **Comandos nuevos:** `existe_partida_guardada() -> bool` (para habilitar/deshabilitar "Cargar partida" en el Menú), `cargar_partida() -> Result<EstadoJuegoView, String>`, `iniciar_partida() -> EstadoJuegoView` — ambos reemplazan todo el estado gestionado (`Jugador`, `Turno`, `AppState`) de la misma forma en que `confirmar_transicion_agencia` (Plan 8) ya reemplaza `Turno`/`AppState` en caliente, y devuelven una vista combinada que el Hub usa para pintarse por primera vez.
- `setup()` deja de construir el turno inicial de Hospital Arcángel automáticamente: solo prepara Postgres embebido y deja el resto del estado en un valor por defecto neutro hasta que el jugador elija "Cargar partida" o "Iniciar partida" en el Menú.

## Fuera de alcance

- **Vestido visual** (escritorio apagado, marco de terminal con semáforo, retratos placeholder de solicitantes/Mentor/Auditor): Plan 10, construido sobre las 3 pantallas de este plan.
- **Multijugador**: solo la entrada deshabilitada en el Menú; ninguna lógica real.
- **Múltiples slots de guardado con nombre**: un solo slot alcanza para esta versión; slots con nombre son Fase 1+.
- **Guardado en la nube / sincronización**: fuera de alcance, no forma parte de ningún plan actual.
- **Persistencia de Postafeta si nunca se transiciona**: no aplica — el guardado siempre refleja la empresa activa real (Hospital Arcángel o Postafeta, la que sea que esté cargada al momento de guardar).

## Arquitectura

- `app/src-tauri/src/guardado/mod.rs` (nuevo módulo): tipo `PartidaGuardada` (serializable/deserializable) con los campos listados arriba; funciones `guardar(app_data_dir, estado) -> anyhow::Result<()>` y `cargar(app_data_dir) -> anyhow::Result<Option<PartidaGuardada>>` que leen/escriben el archivo JSON. `Rango`, `Arquetipo`, `tickets::db::Company`, y el nuevo `FaseArco` (Plan 8, hoy privado a `lib.rs`) necesitan derivar `serde::Deserialize` además de `Serialize` — cambio mecánico, no de comportamiento.
- `app/src-tauri/src/lib.rs`: nuevos comandos `existe_partida_guardada`, `cargar_partida`, `iniciar_partida`; hooks de autosave al final de `resolver_ticket`, `cerrar_dia`, `confirmar_transicion_agencia`; `setup()` deja de armar el turno inicial automáticamente.
- `app/src/index.html` / `main.js`: 3 secciones de pantalla + `mostrarPantalla()`; el Hub deja de incluir `.console`/`.output` (se mueven a la nueva sección de Consola); nuevo botón "‹ Volver".

## Testing

- Rust: tests de round-trip de `guardado::guardar`/`cargar` (guardar un `PartidaGuardada` de prueba, cargarlo, comparar campo por campo) — lógica pura de serialización, sin necesidad de Postgres embebido. Tests de que `cargar_partida`/`iniciar_partida` reconstruyen correctamente los tickets pendientes por id contra el catálogo/mini-boss correcto, siguiendo el mismo patrón de integración contra Postgres embebido ya usado en planes anteriores.
- Verificación manual guiada en la app real (mismo patrón que Planes 6/7/8): navegar Menú→Hub→Consola→Hub, cerrar la app a mitad de una partida y confirmar que "Cargar partida" la recupera exactamente donde quedó.

## Auto-revisión del spec

- **Placeholders:** ninguno — el mecanismo de guardado (ids de tickets, no el `Ticket` completo), los momentos de autosave, y la mecánica de navegación (mostrar/ocultar secciones) están fijados con su razón.
- **Consistencia interna:** reutiliza el mismo patrón de "reemplazar estado gestionado en caliente" que `confirmar_transicion_agencia` (Plan 8) ya estableció, en vez de inventar un mecanismo nuevo.
- **Alcance:** separa deliberadamente la estructura (este plan) del vestido visual (Plan 10) — evita mezclar una feature de persistencia con una de arte, aunque ambas nacieron de la misma conversación.
- **Ambigüedad:** la decisión de persistir por id de ticket (no el `Ticket` completo) fue una decisión de diseño explícita durante la escritura de este documento, no un supuesto silencioso — ver Arquitectura arriba para la razón (evita exponer/reconstruir campos internos como `sql_dorada` de forma redundante).
