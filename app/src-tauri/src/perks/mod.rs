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

/// Busca un perk por id en el catálogo. Hoy solo lo usan los tests de este
/// módulo y de `economia` (Etapa 13, Plan 5) — se mantiene `pub` porque es
/// parte de la API pública del catálogo y un llamador real (comando Tauri o
/// UI) es un uso natural futuro, no un descuido.
#[allow(dead_code)]
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
