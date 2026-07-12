use sqlparser::ast::{JoinConstraint, JoinOperator, SelectItem, SetExpr, Statement, TableFactor};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;

/// Una regla de buenas prácticas violada (Etapa 17-C). Cada variante tiene un
/// comentario pre-escrito del Mentor asociado (Etapa 17-D, ver `mentor.rs`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Regla {
    /// `SELECT *` en vez de columnas explícitas.
    SelectStar,
    /// JOIN (o comma-join) sin condición de unión — riesgo de producto cartesiano.
    JoinSinCondicion,
    /// Dos o más tablas en la consulta, pero al menos una sin alias.
    AliasFaltante,
}

const PENALIZACION_POR_REGLA: f64 = 30.0;

fn agregar_regla(violaciones: &mut Vec<Regla>, regla: Regla) {
    if !violaciones.contains(&regla) {
        violaciones.push(regla);
    }
}

fn constraint_de(op: &JoinOperator) -> Option<&JoinConstraint> {
    match op {
        JoinOperator::Join(c)
        | JoinOperator::Inner(c)
        | JoinOperator::Left(c)
        | JoinOperator::LeftOuter(c)
        | JoinOperator::Right(c)
        | JoinOperator::RightOuter(c)
        | JoinOperator::FullOuter(c)
        | JoinOperator::CrossJoin(c) => Some(c),
        _ => None,
    }
}

fn sin_alias(factor: &TableFactor) -> bool {
    matches!(factor, TableFactor::Table { alias: None, .. })
}

/// Analiza el AST de la query del jugador y devuelve las reglas violadas.
/// Si la query no parsea como un SELECT simple, no reporta violaciones — la
/// responsabilidad de "SQL inválido" es de la ejecución (Etapa 17-A), no del
/// linter.
pub fn analizar(sql: &str) -> Vec<Regla> {
    let mut violaciones = Vec::new();

    let Ok(statements) = Parser::parse_sql(&PostgreSqlDialect {}, sql) else {
        return violaciones;
    };
    let Some(Statement::Query(query)) = statements.into_iter().next() else {
        return violaciones;
    };
    let SetExpr::Select(select) = *query.body else {
        return violaciones;
    };

    if select
        .projection
        .iter()
        .any(|item| matches!(item, SelectItem::Wildcard(_)))
    {
        agregar_regla(&mut violaciones, Regla::SelectStar);
    }

    let total_tablas: usize =
        select.from.len() + select.from.iter().map(|t| t.joins.len()).sum::<usize>();

    if select.from.len() > 1 {
        agregar_regla(&mut violaciones, Regla::JoinSinCondicion);
    }

    for tabla in &select.from {
        if total_tablas > 1 && sin_alias(&tabla.relation) {
            agregar_regla(&mut violaciones, Regla::AliasFaltante);
        }
        for join in &tabla.joins {
            if let Some(constraint) = constraint_de(&join.join_operator) {
                if matches!(constraint, JoinConstraint::None) {
                    agregar_regla(&mut violaciones, Regla::JoinSinCondicion);
                }
            }
            if total_tablas > 1 && sin_alias(&join.relation) {
                agregar_regla(&mut violaciones, Regla::AliasFaltante);
            }
        }
    }

    violaciones
}

/// Puntaje 0-100: 100 menos 30 por cada regla violada (piso en 0) — nunca
/// pase/falla binario (Etapa 17-C).
pub fn puntaje(violaciones: &[Regla]) -> f64 {
    (100.0 - PENALIZACION_POR_REGLA * violaciones.len() as f64).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detecta_select_star() {
        assert_eq!(analizar("SELECT * FROM pacientes"), vec![Regla::SelectStar]);
    }

    #[test]
    fn query_limpia_no_viola_nada() {
        assert_eq!(
            analizar("SELECT nombre FROM pacientes WHERE departamento_id = 1 ORDER BY fecha_ingreso DESC"),
            Vec::<Regla>::new()
        );
    }

    #[test]
    fn detecta_comma_join_sin_condicion() {
        assert_eq!(
            analizar(
                "SELECT p.nombre, d.nombre FROM pacientes p, departamentos d WHERE p.departamento_id = d.id"
            ),
            vec![Regla::JoinSinCondicion]
        );
    }

    #[test]
    fn detecta_join_sin_on() {
        assert_eq!(
            analizar("SELECT p.nombre FROM pacientes p JOIN departamentos d"),
            vec![Regla::JoinSinCondicion]
        );
    }

    #[test]
    fn detecta_alias_faltante() {
        assert_eq!(
            analizar(
                "SELECT p.nombre FROM pacientes p JOIN departamentos ON p.departamento_id = departamentos.id"
            ),
            vec![Regla::AliasFaltante]
        );
    }

    #[test]
    fn join_correcto_con_alias_no_viola_nada() {
        assert_eq!(
            analizar("SELECT p.nombre FROM pacientes p JOIN departamentos d ON p.departamento_id = d.id"),
            Vec::<Regla>::new()
        );
    }

    #[test]
    fn query_de_3_tablas_bien_escrita_no_viola_nada() {
        assert_eq!(
            analizar(
                "SELECT d.nombre, COUNT(t.id) FROM tratamientos t \
                 JOIN pacientes p ON p.id = t.paciente_id \
                 JOIN departamentos d ON d.id = p.departamento_id \
                 GROUP BY d.nombre"
            ),
            Vec::<Regla>::new()
        );
    }

    #[test]
    fn detecta_varias_violaciones_a_la_vez() {
        assert_eq!(
            analizar("SELECT * FROM pacientes p, departamentos"),
            vec![Regla::SelectStar, Regla::JoinSinCondicion, Regla::AliasFaltante]
        );
    }

    #[test]
    fn puntaje_baja_30_por_regla() {
        assert_eq!(puntaje(&[]), 100.0);
        assert_eq!(puntaje(&[Regla::SelectStar]), 70.0);
        assert_eq!(puntaje(&[Regla::SelectStar, Regla::AliasFaltante]), 40.0);
    }

    #[test]
    fn puntaje_nunca_baja_de_cero() {
        assert_eq!(
            puntaje(&[
                Regla::SelectStar,
                Regla::JoinSinCondicion,
                Regla::AliasFaltante,
                Regla::AliasFaltante
            ]),
            0.0
        );
    }
}
