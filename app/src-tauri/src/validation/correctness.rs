use serde_json::Value;

/// Convierte una fila (objeto JSON con columnas en el orden de la proyección)
/// a una representación normalizada por posición: se ignoran los nombres de
/// columna (Etapa 17-A: "se tolera nombres de columna/alias distintos si los
/// valores coinciden en posición y tipo") y los números se redondean a 2
/// decimales (Etapa 17-A: "se tolera redondeo menor en decimales").
fn normalizar_fila(fila: &Value) -> Vec<String> {
    match fila {
        Value::Object(mapa) => mapa.values().map(normalizar_valor).collect(),
        otro => vec![normalizar_valor(otro)],
    }
}

fn normalizar_valor(valor: &Value) -> String {
    match valor {
        Value::Number(n) => match n.as_f64() {
            Some(f) => format!("{:.2}", f),
            None => n.to_string(),
        },
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "NULL".to_string(),
        otro => otro.to_string(),
    }
}

/// Compara el resultado del jugador contra el resultado dorado (Etapa 17-A).
/// Si `requiere_orden` es true, las filas deben coincidir en el mismo orden
/// (la solicitud del ticket pidió un orden específico); si no, se comparan
/// como conjunto (multiset), ignorando el orden de filas.
pub fn son_equivalentes(doradas: &[Value], jugador: &[Value], requiere_orden: bool) -> bool {
    if doradas.len() != jugador.len() {
        return false;
    }

    let doradas: Vec<Vec<String>> = doradas.iter().map(normalizar_fila).collect();
    let jugador: Vec<Vec<String>> = jugador.iter().map(normalizar_fila).collect();

    if requiere_orden {
        return doradas == jugador;
    }

    let mut restantes = jugador.clone();
    for fila_dorada in &doradas {
        let Some(pos) = restantes.iter().position(|fila| fila == fila_dorada) else {
            return false;
        };
        restantes.remove(pos);
    }
    restantes.is_empty()
}

/// Puntaje binario (Etapa 17-A no describe crédito parcial): 100 si el
/// resultado del jugador es equivalente al dorado, 0 si no.
pub fn puntaje(doradas: &[Value], jugador: &[Value], requiere_orden: bool) -> f64 {
    if son_equivalentes(doradas, jugador, requiere_orden) {
        100.0
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn filas_identicas_son_equivalentes() {
        let doradas = vec![json!({"nombre": "Juan", "edad": 30})];
        let jugador = vec![json!({"nombre": "Juan", "edad": 30})];
        assert!(son_equivalentes(&doradas, &jugador, false));
    }

    #[test]
    fn nombres_de_columna_distintos_no_importan_si_los_valores_coinciden_en_posicion() {
        let doradas = vec![json!({"nombre": "Juan", "edad": 30})];
        let jugador = vec![json!({"nombre_paciente": "Juan", "anios": 30})];
        assert!(son_equivalentes(&doradas, &jugador, false));
    }

    #[test]
    fn tolera_redondeo_menor_en_decimales() {
        let doradas = vec![json!({"costo": 100.001})];
        let jugador = vec![json!({"costo": 100.0})];
        assert!(son_equivalentes(&doradas, &jugador, false));
    }

    #[test]
    fn sin_requerir_orden_las_filas_pueden_venir_en_otro_orden() {
        let doradas = vec![json!({"n": 1}), json!({"n": 2})];
        let jugador = vec![json!({"n": 2}), json!({"n": 1})];
        assert!(son_equivalentes(&doradas, &jugador, false));
    }

    #[test]
    fn requiriendo_orden_el_orden_distinto_falla() {
        let doradas = vec![json!({"n": 1}), json!({"n": 2})];
        let jugador = vec![json!({"n": 2}), json!({"n": 1})];
        assert!(!son_equivalentes(&doradas, &jugador, true));
    }

    #[test]
    fn valores_distintos_no_son_equivalentes() {
        let doradas = vec![json!({"n": 1})];
        let jugador = vec![json!({"n": 2})];
        assert!(!son_equivalentes(&doradas, &jugador, false));
    }

    #[test]
    fn cantidad_de_filas_distinta_no_es_equivalente() {
        let doradas = vec![json!({"n": 1}), json!({"n": 2})];
        let jugador = vec![json!({"n": 1})];
        assert!(!son_equivalentes(&doradas, &jugador, false));
    }

    #[test]
    fn puntaje_es_100_o_0() {
        let doradas = vec![json!({"n": 1})];
        assert_eq!(puntaje(&doradas, &doradas, false), 100.0);
        let jugador = vec![json!({"n": 2})];
        assert_eq!(puntaje(&doradas, &jugador, false), 0.0);
    }
}
