# Fase 0 / Plan 5: Sistema RPG/Perks Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reemplazar el stub de un solo perk falso (`perk_desbloqueado: bool`, costo fijo 300) por un catálogo real de 8 perks (Etapa 13: 2 por categoría), con desbloqueo permanente gatiado por dinero + reputación + maestría por arquetipo, y un loadout de 2 slots equipables/desequipables gratis.

**Architecture:** `app/src-tauri/src/perks/mod.rs` define el catálogo estático (datos puros, sin lógica de estado). `economia/mod.rs` (Plan 4) gana la lógica de desbloqueo/equipo sobre `EstadoJugador`, y `calcular()` pasa de un multiplicador único a dos separados (dinero/reputación), porque los únicos 2 perks con efecto real afectan recursos distintos. `lib.rs` expone 4 comandos Tauri nuevos y retira el viejo `unlock_perk`.

**Tech Stack:** Rust puro, sin dependencias nuevas.

## Global Constraints

- Catálogo de 8 perks, 2 por cada una de las 4 categorías (Detective, Manos Rápidas, Billetera y Fama, Ritmo) — nombres y descripciones en lenguaje de jugador, nunca jerga SQL (Etapa 13).
- **Solo Billetera y Fama tiene efecto mecánico real** ("Buena Fama" +20% reputación, "Bono Bajo la Mesa" +20% dinero) — las otras 3 categorías son desbloqueables/equipables de verdad pero mecánicamente inertes hasta que existan sus sistemas (consola SQL real, sistema de turnos).
- Desbloqueo permanente gatiado por 3 condiciones simultáneas (Etapa 13): dinero, reputación mínima, y maestría (XP acumulado, Plan 4) en el arquetipo que el perk requiere. Equipar/desequipar es gratis (Etapa 11-D).
- Loadout fijo en 2 slots (Etapa 19 MVP) — la escalera de hasta 7 slots por rango (Etapa 13 completa) queda fuera de alcance.
- `economia::calcular` cambia de `multiplicador_perks: f64` a `multiplicador_dinero: f64, multiplicador_reputacion: f64` — cada uno 1.0 si no hay perk equipado que lo afecte.
- El stub viejo (`perk_desbloqueado: bool`, comando `unlock_perk`, struct `PerkStatus`) se retira por completo.
- Sin combos entre perks (Etapa 13, contenido de producción futuro) ni escalera de slots por rango.
- Sin recalibración del umbral de ascenso de rango (`UMBRAL_ASCENSO_AUXILIAR`, Plan 4) — los umbrales de reputación de los perks de este plan se calibran de forma independiente, a una escala alcanzable en pocos tickets.

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

---

## File Structure

- Create: `app/src-tauri/src/perks/mod.rs` — `Perk`, `Categoria`, `Efecto`, catálogo estático de 8, `catalogo()`, `buscar(id)`
- Modify: `app/src-tauri/src/economia/mod.rs` — `calcular()` recibe 2 multiplicadores; `EstadoJugador` gana `perks_desbloqueados`/`perks_equipados` (pierde `perk_desbloqueado`) + `puede_desbloquear`/`desbloquear_perk`/`equipar_perk`/`desequipar_perk`/`multiplicador_dinero`/`multiplicador_reputacion`
- Modify: `app/src-tauri/src/lib.rs` — agrega `mod perks;`, 4 comandos nuevos, retira `unlock_perk`/`PerkStatus`, rewira `submit_ticket`
- Modify: `app/src/index.html` y `app/src/main.js` — ajuste mínimo de compatibilidad: un `<select>` + 2 botones en vez del botón único

---

### Task 1: Catálogo de perks

**Files:**
- Create: `app/src-tauri/src/perks/mod.rs`
- Modify: `app/src-tauri/src/lib.rs` (agrega `mod perks;`)

**Interfaces:**
- Produces: `perks::Perk` (struct pública, `Serialize + Clone + Copy + PartialEq`), `perks::{Categoria, Efecto}` (enums públicos), `perks::catalogo() -> &'static [Perk]`, `perks::buscar(id: &str) -> Option<&'static Perk>`

- [ ] **Step 1: Escribir `app/src-tauri/src/perks/mod.rs`**

