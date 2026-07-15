import {
  sfxClick,
  sfxTecleo,
  sfxCierreDia,
  sfxTick,
  sfxExito,
  sfxError,
  sfxAscenso,
  iniciarAmbiente,
  alternarMusica,
  alternarEfectos,
  establecerModoMusica,
} from "./audio.js";
import {
  iniciarTutorial,
  tutorialActivo,
  saltarTutorial,
  notificarClicPrimerTicket,
  notificarSqlCambiado,
  notificarClicPlay,
  notificarClicEnviar,
  notificarCierreScoring,
} from "./tutorial.js";
import { mostrarDialogo, ocultarDialogo } from "./dialogo.js";
import {
  intentarCapitulosHasta,
  forzarSiguienteCapituloSubtrama,
  subtramaActiva,
  reiniciarSubtramaDebug,
} from "./subtrama.js";
import { retratoDelsyHabla } from "./sprites.js";
import {
  sincronizarPerksEquipados,
  pintarPistaInstinto,
  aplicarResaltadoInstintoEnEsquema,
  registrarVocabularioEsquema,
  empujarHistorialSql,
  deshacerSql,
  reiniciarHistorialSql,
  avisoSqlSospechoso,
  actualizarSugerenciasSql,
  aplicarSugerenciaSql,
  perkEquipado,
} from "./perks-efectos.js";

const { invoke } = window.__TAURI__.core;

let sqlInput, statusMsg, resultTable, dineroEl, reputacionEl, rangoEl;
let listaPerks, perksEquipadosMsg, perkSlotsEl, perkSlotsLabelEl;
let perkIdsPrevios = { desbloqueados: new Set(), equipados: new Set(), vistos: new Set() };
let perksYaRenderizados = false;
let presupuestoEl, listaTickets, ticketActivoInfo, bandejaTitulo;
let scoringOverlay, scoringAscenso, agenciaOverlay, cerrarDiaOverlay;
let cerrarDiaMensajeEl;
let cerrandoDia = false;
let pantallaMenu, appShell, pantallaHub, pantallaConsola, btnCargarPartida;
let pausaOverlay;
let ticketRetrato, consolaTitulo;
let btnMuteMusica, btnMuteEfectos;
let btnCerrarScoring;
let btnSaltarTutorial;
let headerAppShell, dineroHubEl, reputacionHubEl;
let rangoPerfilEl, progresoRangoActualTextoEl, progresoRangoSiguienteEl;
let sueldoPerfilEl, sueldoPerfilNotaEl;
let sueldoDiarioActual = 0;
let tooltipGlobal;
let empresaNombreEl, empresaDescripcionEl;
let esquemaOverlay, esquemaLienzo, esquemaSvg;
let wikiOverlay, wikiIndice, wikiArticulo;
let posicionesActuales = {};
let esquemaRelacionesActuales = [];
let cajaArrastrando = null;
let offsetArrastreX = 0;
let offsetArrastreY = 0;

const RETRATOS = {
  generico: `<svg viewBox="0 0 8 8"><rect width="8" height="8" fill="#4a4a3a"/><rect x="2" y="1" width="4" height="3" fill="#8a8266"/><rect x="1" y="4" width="6" height="3" fill="#6b6b52"/><rect x="3" y="2" width="1" height="1" fill="#2a2a1f"/><rect x="5" y="2" width="1" height="1" fill="#2a2a1f"/></svg>`,
  "El Mentor": `<img src="assets/mentor.png" alt="El Mentor" width="112" height="112" draggable="false" />`,
  "Auditor de Cumplimiento": `<svg viewBox="0 0 8 8"><rect width="8" height="8" fill="#2a2a35"/><rect x="2" y="1" width="4" height="3" fill="#7a7a8a"/><rect x="1" y="4" width="6" height="3" fill="#5a5a6a"/><rect x="3" y="4" width="2" height="3" fill="#1c1c22"/></svg>`,
};

function retratoParaSolicitante(solicitante) {
  return RETRATOS[solicitante] || RETRATOS.generico;
}

