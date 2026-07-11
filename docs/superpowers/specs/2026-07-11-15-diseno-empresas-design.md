# Etapa 15: Diseño de Empresas

**Estado:** Aprobado
**Fecha:** 2026-07-11

## Roster final: 8 empresas, parodias reconocibles de marcas reales (sabor mexicano)

| Empresa ficticia | Guiño directo a | Franja de rango | Mini-boss | Peculiaridad absurda (envoltorio, Etapa 9) |
|---|---|---|---|---|
| **Hospital Arcángel** | Hospital Ángeles (cadena hospitalaria privada real) | Becario → Analista | Auditor de Cumplimiento | El CEO solo se comunica por galletas de la fortuna motivacionales |
| **Postafeta** | Estafeta (paquetería real) | Becario → Auxiliar | Auditoría de pérdidas de inventario | Todo el Slack de la empresa lo administra un becario invisible llamado Kevin; todo viene firmado "- Kevin" |
| **AeroMex** | Aeroméxico | Auxiliar → Analista | Cascada de cancelaciones por tormenta | El video de seguridad dura 45 minutos y es obligatorio cada mes; nadie lo ha terminado nunca |
| **Banamix** | Banamex | Analista → DBA Junior | Auditor regulador (AML) | La bóveda tiene seguridad militar; el WiFi sigue siendo "contraseña123" |
| **Vox** | Vix (streaming) | DBA Junior → DBA | Crisis de renovación de licencias / pico de tráfico viral | El algoritmo de recomendación está personificado como una mascota ligeramente inestable, "Al Goritmo" |
| **Amazonia** | Amazon | DBA → Senior DBA | El Black Friday (crisis ya establecida en la Etapa 1) | El "Muro de la Fama" del almacén en realidad es una cuenta regresiva a Black Friday, todo el año |
| **Casino Candente** | Casino Caliente (cadena de casinos/apuestas real) | Senior DBA → Arquitecto de Datos | Investigador de fraude | RRHH exige consultar a un gato de la suerte antes de cualquier despido |
| **Gobierno del Estado de Miramar** *(estado ficticio, sin equivalente real específico)* | — | Lead DBA → Chief Data Officer | Migración/colapso de sistema legado en plena elección | La "Oficina de Transformación Digital" exige formularios en papel por triplicado; llevan 11 años "revisando" esa política |

La Agencia que conecta todas las empresas (Etapa 9) se llama **"Grupo Ómega RH"**.

## Regla de nomenclatura (para producción futura, Pilar 5)

Cualquier empresa/marca nueva que se agregue al roster debe ser un guiño directo y reconocible a una marca real específica (no solo "de la industria X"), lo bastante distinto en nombre/logo/slogan para no ser una copia literal, siguiendo el mismo patrón de parodia (cambio fonético mínimo, mismo campo semántico). La única excepción es **Gobierno**, que se mantiene deliberadamente genérico — satirizar una entidad gubernamental real específica implica un riesgo legal/reputacional distinto al de satirizar una marca comercial.

## Por qué este orden (arco de dificultad narrativo + técnico)

Las empresas tempranas tienen esquemas pequeños y humanos (Hospital Arcángel, Postafeta); las intermedias suben escala y sensibilidad de datos (AeroMex, Banamix, Vox, Amazonia); las tardías introducen el caos de sistemas reales de largo plazo — legado, deuda técnica, burocracia (Casino Candente, Gobierno de Miramar) — exactamente el tipo de problema que un Lead DBA/Arquitecto/CDO real enfrenta.

## La complejidad estructural del esquema escala con el rango

| Franja de rango | Empresas | Tamaño aproximado del esquema | Profundidad de joins típica |
|---|---|---|---|
| Becario / Auxiliar | Hospital Arcángel, Postafeta | ~5-8 tablas | 1-2 saltos de join |
| Analista / DBA Junior | AeroMex, Banamix | ~8-12 tablas | 2-3 saltos |
| DBA / Senior DBA | Vox, Amazonia | ~12-18 tablas, tablas de hechos más grandes | 3-4 saltos |
| Arquitecto / Lead DBA | Casino Candente | ~18-22 tablas, incluye tablas de auditoría/histórico | 4+ saltos |
| Data Engineer / CDO | Gobierno de Miramar | ~25+ tablas, esquema "legado" con inconsistencias deliberadas (nombres viejos, tablas duplicadas/deprecadas, normalización incompleta) | Joins profundos y confusos a propósito |

Este principio es requisito de diseño obligatorio para la **Etapa 16 (Diseño de bases de datos)**: la dificultad debe subir tanto en conceptos SQL (Etapa 10) como en la complejidad estructural real del esquema.

## Nota de alcance

Este roster de 8 es la visión completa. Cuántas de estas 8 entran en el MVP (probablemente 2-3) se decide en la **Etapa 19 (MVP)**.