```rust
use crate::tickets::Arquetipo;

/// Un perk del sistema RPG (Etapa 13): nombre y descripción siempre en
/// lenguaje de jugador, nunca en jerga técnica SQL — la maestría por
/// arquetipo que lo desbloquea por debajo es invisible para el jugador.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub struct Perk {
    pub id: &'static str,
    pub nombre: &'static str,
    pub categoria: Categoria,
    pub descripcion: &'static str,
    pub costo_dinero: i64,
    pub reputacion_minima: f64,
    pub arquetipo_requerido: Arquetipo,
    pub xp_minimo: i64,
    pub efecto: Efecto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum Categoria {
    Detective,
    ManosRapidas,
    BilleteraYFama,
    Ritmo,
}

/// El efecto mecánico real de un perk. Solo Billetera y Fama tiene efecto
/// hoy (Etapa 12/13) — Detective/Manos Rápidas/Ritmo dependen de sistemas
/// que no existen todavía (consola SQL real con ERD/autocompletado, sistema
/// de turnos) y usan `SinEfectoMecanico`: se pueden desbloquear/equipar de
/// verdad, pero no hacen nada todavía.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub enum Efecto {
    BonoDinero(f64),
    BonoReputacion(f64),
    SinEfectoMecanico,
}

const CATALOGO: [Perk; 8] = [
    Perk {
        id: "instinto",
        nombre: "Instinto",
        categoria: Categoria::Detective,
        descripcion: "Al abrir un ticket, resalta automáticamente qué tablas probablemente necesitas.",
        costo_dinero: 200,
        reputacion_minima: 3.0,
        arquetipo_requerido: Arquetipo::Select,
        xp_minimo: 20,
        efecto: Efecto::SinEfectoMecanico,
    },
    Perk {
        id: "rayos_x",
        nombre: "Rayos X",
        categoria: Categoria::Detective,
        descripcion: "Ves las relaciones entre tablas al instante en el visor de esquema.",
        costo_dinero: 300,
        reputacion_minima: 5.0,
        arquetipo_requerido: Arquetipo::Join,
        xp_minimo: 40,
        efecto: Efecto::SinEfectoMecanico,
    },
    Perk {
        id: "piloto_automatico",
        nombre: "Piloto Automático",
        categoria: Categoria::ManosRapidas,
        descripcion: "Autocompletado más inteligente en el editor.",
        costo_dinero: 250,
        reputacion_minima: 4.0,
        arquetipo_requerido: Arquetipo::Select,
        xp_minimo: 30,
        efecto: Efecto::SinEfectoMecanico,
    },
    Perk {
        id: "red_de_seguridad",
        nombre: "Red de Seguridad",
        categoria: Categoria::ManosRapidas,
        descripcion: "Deshacer ilimitado + aviso antes de ejecutar algo probablemente erróneo.",
        costo_dinero: 350,
        reputacion_minima: 6.0,
        arquetipo_requerido: Arquetipo::Agregacion,
        xp_minimo: 50,
        efecto: Efecto::SinEfectoMecanico,
    },
    Perk {
        id: "buena_fama",
        nombre: "Buena Fama",
        categoria: Categoria::BilleteraYFama,
        descripcion: "Reputación extra por cada ticket entregado.",
        costo_dinero: 300,
        reputacion_minima: 5.0,
        arquetipo_requerido: Arquetipo::Join,
        xp_minimo: 40,
        efecto: Efecto::BonoReputacion(0.2),
    },
    Perk {
        id: "bono_bajo_la_mesa",
        nombre: "Bono Bajo la Mesa",
        categoria: Categoria::BilleteraYFama,
        descripcion: "Dinero extra por cada ticket entregado.",
        costo_dinero: 350,
        reputacion_minima: 6.0,
        arquetipo_requerido: Arquetipo::Agregacion,
        xp_minimo: 50,
        efecto: Efecto::BonoDinero(0.2),
    },
    Perk {
        id: "cafe_cargado",
        nombre: "Café Cargado",
        categoria: Categoria::Ritmo,
        descripcion: "Reduce el costo de tiempo de los tickets.",
        costo_dinero: 200,
        reputacion_minima: 3.0,
        arquetipo_requerido: Arquetipo::Select,
        xp_minimo: 20,
        efecto: Efecto::SinEfectoMecanico,
    },
    Perk {
        id: "modo_turbo",
        nombre: "Modo Turbo",
        categoria: Categoria::Ritmo,
        descripcion: "Más presupuesto de tiempo por turno.",
        costo_dinero: 400,
        reputacion_minima: 7.0,
        arquetipo_requerido: Arquetipo::Join,
        xp_minimo: 60,
        efecto: Efecto::SinEfectoMecanico,
    },
];

/// Catálogo completo de perks (Etapa 13) — 8 perks, 2 por categoría.
pub fn catalogo() -> &'static [Perk] {
    &CATALOGO
}

/// Busca un perk por id en el catálogo.
pub fn buscar(id: &str) -> Option<&'static Perk> {
    CATALOGO.iter().find(|p| p.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogo_tiene_8_perks() {
        assert_eq!(catalogo().len(), 8);
    }

    #[test]
    fn catalogo_tiene_2_perks_por_categoria() {
        for categoria in [Categoria::Detective, Categoria::ManosRapidas, Categoria::BilleteraYFama, Categoria::Ritmo] {
            let cantidad = catalogo().iter().filter(|p| p.categoria == categoria).count();
            assert_eq!(cantidad, 2, "{categoria:?} debe tener exactamente 2 perks");
        }
    }

    #[test]
    fn buscar_encuentra_un_perk_por_id() {
        let perk = buscar("buena_fama").expect("buena_fama debe existir");
        assert_eq!(perk.nombre, "Buena Fama");
        assert_eq!(perk.efecto, Efecto::BonoReputacion(0.2));
    }

    #[test]
    fn buscar_devuelve_none_para_id_invalido() {
        assert!(buscar("perk_que_no_existe").is_none());
    }

    #[test]
    fn solo_billetera_y_fama_tiene_efecto_mecanico_real() {
        for perk in catalogo() {
            let tiene_efecto_real = !matches!(perk.efecto, Efecto::SinEfectoMecanico);
            assert_eq!(
                tiene_efecto_real,
                perk.categoria == Categoria::BilleteraYFama,
                "'{}' (categoría {:?}) no debe tener un efecto real fuera de Billetera y Fama",
                perk.nombre,
                perk.categoria
            );
        }
    }
}
```

- [ ] **Step 2: Registrar el módulo en `app/src-tauri/src/lib.rs`**

Localizar:

```rust
mod db;
mod economia;
mod tickets;
mod validation;
```

Reemplazar por (orden alfabético):

```rust
mod db;
mod economia;
mod perks;
mod tickets;
mod validation;
```

- [ ] **Step 3: Correr la suite completa**

Run: `cd app/src-tauri && cargo test --lib -- --nocapture`
Expected: todos los tests en verde, 0 failed, más los 5 nuevos de `perks::tests`.

- [ ] **Step 4: Commit**

```bash
git add app/src-tauri/src/perks/mod.rs app/src-tauri/src/lib.rs
git commit -m "Add the 8-perk catalog (Stage 13)"
```

---

### Task 2: Desbloqueo, loadout, y multiplicadores separados en `economia`

**Files:**
- Modify: `app/src-tauri/src/economia/mod.rs` (reemplazo completo del archivo)

**Interfaces:**
- Consumes: `crate::perks::{Efecto, Perk}` (Tarea 1)
- Produces: `economia::calcular(evaluacion, ticket, multiplicador_dinero: f64, multiplicador_reputacion: f64) -> Resultado` (firma cambiada); `EstadoJugador` gana `perks_desbloqueados: Vec<&'static str>`, `perks_equipados: Vec<&'static str>` (pierde `perk_desbloqueado: bool`), y los métodos `puede_desbloquear`, `desbloquear_perk`, `equipar_perk`, `desequipar_perk`, `multiplicador_dinero`, `multiplicador_reputacion`

- [ ] **Step 1: Reemplazar `app/src-tauri/src/economia/mod.rs` completo**