const ICONOS_TIPO_TICKET = {
  ReporteAnalisis: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round" stroke-linecap="round"><rect x="4" y="3" width="16" height="18" rx="1"/><path d="M8 12v4M12 9v7M16 13v3"/></svg>`,
  InvestigacionDepuracion: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="10" cy="10" r="6"/><path d="M20 20l-5.5-5.5"/></svg>`,
};

const PRIORIDAD_INFO = {
  Baja: { color: "#5a7a3a", etiqueta: "BAJA" },
  Media: { color: "#9a7a2a", etiqueta: "MEDIA" },
  Urgente: { color: "#a13a54", etiqueta: "URGENTE" },
};

const DURACION_TRANSICION_MS = 250;

const TICKET_TUTORIAL_ID_PASO1 = "hospital_reporte_departamentos";
const TICKET_TUTORIAL_ID_PASO2 = "hospital_reporte_pacientes_cardiologia";

function alternarPantalla(el, mostrar) {
  if (mostrar) {
    el.classList.remove("oculto");
    void el.offsetHeight;
    el.classList.remove("fade-out");
  } else {
    el.classList.add("fade-out");
    setTimeout(() => {
      if (el.classList.contains("fade-out")) {
        el.classList.add("oculto");
      }
    }, DURACION_TRANSICION_MS);
  }
}

function mostrarPantalla(nombre) {
  alternarPantalla(pantallaMenu, nombre === "menu");
  alternarPantalla(appShell, nombre !== "menu");
  alternarPantalla(pantallaHub, nombre === "hub");
  alternarPantalla(pantallaConsola, nombre === "consola");
  headerAppShell.classList.toggle("oculto", nombre === "hub");
}

const TITULO_FASE = {
  MiniBoss: "El Auditor de Cumplimiento quiere verte",
};

const EMPRESA_INFO = {
  HospitalArcangel: {
    nombre: "Hospital Arcángel",
    descripcion:
      "Hospital Arcángel es una cadena hospitalaria donde cada expediente pasa por al menos tres departamentos antes de llegar a quien realmente lo necesita. Nadie recuerda quién aprobó el sistema actual, pero todos coinciden en que cambiarlo tomaría más tiempo del que llevan usándolo. Hay un letrero de “sonrían a la cámara” junto a la cafetera — nadie sabe a quién miran.",
  },
  Postafeta: {
    nombre: "Postafeta",
    descripcion:
      "Postafeta mueve paquetes por todo el país con una precisión sorprendente, considerando que nadie ha visto un manual de procesos desde hace años. Las decisiones importantes se toman en un canal de Slack administrado por un becario invisible llamado Kevin — todo viene firmado \"- Kevin\".",
  },
};

const POSTITS_OFICINA = [
  {
    titulo: "IMPORTANTE",
    textos: [
      "Si dices “lo antes posible” en un pedido, se entiende como “cuando puedas… o nunca”.",
      "“Urgente” significa: urgente para ellos, opcional para tu tiempo del día.",
      "No marques el pedido como listo si solo “casi” funciona. Casi no alcanza.",
      "Antes de preguntar: mira la guía del juego. Después pregunta. O mírala otra vez.",
    ],
  },
  {
    titulo: "RRHH",
    textos: [
      "El “Día del Pretzel” se reprogramó. Otra vez. No pregunten por qué.",
      "Foto del gafete: sonrían. Si no pueden, al menos no miren a la cámara muy intenso.",
      "El microondas no es un armario. Empaqueten, etiqueten, y no calienten pescado. Nunca.",
      "Premios inventados de oficina: categoría “mejor cara de “no me pagan suficiente””. Sin estatua.",
    ],
  },
  {
    titulo: "SOPORTE",
    textos: [
      "Osos. Remolachas. Pedidos de la bandeja. (En ese orden de prioridad.)",
      "Si apagas y prendes el aparato y “se arregla”, anótalo en el pedido. No digas que fue magia.",
      "Casting para película de oficina: se buscan extras. Traer suéter negro. Saber ordenar A→Z.",
      "El jefe quiere una taza de “mejor jefe del mundo”. Solo podemos imprimir etiquetas. Improvisen.",
    ],
  },
];

const FAXES_OFICINA = [
  "ASUNTO: casting interno — película de oficina. Se buscan extras que sepan ordenar de la A a la Z. Traer suéter negro.",
  "URGENTE: desapareció la taza del “mejor jefe del mundo”. Si la encuentran, no la usen. Está… sentimentalmente complicada.",
  "De: Contabilidad / Para: Todos — “¿Quién gastó en pasta y salsa para un “club de italiano”? Respondan o lo hablamos con Oscar.”",
  "FAX ilegible (como siempre): algo sobre remolachas, una granja y una serie vieja de ciencia ficción.",
  "Recordatorio: no compartan contraseñas. Compartir el almuerzo tampoco, pero eso es menos grave.",
  "Convocatoria: reunión de 5 minutos que durará 47. Traer café. No traer ideas nuevas sin un plan escrito.",
  "Atención Postafeta: si el mensaje dice “- Kevin”, archívenlo. Si no dice “- Kevin”, sospechen.",
  "Nota del Mentor (tachada): “dejar de enseñar con chistes de oficina”. (Nota nueva: seguir igual.)",
];

let faxOficinaTextoEl;
let indiceFaxOficina = 0;
/** @type {number[]} */
let indicesPostitDia = [0, 0, 0];

function elegirIndiceDistinto(largo, anterior) {
  if (largo <= 1) return 0;
  let siguiente = anterior;
  while (siguiente === anterior) {
    siguiente = Math.floor(Math.random() * largo);
  }
  return siguiente;
}

function rotarPostitsOficina(nuevoDia = false) {
  const notas = document.querySelectorAll(".hub-postits .nota-postit");
  notas.forEach((nota, i) => {
    const pool = POSTITS_OFICINA[i];
    if (!pool) return;
    const tituloEl = nota.querySelector("[data-postit-titulo]");
    const textoEl = nota.querySelector("[data-postit-texto]");
    if (tituloEl) tituloEl.textContent = pool.titulo;
    if (!textoEl) return;
    if (nuevoDia) {
      indicesPostitDia[i] = elegirIndiceDistinto(pool.textos.length, indicesPostitDia[i] ?? 0);
    } else if (textoEl.textContent.trim() === "") {
      indicesPostitDia[i] = Math.floor(Math.random() * pool.textos.length);
    }
    textoEl.textContent = pool.textos[indicesPostitDia[i]];
  });
}

function rotarFaxOficina(forzarSiguiente = false) {
  if (!faxOficinaTextoEl || FAXES_OFICINA.length === 0) return;
  if (forzarSiguiente) {
    indiceFaxOficina = (indiceFaxOficina + 1) % FAXES_OFICINA.length;
  } else {
    indiceFaxOficina = Math.floor(Math.random() * FAXES_OFICINA.length);
  }
  faxOficinaTextoEl.textContent = FAXES_OFICINA[indiceFaxOficina];
}

/** Al cerrar el día: refresca post-its y fax del hub. */
function refrescarHumorOficinaPorNuevoDia() {
  rotarPostitsOficina(true);
  rotarFaxOficina(true);
}

const POSICIONES_TABLAS = {
  HospitalArcangel: {
    departamentos: { x: 80, y: 80 },
    empleados: { x: 420, y: 60 },
    seguros: { x: 780, y: 80 },
    habitaciones: { x: 80, y: 380 },
    pacientes: { x: 420, y: 320 },
    tratamientos: { x: 420, y: 560 },
  },
  Postafeta: {
    sucursales: { x: 420, y: 60 },
    empleados: { x: 80, y: 300 },
    clientes: { x: 780, y: 300 },
    paquetes: { x: 420, y: 320 },
    incidencias: { x: 420, y: 560 },
  },
};

const NOMBRE_RANGO = {
  Becario: "Becario",
  AuxiliarDeSistemas: "Auxiliar de Sistemas",
};

const ORDEN_RANGOS = ["Becario", "AuxiliarDeSistemas"];

/** 4 etapas visuales: más cansado conforme avanza la carrera en Hospital. */
const RETRATOS_JUGADOR = [
  {
    src: "assets/jugador-1.png",
    etiqueta: "Becario · fresco",
    rangoVista: "Becario",
  },
  {
    src: "assets/jugador-2.png",
    etiqueta: "Becario · ya rodado",
    rangoVista: "Becario",
  },
  {
    src: "assets/jugador-3.png",
    etiqueta: "Auxiliar · cansado",
    rangoVista: "AuxiliarDeSistemas",
  },
  {
    src: "assets/jugador-4.png",
    etiqueta: "Post-Auditor · quemado",
    rangoVista: "AuxiliarDeSistemas",
  },
];

let retratoJugadorEl;
let faseActual = "TrabajoNormal";
/** `null` = seguir progreso real; 0–3 = forzado desde debug. */
let debugRetratoEtapa = null;
let debugRetratoEtiquetaEl;

function indiceRetratoDesdeProgreso({ rango, fase, empresa }) {
  if (empresa === "Postafeta" || fase === "ArcoCompletado") return 3;
  if (fase === "MiniBoss" || rango === "AuxiliarDeSistemas") return 2;
  // Becario: fresco al inicio; más cansado al acercarse al umbral de ascenso.
  if (reputacionActual >= UMBRAL_ASCENSO_AUXILIAR * 0.55) return 1;
  return 0;
}

function aplicarRetratoJugador(indice, { soloVista = false } = {}) {
  const etapa = RETRATOS_JUGADOR[Math.max(0, Math.min(RETRATOS_JUGADOR.length - 1, indice))];
  if (!etapa) return;
  if (retratoJugadorEl) {
    retratoJugadorEl.src = etapa.src;
    retratoJugadorEl.alt = etapa.etiqueta;
  }
  if (debugRetratoEtiquetaEl) {
    debugRetratoEtiquetaEl.textContent =
      debugRetratoEtapa == null
        ? `Retrato: ${etapa.etiqueta} (progreso real)`
        : `Retrato: ${etapa.etiqueta} (debug ${debugRetratoEtapa + 1}/${RETRATOS_JUGADOR.length})`;
  }
  if (soloVista && rangoPerfilEl) {
    rangoPerfilEl.textContent = NOMBRE_RANGO[etapa.rangoVista] || etapa.rangoVista;
  }
}

function indiceEmpleoActual() {
  if (debugRetratoEtapa != null) return debugRetratoEtapa;
  return indiceRetratoDesdeProgreso({
    rango: rangoActual,
    fase: faseActual,
    empresa: empresaActual,
  });
}

function actualizarRetratoJugador() {
  const indice = indiceEmpleoActual();
  aplicarRetratoJugador(indice, { soloVista: debugRetratoEtapa != null });
}

async function considerarSubtramaEmpleo() {
  if (tutorialActivo() || subtramaActiva()) return;
  if (appShell?.classList.contains("oculto")) return;
  if (pantallaHub?.classList.contains("oculto")) return;
  try {
    await intentarCapitulosHasta(indiceEmpleoActual());
  } catch (_) {
    /* ignore */
  }
}

function ciclarPuestoRetratoDebug() {
  if (debugRetratoEtapa == null) {
    debugRetratoEtapa = 0;
  } else if (debugRetratoEtapa >= RETRATOS_JUGADOR.length - 1) {
    debugRetratoEtapa = null;
    actualizarRetratoJugador();
    setStatus("Retrato: otra vez según progreso real.", "ok");
    return;
  } else {
    debugRetratoEtapa += 1;
  }
  actualizarRetratoJugador();
  const etapa = RETRATOS_JUGADOR[debugRetratoEtapa];
  setStatus(`Debug puesto: ${etapa.etiqueta}`, "ok");
}

function sueldoDiarioDesdeEstado(estado) {
  if (estado && typeof estado.sueldo_diario === "number") {
    return estado.sueldo_diario;
  }
  // Respaldo si alguna vista vieja no trae el campo.
  if (rangoActual === "Becario") return 0;
  if (empresaActual === "Postafeta") return 140;
  return 100;
}

function actualizarSueldoPerfil(sueldo) {
  sueldoDiarioActual = Number(sueldo) || 0;
  if (sueldoPerfilEl) {
    sueldoPerfilEl.textContent =
      sueldoDiarioActual <= 0 ? "$0 / día" : `$${sueldoDiarioActual} / día`;
  }
  if (sueldoPerfilNotaEl) {
    if (sueldoDiarioActual <= 0) {
      sueldoPerfilNotaEl.textContent = "Practicante · sin nómina";
      sueldoPerfilNotaEl.classList.remove("oculto");
    } else {
      sueldoPerfilNotaEl.textContent = "Se paga al cerrar el día";
      sueldoPerfilNotaEl.classList.remove("oculto");
    }
  }
}

function renderRango(rango) {
  const nombre = NOMBRE_RANGO[rango] || rango;
  rangoEl.textContent = nombre;
  if (debugRetratoEtapa == null) {
    rangoPerfilEl.textContent = nombre;
  }
  progresoRangoActualTextoEl.textContent = nombre;

  const indiceActual = ORDEN_RANGOS.indexOf(rango);
  const siguienteRango = ORDEN_RANGOS[indiceActual + 1];
  progresoRangoSiguienteEl.textContent = siguienteRango
    ? `➜ ${NOMBRE_RANGO[siguienteRango]}`
    : "Alcanzaste el máximo rango disponible";

  rangoActual = rango;
  // Si no llegó sueldo desde el backend en este frame, recalcula por rango/empresa.
  actualizarSueldoPerfil(sueldoDiarioDesdeEstado(null));
  actualizarRetratoJugador();
}

let dineroPendienteHubEl;
let conteoTicketsPendientes = 0;

function actualizarDinero(valor, pendiente) {
  dineroEl.textContent = valor;
  dineroHubEl.textContent = valor;
  const pendienteNum = Number(pendiente) || 0;
  if (dineroPendienteHubEl) {
    if (pendienteNum > 0) {
      dineroPendienteHubEl.textContent = `(+${pendienteNum})`;
      dineroPendienteHubEl.classList.remove("oculto");
      dineroPendienteHubEl.setAttribute("aria-hidden", "false");
      dineroPendienteHubEl.title = "Sueldo ganado hoy — se cobra al cerrar el día";
    } else {
      dineroPendienteHubEl.textContent = "";
      dineroPendienteHubEl.classList.add("oculto");
      dineroPendienteHubEl.setAttribute("aria-hidden", "true");
      dineroPendienteHubEl.removeAttribute("title");
    }
  }
}

function sincronizarEconomiaDesdeEstado(estado) {
  if (estado == null) return;
  if (typeof estado.dinero === "number") {
    actualizarDinero(estado.dinero, estado.dinero_pendiente);
  }
  if (typeof estado.reputacion === "number") {
    actualizarReputacion(estado.reputacion.toFixed(1));
  }
  if (typeof estado.sueldo_diario === "number") {
    actualizarSueldoPerfil(estado.sueldo_diario);
  }
}

function actualizarReputacion(valorFormateado) {
  reputacionEl.textContent = valorFormateado;
  reputacionHubEl.textContent = valorFormateado;
  reputacionActual = Number.parseFloat(valorFormateado) || 0;
  actualizarRetratoJugador();
}

let ticketActivoId = null;
let ticketActivoArquetipos = [];
let wikiArticuloActual = "select";

const UMBRAL_ASCENSO_AUXILIAR = 2.5;
const UMBRAL_SCORE_EXCELENTE = 85;
const UMBRAL_VELOCIDAD_EXCELENTE = 95;
const PRESUPUESTO_ALERTA = 20;

/** @param {{ pass: boolean, puntaje_correctitud: number, puntaje_velocidad: number, puntaje_practicas: number }} score */
function clasificarTierScore(score) {
  if (!score.pass) return "fail";
  const promedio =
    (Number(score.puntaje_correctitud) +
      Number(score.puntaje_velocidad) +
      Number(score.puntaje_practicas)) /
    3;
  if (promedio >= UMBRAL_SCORE_EXCELENTE || Number(score.puntaje_velocidad) >= UMBRAL_VELOCIDAD_EXCELENTE) {
    return "excelente";
  }
  return "pass";
}

/** @param {{ puntaje_correctitud: number, puntaje_velocidad: number, puntaje_practicas: number }} score */
function metricaMasDebil(score) {
  const pares = [
    ["correctitud", Number(score.puntaje_correctitud)],
    ["practicas", Number(score.puntaje_practicas)],
    ["velocidad", Number(score.puntaje_velocidad)],
  ];
  pares.sort((a, b) => a[1] - b[1]);
  return pares[0][0];
}

/** @type {{ titulo: string, pass: boolean, tier: string, deltaDinero: number, deltaRep: number, ascendio: boolean } | null} */
let ultimoFeedback = null;
let modoBossActivo = false;
let bannerBossMostrado = false;
let reputacionActual = 0;
let rangoActual = "Becario";
let ticketActivoMotivo = "";

let panelArcoCaminoEl, panelArcoFillEl, panelArcoRepEl, panelArcoTurnoEl, panelArcoLabelEl;
let ticketToastEl, bossBannerEl;
let dineroHubPopEl, reputacionHubPopEl;
let toastTimer = null;
let bossBannerTimer = null;
/** @type {Record<string, number>} */
let intentosRestantesPorTicket = {};
let intentosLimite = 3;
let ticketIntentosEl;
let enviandoTicket = false;
/** @type {Set<string>} */
let leccionesMentorMostradas = new Set();
let btnSubmit;

let empresaActual = null;

function crearTablaFilas(rows) {
  if (!rows || rows.length === 0) {
    const vacio = document.createElement("p");
    vacio.textContent = "(sin filas)";
    return vacio;
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
  return table;
}

let presupuestoRestanteActual = 100;
let ventanaConsolaEl = null;
let resultRunningEl = null;
let consolaModoActual = "idle"; // idle | querying | ok | error

function obtenerVentanaConsola() {
  if (!ventanaConsolaEl) {
    ventanaConsolaEl = document.querySelector("#pantalla-consola .ventana-terminal");
  }
  return ventanaConsolaEl;
}

function refrescarAlertaConsola() {
  const ventana = obtenerVentanaConsola();
  if (!ventana) return;
  const intentos =
    ticketActivoId != null
      ? intentosRestantesPorTicket[ticketActivoId] ?? intentosLimite
      : intentosLimite;
  const enAlerta = intentos <= 1 || presupuestoRestanteActual <= PRESUPUESTO_ALERTA;
  ventana.classList.toggle("consola-alerta", enAlerta);
}

function setEstadoConsola(modo) {
  const ventana = obtenerVentanaConsola();
  if (!ventana) return;
  consolaModoActual = modo;
  ventana.classList.remove("consola-idle", "consola-querying", "consola-ok", "consola-error");
  ventana.classList.add(`consola-${modo}`);
  refrescarAlertaConsola();

  if (!consolaTitulo) return;
  const base = ticketActivoId ? `query-path — ${ticketActivoId}` : "query-path";
  if (modo === "querying") consolaTitulo.textContent = `${base} · ejecutando…`;
  else if (modo === "ok") consolaTitulo.textContent = `${base} · ok`;
  else if (modo === "error") consolaTitulo.textContent = `${base} · error`;
  else consolaTitulo.textContent = base;
}

function limpiarEstadoConsola() {
  const ventana = obtenerVentanaConsola();
  if (ventana) {
    ventana.classList.remove(
      "consola-idle",
      "consola-querying",
      "consola-ok",
      "consola-error",
      "consola-alerta"
    );
  }
  consolaModoActual = "idle";
  if (resultRunningEl) resultRunningEl.classList.add("oculto");
  if (resultTable) {
    resultTable.classList.remove("es-flash", "es-shake-error");
  }
}

function mostrarRunningResultado(visible) {
  if (!resultRunningEl) resultRunningEl = document.querySelector("#result-running");
  if (!resultRunningEl) return;
  resultRunningEl.classList.toggle("oculto", !visible);
}

function reaccionRetrato(kind) {
  if (!ticketRetrato) return;
  ticketRetrato.classList.remove("retrato-reaccion-ok", "retrato-reaccion-error");
  void ticketRetrato.offsetHeight;
  ticketRetrato.classList.add(
    kind === "ok" ? "retrato-reaccion-ok" : "retrato-reaccion-error"
  );
}

function renderResultados(resultados) {
  resultTable.innerHTML = "";
  resultTable.classList.remove("es-shake-error");
  const mostrarEtiquetas = resultados.length > 1;
  let huboError = false;
  resultados.forEach((resultado, indice) => {
    const bloque = document.createElement("div");
    bloque.className = "resultado-bloque es-stagger";
    bloque.style.animationDelay = `${indice * 55}ms`;
    if (mostrarEtiquetas) {
      const etiqueta = document.createElement("h3");
      etiqueta.className = "resultado-etiqueta";
      etiqueta.textContent = `Resultado ${indice + 1}`;
      bloque.appendChild(etiqueta);
    }
    if (resultado.error) {
      huboError = true;
      const error = document.createElement("p");
      error.className = "resultado-error";
      error.textContent = resultado.error;
      bloque.appendChild(error);
    } else {
      bloque.appendChild(crearTablaFilas(resultado.rows));
    }
    resultTable.appendChild(bloque);
  });
  if (huboError) {
    resultTable.classList.add("es-shake-error");
    setEstadoConsola("error");
    reaccionRetrato("error");
  } else {
    resultTable.classList.remove("es-flash");
    void resultTable.offsetHeight;
    resultTable.classList.add("es-flash");
    setEstadoConsola("ok");
    reaccionRetrato("ok");
  }
}

function setStatus(text, kind) {
  statusMsg.textContent = text;
  statusMsg.className = kind || "";
}

function textoAEjecutar() {
  const inicio = sqlInput.selectionStart;
  const fin = sqlInput.selectionEnd;
  if (inicio !== fin) {
    return sqlInput.value.slice(inicio, fin);
  }
  return sqlInput.value;
}

function dividirSentencias(sql) {
  const sentencias = [];
  let actual = "";
  let comilla = null;
  for (const caracter of sql) {
    if (comilla) {
      actual += caracter;
      if (caracter === comilla) comilla = null;
      continue;
    }
    if (caracter === "'" || caracter === '"') {
      comilla = caracter;
      actual += caracter;
      continue;
    }
    if (caracter === ";") {
      sentencias.push(actual);
      actual = "";
      continue;
    }
    actual += caracter;
  }
  sentencias.push(actual);
  return sentencias.map((s) => s.trim()).filter((s) => s.length > 0);
}

async function confirmarSiSqlSospechoso(sql) {
  const aviso = avisoSqlSospechoso(sql);
  if (!aviso) return true;
  return window.confirm(aviso);
}

async function runQuery() {
  const sql = textoAEjecutar();
  if (!(await confirmarSiSqlSospechoso(sql))) {
    setStatus("Ejecución cancelada.", "");
    return;
  }
  setStatus("Ejecutando...", "");
  setEstadoConsola("querying");
  mostrarRunningResultado(true);
  try {
    const result = await invoke("run_query", { sql });
    setStatus(`OK — ${result.rows.length} fila(s)`, "ok");
    renderResultados([{ rows: result.rows }]);
  } catch (err) {
    setStatus(String(err), "error");
    resultTable.innerHTML = "";
    setEstadoConsola("error");
    reaccionRetrato("error");
    resultTable.classList.add("es-shake-error");
  } finally {
    mostrarRunningResultado(false);
  }
}

async function runAllQueries() {
  const sentencias = dividirSentencias(sqlInput.value);
  if (sentencias.length === 0) {
    setStatus("No hay ninguna consulta que ejecutar.", "error");
    return;
  }
  if (!(await confirmarSiSqlSospechoso(sqlInput.value))) {
    setStatus("Ejecución cancelada.", "");
    return;
  }
  setStatus("Ejecutando...", "");
  setEstadoConsola("querying");
  mostrarRunningResultado(true);
  const resultados = [];
  let errores = 0;
  try {
    for (const sentencia of sentencias) {
      try {
        const result = await invoke("run_query", { sql: sentencia });
        resultados.push({ rows: result.rows });
      } catch (err) {
        errores += 1;
        resultados.push({ error: String(err) });
      }
    }
    renderResultados(resultados);
    if (errores === 0) {
      setStatus(`OK — ${sentencias.length} consulta(s) ejecutada(s)`, "ok");
    } else {
      setStatus(`${errores} de ${sentencias.length} consulta(s) con error`, "error");
    }
  } finally {
    mostrarRunningResultado(false);
  }
}

async function seleccionarTicket(ticket) {
  ticketActivoId = ticket.id;
  ticketActivoMotivo = ticket.motivo || ticket.id;
  ticketActivoArquetipos = Array.isArray(ticket.arquetipos) ? [...ticket.arquetipos] : [];
  ticketActivoInfo.textContent = `Motivo: ${ticket.motivo}\nSolicitud: ${ticket.solicitud}`;
  actualizarEtiquetaIntentos(ticket.id);
  const sqlInicial = tutorialActivo() ? "" : (ticket.sql_inicial || "SELECT * FROM pacientes;");
  sqlInput.value = sqlInicial;
  reiniciarHistorialSql(sqlInicial);
  ticketRetrato.innerHTML = retratoParaSolicitante(ticket.solicitante);
  consolaTitulo.textContent = `query-path — ${ticket.id}`;
  mostrarPantalla("consola");
  setEstadoConsola("idle");
  if (resultTable) resultTable.innerHTML = "";
  mostrarRunningResultado(false);
  notificarClicPrimerTicket();
  try {
    const tablas = await invoke("pista_instinto", { id: ticket.id });
    pintarPistaInstinto(tablas);
  } catch {
    pintarPistaInstinto([]);
  }
  if (!tutorialActivo()) {
    mostrarLeccionesDelTicket(ticket);
  }
}

function ticketTieneArquetipo(ticket, arquetipo) {
  return Array.isArray(ticket.arquetipos) && ticket.arquetipos.includes(arquetipo);
}

function mostrarLeccionesDelTicket(ticket) {
  const pasos = [];
  if (ticketTieneArquetipo(ticket, "Join") && !leccionesMentorMostradas.has("Join")) {
    pasos.push(leccionJoinPasos);
  }
  if (ticketTieneArquetipo(ticket, "Agregacion") && !leccionesMentorMostradas.has("Agregacion")) {
    pasos.push(leccionAgregacionPasos);
  }
  if (pasos.length === 0) return;

  const retrato = RETRATOS["El Mentor"] || RETRATOS.generico;
  let indice = 0;
  const correrSiguiente = () => {
    if (indice >= pasos.length) {
      ocultarDialogo();
      return;
    }
    const fábrica = pasos[indice];
    indice += 1;
    fábrica(retrato, correrSiguiente);
  };
  correrSiguiente();
}

function leccionJoinPasos(retrato, alTerminar) {
  leccionesMentorMostradas.add("Join");
  mostrarDialogo(
    retrato,
    "El Mentor",
    "Este ticket pide juntar información de dos tablas (pacientes y departamentos, por ejemplo). JOIN es la pieza que las une: «esta fila de aquí va con aquella de allá» porque comparten un número en común.",
    {
      permitir: ["#ticket-activo-info", "#sql-input"],
      alContinuar: () => {
        mostrarDialogo(
          retrato,
          "El Mentor",
          "En la caja ya tienes casi todo escrito. Solo completa lo que pide la solicitud (filtro y orden). Si ves dos columnas llamadas «nombre», renómbralas con AS para no mezclarlas — por ejemplo AS paciente y AS departamento. Prueba con ▶ Play y envía cuando se vea bien.",
          {
            permitir: ["#sql-input", "#btn-play", "#btn-submit", "#btn-ver-wiki", "#btn-ver-esquema", "#wiki-overlay", "#esquema-overlay"],
            alContinuar: alTerminar,
          }
        );
      },
    }
  );
}

function leccionAgregacionPasos(retrato, alTerminar) {
  leccionesMentorMostradas.add("Agregacion");
  mostrarDialogo(
    retrato,
    "El Mentor",
    "Aquí no quieres una lista de filas sueltas: quieres un resumen. «Agrupar» (GROUP BY) junta todo lo que sea del mismo tipo, y COUNT(*) cuenta cuántas hay en cada grupo.",
    {
      permitir: ["#ticket-activo-info", "#sql-input"],
      alContinuar: () => {
        mostrarDialogo(
          retrato,
          "El Mentor",
          "La base se ve así:\nSELECT tipo, COUNT(*) AS total\nFROM tratamientos\nGROUP BY tipo\nORDER BY total DESC, tipo;\n\nLee la solicitud con calma: te dice qué orden quiere el Auditor.",
          {
            permitir: ["#ticket-activo-info", "#sql-input"],
            alContinuar: () => {
              mostrarDialogo(
                retrato,
                "El Mentor",
                "Truco que casi siempre se olvida: si dos tipos salen empatados en cantidad, la solicitud pide ordenarlos por nombre (A → Z). Por eso al final va «, tipo». Si solo ordenas por el número, el Auditor puede marcarla mal aunque hayas contado bien. Prueba con ▶ Play y, cuando cuadre, envía.",
                {
                  permitir: ["#sql-input", "#btn-play", "#btn-submit", "#btn-ver-wiki", "#btn-ver-esquema", "#wiki-overlay", "#esquema-overlay"],
                  alContinuar: alTerminar,
                }
              );
            },
          }
        );
      },
    }
  );
}

const WIKI_ARTICULOS = [
  {
    id: "select",
    titulo: "SELECT",
    html: `
      <h3>SELECT — ¿qué quiero ver?</h3>
      <p>Es como pedir columnas de una hoja de cálculo. Dices qué campos quieres y de qué tabla.</p>
      <code class="wiki-ejemplo">SELECT nombre, fecha_ingreso
FROM pacientes;</code>
      <p><code>*</code> trae todo. Mejor pedir solo lo que dice el ticket.</p>
      <p class="wiki-tip">Tip: escribe el SELECT, dale a ▶ Play y mira el resultado antes de complicarlo.</p>
    `,
  },
  {
    id: "where",
    titulo: "WHERE",
    html: `
      <h3>WHERE — filtrar</h3>
      <p>Deja solo las filas que cumplen una condición. Sin WHERE ves la tabla completa.</p>
      <code class="wiki-ejemplo">SELECT nombre, tipo
FROM tratamientos
WHERE tipo = 'cirugia';</code>
      <ul>
        <li>Textos entre comillas simples: <code>'urgencia'</code></li>
        <li><code>AND</code> = deben cumplirse las dos; <code>OR</code> = basta con una</li>
      </ul>
    `,
  },
  {
    id: "order-by",
    titulo: "ORDER BY",
    html: `
      <h3>ORDER BY — ordenar la lista</h3>
      <p>Decide quién sale primero. Muchos tickets son estrictos con el orden.</p>
      <code class="wiki-ejemplo">SELECT nombre, fecha_ingreso
FROM pacientes
ORDER BY fecha_ingreso DESC, nombre;</code>
      <ul>
        <li><code>DESC</code> = de mayor a menor (o más reciente primero)</li>
        <li>Si pides dos criterios, el primero manda; el segundo solo desempata</li>
        <li>Ejemplo: primero por cantidad, y si empatan, por nombre A→Z: <code>ORDER BY total DESC, tipo</code></li>
      </ul>
    `,
  },
  {
    id: "join",
    titulo: "JOIN",
    html: `
      <h3>JOIN — juntar dos tablas</h3>
      <p>Imagina dos listas del hospital. JOIN las cruza cuando comparten un mismo código (por ejemplo el departamento).</p>
      <code class="wiki-ejemplo">SELECT p.nombre AS paciente, d.nombre AS departamento
FROM pacientes p
JOIN departamentos d ON p.departamento_id = d.id
ORDER BY p.nombre;</code>
      <p class="wiki-tip">Si las dos tablas tienen una columna «nombre», ponles apodo con AS para no confundirlas.</p>
    `,
  },
  {
    id: "group-by",
    titulo: "GROUP BY",
    html: `
      <h3>GROUP BY — resumir y contar</h3>
      <p>En vez de cada fila suelta, juntas las que se parecen (mismo tipo) y cuentas cuántas hay en cada montón.</p>
      <code class="wiki-ejemplo">SELECT tipo, COUNT(*) AS total
FROM tratamientos
GROUP BY tipo
ORDER BY total DESC, tipo;</code>
      <ul>
        <li><code>COUNT(*)</code> = «¿cuántas hay en este grupo?»</li>
        <li>Si dos grupos empatan en cantidad y piden orden alfabético, añade el nombre al final del ORDER BY</li>
      </ul>
    `,
  },
  {
    id: "alias",
    titulo: "Alias (AS)",
    html: `
      <h3>AS — poner apodos</h3>
      <p>Sirve para renombrar una columna o acortar el nombre de una tabla. Útil cuando dos tablas se llaman igual en una columna.</p>
      <code class="wiki-ejemplo">SELECT p.nombre AS paciente, d.nombre AS departamento
FROM pacientes AS p
JOIN departamentos AS d ON p.departamento_id = d.id;</code>
    `,
  },
  {
    id: "intentos",
    titulo: "Tickets e intentos",
    html: `
      <h3>Cómo entregar un ticket</h3>
      <ol style="margin:0 0 0.7rem;padding-left:1.15rem;font-size:0.82rem;line-height:1.45">
        <li>Lee el motivo y la solicitud (qué quieren ver y en qué orden)</li>
        <li>Si te pierdes, abre el <strong>esquema</strong> o la <strong>Wiki SQL</strong></li>
        <li>Escribe, prueba con <strong>▶ Play</strong> y revisa la tabla de resultado</li>
        <li>Cuando se vea bien, <strong>✓ Enviar ticket</strong></li>
      </ol>
      <p>Tienes pocos intentos por ticket. Un fallo gasta uno; si se acaban, ese ticket se pierde.</p>
      <p class="wiki-tip">No hace falta memorizar: la wiki y el mentor están para cuando se te olvide.</p>
    `,
  },
];

function articuloWikiParaTicket() {
  if (ticketActivoArquetipos.includes("Join")) return "join";
  if (ticketActivoArquetipos.includes("Agregacion")) return "group-by";
  return "select";
}

function pintarIndiceWiki() {
  if (!wikiIndice) return;
  wikiIndice.innerHTML = "";
  WIKI_ARTICULOS.forEach((art) => {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = `wiki-indice-btn${art.id === wikiArticuloActual ? " activo" : ""}`;
    btn.textContent = art.titulo;
    btn.dataset.wiki = art.id;
    btn.addEventListener("click", () => mostrarWiki(art.id));
    wikiIndice.appendChild(btn);
  });
}

function pintarArticuloWiki(id) {
  const art = WIKI_ARTICULOS.find((a) => a.id === id) || WIKI_ARTICULOS[0];
  wikiArticuloActual = art.id;
  if (wikiArticulo) {
    wikiArticulo.innerHTML = art.html;
  }
  pintarIndiceWiki();
}

function mostrarWiki(articuloId) {
  const id = articuloId || articuloWikiParaTicket();
  pintarArticuloWiki(id);
  if (wikiOverlay) {
    wikiOverlay.classList.remove("oculto");
  }
}

function sincronizarModoBoss(fase) {
  if (!pantallaHub || !bossBannerEl) return;

  const enBoss = fase === "MiniBoss";
  modoBossActivo = enBoss;
  pantallaHub.classList.toggle("hub-boss", enBoss);
  establecerModoMusica(enBoss ? "boss" : "ambiente");

  if (!enBoss) {
    bannerBossMostrado = false;
    bossBannerEl.classList.add("oculto");
    return;
  }

  if (!bannerBossMostrado) {
    mostrarBannerBoss();
  }
}

function mostrarBannerBoss() {
  if (!bossBannerEl) return;

  bannerBossMostrado = true;
  bossBannerEl.classList.remove("oculto");
  if (bossBannerTimer) clearTimeout(bossBannerTimer);
  bossBannerTimer = setTimeout(() => {
    bossBannerEl.classList.add("oculto");
  }, 2800);
}

function actualizarEtiquetaIntentos(ticketId) {
  if (!ticketIntentosEl) return;
  if (!ticketId) {
    ticketIntentosEl.textContent = "";
    ticketIntentosEl.classList.add("oculto");
    return;
  }
  const restantes = intentosRestantesPorTicket[ticketId] ?? intentosLimite;
  ticketIntentosEl.textContent = `Intentos restantes: ${restantes} / ${intentosLimite}`;
  ticketIntentosEl.classList.remove("oculto");
}

function sincronizarIntentosDesdeEstado(estadoTurno) {
  intentosLimite = estadoTurno.intentos_limite ?? 3;
  intentosRestantesPorTicket = { ...(estadoTurno.intentos_restantes || {}) };
  if (ticketActivoId) {
    actualizarEtiquetaIntentos(ticketActivoId);
  }
}

function actualizarPanelArco({ empresa, fase, pendientesCount, presupuesto }) {
  if (
    !panelArcoCaminoEl ||
    !panelArcoFillEl ||
    !panelArcoRepEl ||
    !panelArcoTurnoEl ||
    !panelArcoLabelEl
  ) {
    return;
  }

  const mostrarCamino =
    empresa === "HospitalArcangel" &&
    (rangoActual === "Becario" || fase === "MiniBoss");

  panelArcoCaminoEl.classList.toggle("oculto", !mostrarCamino);

  if (mostrarCamino) {
    const enBoss = fase === "MiniBoss";
    const rep = Math.min(reputacionActual, UMBRAL_ASCENSO_AUXILIAR);
    const pct = enBoss
      ? 100
      : Math.max(0, Math.min(100, (rep / UMBRAL_ASCENSO_AUXILIAR) * 100));
    panelArcoFillEl.style.width = `${pct}%`;
    panelArcoFillEl.classList.toggle("es-completo", enBoss || rep >= UMBRAL_ASCENSO_AUXILIAR);
    panelArcoLabelEl.textContent = enBoss ? "Camino al Auditor — completo" : "Camino al Auditor";
    panelArcoRepEl.textContent = enBoss
      ? `${UMBRAL_ASCENSO_AUXILIAR} / ${UMBRAL_ASCENSO_AUXILIAR} rep`
      : `${rep.toFixed(1)} / ${UMBRAL_ASCENSO_AUXILIAR} rep`;
  }

  panelArcoTurnoEl.textContent = `Bandeja · ${pendientesCount} pendientes · presupuesto ${presupuesto}`;
}

function mostrarPopBadge(el, texto, esNegativo = false) {
  if (!el) return;
  el.textContent = texto;
  el.classList.toggle("es-negativo", esNegativo);
  el.classList.remove("oculto");
  el.style.animation = "none";
  void el.offsetHeight;
  el.style.animation = "";
  setTimeout(() => el.classList.add("oculto"), 1600);
}

function mostrarToastTicket(feedback) {
  if (!ticketToastEl || !feedback) return;
  const tier = feedback.tier || (feedback.pass ? "pass" : "fail");
  const lineaResultado =
    tier === "excelente" ? "Query limpia" : tier === "pass" ? "Resuelto" : "Incorrecto";
  const partes = [`${lineaResultado} · ${feedback.titulo}`];
  if (feedback.pass) {
    const repTxt = Number(feedback.deltaRep).toFixed(1);
    const dineroTxt =
      feedback.deltaDinero > 0
        ? `+$${feedback.deltaDinero} (al cerrar el día)`
        : `+$0`;
    partes.push(`${dineroTxt} · +${repTxt} rep`);
  }
  ticketToastEl.textContent = partes.join("\n");
  ticketToastEl.classList.remove("es-excelente", "es-pass", "es-fallo");
  ticketToastEl.classList.add(
    tier === "excelente" ? "es-excelente" : tier === "pass" ? "es-pass" : "es-fallo"
  );
  ticketToastEl.classList.remove("oculto");
  if (toastTimer) clearTimeout(toastTimer);
  toastTimer = setTimeout(() => ticketToastEl.classList.add("oculto"), 3000);
}

function aplicarFeedbackEnHub() {
  if (!ultimoFeedback) return;
  const feedback = ultimoFeedback;
  ultimoFeedback = null;

  mostrarToastTicket(feedback);

  // El sueldo no entra a la billetera hasta cerrar el día — solo pop de reputación aquí.
  if (feedback.pass && feedback.deltaRep !== 0) {
    const repTxt = Number(feedback.deltaRep).toFixed(1);
    mostrarPopBadge(reputacionHubPopEl, `+${repTxt}`);
  }

  if (feedback.ascendio) {
    bannerBossMostrado = false;
    sincronizarModoBoss("MiniBoss");
  }
}

function renderBandeja(estadoTurno) {
  sincronizarIntentosDesdeEstado(estadoTurno);
  sincronizarEconomiaDesdeEstado(estadoTurno);
  conteoTicketsPendientes = Array.isArray(estadoTurno.pendientes)
    ? estadoTurno.pendientes.length
    : 0;
  presupuestoEl.textContent = estadoTurno.presupuesto_restante;
  presupuestoRestanteActual = Number(estadoTurno.presupuesto_restante) || 0;
  refrescarAlertaConsola();
  bandejaTitulo.textContent = TITULO_FASE[estadoTurno.fase] || "Bandeja — turno actual";
  empresaActual = estadoTurno.empresa;
  faseActual = estadoTurno.fase || "TrabajoNormal";
  actualizarRetratoJugador();
  const empresa = EMPRESA_INFO[estadoTurno.empresa];
  if (empresa) {
    empresaNombreEl.textContent = empresa.nombre;
    empresaDescripcionEl.textContent = empresa.descripcion;
  }
  listaTickets.innerHTML = "";
  estadoTurno.pendientes.forEach((ticket, indice) => {
    const li = document.createElement("li");
    li.className = "papel papel-entrando papel-ticket";
    li.style.animationDelay = `${indice * 60}ms`;
    if (indice === 0) {
      li.dataset.primerTicket = "true";
    }

    const clip = document.createElement("div");
    clip.className = "clip-papel";
    li.appendChild(clip);

    const icono = document.createElement("div");
    icono.className = "icono-tipo-ticket";
    icono.innerHTML = ICONOS_TIPO_TICKET[ticket.tipo] || ICONOS_TIPO_TICKET.ReporteAnalisis;
    li.appendChild(icono);

    const detalle = document.createElement("div");
    detalle.className = "papel-ticket-detalle";
    const info = document.createElement("div");
    info.className = "papel-ticket-motivo";
    info.textContent = `[⏱️ ${ticket.costo_tiempo}] ${ticket.motivo}`;
    const prioridad = PRIORIDAD_INFO[ticket.prioridad] || PRIORIDAD_INFO.Baja;
    const etiquetaPrioridad = document.createElement("div");
    etiquetaPrioridad.className = "papel-ticket-prioridad";
    etiquetaPrioridad.style.color = prioridad.color;
    etiquetaPrioridad.textContent = `● ${prioridad.etiqueta}`;
    const intentosEl = document.createElement("div");
    intentosEl.className = "papel-ticket-intentos";
    const restantes = intentosRestantesPorTicket[ticket.id] ?? intentosLimite;
    intentosEl.textContent = `Intentos ${restantes}/${intentosLimite}`;
    detalle.appendChild(info);
    detalle.appendChild(etiquetaPrioridad);
    detalle.appendChild(intentosEl);
    li.appendChild(detalle);

    const boton = document.createElement("button");
    boton.textContent = ticket.id === ticketActivoId ? "En curso" : "Trabajar en este";
    boton.addEventListener("click", () => seleccionarTicket(ticket));
    li.appendChild(boton);

    listaTickets.appendChild(li);
  });
  if (!estadoTurno.pendientes.some((t) => t.id === ticketActivoId)) {
    ticketActivoId = null;
    ticketActivoInfo.textContent = "Elige un ticket de la bandeja para empezar.";
  }
  actualizarPanelArco({
    empresa: estadoTurno.empresa,
    fase: estadoTurno.fase,
    pendientesCount: estadoTurno.pendientes.length,
    presupuesto: estadoTurno.presupuesto_restante,
  });
  sincronizarModoBoss(estadoTurno.fase);
  if (estadoTurno.fase === "ArcoCompletado") {
    agenciaOverlay.classList.remove("oculto");
  }
}

async function cargarTurno() {
  const estadoTurno = await invoke("turno_actual");
  renderBandeja(estadoTurno);
}

function pintarHubDesdeEstadoJuego(estadoJuego) {
  debugRetratoEtapa = null;
  actualizarDinero(estadoJuego.dinero, estadoJuego.dinero_pendiente);
  actualizarReputacion(estadoJuego.reputacion.toFixed(1));
  renderRango(estadoJuego.rango);
  actualizarSueldoPerfil(sueldoDiarioDesdeEstado(estadoJuego));
  renderBandeja(estadoJuego);
  rotarPostitsOficina(false);
  rotarFaxOficina(false);
  ticketActivoId = null;
  actualizarEtiquetaIntentos(null);
  mostrarPantalla("hub");
}

function leerNumeroOpcional(inputEl) {
  if (!inputEl || inputEl.value === "") return null;
  const valor = Number(inputEl.value);
  return Number.isFinite(valor) ? valor : null;
}

function alternarDebugOverlay(forzar) {
  const overlay = document.querySelector("#debug-overlay");
  if (!overlay) return;
  const mostrar = forzar === true ? true : forzar === false ? false : overlay.classList.contains("oculto");
  overlay.classList.toggle("oculto", !mostrar);
  if (mostrar) {
    const dineroInput = document.querySelector("#debug-dinero");
    const repInput = document.querySelector("#debug-reputacion");
    if (dineroInput && dineroHubEl) dineroInput.value = dineroHubEl.textContent || "0";
    if (repInput && reputacionHubEl) repInput.value = reputacionHubEl.textContent || "0";
  }
}

async function aplicarDebugEstado(forzarAuditor) {
  try {
    const estadoJuego = await invoke("debug_set_estado", {
      dinero: leerNumeroOpcional(document.querySelector("#debug-dinero")),
      reputacion: leerNumeroOpcional(document.querySelector("#debug-reputacion")),
      xpSelect: leerNumeroOpcional(document.querySelector("#debug-xp-select")),
      xpJoin: leerNumeroOpcional(document.querySelector("#debug-xp-join")),
      xpAgregacion: leerNumeroOpcional(document.querySelector("#debug-xp-agregacion")),
      forzarAuditor: Boolean(forzarAuditor),
    });
    bannerBossMostrado = false;
    pintarHubDesdeEstadoJuego(estadoJuego);
    await cargarPerks();
    alternarDebugOverlay(false);
    setStatus(
      forzarAuditor ? "Debug: Auditor forzado." : "Debug: estado aplicado.",
      "ok"
    );
  } catch (err) {
    setStatus(String(err), "error");
  }
}

async function mostrarMenu() {
  mostrarPantalla("menu");
  const existePartida = await invoke("existe_partida_guardada");
  btnCargarPartida.disabled = !existePartida;
}

async function iniciarPartida() {
  const estadoJuego = await invoke("iniciar_partida");
  leccionesMentorMostradas = new Set();
  perksYaRenderizados = false;
  reiniciarSubtramaDebug();
  pintarHubDesdeEstadoJuego(estadoJuego);
  await cargarPerks();
  await refrescarVocabularioEsquema();
  setStatus("Partida nueva iniciada.", "ok");
  const pendientes = estadoJuego.pendientes || [];
  const primerTicket = pendientes[0];
  const segundoTicket = pendientes[1];
  const esInicioDeTutorial =
    primerTicket &&
    primerTicket.id === TICKET_TUTORIAL_ID_PASO1 &&
    segundoTicket &&
    segundoTicket.id === TICKET_TUTORIAL_ID_PASO2;
  if (esInicioDeTutorial) {
    btnSaltarTutorial.classList.remove("oculto");
    iniciarTutorial(RETRATOS["El Mentor"], () => {
      btnSaltarTutorial.classList.add("oculto");
      considerarSubtramaEmpleo();
    });
  } else {
    considerarSubtramaEmpleo();
  }
}

async function cargarPartida() {
  try {
    const estadoJuego = await invoke("cargar_partida");
    perksYaRenderizados = false;
    pintarHubDesdeEstadoJuego(estadoJuego);
    await cargarPerks();
    await refrescarVocabularioEsquema();
    setStatus("Partida cargada.", "ok");
    considerarSubtramaEmpleo();
  } catch (err) {
    setStatus(String(err), "error");
  }
}

function ocultarConfirmarCerrarDia() {
  if (cerrarDiaOverlay) cerrarDiaOverlay.classList.add("oculto");
}

function pedirConfirmarCerrarDia() {
  return new Promise((resolve) => {
    if (!cerrarDiaOverlay || !cerrarDiaMensajeEl) {
      resolve(false);
      return;
    }

    const n = conteoTicketsPendientes;
    cerrarDiaMensajeEl.textContent =
      n === 1
        ? "Todavía te queda 1 ticket sin resolver."
        : `Todavía te quedan ${n} tickets sin resolver.`;

    const btnConfirmar = document.querySelector("#btn-confirmar-cerrar-dia");
    const btnCancelar = document.querySelector("#btn-cancelar-cerrar-dia");

    const finalizar = (ok) => {
      btnConfirmar.removeEventListener("click", onConfirmar);
      btnCancelar.removeEventListener("click", onCancelar);
      document.removeEventListener("keydown", onTecla);
      ocultarConfirmarCerrarDia();
      resolve(ok);
    };
    const onConfirmar = () => finalizar(true);
    const onCancelar = () => finalizar(false);
    const onTecla = (evento) => {
      if (evento.key === "Escape") {
        evento.preventDefault();
        finalizar(false);
      }
    };

    btnConfirmar.addEventListener("click", onConfirmar);
    btnCancelar.addEventListener("click", onCancelar);
    document.addEventListener("keydown", onTecla);
    cerrarDiaOverlay.classList.remove("oculto");
    btnCancelar.focus();
  });
}

async function ejecutarCerrarDia() {
  if (cerrandoDia) return;
  cerrandoDia = true;
  try {
    const estadoTurno = await invoke("cerrar_dia");
    ticketActivoId = null;
    renderBandeja(estadoTurno);

    const cobrado = Number(estadoTurno.dinero_cobrado) || 0;
    const nomina = Number(estadoTurno.sueldo_diario) || 0;
    if (cobrado > 0) {
      mostrarPopBadge(dineroHubPopEl, `+$${cobrado}`);
      const detalleNomina =
        nomina > 0 ? ` (incluye nómina $${nomina})` : rangoActual === "Becario" ? " (practicante: sin nómina)" : "";
      setStatus(`Día cerrado. Cobras $${cobrado}${detalleNomina}. Turno nuevo.`, "ok");
    } else {
      setStatus(
        rangoActual === "Becario"
          ? "Día cerrado. Sin nómina (practicante). Turno nuevo."
          : "Día cerrado. Turno nuevo.",
        "ok"
      );
    }
    refrescarHumorOficinaPorNuevoDia();
    sfxCierreDia();
    await considerarSubtramaEmpleo();
  } finally {
    cerrandoDia = false;
  }
}

async function cerrarDia() {
  if (cerrandoDia) return;
  if (conteoTicketsPendientes > 0) {
    const ok = await pedirConfirmarCerrarDia();
    if (!ok) return;
  }
  await ejecutarCerrarDia();
}

async function confirmarTransicionAgencia() {
  try {
    const estadoTurno = await invoke("confirmar_transicion_agencia");
    actualizarReputacion("0.0");
    agenciaOverlay.classList.add("oculto");
    ticketActivoId = null;
    renderBandeja(estadoTurno);
    setStatus("Bienvenido a Postafeta.", "ok");
    await considerarSubtramaEmpleo();
  } catch (err) {
    setStatus(String(err), "error");
  }
}

function animarNumero(el, valorFinal, decimales) {
  const duracionMs = 600;
  const inicio = performance.now();
  function paso(ahora) {
    const progreso = Math.min((ahora - inicio) / duracionMs, 1);
    el.textContent = (valorFinal * progreso).toFixed(decimales);
    if (progreso < 1) {
      requestAnimationFrame(paso);
    } else {
      el.textContent = valorFinal.toFixed(decimales);
    }
  }
  requestAnimationFrame(paso);
}

const DURACION_LINEA_SCORING_MS = 350;

function esperar(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function mostrarScoring(score) {
  const tituloEl = document.querySelector("#scoring-titulo");
  const mentorEl = document.querySelector("#scoring-mentor");

  btnCerrarScoring.disabled = true;
  tituloEl.textContent = "";
  tituloEl.className = "";
  mentorEl.textContent = "";
  scoringAscenso.textContent = "";

  const notaDineroEl = document.querySelector("#scoring-dinero-nota");
  if (notaDineroEl) {
    notaDineroEl.textContent =
      score.pass && score.dinero_ganado > 0 ? "(se paga al cerrar el día)" : "";
  }

  const lineas = [
    { span: document.querySelector("#scoring-correctitud"), valor: score.puntaje_correctitud, decimales: 0 },
    { span: document.querySelector("#scoring-velocidad"), valor: score.puntaje_velocidad, decimales: 0 },
    { span: document.querySelector("#scoring-practicas"), valor: score.puntaje_practicas, decimales: 0 },
    { span: document.querySelector("#scoring-dinero"), valor: score.dinero_ganado, decimales: 0 },
    { span: document.querySelector("#scoring-reputacion"), valor: score.reputacion_ganada, decimales: 1 },
  ].map((linea) => ({ ...linea, fila: linea.span.closest("p") }));

  for (const linea of lineas) {
    linea.fila.classList.add("linea-oculta");
  }

  scoringOverlay.classList.remove("oculto");

  for (const linea of lineas) {
    linea.fila.classList.remove("linea-oculta");
    animarNumero(linea.span, linea.valor, linea.decimales);
    sfxTick();
    await esperar(DURACION_LINEA_SCORING_MS);
  }

  mentorEl.textContent = score.comentario_mentor || "";

  tituloEl.textContent = score.pass ? "✅ Resuelto" : "❌ Incorrecto";
  tituloEl.className = score.pass ? "pulso" : "shake";
  if (score.pass) {
    sfxExito();
  } else {
    sfxError();
  }

  if (score.ascendio) {
    scoringAscenso.textContent = `¡Ascendiste a ${NOMBRE_RANGO[score.rango_actual] || score.rango_actual}! +1 slot de perk. Nuevos tickets disponibles.`;
    sfxAscenso();
  }

  btnCerrarScoring.disabled = false;
}

async function submitTicket() {
  if (!ticketActivoId) {
    setStatus("Elige un ticket de la bandeja primero.", "error");
    return false;
  }
  if (enviandoTicket) {
    return false;
  }
  enviandoTicket = true;
  if (btnSubmit) btnSubmit.disabled = true;
  setStatus("Enviando ticket...", "");
  setEstadoConsola("querying");
  try {
    const score = await invoke("resolver_ticket", { id: ticketActivoId, sql: sqlInput.value });
    if (score.intentos_restantes) {
      setStatus(score.mensaje, "error");
      await cargarTurno();
      actualizarEtiquetaIntentos(ticketActivoId);
      setEstadoConsola("error");
      refrescarAlertaConsola();
      reaccionRetrato("error");
      return false;
    }
    const tier = clasificarTierScore(score);
    ultimoFeedback = {
      titulo: ticketActivoMotivo || ticketActivoId || "Ticket",
      pass: score.pass,
      tier,
      deltaDinero: score.dinero_ganado,
      deltaRep: score.reputacion_ganada,
      ascendio: score.ascendio,
    };
    actualizarDinero(score.dinero_total, score.dinero_pendiente);
    actualizarReputacion(score.reputacion_total.toFixed(1));
    renderRango(score.rango_actual);
    setEstadoConsola("idle");
    mostrarScoring(score);
    setStatus(score.mensaje, score.pass ? "ok" : "error");
    await cargarTurno();
    await cargarPerks();
    return true;
  } catch (err) {
    setStatus(String(err), "error");
    setEstadoConsola("error");
    refrescarAlertaConsola();
    reaccionRetrato("error");
    // Tras un error de evaluación el backend ya reinsertó el ticket; refresca la bandeja.
    try {
      await cargarTurno();
      actualizarEtiquetaIntentos(ticketActivoId);
    } catch (_) {
      /* ignore */
    }
    return false;
  } finally {
    enviandoTicket = false;
    if (btnSubmit) btnSubmit.disabled = false;
  }
}

const PERK_CATEGORIA_LABEL = {
  Detective: "Detective",
  ManosRapidas: "Manos rápidas",
  BilleteraYFama: "Billetera",
  Ritmo: "Ritmo",
};

const PERK_CATEGORIA_CLASE = {
  Detective: "detective",
  ManosRapidas: "manos",
  BilleteraYFama: "billetera",
  Ritmo: "ritmo",
};

function iconoSvgPerk(categoria, bloqueado) {
  if (bloqueado) {
    return `<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round"><rect x="5" y="11" width="14" height="9" rx="1"/><path d="M8 11V7a4 4 0 0 1 8 0v4"/></svg>`;
  }
  switch (categoria) {
    case "Detective":
      return `<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="11" cy="11" r="6"/><path d="M20 20l-4-4"/></svg>`;
    case "ManosRapidas":
      return `<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M13 2L4 14h7l-1 8 9-12h-7l1-8z"/></svg>`;
    case "BilleteraYFama":
      return `<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round"><rect x="2" y="6" width="20" height="14" rx="2"/><path d="M2 10h20"/><circle cx="16" cy="14" r="1.5"/></svg>`;
    case "Ritmo":
      return `<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M12 4v10"/><circle cx="12" cy="17" r="3"/><path d="M12 4c3 2 6 2 8 0"/></svg>`;
    default:
      return `<svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2l2.9 6.3 6.9.6-5.2 4.6 1.6 6.8L12 17l-6.2 3.3 1.6-6.8L2.2 8.9l6.9-.6z"/></svg>`;
  }
}

