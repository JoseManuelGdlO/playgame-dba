# Fase 0 / Plan 17: Sistema de Intentos + Perk "Segunda Opinión"

**Estado:** Aprobado
**Fecha:** 2026-07-14

## Problema

Hoy, `EstadoTurno::resolver` retira un ticket de la bandeja en el mismo instante en que se envía — acierto o error, consumiendo su costo de tiempo de inmediato y sin posibilidad de corregir. Un jugador que comete un error tipográfico o de lógica pierde el ticket por completo (tiempo gastado, $0/0 de pago), sin ninguna oportunidad de intentarlo de nuevo. El jugador pidió explícitamente 2-3 intentos por ticket antes de perderlo, más un perk que sume 2 intentos extra.

## Mecánica central

Cada ticket permite **3 intentos base** (configurable vía una constante, `INTENTOS_BASE = 3`), o **5 con el perk "Segunda Opinión" equipado** (+2). Un intento **incorrecto que todavía tiene reintentos disponibles**:

- No consume tiempo del presupuesto del turno.
- No paga dinero ni reputación.
- El ticket permanece (o regresa) a la bandeja, y sigue abierto en la consola para que el jugador corrija y vuelva a enviar.

Solo dos cosas cierran un ticket de verdad, igual que hoy: **acertar** (paga completo según los puntajes de esa entrega, sin importar en qué intento haya sido — Etapa/Plan 17: no hay penalización económica por tardarte en encontrar la respuesta) o **agotar todos los intentos disponibles** (se cobra el tiempo, se paga $0/0, exactamente como el comportamiento de fallo de hoy).

## Preservar la protección contra doble-envío

`resolver_ticket` (`lib.rs`) hoy llama a `EstadoTurno::resolver(&id)` **antes** de validar/premiar, deliberadamente — es una operación atómica de "quitar y verificar" bajo el mismo lock, para que un doble-clic concurrente en "✓ Enviar ticket" nunca pueda premiar el mismo ticket dos veces (un hallazgo de revisión de código de un plan anterior). Este plan preserva ese mecanismo intacto: `resolver()` no cambia. Se agrega un método nuevo, `EstadoTurno::reintentar(ticket: Ticket)`, que hace lo opuesto — reembolsa el `costo_tiempo` del ticket al presupuesto restante y lo reinserta en `pendientes` — usado únicamente cuando un intento falla pero todavía quedan reintentos. La secuencia en `resolver_ticket` pasa a ser: quitar (atómico, como hoy) → validar → si es correcto o si ya no quedan intentos, se queda retirado (se paga lo que corresponda); si es incorrecto y sobran intentos, se reinserta vía `reintentar()` y no se paga nada.

## Conteo de intentos

Un nuevo campo privado en `EstadoTurno` (no serializado al frontend, `#[serde(skip)]`), `intentos_usados: HashMap<String, u32>` (clave: id del ticket), incrementado cada vez que una entrega de ese ticket resulta incorrecta. Se limpia (se remueve la entrada) cuando el ticket finalmente se resuelve (acierto o agotamiento) — un ticket que vuelve a aparecer en un turno futuro (poco común, pero posible si el catálogo rota) empieza con el contador en cero.

## Respuesta al frontend

`ScoreResult` (la respuesta de `resolver_ticket`) gana un campo nuevo: `intentos_restantes: Option<u32>`.
- `None`: el resultado es **final** — acierto, o intentos agotados. El frontend se comporta exactamente como hoy: overlay de puntaje completo, dinero/reputación actualizados, `cargarTurno()`.
- `Some(n)` con `n > 0`: el intento falló pero **sobran `n` intentos**. El frontend no muestra el overlay de puntaje — solo un mensaje corto vía `setStatus`: *"No es correcto todavía — te quedan N intento(s)."* — y no llama a `cargarTurno()` de forma que interrumpa la sesión de consola activa (el ticket sigue siendo el mismo `ticketActivoId`).

## Perk nuevo: "Segunda Opinión"

- Categoría: `ManosRapidas` (junto con autocompletado y deshacer ilimitado — es, en esencia, otra herramienta que te ayuda a corregirte antes de perder el ticket).
- Efecto: nueva variante `Efecto::BonoIntentos(u32)` en el enum `Efecto` de `perks/mod.rs`, con valor `2`.
- Descripción jugador (sin jerga SQL): *"Antes de rendirte con un ticket difícil, tienes 2 intentos extra para corregir tu respuesta."*
- `EstadoJugador` gana un método `intentos_extra(&self, catalogo: Vec<Perk>) -> u32`, calcado del patrón ya existente de `multiplicador_dinero`/`multiplicador_reputacion` (recorre perks equipados, suma los `BonoIntentos` encontrados).

## Ajuste al tutorial (Plan 15/16)

El tutorial (`app/src/tutorial.js`) espera hoy que cada clic en "✓ Enviar ticket" lleve, tarde o temprano, a un resultado final (para disparar su remate vía `notificarCierreScoring`). Con intentos múltiples, un typo durante el tutorial ahora puede devolver un resultado **no final** (reintento disponible) — el tutorial no debe ocultar su diálogo ni armar la espera del overlay en ese caso. `app/src/main.js`'s manejador de "✓ Enviar ticket" debe llamar a `notificarClicEnviar()` **solo cuando `score.intentos_restantes` sea `null`/`undefined`** (resultado final); si es un número, el tutorial simplemente no se entera — su diálogo sigue visible, y el jugador ve el mensaje corto de reintento como cualquier otro jugador, corrige, y vuelve a intentar sin que el flujo del Mentor se rompa.

## Fuera de alcance

- No se toca el sistema de "concepto nuevo aparece por primera vez" (LIKE, JOIN) — ese es un plan aparte, todavía sin diseñar.
- No cambia el costo de tiempo por ticket ni el presupuesto de turno más allá de lo descrito arriba.
- No se penaliza económicamente por usar más de un intento (decisión explícita del jugador).

## Testing

Rust: TDD vía `cargo test` — nuevas pruebas para `EstadoTurno::reintentar` (reembolsa tiempo, reinserta el ticket), para el conteo de intentos en `resolver_ticket` (falla con reintentos → no cobra, no paga, ticket sigue pendiente; falla agotando intentos → cobra y paga $0 como hoy; acierta en el intento 2 o 3 → paga completo), y para `EstadoJugador::intentos_extra` (0 sin el perk, 2 con él equipado) y el nuevo conteo por categoría de perks (`ManosRapidas` ahora tiene 3, las demás siguen en 2). Frontend: mismo patrón manual de siempre (sin runner) — confirmar que un intento fallido con reintentos muestra el mensaje corto sin overlay y deja la consola abierta, que el intento final (acierto o agotamiento) se ve exactamente igual que hoy, y que un typo durante el tutorial no rompe su flujo.
