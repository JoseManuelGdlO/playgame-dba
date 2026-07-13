# Fase 0 / Plan 5: Sistema RPG/Perks — Design

**Estado:** Aprobado (brainstorming)
**Fecha:** 2026-07-12
**Etapa de referencia:** Etapa 13 (Sistema RPG), dentro del alcance de Etapa 19 (MVP)

## Contexto

Hoy el "sistema de perks" es el stub literal del spike original: un solo perk falso (`perk_desbloqueado: bool`) que cuesta 300 fijo, sin nombre, sin categoría, sin conexión real a la economía. Este plan lo reemplaza por un catálogo real de 8 perks (2 por categoría, Etapa 19: "6-8 perks... 2 slots de loadout"), con desbloqueo permanente (dinero + reputación + maestría por arquetipo, Etapa 13) y un loadout de 2 slots equipables/desequipables gratis (Etapa 11-D).

## Alcance

- Catálogo de 8 perks, 2 por cada una de las 4 categorías de la Etapa 13 (Detective, Manos Rápidas, Billetera y Fama, Ritmo), en lenguaje de jugador, nunca jerga SQL.
- Desbloqueo permanente gatiado por 3 condiciones simultáneas (Etapa 13): dinero, reputación mínima, y maestría por arquetipo (reutilizando `xp_por_arquetipo` ya trackeado en el Plan 4 como la métrica de "uso acumulado").
- Loadout de 2 slots (Etapa 19 MVP; la escalera de hasta 7 slots por rango es Etapa 13 completa, fuera de alcance de este plan). Equipar/desequipar es gratis.
- **Solo la categoría Billetera y Fama tiene efecto mecánico real** ("Buena Fama" +20% reputación, "Bono Bajo la Mesa" +20% dinero) — las otras 3 categorías (Detective, Manos Rápidas, Ritmo) son desbloqueables/equipables de verdad, pero su efecto depende de sistemas que no existen todavía (consola SQL real con ERD/autocompletado, sistema de turnos) y quedan sin comportamiento mecánico por ahora — metadata capturada, no consumida, mismo patrón que `arquetipos` en los tickets (Plan 3).
- Cambio técnico necesario: `economia::calcular` pasa de un `multiplicador_perks: f64` único a `multiplicador_dinero: f64` + `multiplicador_reputacion: f64` separados, porque los dos perks con efecto real afectan recursos distintos — un multiplicador compartido los volvería indistinguibles.
- El stub viejo (`perk_desbloqueado: bool`, "Desbloquear perk (300)") se retira por completo — "Café Cargado" recupera ese nombre y ese costo aproximado dentro del catálogo real.
- Ajuste mínimo de frontend: el botón único se reemplaza por un `<select>` simple + botón "Desbloquear" que demuestra que el backend funciona — no se construye la UI de colección real tipo tienda (Etapa 13), que es trabajo de un plan de UI posterior.

## Fuera de alcance

- UI de colección real (grid/tienda filtrable, Etapa 13).
- Efectos mecánicos de Detective/Manos Rápidas/Ritmo (dependen de la consola SQL real y del sistema de turnos, ninguno construido todavía).
- Escalera de slots por rango (2→7, Etapa 13) — fija en 2 para todo este plan.
- Combos entre perks activos (Etapa 13, contenido de producción — Etapa 21 Backlog).
- Recalibración del umbral de ascenso de rango (`UMBRAL_ASCENSO_AUXILIAR = 500.0`, Plan 4) — se detectó durante el brainstorming que, al ritmo real de reputación por ticket (~0.5-1.2), cruzarlo tomaría muchos más tickets de los estimados originalmente. Es un ajuste de balance para una pasada posterior, no parte de este plan; los umbrales de reputación de los perks de este plan se calibran de forma independiente, a una escala que sí es alcanzable en pocos tickets.

## Catálogo (valores de partida, sujetos a ajuste en playtesting)

| Perk | Categoría | Efecto | Arquetipo / XP mínimo | Costo | Reputación mínima |
|---|---|---|---|---|---|
| Instinto | Detective | Sin efecto mecánico aún | Select / 20 | 200 | 3.0 |
| Rayos X | Detective | Sin efecto mecánico aún | Join / 40 | 300 | 5.0 |
| Piloto Automático | Manos Rápidas | Sin efecto mecánico aún | Select / 30 | 250 | 4.0 |
| Red de Seguridad | Manos Rápidas | Sin efecto mecánico aún | Agregación / 50 | 350 | 6.0 |
| Buena Fama | Billetera y Fama | +20% reputación ganada | Join / 40 | 300 | 5.0 |
| Bono Bajo la Mesa | Billetera y Fama | +20% dinero ganado | Agregación / 50 | 350 | 6.0 |
| Café Cargado | Ritmo | Sin efecto mecánico aún | Select / 20 | 200 | 3.0 |
| Modo Turbo | Ritmo | Sin efecto mecánico aún | Join / 60 | 400 | 7.0 |

## Arquitectura

- `app/src-tauri/src/perks/mod.rs`: modelo de datos (`Perk`, `Categoria`, `Efecto`), catálogo estático de 8 perks, funciones de verificación de desbloqueo.
- `economia/mod.rs` (Plan 4): `calcular()` cambia su tercer parámetro de `multiplicador_perks: f64` a `multiplicador_dinero: f64, multiplicador_reputacion: f64`.
- `EstadoJugador` (Plan 4): gana `perks_desbloqueados: Vec<&'static str>` y `perks_equipados: Vec<&'static str>` (máx. 2); pierde `perk_desbloqueado: bool`.
- `lib.rs`: nuevos comandos `catalogo_perks()`, `desbloquear_perk(id)`, `equipar_perk(id)`, `desequipar_perk(id)`; `submit_ticket` deriva los 2 multiplicadores de los perks equipados antes de llamar a `economia::calcular`; se retira `unlock_perk`/`PerkStatus`.
- `main.js`: reemplaza el botón único por un `<select>` + botón, ajuste mínimo de compatibilidad.

## Testing

Todo lo nuevo en `perks/` y los cambios en `economia::calcular` son puros (sin base de datos) — tests unitarios directos: condiciones de desbloqueo (dinero/reputación/XP, cada una faltante por separado), límite de 2 slots al equipar, y que los multiplicadores de dinero/reputación solo se apliquen cuando el perk correspondiente está *equipado* (no solo desbloqueado). Regresión completa + smoke test de la app real al final.

## Auto-revisión del spec

- **Placeholders:** ninguno — la decisión de qué categorías tienen efecto real, y cuáles no, está explícita con su razón (dependencia de sistemas no construidos).
- **Consistencia interna:** el catálogo, el modelo de datos y la integración usan los mismos nombres (`perks_desbloqueados`, `perks_equipados`, `multiplicador_dinero`, `multiplicador_reputacion`) de principio a fin.
- **Alcance:** un solo subsistema (RPG/perks), con un recorte explícito de 3 de 4 categorías a "sin efecto mecánico todavía" — cabe en un plan de implementación.
- **Ambigüedad:** ninguna — el catálogo completo de 8 perks con sus valores queda fijado arriba; la recalibración del umbral de ascenso queda explícitamente fuera de alcance, no es un olvido.