function renderPerkSlots(equipadosCount, maxSlots) {
  if (!perkSlotsEl) return;
  perkSlotsEl.innerHTML = "";
  for (let i = 0; i < maxSlots; i += 1) {
    const slot = document.createElement("span");
    slot.className = `perk-slot${i < equipadosCount ? " perk-slot--lleno" : ""}`;
    perkSlotsEl.appendChild(slot);
  }
  if (perkSlotsLabelEl) {
    perkSlotsLabelEl.textContent = `Slots ${equipadosCount}/${maxSlots}`;
  }
}

function renderPerks({ perks, max_slots, ocultos = 0 }) {
  listaPerks.innerHTML = "";
  const equipados = perks.filter((p) => p.equipado);
  renderPerkSlots(equipados.length, max_slots);

  perks.forEach((perk, indice) => {
    const estadoClase = perk.equipado ? "equipado" : perk.desbloqueado ? "desbloqueado" : "bloqueado";
    const catClase = PERK_CATEGORIA_CLASE[perk.categoria] || "default";
    const catLabel = PERK_CATEGORIA_LABEL[perk.categoria] || perk.categoria;
    const estadoLabel = perk.equipado ? "Equipado" : perk.desbloqueado ? "Listo" : "Bloqueado";

    const li = document.createElement("li");
    li.className = `perk-card perk-card--${estadoClase} perk-card--${catClase} perk-card-entrando`;
    li.style.animationDelay = `${indice * 55}ms`;
    li.dataset.tooltip = perk.descripcion;
    li.dataset.perkId = perk.id;

    if (perksYaRenderizados && perk.desbloqueado && !perkIdsPrevios.desbloqueados.has(perk.id)) {
      li.classList.add("perk-card--flash");
    }
    if (perksYaRenderizados && perk.equipado && !perkIdsPrevios.equipados.has(perk.id)) {
      li.classList.add("perk-card--pulse-in");
    }
    if (perksYaRenderizados && !perkIdsPrevios.vistos.has(perk.id)) {
      li.classList.add("perk-card--reveal");
    }

    const icono = document.createElement("div");
    icono.className = "perk-icon";
    icono.innerHTML = iconoSvgPerk(perk.categoria, !perk.desbloqueado && !perk.equipado);

    const cuerpo = document.createElement("div");
    cuerpo.className = "perk-cuerpo";
    const meta = document.createElement("div");
    meta.className = "perk-meta";
    meta.textContent = `${catLabel} · ${estadoLabel}`;
    const nombre = document.createElement("div");
    nombre.className = "perk-nombre";
    nombre.textContent = perk.nombre;
    const detalle = document.createElement("div");
    detalle.className = "perk-detalle";
    detalle.textContent = perk.desbloqueado
      ? perk.descripcion
      : `$${perk.costo_dinero} · ⭐${perk.reputacion_minima}`;
    cuerpo.appendChild(meta);
    cuerpo.appendChild(nombre);
    cuerpo.appendChild(detalle);

    const boton = document.createElement("button");
    boton.type = "button";
    boton.className = "perk-accion";
    boton.textContent = perk.equipado ? "Quitar" : perk.desbloqueado ? "Equipar" : "Comprar";
    boton.title = perk.equipado ? "Dejar de usar este perk" : perk.desbloqueado ? "Usar este perk" : "Desbloquear con dinero";
    boton.addEventListener("click", (evento) => {
      evento.stopPropagation();
      accionPerk(perk);
    });

    li.appendChild(icono);
    li.appendChild(cuerpo);
    li.appendChild(boton);
    listaPerks.appendChild(li);
  });

  if (ocultos > 0) {
    const teaser = document.createElement("li");
    teaser.className = "perk-card perk-card--teaser perk-card-entrando";
    teaser.style.animationDelay = `${perks.length * 55}ms`;
    teaser.innerHTML = `
      <div class="perk-icon">${iconoSvgPerk("Detective", true)}</div>
      <div class="perk-cuerpo">
        <div class="perk-meta">Próximamente</div>
        <div class="perk-nombre">${ocultos} perk${ocultos === 1 ? "" : "s"} más</div>
        <div class="perk-detalle">Sube de rango o reputación para revelar mejores bonos.</div>
      </div>
    `;
    listaPerks.appendChild(teaser);
  }

  perkIdsPrevios = {
    desbloqueados: new Set(perks.filter((p) => p.desbloqueado || p.equipado).map((p) => p.id)),
    equipados: new Set(equipados.map((p) => p.id)),
    vistos: new Set(perks.map((p) => p.id)),
  };
  perksYaRenderizados = true;

  const resumen = `Perks equipados: ${equipados.length}/${max_slots}`;
  perksEquipadosMsg.textContent = equipados.length
    ? `${resumen} — ${equipados.map((p) => p.nombre).join(", ")}`
    : `${resumen} — ninguno equipado.`;
}

