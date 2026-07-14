# Fase 0 / Plan 14: Mapa de Esquema Visual por Empresa — Design

**Estado:** Aprobado (brainstorming)
**Fecha:** 2026-07-13
**Etapa de referencia:** Etapa 16 (Diseño de bases de datos) — cada esquema (Hospital Arcángel: 6 tablas; Postafeta: 5 tablas) ya trae `COMMENT ON TABLE`/`COMMENT ON COLUMN` reales en el SQL de creación, escritos con el mismo tono del juego. Etapa 8 (Dirección Artística) para el estilo visual del overlay.

## Contexto

El jugador no tiene forma de ver qué tablas existen antes de escribir una query — hoy debe adivinar o inferir la estructura a partir del texto de cada ticket. Esto ya se había identificado como una pieza pendiente ("pestaña Base de Datos") durante el brainstorming del Plan 12, deliberadamente diferida a su propio spec. Este plan la construye: un mapa de esquema visual con relaciones, leído en vivo desde Postgres — sin inventar contenido nuevo, ya que cada tabla y columna real ya tiene una descripción humana escrita en el SQL de cada empresa (`app/src-tauri/src/db/hospital_arcangel.rs`, `.../postafeta.rs`).

## Alcance

- **Backend — introspección en vivo:** un nuevo comando de Tauri `esquema_actual()` que consulta la base de datos de la empresa activa (mismo pool que ya usa `run_query`) contra `information_schema`/`pg_catalog` para obtener: lista de tablas con su comentario real (`obj_description`), columnas por tabla (nombre, tipo, si acepta NULL, comentario real de columna vía `col_description`), y relaciones de llave foránea reales (tabla/columna origen → tabla/columna destino, vía `information_schema.table_constraints`/`key_column_usage`/`constraint_column_usage`). Como la introspección lee el pool activo, refleja automáticamente la empresa correcta incluso después de una transición (Hospital Arcángel → Postafeta) sin lógica especial.
- **Frontend — diagrama visual arrastrable:** un nuevo overlay `#esquema-overlay` (mismo estilo de ventana de terminal oscura que el menú de pausa del Plan 13, por consistencia — es un overlay de sistema accesible desde más de una pantalla). Cada tabla se dibuja como una caja de papel con: nombre, lista completa de columnas (nombre + tipo), y las descripciones reales de tabla/columnas — todo visible siempre, sin ocultar nada detrás de un hover. Las relaciones FK se dibujan como líneas SVG conectando las cajas correspondientes.
- **Posición inicial por empresa:** un lookup fijo en el frontend (`{empresa: {tabla: {x, y}}}`) con la posición de cada caja acomodada a mano — no hay algoritmo de auto-layout de grafos, ya que con 5-6 tablas por empresa un acomodo manual es más simple y confiable.
- **Arrastre del jugador:** el jugador puede arrastrar cualquier caja a una nueva posición; las líneas de relación se recalculan en tiempo real mientras arrastra. Esta posición solo vive en memoria mientras el overlay está abierto — al cerrarlo y volver a abrirlo (o reiniciar la partida), vuelve al acomodo por defecto. No se guarda en ningún lado.
- **Puntos de entrada:** un botón "Ver esquema" junto al editor SQL en la Consola, y la pestaña "Base de Datos" de vuelta en la barra de pestañas del Hub (existía como concepto en el Plan 12, se había quitado explícitamente de esa entrega) — ambos abren el mismo overlay con el mismo contenido, reflejando siempre la empresa activa.

## Fuera de alcance

- **Auto-layout de grafos:** decisión explícita — acomodo manual por empresa en el frontend.
- **Persistir el acomodo del jugador entre sesiones:** decisión explícita — solo vive en memoria mientras el overlay está abierto.
- **Contenido nuevo de descripciones:** todo el texto ya existe como `COMMENT ON TABLE`/`COMMENT ON COLUMN` real en el SQL de cada empresa — este plan solo lo lee e muestra, no escribe descripciones nuevas.
- **Ejecutar SQL desde el diagrama, o cualquier interacción más allá de ver/arrastrar:** el diagrama es de solo lectura respecto al juego — no reemplaza ni modifica el editor de la Consola.
- **Diagramas para empresas futuras (Fase 1+):** el lookup de posiciones se define solo para Hospital Arcángel y Postafeta, las únicas 2 empresas del MVP — agregar una empresa nueva requerirá agregar su propia entrada al lookup en su momento, no es responsabilidad de este plan.

## Arquitectura

- **`app/src-tauri/src/lib.rs`:** nuevo comando `esquema_actual(state: tauri::State<'_, AppState>) -> Result<EsquemaView, String>`, reutilizando `state.0.lock().unwrap().clone()` (mismo patrón que `run_query`) para obtener el pool activo. Nuevos structs serializables: `EsquemaView { tablas: Vec<TablaEsquema>, relaciones: Vec<RelacionEsquema> }`, `TablaEsquema { nombre: String, descripcion: Option<String>, columnas: Vec<ColumnaEsquema> }`, `ColumnaEsquema { nombre: String, tipo: String, nullable: bool, descripcion: Option<String> }`, `RelacionEsquema { tabla_origen: String, columna_origen: String, tabla_destino: String, columna_destino: String }`.
- **`app/src-tauri/src/db/mod.rs`:** nueva función `obtener_esquema(pool: &PgPool) -> anyhow::Result<EsquemaView>` (o similar) que ejecuta las 3 consultas de introspección (tablas+comentarios, columnas+comentarios, FKs) contra `information_schema`/`pg_catalog` y arma la vista combinada.
- **`app/src/index.html`:** nuevo `#esquema-overlay` (mismo patrón `.scoring-overlay` > `.ventana-terminal` que el menú de pausa), con un contenedor para las cajas de tabla (posicionadas absolutamente) y un `<svg>` superpuesto para las líneas de relación. Botón "Ver esquema" agregado junto a los botones de la Consola. Pestaña "Base de Datos" agregada de vuelta a `.barra-pestanas` en el Hub.
- **`app/src/main.js`:** nuevo `POSICIONES_TABLAS` (lookup fijo por empresa/tabla → {x, y}), función `mostrarEsquema()` que invoca `esquema_actual()`, construye las cajas de tabla y las líneas SVG; lógica de arrastre (mousedown/mousemove/mouseup) por caja que actualiza su posición en memoria y recalcula las líneas conectadas a esa caja en cada movimiento.

## Testing

Frontend: sin runner de tests (mismo patrón que planes anteriores) — revisión cuidadosa del diff + verificación manual guiada en la app real (visual, arrastre, ambas empresas). Backend: se agregan tests de integración para `obtener_esquema` contra ambas bases de datos reales (mismo patrón que los tests existentes en `db::hospital_arcangel::tests`/`db::postafeta::tests`), verificando que las tablas/columnas/relaciones esperadas aparezcan con sus comentarios reales.

## Auto-revisión del spec

- **Placeholders:** ninguno — el alcance exacto de la introspección, el acomodo manual de posiciones, y los 2 puntos de entrada están fijados con su razón.
- **Consistencia interna:** reutiliza el mismo patrón de acceso al pool que `run_query`, el mismo estilo visual que el menú de pausa (Plan 13), y contenido (comentarios SQL) que ya existe — no se inventa nada nuevo.
- **Alcance:** un plan, con trabajo real tanto en Rust (introspección) como en frontend (diagrama arrastrable) — cohesivo alrededor de un solo objetivo (ver el esquema), no se descompone en sub-proyectos.
