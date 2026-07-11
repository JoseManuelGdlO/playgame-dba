# Etapa 9: Mundo y Ambientación

**Estado:** Aprobado
**Fecha:** 2026-07-11

## Decisión de tono: absurdismo corporativo heightened, con una frontera crítica

Se eligió un nivel de sátira "heightened" (tipo *Severance* / *Sorry to Bother You*) en vez de un realismo contenido tipo *Office Space*. Esto entra en tensión directa con el Pilar 1 (Etapa 4: "SQL real, consecuencias reales") y la Propuesta de Valor (Etapa 6: los problemas técnicos deben ser honestos, no una caricatura) — se resuelve con una regla de oro explícita.

## Regla de oro: absurdismo en el envoltorio, realismo en los datos

- **Sí puede ser absurdo:** políticas de oficina sin sentido literal, jerarquías imposibles, rituales corporativos extraños, personajes excéntricos, la premisa de cómo el jugador rota de empresa en empresa.
- **Nunca puede ser absurdo:** el esquema de datos, la lógica de negocio del ticket, lo que hace que una query sea correcta o eficiente. Un hospital ficticio puede tener un CEO que se comunica solo por acertijos, pero su tabla `pacientes` se comporta como una tabla de pacientes real.

Referencia de diseño: en *Severance*, la oficina es profundamente surrealista, pero la tarea que hacen los personajes ("refinar números") no necesita tener sentido real. En Query Path sí necesita tenerlo, porque es la propuesta de valor central — de ahí la frontera.

## Geografía y época

Un universo satírico globalizado y sin ubicación real específica ("anytown corporativo" contemporáneo, ligeramente futurista/atemporal) — como el pueblo sin nombre de *Severance* o el "no-lugar" de *Sorry to Bother You*. Las empresas son versiones ficticias/paródicas reconocibles (un "Netflix", un "Amazon", un banco, un casino) sin atarse a un país real, evitando alienar jugadores de mercados específicos y esquivando especificidad legal/política real.

## El dispositivo que conecta las empresas: la Agencia

El jugador nunca es contratado directamente por ninguna empresa — siempre lo asigna la misma agencia de staffing omnipresente y ligeramente siniestra (nombre de trabajo: "Ω Staffing Solutions"), que reasigna analistas de datos entre industrias completamente distintas sin explicación, como si fuera perfectamente normal.

Funciones de este dispositivo de lore:
- Explica diegéticamente por qué el jugador salta de un hospital a un casino sin necesitar una "trama" de transición.
- Es la fuente natural del chiste recurrente de RRHH ya establecido en la Etapa 7 — es literalmente la misma entidad en todas las empresas, porque es la misma agencia detrás de todas.
- Da textura absurdista real (burocracia opaca, políticas sin sentido, tono ligero "Kafka/Severance") sin convertirse en una trama/antagonista que haya que mantener consistente entre empresas (protege el Pilar 5, Etapa 4).

## Pendiente para etapas siguientes

- Nombres finales/parodias específicas de cada empresa → Etapa 15 (Diseño de empresas).
- Cómo se presenta narrativamente la transición de la Agencia entre empresas (cinemática, mensaje, evento) → Etapa 11 (Mecánicas).