```rust
use crate::perks::{Efecto, Perk};
use crate::tickets::{Arquetipo, Ticket};
use crate::validation::Evaluacion;

/// Puntos de XP que otorga usar cada arquetipo SQL una vez, antes de escalar
/// por el puntaje final (Etapa 10: la dificultad del concepto define el XP
/// base — Join vale más que Select, Agregación más que Join).
fn xp_base_por_arquetipo(arquetipo: Arquetipo) -> i64 {
    match arquetipo {
        Arquetipo::Select => 10,
        Arquetipo::Join => 20,
        Arquetipo::Agregacion => 25,
    }
}

/// Resultado de aplicar la fórmula de economía (Etapa 12) a una entrega ya
/// evaluada (Plan 2) contra un ticket (Plan 3).
#[derive(Debug, Clone, PartialEq)]
pub struct Resultado {
    pub puntaje_base: f64,
    pub puntaje_final: f64,
    pub dinero_ganado: i64,
    pub reputacion_ganada: f64,
    pub xp_ganado: Vec<(Arquetipo, i64)>,
}

/// Calcula dinero/reputación/XP ganados por una entrega, siguiendo la
/// fórmula literal de la Etapa 12. `multiplicador_dinero`/`multiplicador_reputacion`
/// representan el efecto de los perks de la categoría Billetera y Fama
/// actualmente equipados (Etapa 13, Plan 5) — cada uno 1.0 si no hay ninguno
/// activo. A diferencia del multiplicador único que existía antes de la
/// Etapa 13, cada recurso escala por separado porque los perks reales
/// afectan solo uno de los dos ("Buena Fama" solo reputación, "Bono Bajo la
/// Mesa" solo dinero) — un multiplicador compartido los volvería
/// indistinguibles. Si la entrega es incorrecta, no se otorga
/// dinero/reputación/XP (la penalización por tickets escalados es solo de
/// reputación y depende del sistema de turnos, Etapa 11-A, no construido —
/// este cálculo no la implementa).
pub fn calcular(
    evaluacion: &Evaluacion,
    ticket: &Ticket,
    multiplicador_dinero: f64,
    multiplicador_reputacion: f64,
) -> Resultado {
    let puntaje_base = evaluacion.puntaje_correctitud * ticket.peso_correctitud
        + evaluacion.puntaje_velocidad * ticket.peso_velocidad
        + evaluacion.puntaje_practicas * ticket.peso_practicas;
    // Ya no hay un multiplicador genérico compartido (Etapa 13, Plan 5): los
    // perks con efecto real escalan dinero/reputación por separado, más
    // abajo — puntaje_final se mantiene igual a puntaje_base.
    let puntaje_final = puntaje_base;

    if !evaluacion.correcta {
        return Resultado {
            puntaje_base,
            puntaje_final,
            dinero_ganado: 0,
            reputacion_ganada: 0.0,
            xp_ganado: Vec::new(),
        };
    }

    let dinero_ganado =
        (puntaje_final * ticket.valor_base as f64 / 100.0 * multiplicador_dinero).round() as i64;
    let reputacion_ganada =
        puntaje_final * ticket.factor_reputacion / 100.0 * multiplicador_reputacion;
    let xp_ganado = ticket
        .arquetipos
        .iter()
        .map(|&arquetipo| {
            let xp = (xp_base_por_arquetipo(arquetipo) as f64 * puntaje_final / 100.0).round() as i64;
            (arquetipo, xp)
        })
        .collect();

    Resultado {
        puntaje_base,
        puntaje_final,
        dinero_ganado,
        reputacion_ganada,
        xp_ganado,
    }
}

/// Máximo de perks equipados simultáneamente (Etapa 19, MVP): 2 slots fijos.
/// La escalera de hasta 7 slots por hito de rango (Etapa 13 completa) es de
/// un plan posterior.
const MAX_SLOTS_EQUIPADOS: usize = 2;

/// Estado acumulado del jugador (Etapa 12/13): dinero, reputación, XP por
/// arquetipo, y los perks desbloqueados/equipados.
#[derive(Debug, Clone, Default)]
pub struct EstadoJugador {
    pub dinero: i64,
    pub reputacion: f64,
    pub xp_por_arquetipo: Vec<(Arquetipo, i64)>,
    pub perks_desbloqueados: Vec<&'static str>,
    pub perks_equipados: Vec<&'static str>,
}

/// Umbral de reputación para ascender de Becario a Auxiliar de Sistemas en
/// Hospital Arcángel (Etapa 10). El ascenso real (superar el mini-boss,
/// cambiar de rango) es responsabilidad de un plan posterior — esta
/// constante solo define cuándo se cumple la condición de reputación.
const UMBRAL_ASCENSO_AUXILIAR: f64 = 500.0;

impl EstadoJugador {
    /// Aplica el resultado de una entrega (Etapa 12): acumula dinero,
    /// reputación y XP por arquetipo sobre el estado existente.
    pub fn aplicar_resultado(&mut self, resultado: &Resultado) {
        self.dinero += resultado.dinero_ganado;
        self.reputacion += resultado.reputacion_ganada;
        for &(arquetipo, xp) in &resultado.xp_ganado {
            match self.xp_por_arquetipo.iter_mut().find(|(a, _)| *a == arquetipo) {
                Some((_, existente)) => *existente += xp,
                None => self.xp_por_arquetipo.push((arquetipo, xp)),
            }
        }
    }

    /// Etapa 10: señal de que la reputación ya cruzó el umbral de ascenso —
    /// no dispara ningún cambio de estado por sí sola.
    pub fn puede_ascender(&self) -> bool {
        self.reputacion >= UMBRAL_ASCENSO_AUXILIAR
    }

    /// Etapa 13: un perk se desbloquea cuando se cumplen 3 condiciones a la
    /// vez — dinero suficiente, reputación mínima, y maestría (XP) suficiente
    /// en el arquetipo que ese perk requiere.
    pub fn puede_desbloquear(&self, perk: &Perk) -> bool {
        let xp_en_arquetipo = self
            .xp_por_arquetipo
            .iter()
            .find(|&&(a, _)| a == perk.arquetipo_requerido)
            .map(|&(_, xp)| xp)
            .unwrap_or(0);

        self.dinero >= perk.costo_dinero
            && self.reputacion >= perk.reputacion_minima
            && xp_en_arquetipo >= perk.xp_minimo
    }

    /// Desbloquea un perk permanentemente (Etapa 13): gasta el dinero, no
    /// toca la reputación ni el XP (esos solo se verifican, nunca se
    /// consumen). Idempotente si ya estaba desbloqueado.
    pub fn desbloquear_perk(&mut self, catalogo: &[Perk], id: &str) -> Result<(), String> {
        if self.perks_desbloqueados.contains(&id) {
            return Ok(());
        }
        let perk = catalogo
            .iter()
            .find(|p| p.id == id)
            .ok_or_else(|| format!("perk desconocido: {id}"))?;
        if !self.puede_desbloquear(perk) {
            return Err(format!("No cumples los requisitos para '{}' todavía.", perk.nombre));
        }
        self.dinero -= perk.costo_dinero;
        self.perks_desbloqueados.push(perk.id);
        Ok(())
    }

    /// Equipa un perk ya desbloqueado (Etapa 11-D: equipar es gratis).
    /// Falla si no está desbloqueado, o si ya se ocuparon los 2 slots.
    /// Idempotente si ya estaba equipado.
    pub fn equipar_perk(&mut self, id: &str) -> Result<(), String> {
        if !self.perks_desbloqueados.contains(&id) {
            return Err(format!("'{id}' no está desbloqueado todavía."));
        }
        if self.perks_equipados.contains(&id) {
            return Ok(());
        }
        if self.perks_equipados.len() >= MAX_SLOTS_EQUIPADOS {
            return Err(format!(
                "Ya tienes {MAX_SLOTS_EQUIPADOS} perks equipados — desequipa uno primero."
            ));
        }
        let id_estatico = self
            .perks_desbloqueados
            .iter()
            .find(|&&d| d == id)
            .copied()
            .expect("ya se confirmó arriba que está desbloqueado");
        self.perks_equipados.push(id_estatico);
        Ok(())
    }

    /// Desequipa un perk (gratis, Etapa 11-D). No falla si no estaba
    /// equipado.
    pub fn desequipar_perk(&mut self, id: &str) {
        self.perks_equipados.retain(|&equipado| equipado != id);
    }

    /// Multiplicador de dinero (Etapa 12/13) por los perks "Billetera y Fama"
    /// actualmente equipados — 1.0 si ninguno está activo.
    pub fn multiplicador_dinero(&self, catalogo: &[Perk]) -> f64 {
        let mut multiplicador = 1.0;
        for &id in &self.perks_equipados {
            if let Some(perk) = catalogo.iter().find(|p| p.id == id) {
                if let Efecto::BonoDinero(bono) = perk.efecto {
                    multiplicador *= 1.0 + bono;
                }
            }
        }
        multiplicador
    }

    /// Multiplicador de reputación (Etapa 12/13) por los perks "Billetera y
    /// Fama" actualmente equipados — 1.0 si ninguno está activo.
    pub fn multiplicador_reputacion(&self, catalogo: &[Perk]) -> f64 {
        let mut multiplicador = 1.0;
        for &id in &self.perks_equipados {
            if let Some(perk) = catalogo.iter().find(|p| p.id == id) {
                if let Efecto::BonoReputacion(bono) = perk.efecto {
                    multiplicador *= 1.0 + bono;
                }
            }
        }
        multiplicador
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::perks;
    use crate::tickets::{Prioridad, TipoTicket};

    fn ticket_de_prueba(arquetipos: Vec<Arquetipo>) -> Ticket {
        Ticket {
            id: "ticket_de_prueba",
            tipo: TipoTicket::ReporteAnalisis,
            solicitante: "Alguien",
            motivo: "un motivo".to_string(),
            solicitud: "una solicitud".to_string(),
            prioridad: Prioridad::Media,
            costo_tiempo: 10,
            arquetipos,
            sql_dorada: "SELECT 1".to_string(),
            sql_inicial: None,
            requiere_orden: true,
            peso_correctitud: 0.6,
            peso_velocidad: 0.2,
            peso_practicas: 0.2,
            valor_base: 100,
            factor_reputacion: 0.5,
        }
    }

    fn evaluacion_perfecta() -> Evaluacion {
        Evaluacion {
            correcta: true,
            puntaje_correctitud: 100.0,
            puntaje_velocidad: 100.0,
            puntaje_practicas: 100.0,
            comentario_mentor: None,
        }
    }

    #[test]
    fn calcular_ticket_correcto_otorga_recompensa_proporcional() {
        let ticket = ticket_de_prueba(vec![Arquetipo::Select]);
        let evaluacion = evaluacion_perfecta();

        let resultado = calcular(&evaluacion, &ticket, 1.0, 1.0);

        assert_eq!(resultado.puntaje_base, 100.0);
        assert_eq!(resultado.puntaje_final, 100.0);
        assert_eq!(resultado.dinero_ganado, 100);
        assert_eq!(resultado.reputacion_ganada, 0.5);
        assert_eq!(resultado.xp_ganado, vec![(Arquetipo::Select, 10)]);
    }

    #[test]
    fn calcular_ticket_incorrecto_no_otorga_dinero_ni_reputacion_ni_xp() {
        let ticket = ticket_de_prueba(vec![Arquetipo::Select]);
        let mut evaluacion = evaluacion_perfecta();
        evaluacion.correcta = false;

        let resultado = calcular(&evaluacion, &ticket, 1.0, 1.0);

        assert_eq!(resultado.dinero_ganado, 0);
        assert_eq!(resultado.reputacion_ganada, 0.0);
        assert!(resultado.xp_ganado.is_empty());
        assert_eq!(
            resultado.puntaje_base, 100.0,
            "el puntaje de calidad se calcula aunque el resultado sea incorrecto"
        );
    }

    #[test]
    fn calcular_reparte_xp_entre_varios_arquetipos() {
        let mut ticket = ticket_de_prueba(vec![Arquetipo::Join, Arquetipo::Agregacion]);
        ticket.peso_correctitud = 0.4;
        ticket.peso_velocidad = 0.3;
        ticket.peso_practicas = 0.3;
        let evaluacion = evaluacion_perfecta();

        let resultado = calcular(&evaluacion, &ticket, 1.0, 1.0);

        assert_eq!(
            resultado.xp_ganado,
            vec![(Arquetipo::Join, 20), (Arquetipo::Agregacion, 25)]
        );
    }

    #[test]
    fn calcular_aplica_los_multiplicadores_de_dinero_y_reputacion_por_separado() {
        let ticket = ticket_de_prueba(vec![Arquetipo::Select]);
        let evaluacion = evaluacion_perfecta();

        let resultado = calcular(&evaluacion, &ticket, 2.0, 1.5);

        assert_eq!(
            resultado.puntaje_final, 100.0,
            "puntaje_final ya no lleva un multiplicador genérico"
        );
        assert_eq!(resultado.dinero_ganado, 200, "100 (valor_base) * 2.0");
        assert_eq!(resultado.reputacion_ganada, 0.75, "0.5 (factor_reputacion) * 1.5");
    }

    #[test]
    fn aplicar_resultado_acumula_dinero_reputacion_y_xp() {
        let mut estado = EstadoJugador::default();
        let resultado = Resultado {
            puntaje_base: 100.0,
            puntaje_final: 100.0,
            dinero_ganado: 100,
            reputacion_ganada: 0.5,
            xp_ganado: vec![(Arquetipo::Select, 10)],
        };

        estado.aplicar_resultado(&resultado);

        assert_eq!(estado.dinero, 100);
        assert_eq!(estado.reputacion, 0.5);
        assert_eq!(estado.xp_por_arquetipo, vec![(Arquetipo::Select, 10)]);
    }

    #[test]
    fn aplicar_resultado_suma_xp_al_mismo_arquetipo_en_llamadas_sucesivas() {
        let mut estado = EstadoJugador::default();
        let resultado = Resultado {
            puntaje_base: 100.0,
            puntaje_final: 100.0,
            dinero_ganado: 100,
            reputacion_ganada: 0.5,
            xp_ganado: vec![(Arquetipo::Select, 10)],
        };

        estado.aplicar_resultado(&resultado);
        estado.aplicar_resultado(&resultado);

        assert_eq!(estado.dinero, 200);
        assert_eq!(
            estado.xp_por_arquetipo,
            vec![(Arquetipo::Select, 20)],
            "debe acumular en la misma entrada, no duplicarla"
        );
    }

    #[test]
    fn puede_ascender_es_false_bajo_el_umbral_y_true_al_cruzarlo() {
        let mut estado = EstadoJugador::default();
        assert!(!estado.puede_ascender());

        estado.reputacion = 499.9;
        assert!(!estado.puede_ascender());

        estado.reputacion = 500.0;
        assert!(estado.puede_ascender());
    }

    #[test]
    fn calcular_distingue_cual_peso_multiplica_a_cual_puntaje() {
        let mut ticket = ticket_de_prueba(vec![Arquetipo::Select]);
        ticket.peso_correctitud = 0.5;
        ticket.peso_velocidad = 0.3125;
        ticket.peso_practicas = 0.1875;
        let evaluacion = Evaluacion {
            correcta: true,
            puntaje_correctitud: 80.0,
            puntaje_velocidad: 64.0,
            puntaje_practicas: 32.0,
            comentario_mentor: None,
        };

        let resultado = calcular(&evaluacion, &ticket, 1.0, 1.0);

        assert_eq!(resultado.puntaje_base, 66.0);
        assert_eq!(resultado.puntaje_final, 66.0);
    }

    #[test]
    fn calcular_redondea_dinero_y_xp_cuando_el_puntaje_final_es_fraccionario() {
        let mut ticket = ticket_de_prueba(vec![Arquetipo::Select]);
        ticket.peso_correctitud = 0.625;
        ticket.peso_velocidad = 0.25;
        ticket.peso_practicas = 0.125;
        let evaluacion = Evaluacion {
            correcta: true,
            puntaje_correctitud: 70.0,
            puntaje_velocidad: 51.0,
            puntaje_practicas: 11.0,
            comentario_mentor: None,
        };

        let resultado = calcular(&evaluacion, &ticket, 1.0, 1.0);

        assert_eq!(resultado.puntaje_base, 57.875);
        assert_eq!(resultado.puntaje_final, 57.875);
        assert_eq!(resultado.dinero_ganado, 58);
        assert_eq!(resultado.xp_ganado, vec![(Arquetipo::Select, 6)]);
    }

    #[test]
    fn aplicar_resultado_agrega_arquetipo_nuevo_sin_afectar_los_existentes() {
        let mut estado = EstadoJugador::default();
        let resultado_select = Resultado {
            puntaje_base: 100.0,
            puntaje_final: 100.0,
            dinero_ganado: 100,
            reputacion_ganada: 0.5,
            xp_ganado: vec![(Arquetipo::Select, 10)],
        };
        let resultado_join = Resultado {
            puntaje_base: 100.0,
            puntaje_final: 100.0,
            dinero_ganado: 50,
            reputacion_ganada: 0.3,
            xp_ganado: vec![(Arquetipo::Join, 20)],
        };

        estado.aplicar_resultado(&resultado_select);
        estado.aplicar_resultado(&resultado_join);

        assert_eq!(
            estado.xp_por_arquetipo,
            vec![(Arquetipo::Select, 10), (Arquetipo::Join, 20)],
            "la entrada existente de Select no debe alterarse y Join debe agregarse como nueva entrada"
        );
    }

    #[test]
    fn puede_desbloquear_requiere_dinero_reputacion_y_xp_simultaneamente() {
        let perk = perks::buscar("buena_fama").expect("buena_fama debe existir en el catálogo");

        let mut estado = EstadoJugador::default();
        assert!(!estado.puede_desbloquear(perk), "sin nada, no debe poder desbloquear");

        estado.dinero = perk.costo_dinero;
        assert!(!estado.puede_desbloquear(perk), "dinero solo no basta");

        estado.reputacion = perk.reputacion_minima;
        assert!(!estado.puede_desbloquear(perk), "falta el XP del arquetipo requerido");

        estado.xp_por_arquetipo.push((perk.arquetipo_requerido, perk.xp_minimo));
        assert!(estado.puede_desbloquear(perk), "con las 3 condiciones cumplidas ya debe poder");
    }

    #[test]
    fn desbloquear_perk_gasta_dinero_y_es_idempotente() {
        let catalogo = perks::catalogo();
        let perk = perks::buscar("buena_fama").unwrap();

        let mut estado = EstadoJugador::default();
        estado.dinero = perk.costo_dinero;
        estado.reputacion = perk.reputacion_minima;
        estado.xp_por_arquetipo.push((perk.arquetipo_requerido, perk.xp_minimo));

        estado.desbloquear_perk(catalogo, "buena_fama").expect("debe poder desbloquear");
        assert_eq!(estado.dinero, 0);
        assert!(estado.perks_desbloqueados.contains(&"buena_fama"));

        estado.dinero = 1000;
        estado.desbloquear_perk(catalogo, "buena_fama").expect("ya desbloqueado, no debe fallar");
        assert_eq!(estado.dinero, 1000, "no debe cobrar de nuevo un perk ya desbloqueado");
    }

    #[test]
    fn desbloquear_perk_falla_si_no_cumple_los_requisitos() {
        let catalogo = perks::catalogo();
        let mut estado = EstadoJugador::default();
        assert!(estado.desbloquear_perk(catalogo, "buena_fama").is_err());
    }

    #[test]
    fn equipar_perk_respeta_el_limite_de_2_slots() {
        let mut estado = EstadoJugador::default();
        estado.perks_desbloqueados = vec!["instinto", "rayos_x", "piloto_automatico"];

        estado.equipar_perk("instinto").unwrap();
        estado.equipar_perk("rayos_x").unwrap();
        let resultado = estado.equipar_perk("piloto_automatico");

        assert!(resultado.is_err(), "un tercer perk no debe caber en 2 slots");
        assert_eq!(estado.perks_equipados, vec!["instinto", "rayos_x"]);
    }

    #[test]
    fn equipar_perk_falla_si_no_esta_desbloqueado() {
        let mut estado = EstadoJugador::default();
        assert!(estado.equipar_perk("instinto").is_err());
    }

    #[test]
    fn desequipar_perk_libera_un_slot() {
        let mut estado = EstadoJugador::default();
        estado.perks_desbloqueados = vec!["instinto", "rayos_x"];
        estado.equipar_perk("instinto").unwrap();
        estado.equipar_perk("rayos_x").unwrap();

        estado.desequipar_perk("instinto");
        assert_eq!(estado.perks_equipados, vec!["rayos_x"]);

        estado.perks_desbloqueados.push("piloto_automatico");
        estado.equipar_perk("piloto_automatico").unwrap();
        assert_eq!(estado.perks_equipados, vec!["rayos_x", "piloto_automatico"]);
    }

    #[test]
    fn multiplicador_dinero_solo_cuenta_perks_equipados_no_solo_desbloqueados() {
        let catalogo = perks::catalogo();
        let mut estado = EstadoJugador::default();
        estado.perks_desbloqueados.push("bono_bajo_la_mesa");

        assert_eq!(
            estado.multiplicador_dinero(catalogo),
            1.0,
            "desbloqueado pero no equipado no debe aplicar"
        );

        estado.equipar_perk("bono_bajo_la_mesa").unwrap();
        assert_eq!(estado.multiplicador_dinero(catalogo), 1.2, "equipado, +20%");
    }

    #[test]
    fn multiplicador_reputacion_solo_cuenta_perks_equipados_no_solo_desbloqueados() {
        let catalogo = perks::catalogo();
        let mut estado = EstadoJugador::default();
        estado.perks_desbloqueados.push("buena_fama");

        assert_eq!(estado.multiplicador_reputacion(catalogo), 1.0);

        estado.equipar_perk("buena_fama").unwrap();
        assert_eq!(estado.multiplicador_reputacion(catalogo), 1.2);
    }
}
```

