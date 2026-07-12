use super::practices::Regla;

const UMBRAL_VELOCIDAD_BAJA: f64 = 70.0;

/// Comentario del Mentor (Etapa 17-D): nunca se genera con IA en vivo — cada
/// regla de buenas prácticas (o un patrón de costo de plan) tiene un
/// comentario pre-escrito. Se muestra el de la primera regla violada; si
/// ninguna regla falló pero el plan es notablemente costoso, se muestra un
/// comentario sobre el plan. Nunca dos comentarios a la vez (Etapa 11-E: "no
/// en cada ticket, se volvería ruido").
pub fn comentario(violaciones: &[Regla], puntaje_velocidad: f64) -> Option<&'static str> {
    for regla in violaciones {
        let texto = match regla {
            Regla::SelectStar => {
                "Vi que usaste SELECT *. Funciona, pero listar las columnas que de verdad \
                 necesitas hace la query más clara y evita traer datos de más."
            }
            Regla::JoinSinCondicion => {
                "Tu JOIN no tiene una condición clara de unión — eso puede generar un \
                 producto cartesiano (cada fila de una tabla contra cada fila de la otra). \
                 Revisa tu ON."
            }
            Regla::AliasFaltante => {
                "Cuando unes varias tablas, ponerles un alias corto a cada una hace la query \
                 mucho más fácil de leer — la tuya funciona, pero le vendría bien."
            }
        };
        return Some(texto);
    }

    if puntaje_velocidad < UMBRAL_VELOCIDAD_BAJA {
        return Some(
            "Tu query encontró la respuesta correcta, pero el plan de ejecución cuesta más \
             de lo necesario. Vale la pena revisar si hay una forma más directa de llegar al \
             mismo resultado.",
        );
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn muestra_comentario_de_la_primera_regla_violada() {
        assert!(comentario(&[Regla::SelectStar], 100.0)
            .unwrap()
            .contains("SELECT *"));
    }

    #[test]
    fn prioriza_la_primera_regla_sobre_las_siguientes() {
        assert!(comentario(&[Regla::JoinSinCondicion, Regla::AliasFaltante], 100.0)
            .unwrap()
            .contains("JOIN"));
    }

    #[test]
    fn muestra_comentario_de_velocidad_si_no_hay_reglas_violadas_pero_el_plan_es_costoso() {
        assert!(comentario(&[], 40.0).is_some());
    }

    #[test]
    fn no_hay_comentario_si_todo_esta_bien() {
        assert_eq!(comentario(&[], 100.0), None);
    }
}
