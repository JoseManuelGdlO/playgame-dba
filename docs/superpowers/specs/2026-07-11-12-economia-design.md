# Etapa 12: Economía

**Estado:** Aprobado
**Fecha:** 2026-07-11

## Tres recursos, tres roles distintos (nunca se mezclan)

| Recurso | Rol | Se gasta en |
|---|---|---|
| **Dinero** | Recompensa transaccional por trabajo entregado | Desbloquear perks permanentemente (único sink, Pilar 5 — evitar múltiples economías paralelas) |
| **Reputación** | Gate cualitativo de confianza/desempeño | No se gasta — solo sube/baja; cruzar el umbral dispara el ascenso/mini-boss, y también actúa como requisito de acceso para comprar perks (ver abajo) |
| **XP por arquetipo** | Maestría técnica específica (JOIN, CTE, Window Functions...) | No se gasta — desbloquea tiers de perks propios de ese arquetipo (Etapa 1) |

Mantenerlos con roles distintos evita el problema de "un solo recurso todopoderoso" donde el jugador elige entre ahorrar para progresar o gastar para mejorar — aquí ambas cosas ocurren en paralelo sin fricción.

## Fórmula de recompensa por ticket (al hacer "Enviar")

```
puntaje_base = (correctitud × peso_correctitud) + (velocidad_plan × peso_velocidad) + (buenas_prácticas × peso_prácticas)
puntaje_final = puntaje_base × multiplicador_perks_activos

dinero_ganado = puntaje_final × valor_base_del_ticket   (valor_base sube con prioridad/complejidad del ticket)
reputación_ganada = puntaje_final × factor_reputación_ticket   (mayor en tickets de mini-boss/alta prioridad)
xp_arquetipo = xp_fijo_por_uso(concepto_usado) × puntaje_final/100   (se reparte entre los arquetipos SQL usados en la query)
```

## Reputación: local por empresa, con arranque influido por el rango

- La reputación se reinicia a una base al empezar en una empresa nueva (eres "el nuevo", realismo narrativo) — pero esa base escala levemente con el rango del jugador (un Senior DBA no empieza en cero absoluto de confianza como un becario).
- Cruzar el umbral de reputación de esa empresa + superar el mini-boss = dispara el ascenso (Etapa 10).
- Reputación no se gasta nunca — solo puede bajar por tickets escalados/ignorados (Etapa 11-A).

## Perks: costo doble — dinero (se gasta) + reputación (se verifica, no se gasta)

- Cada perk tiene un costo en dinero (se descuenta al comprar) **y** un requisito mínimo de reputación (se verifica, no se consume — igual que un requisito de nivel).
- Esto evita que el jugador "farmee" tickets fáciles y triviales solo para acumular dinero y comprar perks avanzados sin haber demostrado desempeño real ante esa empresa — ata el poder del build a la confianza genuina ganada, no solo al volumen de trabajo.
- Un perk puede estar "visible pero bloqueado" si el jugador tiene el dinero pero no la reputación (o viceversa) — el panel de habilidades debe comunicar claramente cuál de los dos requisitos falta.
- Los costos en dinero escalan junto con los pagos de tickets: los perks tempranos cuestan poco (~3-5 tickets promedio de esa etapa); los de tiers altos cuestan más (~8-12 tickets promedio de su etapa) — la proporción se mantiene durante todo el juego, evitando que el early game se sienta lento o el late game trivial. Valores sujetos a ajuste en playtesting.

## Único sink de dinero: perks

Deliberadamente no hay cosméticos, oficina personalizable, ni otras economías paralelas — mantiene el sistema simple de balancear para un solo dev (Pilar 5). Si en producción se ve "hueco" de contenido, la expansión más barata es más perks, no una segunda economía.

## Penalización: solo reputación, nunca dinero

Los tickets perdidos/escalados (Etapa 11-A) penalizan reputación, no dinero — un solo canal de castigo, más fácil de balancear y de comunicar al jugador ("perdiste confianza", no "te multaron").

## Pendiente para etapas siguientes

- Catálogo completo de perks, sus costos/requisitos específicos y slots de loadout → Etapa 13 (Sistema RPG).