- [ ] **Step 2: Correr la suite completa**

Run: `cd app/src-tauri && cargo test --lib -- --nocapture`
Expected: todos los tests en verde, 0 failed — este archivo pasa de 10 a 18 tests en `economia::tests` (los 9 anteriores sin cambios + 1 renombrado/adaptado a los 2 multiplicadores + 8 nuevos de desbloqueo/equipo/multiplicadores). **Nota:** el crate completo no compilará todavía (`lib.rs` sigue llamando a `economia::calcular` con la firma vieja de 1 argumento) — eso es esperado hasta la Tarea 3; para verificar solo este módulo usa `cargo test --lib economia::` o revisa que el único error de compilación restante sea en `lib.rs`, no en `economia/mod.rs`.

- [ ] **Step 3: Commit**

```bash
git add app/src-tauri/src/economia/mod.rs
git commit -m "Add perk unlock/equip logic and split the reward multiplier (Stage 13)"
```

---

### Task 3: Conectar los perks a la app

**Files:**
- Modify: `app/src-tauri/src/lib.rs`
- Modify: `app/src/index.html`
- Modify: `app/src/main.js`

**Interfaces:**
- Consumes: `perks::{catalogo, Categoria}` (Tarea 1), `economia::EstadoJugador`'s nuevos métodos (Tarea 2)
- Produces: comandos Tauri `catalogo_perks`, `desbloquear_perk(id)`, `equipar_perk(id)`, `desequipar_perk(id)` (nuevos); `unlock_perk`/`PerkStatus` retirados; `submit_ticket` deriva los 2 multiplicadores de los perks equipados

