use crate::tickets::{Arquetipo, Rango};

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

/// Efecto mecánico de un perk (equipado = activo).
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub enum Efecto {
    /// Multiplica dinero ganado (+0.2 → ×1.2).
    BonoDinero(f64),
    /// Multiplica reputación ganada.
    BonoReputacion(f64),
    /// Intentos extra por ticket.
    BonoIntentos(u32),
    /// Multiplica el costo de tiempo del ticket (0.75 = 25% menos).
    FactorCostoTiempo(f64),
    /// Suma al presupuesto de tiempo al empezar un turno.
    BonoPresupuestoTurno(u32),
    /// Resalta tablas útiles al abrir un ticket.
    DestacarTablasTicket,
    /// Resalta más las relaciones en el esquema.
    ResaltarRelacionesEsquema,
    /// Autocompletado de tablas/columnas en el editor.
    AutocompletadoSql,
    /// Historial de deshacer + aviso antes de ejecutar algo raro.
    RedSeguridadSql,
}

const CATALOGO: [Perk; 9] = [
    Perk {
        id: "instinto",
        nombre: "Instinto",
        categoria: Categoria::Detective,
        descripcion: "Al abrir un ticket, marca qué tablas probablemente necesitas.",
        costo_dinero: 200,
        reputacion_minima: 3.0,
        arquetipo_requerido: Arquetipo::Select,
        xp_minimo: 20,
        efecto: Efecto::DestacarTablasTicket,
    },
    Perk {
        id: "rayos_x",
        nombre: "Rayos X",
        categoria: Categoria::Detective,
        descripcion: "En el esquema, las relaciones entre tablas se ven mucho más claras.",
        costo_dinero: 300,
        reputacion_minima: 5.0,
        arquetipo_requerido: Arquetipo::Join,
        xp_minimo: 40,
        efecto: Efecto::ResaltarRelacionesEsquema,
    },
    Perk {
        id: "piloto_automatico",
        nombre: "Piloto Automático",
        categoria: Categoria::ManosRapidas,
        descripcion: "Sugiere nombres de tablas y columnas mientras escribes.",
        costo_dinero: 250,
        reputacion_minima: 4.0,
        arquetipo_requerido: Arquetipo::Select,
        xp_minimo: 30,
        efecto: Efecto::AutocompletadoSql,
    },
    Perk {
        id: "red_de_seguridad",
        nombre: "Red de Seguridad",
        categoria: Categoria::ManosRapidas,
        descripcion: "Puedes deshacer lo que escribes y te avisa si vas a ejecutar algo peligroso.",
        costo_dinero: 350,
        reputacion_minima: 6.0,
        arquetipo_requerido: Arquetipo::Agregacion,
        xp_minimo: 50,
        efecto: Efecto::RedSeguridadSql,
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
        descripcion: "Los tickets cuestan menos tiempo de tu día.",
        costo_dinero: 200,
        reputacion_minima: 3.0,
        arquetipo_requerido: Arquetipo::Select,
        xp_minimo: 20,
        efecto: Efecto::FactorCostoTiempo(0.75),
    },
    Perk {
        id: "modo_turbo",
        nombre: "Modo Turbo",
        categoria: Categoria::Ritmo,
        descripcion: "Empiezas cada día con más presupuesto de tiempo.",
        costo_dinero: 400,
        reputacion_minima: 7.0,
        arquetipo_requerido: Arquetipo::Join,
        xp_minimo: 60,
        efecto: Efecto::BonoPresupuestoTurno(40),
    },
    Perk {
        id: "segunda_opinion",
        nombre: "Segunda Opinión",
        categoria: Categoria::ManosRapidas,
        descripcion: "Antes de rendirte con un ticket difícil, tienes 2 intentos extra para corregir tu respuesta.",
        costo_dinero: 300,
        reputacion_minima: 5.0,
        arquetipo_requerido: Arquetipo::Select,
        xp_minimo: 30,
        efecto: Efecto::BonoIntentos(2),
    },
];

/// Catálogo completo de perks (Etapa 13) — 9 perks.
pub fn catalogo() -> &'static [Perk] {
    &CATALOGO
}

