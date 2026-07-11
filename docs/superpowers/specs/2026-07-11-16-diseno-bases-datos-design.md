# Etapa 16: Diseño de Bases de Datos

**Estado:** Aprobado
**Fecha:** 2026-07-11

## Decisión: datos fijos, generados una vez en desarrollo

Se genera un dataset realista por empresa usando scripts (tipo Faker) una vez durante producción, se revisa a mano que las respuestas a los tickets tengan sentido, y se congela como parte del juego. Se descarta la generación aleatoria por partida por su alto riesgo técnico (garantizar en tiempo real que nunca se produzca un caso ambiguo/roto para un ticket) frente al beneficio de un solo dev.

## Metodología de generación (sistémica, Pilar 5)

1. Definir entidades reales de la industria (ej. para un hospital: pacientes, tratamientos, empleados).
2. Diseñar relaciones/FKs siguiendo la guía de tamaño/profundidad de la Etapa 15 según la franja de rango de esa empresa.
3. Poblar con datos sintéticos realistas vía scripts tipo Faker (nombres, fechas, distribuciones plausibles) — generado una vez en desarrollo, nunca a mano fila por fila.
4. Validar manualmente que cada plantilla de ticket (Etapa 14) aplicada a ese dataset produzca una respuesta limpia, sin ambigüedades ni empates raros — paso de control de calidad obligatorio antes de congelar el dataset.
5. Congelar el dataset como parte del contenido del juego (no se regenera en runtime).

## Ejemplo concreto: Hospital Arcángel (franja Becario → Analista, ~6 tablas)

```
pacientes(id, nombre, fecha_nacimiento, genero, fecha_ingreso, fecha_alta, departamento_id, diagnostico, seguro_id)
departamentos(id, nombre, piso, jefe_id)
empleados(id, nombre, puesto, departamento_id, fecha_contratacion, salario)
tratamientos(id, paciente_id, tipo, fecha, costo, empleado_id)
seguros(id, aseguradora, cobertura_pct)
habitaciones(id, numero, departamento_id, tipo, ocupada)
```

Cumple la guía de la Etapa 15 (~5-8 tablas, 1-2 saltos de join para la mayoría de tickets: `pacientes JOIN departamentos`, `pacientes JOIN tratamientos JOIN empleados`).

## Realismo/"suciedad" de datos escala con el rango (formaliza la Etapa 15)

- **Empresas tempranas** (Hospital Arcángel, Postafeta): datos limpios, nombres de columnas consistentes — el foco es aprender a leer un esquema simple.
- **Empresas tardías** (Casino Candente, Gobierno de Miramar): NULLs deliberados en campos opcionales, columnas duplicadas/deprecadas (`telefono` y `telefono_2_viejo_no_usar`), nombres inconsistentes entre tablas relacionadas — simula deuda técnica real de sistemas longevos, y se convierte en parte del desafío.

## Requisito técnico impuesto a la Etapa 18 (Arquitectura)

El motor de base de datos elegido debe soportar de forma nativa: CTEs (incluyendo recursivos), window functions, índices con `EXPLAIN`/planes de ejecución reales, y procedimientos almacenados/triggers (para los rangos altos, Etapa 14). Restricción de diseño explícita para cuando se decida el stack técnico; no se resuelve en esta etapa.

## Documentación del esquema (alimenta el visor ERD, Etapa 7)

Cada tabla incluye comentarios de columna con sabor (`// campo legado, no usar` en las empresas tardías) que el visor ERD mostrará directamente — el mismo dataset sirve tanto para la lógica del juego como para su propia documentación in-game.