- [ ] **Step 1: Leer el estado actual de los 3 archivos y confirmar que coinciden con lo descrito abajo antes de editar**

- [ ] **Step 2: Reemplazar `app/src-tauri/src/lib.rs` completo**

```rust
mod db;
mod economia;
mod perks;
mod tickets;
mod validation;

use std::sync::Mutex;
use tauri::Manager;

/// Pool de conexión al Postgres embebido, gestionado por Tauri.
struct AppState {
    pool: sqlx::PgPool,
}

/// Estado de economía del jugador (Etapa 12/13), gestionado por Tauri.
struct Jugador(Mutex<economia::EstadoJugador>);

/// Mantiene vivo el servidor Postgres embebido y permite detenerlo al salir.
struct EmbeddedPostgres(Mutex<Option<postgresql_embedded::PostgreSQL>>);

/// Catálogo de tickets de la empresa activa + índice del ticket actual
/// (Etapa 14). Selección round-robin simple — sin bandeja de entrada ni
/// tiempo de turno todavía (Etapa 11-A, plan de UI/loop posterior).
struct Tickets {
    catalogo: Vec<tickets::Ticket>,
    indice_actual: Mutex<usize>,
}

#[derive(serde::Serialize)]
struct ScoreResult {
    pass: bool,
    puntaje_correctitud: f64,
    puntaje_velocidad: f64,
    puntaje_practicas: f64,
    puntaje_base: f64,
    puntaje_final: f64,
    comentario_mentor: Option<String>,
    dinero_ganado: i64,
    dinero_total: i64,
    reputacion_ganada: f64,
    reputacion_total: f64,
    xp_ganado: Vec<(tickets::Arquetipo, i64)>,
    puede_ascender: bool,
    mensaje: String,
}

/// Vista combinada de un perk (Etapa 13): datos estáticos del catálogo +
/// estado dinámico del jugador frente a él.
#[derive(serde::Serialize)]
struct PerkConEstado {
    id: &'static str,
    nombre: &'static str,
    categoria: perks::Categoria,
    descripcion: &'static str,
    costo_dinero: i64,
    reputacion_minima: f64,
    desbloqueado: bool,
    equipado: bool,
}

fn vista_perks(estado: &economia::EstadoJugador) -> Vec<PerkConEstado> {
    perks::catalogo()
        .iter()
        .map(|perk| PerkConEstado {
            id: perk.id,
            nombre: perk.nombre,
            categoria: perk.categoria,
            descripcion: perk.descripcion,
            costo_dinero: perk.costo_dinero,
            reputacion_minima: perk.reputacion_minima,
            desbloqueado: estado.perks_desbloqueados.contains(&perk.id),
            equipado: estado.perks_equipados.contains(&perk.id),
        })
        .collect()
}

#[tauri::command]
fn ticket_actual(tickets: tauri::State<'_, Tickets>) -> tickets::Ticket {
    let indice = *tickets.indice_actual.lock().unwrap();
    tickets.catalogo[indice].clone()
}

#[tauri::command]
async fn run_query(state: tauri::State<'_, AppState>, sql: String) -> Result<db::QueryResult, String> {
    db::run_query(&state.pool, &sql).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn submit_ticket(
    state: tauri::State<'_, AppState>,
    jugador: tauri::State<'_, Jugador>,
    tickets: tauri::State<'_, Tickets>,
    sql: String,
) -> Result<ScoreResult, String> {
    let indice = *tickets.indice_actual.lock().unwrap();
    let ticket = tickets.catalogo[indice].clone();

    let evaluacion = validation::evaluar_entrega(&state.pool, &sql, &ticket.sql_dorada, ticket.requiere_orden)
        .await
        .map_err(|e| e.to_string())?;

    let mut estado = jugador.0.lock().unwrap();
    let multiplicador_dinero = estado.multiplicador_dinero(perks::catalogo());
    let multiplicador_reputacion = estado.multiplicador_reputacion(perks::catalogo());
    let resultado = economia::calcular(&evaluacion, &ticket, multiplicador_dinero, multiplicador_reputacion);
    estado.aplicar_resultado(&resultado);

    if evaluacion.correcta {
        let mut indice_mut = tickets.indice_actual.lock().unwrap();
        *indice_mut = (*indice_mut + 1) % tickets.catalogo.len();
    }

    Ok(ScoreResult {
        pass: evaluacion.correcta,
        puntaje_correctitud: evaluacion.puntaje_correctitud,
        puntaje_velocidad: evaluacion.puntaje_velocidad,
        puntaje_practicas: evaluacion.puntaje_practicas,
        puntaje_base: resultado.puntaje_base,
        puntaje_final: resultado.puntaje_final,
        comentario_mentor: evaluacion.comentario_mentor.map(str::to_string),
        dinero_ganado: resultado.dinero_ganado,
        dinero_total: estado.dinero,
        reputacion_ganada: resultado.reputacion_ganada,
        reputacion_total: estado.reputacion,
        xp_ganado: resultado.xp_ganado,
        puede_ascender: estado.puede_ascender(),
        mensaje: if evaluacion.correcta {
            "Ticket resuelto. Contabilidad procesará tu pago... eventualmente.".to_string()
        } else {
            "El resultado no coincide con lo que pidió la solicitud. Revisa tu consulta.".to_string()
        },
    })
}

#[tauri::command]
fn catalogo_perks(jugador: tauri::State<'_, Jugador>) -> Vec<PerkConEstado> {
    let estado = jugador.0.lock().unwrap();
    vista_perks(&estado)
}

#[tauri::command]
fn desbloquear_perk(jugador: tauri::State<'_, Jugador>, id: String) -> Result<Vec<PerkConEstado>, String> {
    let mut estado = jugador.0.lock().unwrap();
    estado.desbloquear_perk(perks::catalogo(), &id)?;
    Ok(vista_perks(&estado))
}

#[tauri::command]
fn equipar_perk(jugador: tauri::State<'_, Jugador>, id: String) -> Result<Vec<PerkConEstado>, String> {
    let mut estado = jugador.0.lock().unwrap();
    estado.equipar_perk(&id)?;
    Ok(vista_perks(&estado))
}

#[tauri::command]
fn desequipar_perk(jugador: tauri::State<'_, Jugador>, id: String) -> Vec<PerkConEstado> {
    let mut estado = jugador.0.lock().unwrap();
    estado.desequipar_perk(&id);
    vista_perks(&estado)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            ticket_actual,
            run_query,
            submit_ticket,
            catalogo_perks,
            desbloquear_perk,
            equipar_perk,
            desequipar_perk
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::block_on(async move {
                let pg = db::init_embedded_postgres()
                    .await
                    .expect("no se pudo inicializar Postgres embebido");
                let pool = db::load_company(&pg, db::Company::HospitalArcangel)
                    .await
                    .expect("no se pudo cargar Hospital Arcángel");
                // `pool` y `catalogo` deben cargarse siempre con la misma `Company`:
                // `submit_ticket` valida el SQL del jugador (ejecutado contra `pool`)
                // contra `sql_dorada` del ticket actual de `Tickets`, así que si
                // alguna vez divergen, se validaría contra el esquema de otra empresa.
                let catalogo = tickets::catalogo(db::Company::HospitalArcangel);
                handle.manage(AppState { pool });
                handle.manage(Jugador(Mutex::new(economia::EstadoJugador::default())));
                handle.manage(Tickets { catalogo, indice_actual: Mutex::new(0) });
                handle.manage(EmbeddedPostgres(Mutex::new(Some(pg))));
            });
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| {
        if !matches!(event, tauri::RunEvent::Exit) {
            return;
        }
        let Some(embedded) = app_handle.try_state::<EmbeddedPostgres>() else {
            return;
        };
        let Some(pg) = embedded.0.lock().unwrap().take() else {
            return;
        };
        tauri::async_runtime::block_on(async move {
            let _ = pg.stop().await;
        });
    });
}
```

