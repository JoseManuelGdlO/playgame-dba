# Etapa 18: Arquitectura Técnica

**Estado:** Aprobado
**Fecha:** 2026-07-11

## Decisión de stack: Tauri/Electron (web-tech) + PostgreSQL embebido

Se evaluaron tres enfoques:

1. **Web-tech + Tauri/Electron + Postgres embebido (elegido).** Frontend en HTML/CSS/JS — encaja directamente con los mockups ya construidos en la Etapa 8 (son literalmente HTML). Empaquetado con Tauri o Electron para distribución en Steam (ambos frameworks tienen bindings de Steamworks SDK para logros/cloud saves, patrón común y probado en juegos indie shipeados). PostgreSQL corre embebido localmente (binario que arranca junto con el juego, sin necesidad de internet) — cumple TODOS los requisitos técnicos ya fijados (Etapa 16/17): `EXPLAIN` real, triggers, procedimientos almacenados, CTEs recursivos, window functions. Es también el stack con más soporte de herramientas de IA para desarrollo rápido (Pilar 5).
2. Motor de juego tradicional (Godot/Unity) + SQLite embebido. Descartado: SQLite no soporta procedimientos almacenados reales, rompiendo el requisito técnico de la Etapa 16 para los rangos altos (Lead DBA+) — obligaría a simular esas mecánicas en vez de ejecutarlas de verdad, chocando con el Pilar 1.
3. App web pura sin instalador (sql.js/WebAssembly). Descartado: mismas limitaciones de SQLite que la opción 2, y Steam generalmente espera un ejecutable nativo.

## Publicación en Steam: confirmado viable

Electron/Tauri + Steamworks SDK (vía `steamworks.js` o equivalente) es un patrón probado en juegos indie ya publicados. No hay obstáculo técnico para este objetivo.

## Componentes principales

- **Frontend (UI):** HTML/CSS/JS. Implementa la consola SQL "Terminal Moderno" (Etapa 8), el chat corporativo, el visor ERD, el panel de perks/loadout, y la capa de retratos pixel art estilo Papers, Please (adenda a la Etapa 8).
- **Motor de base de datos:** PostgreSQL embebido, una instancia local por partida/empresa activa. Provee `EXPLAIN` real, CTEs, window functions, triggers, procedimientos almacenados — sin dependencias de red.
- **Parser SQL para el linter de buenas prácticas (Etapa 17-C):** librería de parsing SQL a AST (ecosistema JS, ej. tipo `node-sql-parser`), corre localmente contra el texto de la query del jugador antes/después de ejecutar.
- **Capa de aislamiento por ticket (Etapa 14):** para tickets de escritura (`UPDATE/INSERT/DELETE`/DDL, rango DBA+), cada ticket corre sobre una copia/snapshot aislado del esquema de esa empresa, para que un error no afecte tickets subsecuentes ni el estado general de la partida.
- **Persistencia de partida:** guardado local (archivo/DB de progreso del jugador: rango, dinero, reputación, perks desbloqueados/equipados, empresa actual) + integración opcional con Steam Cloud Saves.

## Por qué no hay backend/servidor

El juego es 100% single-player, offline-first. No se requiere infraestructura de servidor propia (evita costos y mantenimiento continuo para un solo dev, Pilar 5). Cualquier función social futura (leaderboards, etc. — fuera de alcance por ahora) podría apoyarse en servicios ya provistos por Steamworks en vez de un backend propio.

## Pendiente para etapas siguientes

- Definir qué parte de este stack se implementa en el MVP vs. se difiere → Etapa 19 (MVP).
