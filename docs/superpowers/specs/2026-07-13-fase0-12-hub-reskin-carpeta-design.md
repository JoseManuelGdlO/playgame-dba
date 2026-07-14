# Fase 0 / Plan 12: Reskin del Hub — escritorio de carpeta/papel rico — Design

**Estado:** Aprobado (brainstorming, con mockups)
**Fecha:** 2026-07-13
**Etapa de referencia:** Etapa 8 (Dirección Artística) — revisa específicamente la capa visual del Hub que el Plan 10 dejó, tras feedback del usuario de que seguía sintiéndose "plano" y "deprimente" incluso después del vestido visual y las animaciones/audio (Planes 10-11).

## Contexto

Después del Plan 11 (animaciones + audio), el usuario seguía sintiendo la app visualmente aburrida y compartió dos mockups generados con IA como referencia de dirección: (A) un dashboard neón estilo synthwave/cyberpunk, y (B) una versión mucho más rica y detallada de nuestro propio concepto de "escritorio de papel" — carpeta manila, stickers, clips, sellos, manchas de café, foto polaroid. El usuario eligió la dirección B por acercarse más a lo que ya veníamos construyendo (Etapa 8: Papers, Please como referencia central), solo que con mucho más detalle y calidez.

La imagen de referencia B trae, además del estilo visual, varias piezas de **funcionalidad nueva** que hoy no existen: barra de XP/Nivel, una escalera visible de 5 rangos (nuestro MVP solo tiene 2: Becario→Auxiliar de Sistemas — los otros 3 son contenido de Fase 2/3 según el propio roadmap del GDD), un panel de estadísticas con gráfica, y pestañas de navegación adicionales ("Base de Datos", "Mis Logros"). Se acordó explícitamente con el usuario descomponer esto en sub-proyectos secuenciales: **este plan cubre únicamente el reskin visual**, reutilizando datos que ya existen; XP/Nivel, el panel de estadísticas real, y las pantallas de Base de Datos y Mis Logros quedan como specs/planes independientes a brainstormear después.

Se iteró el mockup 3 veces en esta sesión de brainstorming (compañero visual): una primera versión demasiado simplificada fue rechazada por no parecerse a la referencia; una segunda versión con la estructura completa de 3 columnas y textura física fue aprobada en espíritu pero con emojis; la versión final reemplaza todos los emojis por iconos SVG de línea y recorta la barra de pestañas a solo lo que aplica a este plan.

## Alcance

Todo lo siguiente aplica **únicamente a `#pantalla-hub`** — Consola SQL y ambos overlays (resultado/agencia) no cambian, siguen exactamente como los dejó el Plan 10 (chrome de terminal oscuro sobre fondo de escritorio).

- **Fondo del Hub:** textura de carpeta manila cálida (degradado marrón/tostado), reemplazando el fondo oliva plano de `.escritorio` — pero solo dentro de `#pantalla-hub`, sin tocar la clase compartida `.escritorio` que Consola también usa.
- **Barra superior:** 💰 (dinero) y ⭐ (reputación) se rediseñan como cajas con borde e icono SVG en vez de emoji. 🎓 (rango) se muda de la barra superior a la nueva tarjeta de perfil.
- **Tarjeta de perfil (nueva, columna izquierda):** avatar genérico (mismo espíritu que los retratos placeholder del Plan 10) + nombre del rango actual, con un clip decorativo.
- **Tarjeta de progreso de carrera (nueva, columna izquierda, debajo del perfil):** muestra el rango actual con un ✓, el siguiente rango alcanzable, y colapsa el resto en una línea honesta "🔒 Próximamente..." — no se inventan nombres de rangos que no existen todavía en el backend (`Rango` solo tiene 2 variantes: `Becario`, `AuxiliarDeSistemas`).
- **Bandeja de tickets:** cada ticket gana un icono por su campo real `tipo` (`ReporteAnalisis` → icono de documento, `InvestigacionDepuracion` → icono de lupa) y una etiqueta de prioridad por su campo real `prioridad` (`Baja`/`Media`/`Urgente`, punto de color + texto). Ambos campos ya se serializan al frontend hoy (no tienen `#[serde(skip_serializing)]`) — cero cambios en Rust. Las tarjetas ganan textura de papel física (sombra, rotación leve, clip decorativo).
- **"Cerrar día":** mismo botón y comportamiento, icono de luna SVG en vez de emoji.
- **Panel de estadísticas (nuevo, placeholder honesto):** atenuado/inerte, bajo la bandeja, con el texto "Estadísticas (próximamente)" — sin números inventados. Es puramente decorativo hasta que exista su propio plan.
- **Perks:** mismo contenido y comandos ya existentes (`catalogo_perks`, `desbloquear_perk`, `equipar_perk`, `desequipar_perk`), restyled a la nueva paleta; mancha de café decorativa en la columna.
- **Barra de pestañas (nueva, decorativa/parcialmente funcional):** "Dashboard" (hace scroll a la bandeja), "Perks" (hace scroll a la sección de perks), "Mis Logros" (deshabilitada, muestra mensaje de estado "Próximamente" al hacer click — mismo patrón que el botón "Multijugador (próximamente)" del Menú). Sin pestañas de "Consola SQL" ni "Base de Datos" — rechazadas explícitamente por el usuario en esta sesión.
- **Iconos SVG de línea:** todo emoji restante (💰, ⭐, 🎓, 📋, 🔎, 🌙, 🔒, ✓, etc.) se reemplaza por un SVG de línea dibujado a mano, mismo espíritu placeholder que los retratos del Plan 10 — reemplazables después sin tocar la lógica que los selecciona.
- **Decoración:** sello rotado "CONFIDENCIAL", clips de papel, mancha de café — puramente visuales, sin interacción.

