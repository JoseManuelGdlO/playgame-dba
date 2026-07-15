use std::path::Path;

const ARCHIVO_GUARDADO: &str = "partida.json";

/// Snapshot serializable de todo el estado del juego (Etapa 11-G/Menú, Plan
/// 9) — se persiste como JSON en el directorio de datos de la app. Los
/// tickets pendientes se guardan por id, no como `Ticket` completo: evita
/// tener que exponer/reconstruir campos internos como `sql_dorada` en el
/// archivo de guardado, y los ids ya alcanzan para reconstruir el `Ticket`
/// completo contra el catálogo correcto al cargar (ver `lib.rs`).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PartidaGuardada {
    pub dinero: i64,
    /// Sueldo ganado hoy y aún no cobrado (partidas viejas → 0).
    #[serde(default)]
    pub dinero_pendiente: i64,
    pub reputacion: f64,
    pub xp_por_arquetipo: Vec<(crate::tickets::Arquetipo, i64)>,
    pub rango: crate::tickets::Rango,
    pub perks_desbloqueados: Vec<String>,
    pub perks_equipados: Vec<String>,
    pub empresa: crate::db::Company,
    pub fase: crate::FaseArco,
    pub indice_siguiente: usize,
    pub presupuesto_restante: u32,
    pub pendientes_ids: Vec<String>,
}

/// Escribe `partida` como JSON en `dir/partida.json`, sobreescribiendo
/// cualquier guardado anterior (un solo slot, Etapa 11-G/Menú, Plan 9).
pub fn guardar(dir: &Path, partida: &PartidaGuardada) -> anyhow::Result<()> {
    std::fs::create_dir_all(dir)?;
    let json = serde_json::to_string_pretty(partida)?;
    std::fs::write(dir.join(ARCHIVO_GUARDADO), json)?;
    Ok(())
}

/// Lee el guardado de `dir/partida.json` si existe. `Ok(None)` si todavía no
/// hay ningún archivo de guardado (primera vez que se abre la app).
pub fn cargar(dir: &Path) -> anyhow::Result<Option<PartidaGuardada>> {
    let ruta = dir.join(ARCHIVO_GUARDADO);
    if !ruta.exists() {
        return Ok(None);
    }
    let json = std::fs::read_to_string(ruta)?;
    Ok(Some(serde_json::from_str(&json)?))
}

/// `true` si ya existe un archivo de guardado en `dir` (Etapa 11-G/Menú,
/// Plan 9) — usado para habilitar "Cargar partida" en el Menú sin tener que
/// leer/parsear el archivo completo.
pub fn existe(dir: &Path) -> bool {
    dir.join(ARCHIVO_GUARDADO).exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Company;
    use crate::tickets::{Arquetipo, Rango};
    use crate::FaseArco;

    fn partida_de_prueba() -> PartidaGuardada {
        PartidaGuardada {
            dinero: 500,
            dinero_pendiente: 80,
            reputacion: 12.5,
            xp_por_arquetipo: vec![(Arquetipo::Select, 30), (Arquetipo::Join, 20)],
            rango: Rango::AuxiliarDeSistemas,
            perks_desbloqueados: vec!["instinto".to_string()],
            perks_equipados: vec!["instinto".to_string()],
            empresa: Company::Postafeta,
            fase: FaseArco::TrabajoNormal,
            indice_siguiente: 2,
            presupuesto_restante: 80,
            pendientes_ids: vec!["postafeta_reporte_paquetes_centro".to_string()],
        }
    }

    fn dir_de_prueba(nombre: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("query_path_test_guardado_{nombre}_{}", std::process::id()))
    }

    #[test]
    fn guardar_y_cargar_hace_round_trip_exacto() {
        let dir = dir_de_prueba("round_trip");
        let partida = partida_de_prueba();

        guardar(&dir, &partida).expect("debe poder guardar");
        let cargada = cargar(&dir).expect("debe poder cargar").expect("debe existir un guardado");

        assert_eq!(cargada.dinero, partida.dinero);
        assert_eq!(cargada.dinero_pendiente, partida.dinero_pendiente);
        assert_eq!(cargada.reputacion, partida.reputacion);
        assert_eq!(cargada.xp_por_arquetipo, partida.xp_por_arquetipo);
        assert_eq!(cargada.rango, partida.rango);
        assert_eq!(cargada.perks_desbloqueados, partida.perks_desbloqueados);
        assert_eq!(cargada.perks_equipados, partida.perks_equipados);
        assert_eq!(cargada.empresa, partida.empresa);
        assert_eq!(cargada.fase, partida.fase);
        assert_eq!(cargada.indice_siguiente, partida.indice_siguiente);
        assert_eq!(cargada.presupuesto_restante, partida.presupuesto_restante);
        assert_eq!(cargada.pendientes_ids, partida.pendientes_ids);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn existe_es_false_sin_guardado_y_true_despues_de_guardar() {
        let dir = dir_de_prueba("existe");
        assert!(!existe(&dir));

        guardar(&dir, &partida_de_prueba()).expect("debe poder guardar");
        assert!(existe(&dir));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn cargar_devuelve_none_si_no_existe_el_archivo() {
        let dir = dir_de_prueba("none");
        assert!(cargar(&dir).expect("no debe fallar, solo no encontrar nada").is_none());
    }
}