async function cargarPerks() {
  const vista = await invoke("catalogo_perks");
  renderPerks(vista);
  sincronizarPerksEquipados(vista);
  if (ticketActivoId && perkEquipado("instinto")) {
    try {
      const tablas = await invoke("pista_instinto", { id: ticketActivoId });
      pintarPistaInstinto(tablas);
    } catch {
      pintarPistaInstinto([]);
    }
  } else {
    pintarPistaInstinto([]);
  }
}

async function accionPerk(perk) {
  try {
    let vista;
    let mensaje = "";
    if (perk.equipado) {
      vista = await invoke("desequipar_perk", { id: perk.id });
      mensaje = "Perk quitado.";
    } else if (perk.desbloqueado) {
      vista = await invoke("equipar_perk", { id: perk.id });
      mensaje = "Perk equipado — ya aplica.";
    } else {
      vista = await invoke("desbloquear_perk", { id: perk.id });
      const auto = (vista.perks || []).find((p) => p.id === perk.id);
      mensaje = auto?.equipado
        ? "Perk comprado y equipado — ya aplica."
        : "Perk comprado. Equípalo en un slot libre para usarlo.";
    }
    renderPerks(vista);
    sincronizarPerksEquipados(vista);
    setStatus(mensaje, "ok");
    try {
      await cargarTurno();
    } catch {
      /* sin turno activo (menú) */
    }
    if (ticketActivoId && perkEquipado("instinto")) {
      const tablas = await invoke("pista_instinto", { id: ticketActivoId });
      pintarPistaInstinto(tablas);
    } else {
      pintarPistaInstinto([]);
    }
  } catch (err) {
    setStatus(String(err), "error");
  }
}

