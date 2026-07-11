# Etapa 3: Público Objetivo

**Estado:** Aprobado
**Fecha:** 2026-07-11

## Comprador principal: "El Fan de Tycoon/Sim"

- Ya compra *Game Dev Tycoon*, *Software Inc*, *Two Point Hospital*, *Prison Architect*.
- No busca aprender SQL — busca un buen sim de gestión/carrera con humor y progresión satisfactoria ("números que suben").
- Requisito de diseño: el juego debe funcionar con **cero conocimiento previo de SQL**. El tutorial y la curva de dificultad inicial son críticos para la conversión.
- Motivación emocional: fantasía de progreso de carrera + humor relatable de oficina, no "aprender una habilidad técnica".

## Comprador secundario: "El Career-Changer / Autodidacta"

- Developers junior, estudiantes de datos, gente en bootcamps o cambiando de carrera hacia tech/datos.
- Ya tiene motivación de aprendizaje activa — busca una alternativa más entretenida a SQLZoo/LeetCode/cursos.
- Mercado más pequeño que el primario, pero alto valor para el word-of-mouth: es el segmento que deja reseñas tipo *"literalmente aprendí SQL jugando esto"* — el motor de marketing orgánico más potente para un juego educativo encubierto.

## Comprador terciario (bonus, sin diseño dedicado): "El Completista de Puzzles"

- Fans de *TIS-100* / *Human Resource Machine* que disfrutan optimizar soluciones al límite.
- Se enganchan con el sistema de scoring (velocidad/plan de ejecución/buenas prácticas) y posibles leaderboards de "la query más elegante".
- Servido de forma natural por el sistema de validación multi-dimensión ya definido en la Etapa 1, sin requerir diseño adicional.

## Anti-persona / validador público: "El DBA/Ingeniero Senior Real"

No es comprador objetivo, pero su opinión pública (reviews, redes, streams) sobre si el SQL "se siente real" afecta la credibilidad frente al comprador secundario. Restricción de diseño: el humor puede exagerar la vida de oficina, pero el SQL y sus problemas de rendimiento deben ser técnicamente honestos, no una caricatura falsa.

## Implicación de diseño principal

Como el comprador principal no busca aprender SQL, el marketing y la primera hora de juego deben venderse 100% como simulador de carrera/gestión con humor, nunca como "app educativa" (analogía: *Kerbal Space Program* con la física orbital real — nadie lo compra para "aprender física", pero la aprende sin darse cuenta porque el juego la vuelve indispensable para progresar).
