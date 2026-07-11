# Etapa 8: Dirección Artística

**Estado:** Aprobado
**Fecha:** 2026-07-11

## Estética central: "Realismo de Software Corporativo"

Todo el juego se presenta como si fuera software real que se usa en el trabajo (consola SQL, chat corporativo, visor de esquema/wiki) en vez de un "mundo ilustrado". Refuerza el Pilar 1 (SQL real) y el Pilar 5 (barato de producir para un solo dev): casi no se necesita arte de personajes tradicional, la mayoría del arte es UI.

Explorado vía compañero visual: se descartaron "Retro Corporate OS" (Windows 95/2000, ventanas biseladas) e "Ilustrado Editorial" (retratos cálidos con textura de papel) a favor de un enfoque de **terminal moderno**, más alineado con el Pilar 1 y más barato de mantener consistente.

## Herramienta principal: Consola SQL "Terminal Moderno"

- Fondo oscuro estilo GNOME Terminal/iTerm2/VS Code, SQL con syntax highlighting a color.
- **Paleta confirmada:** tipo *Catppuccin Mocha* (base `#1e1e2e`, superficie `#313244`, texto `#cdd6f4`, acentos verde `#a6e3a1` / azul `#89b4fa` / morado `#cba6f7` / durazno `#fab387`) — paleta popular entre developers reales, aporta autenticidad y es fácil de tematizar de forma consistente en toda la interfaz.
- **Editor multi-query**: el jugador puede escribir varias sentencias SQL a la vez, separadas por `;`. Un selector permite elegir "ejecutar selección actual" o "ejecutar todas".
- **Botón ▶ Play** (icono universal) reemplaza el "Probar" inicial — ejecuta de verdad (Pilar 1), sin puntuar ni disparar consecuencias narrativas.
- **Resultados en pestañas de grid real**, estilo DBeaver/SQL Workbench: cada ejecución abre su propio "Resultado N" con tabla de encabezados, filas y pie con conteo/tiempo.
- **"✓ Enviar ticket"** sigue siendo la acción deliberada y separada que compromete la respuesta y dispara el scoring multidimensional (Etapas 1, 2 y 5).

## Superficies secundarias (mismo lenguaje visual, misma paleta)

- **Chat corporativo** (entrega de tickets, Etapa 7): mismos tonos oscuros/acentos, burbujas de mensaje con avatares mínimos.
- **Visor de esquema/ERD** (Etapa 7): presentado como otra "herramienta real" (tipo dbdiagram.io/DataGrip), misma paleta — refuerza la identidad de "vives dentro de herramientas de developer reales".

## Personajes: retratos pixel art estilo "Papers, Please" (actualizado en Etapa 18)

**Revisión:** la primera versión de esta etapa proponía avatares mínimos de iniciales/iconos genéricos. Al definir la Etapa 18 (Arquitectura técnica) se decidió añadir una capa de arte adicional: los personajes (jefes, Mentor, mini-bosses) se representan con **retratos pixel art de baja resolución estilo *Papers, Please*** — pequeños, expresivos, paleta apagada/burocrática, animación mínima. Sigue siendo barato de producir (incluso asistible con generación de imagen por IA), pero da mucho más carácter que un ícono de iniciales.

Esto crea dos capas de arte separadas y deliberadamente distintas, ambas coexistiendo sin conflicto:
- **Superficies de herramienta** (consola SQL, chat, visor ERD): Terminal Moderno, paleta Catppuccin Mocha — son software real dentro de la ficción.
- **Superficies de mundo/personajes** (retratos, marco de escritorio/cubículo): pixel art estilo Papers, Please, paleta más apagada — es la "carne" humana/narrativa alrededor del software.

El chiste satírico del "avatar genérico" de la versión anterior se descarta junto con ese enfoque.

## Tipografía

- Monoespaciada (Consolas/Courier-like) para todo lo que es código/consola.
- Sans-serif limpia (Segoe UI/Inter-like) para chrome de UI, chats y texto narrativo.

## Proceso

Explorado con mockups interactivos vía el compañero visual de brainstorming (3 iteraciones): dirección de arte inicial → refinamiento a terminal Linux con distinción Probar/Enviar → editor multi-query con ejecución selectiva y resultados en grid estilo DBeaver/SQL Workbench.
