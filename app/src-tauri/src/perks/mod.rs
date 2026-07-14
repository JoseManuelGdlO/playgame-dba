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

/// El efecto mecánico real de un perk. Solo Billetera y Fama, y "Segunda
/// Opinión" (Plan 17), tienen efecto hoy — el resto de Detective/Manos
/// Rápidas/Ritmo dependen de sistemas que no existen todavía (consola SQL
/// real con ERD/autocompletado) y usan `SinEfectoMecanico`: se pueden
/// desbloquear/equipar de verdad, pero no hacen nada todavía.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub enum Efecto {
    BonoDinero(f64),
    BonoReputacion(f64),
    /// Intentos extra por ticket antes de perderlo (Plan 17).
    BonoIntentos(u32),
    SinEfectoMecanico,
}

const CATALOGO: [Perk; 9] = [
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

/// Catálogo completo de perks (Etapa 13) — 9 perks: 2 por categoría, salvo
/// Manos Rápidas que tiene 3 (Plan 17 agrega "Segunda Opinión").
pub fn catalogo() -> &'static [Perk] {
    &CATALOGO
}

/// Busca un perk por id en el catálogo. Hoy solo lo usan los tests de este
/// módulo y de `economia` (Etapa 13, Plan 5) — se mantiene `pub` porque es
/// parte de la API pública del catálogo y un llamador real (comando Tauri o
/// UI) es un uso natural futuro, no un descuido.
#[allow(dead_code)]
pub fn buscar(id: &str) -> Option<&'static Perk> {
    CATALOGO.iter().find(|p| p.id == id)
}

/// Techo de `reputacion_minima` que el hub puede mostrar según rango/rep.
/// Becario solo ve perks de arranque; Auxiliar va abriendo mejores a medida
/// que sube la reputación (con margen de +2 para ver el próximo objetivo).
pub fn techo_revelacion(rango: Rango, reputacion: f64) -> f64 {
    match rango {
        Rango::Becario => 4.0,
        Rango::AuxiliarDeSistemas => (reputacion + 2.0).max(5.0),
    }
}

/// Un perk aparece en el hub si ya lo tienes o si su requisito de
/// reputación entra bajo el techo de revelación del progreso actual.
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
        assert_eq!(catalogo().len(), 9, "8 originales + Segunda Opinion (Plan 17)");
    }

    #[test]
    fn catalogo_tiene_2_perks_por_categoria_salvo_manos_rapidas_con_3() {
        for categoria in [Categoria::Detective, Categoria::BilleteraYFama, Categoria::Ritmo] {
            let cantidad = catalogo().iter().filter(|p| p.categoria == categoria).count();
            assert_eq!(cantidad, 2, "{categoria:?} debe tener exactamente 2 perks");
        }
        let manos_rapidas = catalogo().iter().filter(|p| p.categoria == Categoria::ManosRapidas).count();
        assert_eq!(manos_rapidas, 3, "ManosRapidas suma Segunda Opinion (Plan 17)");
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
    fn solo_billetera_y_fama_y_segunda_opinion_tienen_efecto_mecanico_real() {
        for perk in catalogo() {
            let tiene_efecto_real = !matches!(perk.efecto, Efecto::SinEfectoMecanico);
            let debe_tener_efecto_real = perk.categoria == Categoria::BilleteraYFama || perk.id == "segunda_opinion";
            assert_eq!(
                tiene_efecto_real,
                debe_tener_efecto_real,
                "'{}' (categoría {:?}) tiene un efecto real inesperado, o le falta uno (Plan 17: Segunda Opinion es la única excepción fuera de Billetera y Fama)",
                perk.nombre,
                perk.categoria
            );
        }
    }

    #[test]
    fn segunda_opinion_da_2_intentos_extra() {
        let perk = buscar("segunda_opinion").expect("segunda_opinion debe existir");
        assert_eq!(perk.categoria, Categoria::ManosRapidas);
        assert_eq!(perk.efecto, Efecto::BonoIntentos(2));
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
        assert!(!visibles.contains(&"buena_fama"));
        assert!(!visibles.contains(&"modo_turbo"));
        assert_eq!(visibles.len(), 3);
    }

    #[test]
    fn auxiliar_con_poca_rep_abre_hasta_umbral_5() {
        let visibles: Vec<_> = catalogo()
            .iter()
            .filter(|p| visible_en_hub(p, Rango::AuxiliarDeSistemas, 2.5, false, false))
            .map(|p| p.id)
            .collect();
        assert!(visibles.contains(&"buena_fama"));
        assert!(visibles.contains(&"segunda_opinion"));
        assert!(visibles.contains(&"rayos_x"));
        assert!(!visibles.contains(&"modo_turbo"));
        assert!(!visibles.contains(&"bono_bajo_la_mesa"));
    }

    #[test]
    fn perk_ya_desbloqueado_sigue_visible_aunque_el_techo_sea_bajo() {
        let turbo = buscar("modo_turbo").expect("modo_turbo");
        assert!(visible_en_hub(turbo, Rango::Becario, 0.0, true, false));
    }
}