- [ ] **Step 3: Reemplazar la sección de stats y el botón de perk en `app/src/index.html`**

Localizar:

```html
        <div class="stats">
          <span>💰 <span id="dinero">0</span></span>
          <span id="perk-indicator" class="perk-locked">🔒 Perk: Café Cargado</span>
        </div>
```

Reemplazar por:

```html
        <div class="stats">
          <span>💰 <span id="dinero">0</span></span>
          <span>⭐ <span id="reputacion">0</span></span>
        </div>
```

Localizar:

```html
        <div class="actions">
          <button id="btn-play">▶ Play</button>
          <button id="btn-submit">✓ Enviar ticket</button>
          <button id="btn-unlock-perk">Desbloquear perk (300)</button>
        </div>
      </section>
```

Reemplazar por:

```html
        <div class="actions">
          <button id="btn-play">▶ Play</button>
          <button id="btn-submit">✓ Enviar ticket</button>
        </div>
      </section>

      <section class="perks">
        <h2>Perks</h2>
        <select id="perks-select"></select>
        <div class="actions">
          <button id="btn-unlock-perk">Desbloquear</button>
          <button id="btn-equip-perk">Equipar/Desequipar</button>
        </div>
        <p id="perks-equipados-msg"></p>
      </section>
```

- [ ] **Step 4: Reemplazar `app/src/main.js` completo**