pub fn buscar(id: &str) -> Option<&'static Perk> {
    CATALOGO.iter().find(|p| p.id == id)
}

/// Extrae nombres de tabla que aparecen tras FROM / JOIN en un SQL.
pub fn tablas_mencionadas_en_sql(sql: &str) -> Vec<String> {
    let lower = sql.to_lowercase();
    let mut tablas = Vec::new();
    for clave in [" from ", " join ", "\nfrom ", "\njoin "] {
        let mut resto = lower.as_str();
        while let Some(pos) = resto.find(clave) {
            let despues = &resto[pos + clave.len()..];
            let nombre: String = despues
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if !nombre.is_empty() && !tablas.iter().any(|t| t == &nombre) {
                tablas.push(nombre);
            }
            resto = &resto[pos + clave.len()..];
        }
    }
    // También cubre "FROM tabla" al inicio.
    if let Some(resto) = lower.strip_prefix("from ") {
        let nombre: String = resto
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if !nombre.is_empty() && !tablas.iter().any(|t| t == &nombre) {
            tablas.insert(0, nombre);
        }
    }
    tablas
}

pub fn techo_revelacion(rango: Rango, reputacion: f64) -> f64 {
    match rango {
        Rango::Becario => 4.0,
        Rango::AuxiliarDeSistemas => (reputacion + 2.0).max(5.0),
    }
}

pub fn visible_en_hub(
    perk: &Perk,
    rango: Rango,
    reputacion: f64,
    desbloqueado: bool,
    equipado: bool,
) -> bool {
    desbloqueado || equipado || perk.reputacion_minima <= techo_revelacion(rango, reputacion)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogo_tiene_9_perks() {
        assert_eq!(catalogo().len(), 9);
    }

    #[test]
    fn todos_los_perks_tienen_efecto_mecanico() {
        for perk in catalogo() {
            // Cualquier variante del enum cuenta; ninguno debe quedar “vacío”.
            let _ = perk.efecto;
            assert!(
                !matches!(format!("{:?}", perk.efecto).as_str(), ""),
                "{} debe tener efecto",
                perk.id
            );
        }
        assert!(buscar("cafe_cargado").is_some());
        assert!(matches!(
            buscar("cafe_cargado").unwrap().efecto,
            Efecto::FactorCostoTiempo(_)
        ));
        assert!(matches!(
            buscar("modo_turbo").unwrap().efecto,
            Efecto::BonoPresupuestoTurno(_)
        ));
        assert!(matches!(
            buscar("instinto").unwrap().efecto,
            Efecto::DestacarTablasTicket
        ));
    }

    #[test]
    fn tablas_mencionadas_extrae_from_y_join() {
        let tablas = tablas_mencionadas_en_sql(
            "SELECT * FROM empleados e JOIN departamentos d ON e.dep = d.id",
        );
        assert!(tablas.contains(&"empleados".to_string()));
        assert!(tablas.contains(&"departamentos".to_string()));
    }

    #[test]
    fn buscar_encuentra_un_perk_por_id() {
        let perk = buscar("buena_fama").expect("buena_fama debe existir");
        assert_eq!(perk.nombre, "Buena Fama");
        assert_eq!(perk.efecto, Efecto::BonoReputacion(0.2));
    }

    #[test]
    fn becario_solo_revela_perks_de_arranque() {
        let visibles: Vec<_> = catalogo()
            .iter()
            .filter(|p| visible_en_hub(p, Rango::Becario, 0.0, false, false))
            .map(|p| p.id)
            .collect();
        assert!(visibles.contains(&"instinto"));
        assert!(visibles.contains(&"cafe_cargado"));
        assert!(visibles.contains(&"piloto_automatico"));
        assert_eq!(visibles.len(), 3);
    }

    #[test]
    fn segunda_opinion_da_2_intentos_extra() {
        let perk = buscar("segunda_opinion").expect("segunda_opinion debe existir");
        assert_eq!(perk.efecto, Efecto::BonoIntentos(2));
    }
}