## Fuera de alcance

- **Consola SQL y overlays (resultado/agencia):** sin cambios, quedan como el Plan 10 los dejó.
- **Menú:** sin cambios, sigue siendo la pantalla de título limpia del Plan 10.
- **XP/Nivel:** sistema de progresión nuevo — spec propio, futuro.
- **Escalera completa de rangos (DBA Junior/Senior/Arquitecto/Director):** esos rangos son contenido de Fase 2/3 según el roadmap del GDD — no se construyen aquí; la tarjeta de progreso de carrera solo muestra honestamente lo que existe hoy.
- **Panel de estadísticas real (con tracking de datos):** requiere nuevo tracking en Rust — spec propio, futuro. Aquí solo es un placeholder visual.
- **Pestañas "Base de Datos" y "Mis Logros" funcionales:** pantallas nuevas — spec propio cada una, futuro. Aquí "Mis Logros" solo existe como pestaña deshabilitada.
- **Cualquier cambio en Rust/Tauri:** este plan reutiliza campos (`tipo`, `prioridad`) que ya viajan al frontend — no se toca `app/src-tauri/`.
- **Interacción de la bandeja/perks:** sin cambios — mismos comandos, mismos botones, mismo flujo de selección de ticket. Solo cambia el estilo visual y qué icono/etiqueta se muestra.

## Arquitectura

- **`app/src/styles.css`:** nuevo fondo con textura de carpeta scopeado a `#pantalla-hub` (no a `.escritorio`/`#app-shell`, para no afectar a Consola). Nuevas clases: `.tarjeta-perfil`, `.tarjeta-progreso-carrera`, `.etiqueta-sticky` (labels de sección estilo nota adhesiva), `.icono-tipo-ticket` (caja de color redondeada que envuelve el SVG), `.panel-stats-placeholder` (atenuado/inerte), `.sello-confidencial`, `.clip-papel`, `.mancha-cafe` (decorativos), `.barra-pestanas`/`.pestana`/`.pestana.activa`/`.pestana.bloqueada`.
- **`app/src/main.js`:** nuevo `ICONOS_TIPO_TICKET` (lookup por `tipo`, mismo patrón que `RETRATOS` del Plan 10) y `PRIORIDAD_INFO` (lookup por `prioridad` → color + etiqueta). `renderBandeja` se actualiza para pintar el icono y la etiqueta de prioridad de cada ticket usando estos lookups. El elemento `#rango` se reubica del header a la nueva tarjeta de perfil — mismo id, misma función `renderRango`, sin cambios de lógica. Nueva wiring de la barra de pestañas: "Dashboard"/"Perks" hacen `scrollIntoView` a sus secciones; "Mis Logros" deshabilitada, dispara `setStatus("Próximamente.", "")` al click.
- **`app/src/index.html`:** el interior de `#pantalla-hub` se reestructura en una cuadrícula de 3 columnas (perfil+progreso de carrera / bandeja+stats-placeholder / perks), se agrega la barra de pestañas al final, y los elementos decorativos (sello, clips, mancha de café) como marcado inerte.

## Testing

Frontend puro (HTML/CSS/JS) — sin comandos de Tauri nuevos, sin cambios en Rust. Sin runner de tests de frontend en este proyecto (mismo patrón que Planes 6-11): corrección por revisión cuidadosa del diff (que `tipo`/`prioridad` mapeen a los iconos/etiquetas correctos, que `#rango` siga siendo actualizado correctamente por `renderRango` tras el movimiento de posición, que los comandos de perks/tickets no cambien) más verificación manual guiada en la app real al final del plan — esta vez puramente visual, sin necesidad de escuchar audio (el Plan 11 no se toca).

## Auto-revisión del spec

- **Placeholders:** los únicos "placeholders" son intencionales y declarados como tales (panel de estadísticas, línea de rangos futuros, pestañas Base de Datos/Mis Logros) — no son huecos del spec, son honestidad de alcance ya acordada con el usuario.
- **Consistencia interna:** reutiliza `tipo`/`prioridad`, campos que ya viajan al frontend sin cambios — no se necesita ningún comando ni campo nuevo de Tauri.
- **Hueco de alcance detectado y cerrado durante el brainstorming:** el mockup original mezclaba reskin visual con funcionalidad nueva (XP, stats reales, más rangos, más pestañas) — se confirmó explícitamente con el usuario descomponer esto en sub-proyectos secuenciales, y este spec cubre solo el primero (reskin visual del Hub).
- **Alcance:** un solo plan, estrictamente frontend, que solo toca `#pantalla-hub` — no reabre Consola, overlays, Menú, ni ningún comando de Tauri.