async function refrescarVocabularioEsquema() {
  try {
    const esquema = await invoke("esquema_actual");
    registrarVocabularioEsquema(esquema.tablas);
  } catch {
    /* DB aún no lista */
  }
}

function dibujarRelaciones(relaciones) {
  esquemaRelacionesActuales = relaciones;
  esquemaSvg.innerHTML = "";
  relaciones.forEach((rel) => {
    if (rel.tabla_origen === rel.tabla_destino) return;
    const origen = posicionesActuales[rel.tabla_origen];
    const destino = posicionesActuales[rel.tabla_destino];
    if (!origen || !destino) return;
    const cajaOrigen = esquemaLienzo.querySelector(`[data-tabla="${rel.tabla_origen}"]`);
    const cajaDestino = esquemaLienzo.querySelector(`[data-tabla="${rel.tabla_destino}"]`);
    if (!cajaOrigen || !cajaDestino) return;

    const x1 = origen.x + cajaOrigen.offsetWidth / 2;
    const y1 = origen.y + cajaOrigen.offsetHeight / 2;
    const x2 = destino.x + cajaDestino.offsetWidth / 2;
    const y2 = destino.y + cajaDestino.offsetHeight / 2;

    const linea = document.createElementNS("http://www.w3.org/2000/svg", "line");
    linea.setAttribute("x1", x1);
    linea.setAttribute("y1", y1);
    linea.setAttribute("x2", x2);
    linea.setAttribute("y2", y2);
    linea.setAttribute("class", "esquema-linea-relacion");
    esquemaSvg.appendChild(linea);
  });
}