```js
const { invoke } = window.__TAURI__.core;

let sqlInput, statusMsg, resultTable, dineroEl, reputacionEl, ticketEnunciado;
let perksSelect, perksEquipadosMsg;

function renderRows(rows) {
  resultTable.innerHTML = "";
  if (!rows || rows.length === 0) {
    resultTable.textContent = "(sin filas)";
    return;
  }
  const columns = Object.keys(rows[0]);
  const table = document.createElement("table");

  const thead = document.createElement("thead");
  const headRow = document.createElement("tr");
  for (const col of columns) {
    const th = document.createElement("th");
    th.textContent = col;
    headRow.appendChild(th);
  }
  thead.appendChild(headRow);
  table.appendChild(thead);

  const tbody = document.createElement("tbody");
  for (const row of rows) {
    const tr = document.createElement("tr");
    for (const col of columns) {
      const td = document.createElement("td");
      td.textContent = row[col] === null ? "NULL" : String(row[col]);
      tr.appendChild(td);
    }
    tbody.appendChild(tr);
  }
  table.appendChild(tbody);
  resultTable.appendChild(table);
}

function setStatus(text, kind) {
  statusMsg.textContent = text;
  statusMsg.className = kind || "";
}

async function runQuery() {
  setStatus("Ejecutando...", "");
  try {
    const result = await invoke("run_query", { sql: sqlInput.value });
    setStatus(`OK — ${result.rows.length} fila(s)`, "ok");
    renderRows(result.rows);
  } catch (err) {
    setStatus(String(err), "error");
    resultTable.innerHTML = "";
  }
}

async function submitTicket() {
  setStatus("Enviando ticket...", "");
  try {
    const score = await invoke("submit_ticket", { sql: sqlInput.value });
    dineroEl.textContent = score.dinero_total;
    reputacionEl.textContent = score.reputacion_total.toFixed(1);
    setStatus(score.mensaje, score.pass ? "ok" : "error");
  } catch (err) {
    setStatus(String(err), "error");
  }
}

function renderPerks(perks) {
  const seleccionado = perksSelect.value;
  perksSelect.innerHTML = "";
  for (const perk of perks) {
    const opt = document.createElement("option");
    opt.value = perk.id;
    const estado = perk.equipado ? "⭐ equipado" : perk.desbloqueado ? "✅ desbloqueado" : "🔒 bloqueado";
    opt.textContent = `${perk.nombre} (${perk.categoria}) — ${estado} — $${perk.costo_dinero}, ⭐${perk.reputacion_minima}`;
    perksSelect.appendChild(opt);
  }
  if (seleccionado) perksSelect.value = seleccionado;

  const equipados = perks.filter((p) => p.equipado).map((p) => p.nombre);
  perksEquipadosMsg.textContent = equipados.length ? `Equipados: ${equipados.join(", ")}` : "Ningún perk equipado.";
}

async function cargarPerks() {
  const perks = await invoke("catalogo_perks");
  renderPerks(perks);
}

async function desbloquearPerkSeleccionado() {
  const id = perksSelect.value;
  if (!id) return;
  try {
    const perks = await invoke("desbloquear_perk", { id });
    renderPerks(perks);
    setStatus("Perk desbloqueado.", "ok");
  } catch (err) {
    setStatus(String(err), "error");
  }
}

async function equiparODesequiparPerkSeleccionado() {
  const id = perksSelect.value;
  if (!id) return;
  const actual = (await invoke("catalogo_perks")).find((p) => p.id === id);
  try {
    const perks = actual && actual.equipado
      ? await invoke("desequipar_perk", { id })
      : await invoke("equipar_perk", { id });
    renderPerks(perks);
  } catch (err) {
    setStatus(String(err), "error");
  }
}

window.addEventListener("DOMContentLoaded", async () => {
  sqlInput = document.querySelector("#sql-input");
  statusMsg = document.querySelector("#status-msg");
  resultTable = document.querySelector("#result-table");
  dineroEl = document.querySelector("#dinero");
  reputacionEl = document.querySelector("#reputacion");
  ticketEnunciado = document.querySelector("#ticket-enunciado");
  perksSelect = document.querySelector("#perks-select");
  perksEquipadosMsg = document.querySelector("#perks-equipados-msg");

  const ticket = await invoke("ticket_actual");
  ticketEnunciado.textContent = `Motivo: ${ticket.motivo}\nSolicitud: ${ticket.solicitud}`;
  sqlInput.value = ticket.sql_inicial || "SELECT * FROM pacientes;";

  await cargarPerks();

  document.querySelector("#btn-play").addEventListener("click", runQuery);
  document.querySelector("#btn-submit").addEventListener("click", submitTicket);
  document.querySelector("#btn-unlock-perk").addEventListener("click", desbloquearPerkSeleccionado);
  document.querySelector("#btn-equip-perk").addEventListener("click", equiparODesequiparPerkSeleccionado);
});
```