function crearCajaTabla(tabla, posicion) {
  const caja = document.createElement("div");
  caja.className = "caja-tabla";
  caja.dataset.tabla = tabla.nombre;
  caja.style.left = `${posicion.x}px`;
  caja.style.top = `${posicion.y}px`;

  const titulo = document.createElement("div");
  titulo.className = "caja-tabla-titulo";
  titulo.textContent = tabla.nombre;
  caja.appendChild(titulo);

  if (tabla.descripcion) {
    const descripcion = document.createElement("div");
    descripcion.className = "caja-tabla-descripcion";
    descripcion.textContent = tabla.descripcion;
    caja.appendChild(descripcion);
  }

  const listaColumnas = document.createElement("ul");
  listaColumnas.className = "caja-tabla-columnas";
  tabla.columnas.forEach((columna) => {
    const li = document.createElement("li");
    const nulo = columna.nullable ? "" : " NOT NULL";
    li.innerHTML = `<span class="columna-nombre">${columna.nombre}</span> <span class="columna-tipo">${columna.tipo}${nulo}</span>`;
    if (columna.descripcion) {
      const descripcion = document.createElement("div");
      descripcion.className = "columna-descripcion";
      descripcion.textContent = columna.descripcion;
      li.appendChild(descripcion);
    }
    listaColumnas.appendChild(li);
  });
  caja.appendChild(listaColumnas);

  caja.addEventListener("mousedown", (evento) => {
    cajaArrastrando = tabla.nombre;
    const rect = esquemaLienzo.getBoundingClientRect();
    const posicion = posicionesActuales[tabla.nombre];
    offsetArrastreX = evento.clientX - rect.left + esquemaLienzo.scrollLeft - posicion.x;
    offsetArrastreY = evento.clientY - rect.top + esquemaLienzo.scrollTop - posicion.y;
    evento.preventDefault();
  });

  return caja;
}