- [ ] **Step 5: Verificar que compila sin advertencias**

Run: `cd app/src-tauri && cargo check`
Expected: `Finished` sin errores ni warnings.

- [ ] **Step 6: Correr la suite completa**

Run: `cd app/src-tauri && cargo test --lib -- --nocapture`
Expected: todos los tests en verde, 0 failed (sin tests nuevos en este paso — es wiring de `lib.rs`/frontend).

- [ ] **Step 7: Smoke test de la app real**

Antes de lanzar nada, correr `ps aux | grep -iE "tauri dev|target/debug/app"` — si ya hay una instancia corriendo (p. ej. una sesión interactiva del usuario), **no** lanzar una segunda ni tocarla; basta con `cargo check`/`cargo test` como verificación en ese caso. Si no hay ninguna corriendo, lanzar `cd app && npm run tauri dev` en segundo plano, esperar a que compile y arranque sin pánico ni warnings, confirmar con `ps aux` que el proceso vive, y verificar que el `<select>` de perks se puede leer (aunque sea por el log, ya que no hay herramienta de captura visual). Detener limpiamente el proceso (`npm`/`tauri dev`/`target/debug/app`, y cualquier Postgres embebido que la app haya arrancado) antes de terminar.

- [ ] **Step 8: Commit**

```bash
git add app/src-tauri/src/lib.rs app/src/index.html app/src/main.js
git commit -m "Wire the perk catalog into the app (unlock/equip/desequip)"
```

---

## Self-Review Notes

- **Cobertura del spec:** catálogo de 8 perks (2 por categoría) ✓, desbloqueo gatiado por dinero+reputación+maestría ✓, loadout de 2 slots gratis para equipar/desequipar ✓, solo Billetera y Fama con efecto mecánico real ✓, multiplicadores de dinero/reputación separados en `economia::calcular` ✓, stub viejo retirado por completo ✓.
- **Fuera de alcance deliberado (para planes posteriores):** efectos mecánicos de Detective/Manos Rápidas/Ritmo (dependen de consola SQL real y sistema de turnos, ninguno construido); escalera de slots por rango (2→7, Etapa 13 completa); combos entre perks (Etapa 21, Backlog); recalibración de `UMBRAL_ASCENSO_AUXILIAR` (Plan 4, detectado durante el brainstorming como probablemente desbalanceado, pero es un ajuste de balance aparte, no de este plan).
- **Consistencia de tipos:** `Perk`/`Categoria`/`Efecto` (Tarea 1) se usan con los mismos nombres en `EstadoJugador` (Tarea 2) y en `PerkConEstado`/los comandos Tauri (Tarea 3) — sin conversiones sorpresa. `economia::calcular`'s nueva firma (2 multiplicadores) se usa consistentemente en sus propios tests (Tarea 2) y en `submit_ticket` (Tarea 3).
- **Patrones de Rust verificados antes de escribir este plan:** se probó en un sandbox aparte que `Vec<&'static str>::contains(&id)` y `.iter().find(|&&d| d == id).copied()` compilan y funcionan correctamente cuando `id: &str` no es `'static` (p. ej. viene de un `String` de un comando Tauri) — no quedó como incógnita para el implementador.

---

## Execution Handoff

Plan completo y guardado en `docs/superpowers/plans/2026-07-12-fase0-05-rpg-perks.md`. Dos opciones de ejecución:

1. **Subagent-Driven (recomendado)** — despacho un subagente fresco por tarea, reviso el resultado entre cada una antes de seguir
2. **Ejecución inline** — ejecuto las tareas en esta sesión con executing-plans, ejecución por lotes con checkpoints

¿Cuál prefieres?