async function mostrarEsquema() {
  const esquema = await invoke("esquema_actual");
  const posiciones = POSICIONES_TABLAS[empresaActual] || {};

  posicionesActuales = {};
  esquemaLienzo.querySelectorAll(".caja-tabla").forEach((el) => el.remove());

  esquema.tablas.forEach((tabla, indice) => {
    const posicion = posiciones[tabla.nombre] || { x: 40 + indice * 260, y: 40 };
    posicionesActuales[tabla.nombre] = { ...posicion };
    esquemaLienzo.appendChild(crearCajaTabla(tabla, posicion));
  });

  registrarVocabularioEsquema(esquema.tablas);
  dibujarRelaciones(esquema.relaciones);
  aplicarResaltadoInstintoEnEsquema(esquemaLienzo);
  esquemaOverlay.classList.remove("oculto");
}

window.addEventListener("DOMContentLoaded", async () => {
  sqlInput = document.querySelector("#sql-input");
  statusMsg = document.querySelector("#status-msg");
  resultTable = document.querySelector("#result-table");
  dineroEl = document.querySelector("#dinero");
  reputacionEl = document.querySelector("#reputacion");
  rangoEl = document.querySelector("#rango");
  listaPerks = document.querySelector("#lista-perks");
  perksEquipadosMsg = document.querySelector("#perks-equipados-msg");
  perkSlotsEl = document.querySelector("#perk-slots");
  perkSlotsLabelEl = document.querySelector("#perk-slots-label");
  presupuestoEl = document.querySelector("#presupuesto");
  listaTickets = document.querySelector("#lista-tickets");
  ticketActivoInfo = document.querySelector("#ticket-activo-info");
  bandejaTitulo = document.querySelector("#bandeja-titulo");
  scoringOverlay = document.querySelector("#scoring-overlay");
  scoringAscenso = document.querySelector("#scoring-ascenso");
  agenciaOverlay = document.querySelector("#agencia-overlay");
  cerrarDiaOverlay = document.querySelector("#cerrar-dia-overlay");
  cerrarDiaMensajeEl = document.querySelector("#cerrar-dia-mensaje");
  esquemaOverlay = document.querySelector("#esquema-overlay");
  esquemaLienzo = document.querySelector("#esquema-lienzo");
  esquemaSvg = document.querySelector("#esquema-svg");
  wikiOverlay = document.querySelector("#wiki-overlay");
  wikiIndice = document.querySelector("#wiki-indice");
  wikiArticulo = document.querySelector("#wiki-articulo");
  pausaOverlay = document.querySelector("#pausa-overlay");
  pantallaMenu = document.querySelector("#pantalla-menu");
  appShell = document.querySelector("#app-shell");
  pantallaHub = document.querySelector("#pantalla-hub");
  pantallaConsola = document.querySelector("#pantalla-consola");
  btnCargarPartida = document.querySelector("#btn-cargar-partida");
  ticketRetrato = document.querySelector("#ticket-retrato");
  consolaTitulo = document.querySelector("#consola-titulo");
  resultRunningEl = document.querySelector("#result-running");
  ventanaConsolaEl = document.querySelector("#pantalla-consola .ventana-terminal");
  btnMuteMusica = document.querySelector("#btn-mute-musica");
  btnMuteEfectos = document.querySelector("#btn-mute-efectos");
  btnCerrarScoring = document.querySelector("#btn-cerrar-scoring");
  btnSaltarTutorial = document.querySelector("#btn-saltar-tutorial");
  headerAppShell = document.querySelector("#header-app-shell");
  dineroHubEl = document.querySelector("#dinero-hub");
  reputacionHubEl = document.querySelector("#reputacion-hub");
  rangoPerfilEl = document.querySelector("#rango-perfil");
  progresoRangoActualTextoEl = document.querySelector("#progreso-rango-actual-texto");
  progresoRangoSiguienteEl = document.querySelector("#progreso-rango-siguiente");
  sueldoPerfilEl = document.querySelector("#sueldo-perfil");
  sueldoPerfilNotaEl = document.querySelector("#sueldo-perfil-nota");
  actualizarSueldoPerfil(0);
  tooltipGlobal = document.querySelector("#tooltip-global");
  empresaNombreEl = document.querySelector("#empresa-nombre");
  empresaDescripcionEl = document.querySelector("#empresa-descripcion");
  panelArcoCaminoEl = document.querySelector("#panel-arco-camino");
  panelArcoFillEl = document.querySelector("#panel-arco-fill");
  panelArcoRepEl = document.querySelector("#panel-arco-rep");
  panelArcoTurnoEl = document.querySelector("#panel-arco-turno");
  panelArcoLabelEl = document.querySelector("#panel-arco-label");
  ticketToastEl = document.querySelector("#ticket-toast");
  bossBannerEl = document.querySelector("#boss-banner");
  dineroHubPopEl = document.querySelector("#dinero-hub-pop");
  dineroPendienteHubEl = document.querySelector("#dinero-pendiente-hub");
  retratoJugadorEl = document.querySelector("#retrato-jugador");
  debugRetratoEtiquetaEl = document.querySelector("#debug-retrato-etapa");
  faxOficinaTextoEl = document.querySelector("#fax-oficina-texto");
  document.querySelector(".fax-oficina")?.addEventListener("click", () => {
    rotarFaxOficina(true);
  });
  const hubDelsy = document.querySelector("#hub-delsy-retrato");
  if (hubDelsy) hubDelsy.innerHTML = retratoDelsyHabla();
  reputacionHubPopEl = document.querySelector("#reputacion-hub-pop");
  actualizarRetratoJugador();
  rotarPostitsOficina(false);
  rotarFaxOficina(false);
  ticketIntentosEl = document.querySelector("#ticket-intentos");

  await mostrarMenu();

  document.querySelector("#btn-play").addEventListener("click", () => {
    runQuery();
    notificarClicPlay();
  });
  document.querySelector("#btn-ejecutar-todas").addEventListener("click", runAllQueries);
  document.querySelector("#btn-deshacer-sql")?.addEventListener("click", () => {
    if (deshacerSql(sqlInput)) {
      setStatus("Deshecho (Red de Seguridad).", "ok");
      const listaSug = document.querySelector("#sql-sugerencias");
      actualizarSugerenciasSql(sqlInput, listaSug);
    } else {
      setStatus("Nada que deshacer.", "");
    }
  });
  btnSubmit = document.querySelector("#btn-submit");
  btnSubmit.addEventListener("click", async () => {
    const exito = await submitTicket();
    if (exito) {
      notificarClicEnviar();
    }
  });
  document.querySelector("#btn-cerrar-dia").addEventListener("click", cerrarDia);
  btnCerrarScoring.addEventListener("click", () => {
    scoringOverlay.classList.add("oculto");
    mostrarPantalla("hub");
    limpiarEstadoConsola();
    aplicarFeedbackEnHub();
    notificarCierreScoring();
    considerarSubtramaEmpleo();
  });
  document.querySelector("#btn-confirmar-agencia").addEventListener("click", confirmarTransicionAgencia);
  document.querySelector("#btn-iniciar-partida").addEventListener("click", iniciarPartida);
  document.querySelector("#btn-cargar-partida").addEventListener("click", cargarPartida);
  document.querySelector("#btn-salir-juego-menu").addEventListener("click", () => invoke("salir_del_juego"));
  document.querySelector("#btn-salir-juego-pausa").addEventListener("click", () => invoke("salir_del_juego"));
  document.querySelector("#btn-volver-hub").addEventListener("click", () => {
    ticketActivoId = null;
    ticketActivoArquetipos = [];
    ticketActivoInfo.textContent = "Elige un ticket de la bandeja para empezar.";
    actualizarEtiquetaIntentos(null);
    pintarPistaInstinto([]);
    limpiarEstadoConsola();
    mostrarPantalla("hub");
  });

  document.addEventListener("click", (evento) => {
    if (evento.target.closest("button")) {
      iniciarAmbiente();
      sfxClick();
    }
  });

  const listaSugerencias = document.querySelector("#sql-sugerencias");
  sqlInput.addEventListener("keydown", (evento) => {
    sfxTecleo();
    if (!perkEquipado("piloto_automatico") || !listaSugerencias) return;
    const abierta = !listaSugerencias.classList.contains("oculto");
    const items = [...listaSugerencias.querySelectorAll("li")];
    if (abierta && items.length > 0 && (evento.key === "Tab" || evento.key === "Enter")) {
      const activa = listaSugerencias.querySelector("li.es-activa") || items[0];
      evento.preventDefault();
      aplicarSugerenciaSql(sqlInput, activa.dataset.sug);
      listaSugerencias.classList.add("oculto");
      return;
    }
    if (abierta && evento.key === "Escape") {
      listaSugerencias.classList.add("oculto");
    }
  });
  sqlInput.addEventListener("input", () => {
    empujarHistorialSql(sqlInput.value);
    notificarSqlCambiado(sqlInput.value);
    actualizarSugerenciasSql(sqlInput, listaSugerencias);
  });
  listaSugerencias?.addEventListener("mousedown", (evento) => {
    const li = evento.target.closest("li[data-sug]");
    if (!li) return;
    evento.preventDefault();
    aplicarSugerenciaSql(sqlInput, li.dataset.sug);
    listaSugerencias.classList.add("oculto");
  });
  document.addEventListener("click", (evento) => {
    if (!evento.target.closest(".sql-editor-wrap")) {
      listaSugerencias?.classList.add("oculto");
    }
  });

  btnMuteMusica.addEventListener("click", () => {
    const activa = alternarMusica();
    btnMuteMusica.textContent = activa ? "🔊" : "🔇";
  });

  btnMuteEfectos.addEventListener("click", () => {
    const activos = alternarEfectos();
    btnMuteEfectos.textContent = activos ? "🔊" : "🔇";
  });

  btnSaltarTutorial.addEventListener("click", () => {
    saltarTutorial();
  });

  document.addEventListener("keydown", (evento) => {
    if (evento.key === "F2") {
      evento.preventDefault();
      alternarDebugOverlay();
    }
    if (evento.key === "Escape") {
      alternarDebugOverlay(false);
    }
  });
  document.querySelector("#debug-aplicar")?.addEventListener("click", () => aplicarDebugEstado(false));
  document.querySelector("#debug-forzar-auditor")?.addEventListener("click", () => aplicarDebugEstado(true));
  document.querySelector("#debug-ciclo-puesto")?.addEventListener("click", () => {
    ciclarPuestoRetratoDebug();
    considerarSubtramaEmpleo();
  });
  document.querySelector("#debug-subtrama")?.addEventListener("click", async () => {
    alternarDebugOverlay(false);
    const cap = await forzarSiguienteCapituloSubtrama();
    if (cap?.titulo) setStatus(`Subtrama: ${cap.titulo}`, "ok");
  });
  document.querySelector("#debug-subtrama-reset")?.addEventListener("click", () => {
    reiniciarSubtramaDebug();
    setStatus("Subtrama reiniciada.", "ok");
  });
  document.querySelector("#debug-cerrar")?.addEventListener("click", () => alternarDebugOverlay(false));

  document.querySelector("#tab-dashboard").addEventListener("click", () => {
    document.querySelector(".hub-columna-bandeja").scrollIntoView({ behavior: "smooth", block: "start" });
  });

  document.querySelector("#tab-perks").addEventListener("click", () => {
    document.querySelector(".hub-columna-perks").scrollIntoView({ behavior: "smooth", block: "start" });
  });

  document.querySelector("#tab-logros").addEventListener("click", () => {
    setStatus("Próximamente.", "");
  });

  document.querySelector("#btn-ver-esquema").addEventListener("click", mostrarEsquema);

  document.querySelector("#tab-base-datos").addEventListener("click", mostrarEsquema);

  document.querySelector("#btn-cerrar-esquema").addEventListener("click", () => {
    esquemaOverlay.classList.add("oculto");
  });

  document.querySelector("#btn-ver-wiki").addEventListener("click", () => mostrarWiki());
  document.querySelector("#tab-wiki").addEventListener("click", () => mostrarWiki("select"));
  document.querySelector("#btn-cerrar-wiki").addEventListener("click", () => {
    wikiOverlay.classList.add("oculto");
  });
  pintarIndiceWiki();
  pintarArticuloWiki("select");

  document.addEventListener("mousemove", (evento) => {
    if (!cajaArrastrando) return;
    const rect = esquemaLienzo.getBoundingClientRect();
    const nuevaX = evento.clientX - rect.left + esquemaLienzo.scrollLeft - offsetArrastreX;
    const nuevaY = evento.clientY - rect.top + esquemaLienzo.scrollTop - offsetArrastreY;
    posicionesActuales[cajaArrastrando] = { x: nuevaX, y: nuevaY };
    const caja = esquemaLienzo.querySelector(`[data-tabla="${cajaArrastrando}"]`);
    caja.style.left = `${nuevaX}px`;
    caja.style.top = `${nuevaY}px`;
    dibujarRelaciones(esquemaRelacionesActuales);
  });

  document.addEventListener("mouseup", () => {
    cajaArrastrando = null;
  });

  document.querySelector("#btn-guardar-pausa").addEventListener("click", () => {
    setStatus("Partida guardada.", "ok");
  });

  document.querySelector("#btn-salir-pausa").addEventListener("click", async () => {
    pausaOverlay.classList.add("oculto");
    await mostrarMenu();
  });

  document.querySelector("#btn-continuar-pausa").addEventListener("click", () => {
    pausaOverlay.classList.add("oculto");
  });

  document.addEventListener("keydown", (evento) => {
    if (evento.key !== "Escape") return;
    if (appShell.classList.contains("oculto")) return;
    if (tutorialActivo() || subtramaActiva()) return;
    const hayOverlayResultado = !scoringOverlay.classList.contains("oculto");
    const hayOverlayAgencia = !agenciaOverlay.classList.contains("oculto");
    const hayOverlayCerrarDia = cerrarDiaOverlay && !cerrarDiaOverlay.classList.contains("oculto");
    if (hayOverlayResultado || hayOverlayAgencia || hayOverlayCerrarDia) return;
    pausaOverlay.classList.toggle("oculto");
  });

  document.addEventListener("mouseover", (evento) => {
    const elConTooltip = evento.target.closest("[data-tooltip]");
    if (elConTooltip) {
      tooltipGlobal.textContent = elConTooltip.dataset.tooltip;
      tooltipGlobal.classList.remove("oculto");
    }
  });

  document.addEventListener("mousemove", (evento) => {
    if (!tooltipGlobal.classList.contains("oculto")) {
      tooltipGlobal.style.left = `${evento.clientX + 16}px`;
      tooltipGlobal.style.top = `${evento.clientY + 16}px`;
    }
  });

  document.addEventListener("mouseout", (evento) => {
    const saliendoDe = evento.target.closest("[data-tooltip]");
    const entrandoA = evento.relatedTarget && evento.relatedTarget.closest ? evento.relatedTarget.closest("[data-tooltip]") : null;
    if (saliendoDe && saliendoDe !== entrandoA) {
      tooltipGlobal.classList.add("oculto");
    }
  });
});
