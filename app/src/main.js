import { sfxClick, sfxTecleo, sfxCierreDia, sfxTick, sfxExito, sfxError, sfxAscenso, iniciarAmbiente, alternarMusica, alternarEfectos } from "./audio.js";

const { invoke } = window.__TAURI__.core;

let sqlInput, statusMsg, resultTable, dineroEl, reputacionEl, rangoEl;
let listaPerks, perksEquipadosMsg;
let presupuestoEl, listaTickets, ticketActivoInfo, bandejaTitulo;
let scoringOverlay, scoringAscenso, agenciaOverlay;
let pantallaMenu, appShell, pantallaHub, pantallaConsola, btnCargarPartida;
let pausaOverlay;
let ticketRetrato, consolaTitulo;
let btnMuteMusica, btnMuteEfectos;
let btnCerrarScoring;
let headerAppShell, dineroHubEl, reputacionHubEl;
let rangoPerfilEl, progresoRangoActualTextoEl, progresoRangoSiguienteEl;
let tooltipGlobal;
let empresaNombreEl, empresaDescripcionEl;
let esquemaOverlay, esquemaLienzo, esquemaSvg;
let posicionesActuales = {};
let esquemaRelacionesActuales = [];

const RETRATOS = {
  generico: `<svg viewBox="0 0 8 8"><rect width="8" height="8" fill="#4a4a3a"/><rect x="2" y="1" width="4" height="3" fill="#8a8266"/><rect x="1" y="4" width="6" height="3" fill="#6b6b52"/><rect x="3" y="2" width="1" height="1" fill="#2a2a1f"/><rect x="5" y="2" width="1" height="1" fill="#2a2a1f"/></svg>`,
  "El Mentor": `<svg viewBox="0 0 8 8"><rect width="8" height="8" fill="#3a3a2e"/><rect x="2" y="1" width="4" height="3" fill="#9a8a7a"/><rect x="1" y="4" width="6" height="3" fill="#7a6a5a"/><rect x="2" y="2" width="4" height="1" fill="#1c1c15"/></svg>`,
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
      "Hospital Arcángel es una cadena hospitalaria donde cada expediente pasa por al menos tres departamentos antes de llegar a quien realmente lo necesita. Nadie recuerda quién aprobó el sistema actual, pero todos coinciden en que cambiarlo tomaría más tiempo del que llevan usándolo.",
  },
  Postafeta: {
    nombre: "Postafeta",
    descripcion:
      "Postafeta mueve paquetes por todo el país con una precisión sorprendente, considerando que nadie ha visto un manual de procesos desde hace años. Las decisiones importantes se toman en un canal de Slack administrado por un becario invisible llamado Kevin — todo viene firmado \"- Kevin\".",
  },
};

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

function renderRango(rango) {
  const nombre = NOMBRE_RANGO[rango] || rango;
  rangoEl.textContent = nombre;
  rangoPerfilEl.textContent = nombre;
  progresoRangoActualTextoEl.textContent = nombre;

  const indiceActual = ORDEN_RANGOS.indexOf(rango);
  const siguienteRango = ORDEN_RANGOS[indiceActual + 1];
  progresoRangoSiguienteEl.textContent = siguienteRango
    ? `➜ ${NOMBRE_RANGO[siguienteRango]}`
    : "Alcanzaste el máximo rango disponible";
}

function actualizarDinero(valor) {
  dineroEl.textContent = valor;
  dineroHubEl.textContent = valor;
}

function actualizarReputacion(valorFormateado) {
  reputacionEl.textContent = valorFormateado;
  reputacionHubEl.textContent = valorFormateado;
}

let ticketActivoId = null;
let empresaActual = null;

function renderRows(rows) {
  resultTable.innerHTML = "";
  if (!rows || rows.length === 0) {
    resultTable.textContent = "(sin filas)";
    return;
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
  resultTable.appendChild(table);
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

async function runQuery() {
  setStatus("Ejecutando...", "");
  try {
    const result = await invoke("run_query", { sql: textoAEjecutar() });
    setStatus(`OK — ${result.rows.length} fila(s)`, "ok");
    renderRows(result.rows);
  } catch (err) {
    setStatus(String(err), "error");
    resultTable.innerHTML = "";
  }
}

function seleccionarTicket(ticket) {
  ticketActivoId = ticket.id;
  ticketActivoInfo.textContent = `Motivo: ${ticket.motivo}\nSolicitud: ${ticket.solicitud}`;
  sqlInput.value = ticket.sql_inicial || "SELECT * FROM pacientes;";
  ticketRetrato.innerHTML = retratoParaSolicitante(ticket.solicitante);
  consolaTitulo.textContent = `query-path — ${ticket.id}`;
  mostrarPantalla("consola");
}

function renderBandeja(estadoTurno) {
  presupuestoEl.textContent = estadoTurno.presupuesto_restante;
  bandejaTitulo.textContent = TITULO_FASE[estadoTurno.fase] || "Bandeja — turno actual";
  empresaActual = estadoTurno.empresa;
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
    detalle.appendChild(info);
    detalle.appendChild(etiquetaPrioridad);
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
  if (estadoTurno.fase === "ArcoCompletado") {
    agenciaOverlay.classList.remove("oculto");
  }
}

async function cargarTurno() {
  const estadoTurno = await invoke("turno_actual");
  renderBandeja(estadoTurno);
}

function pintarHubDesdeEstadoJuego(estadoJuego) {
  actualizarDinero(estadoJuego.dinero);
  actualizarReputacion(estadoJuego.reputacion.toFixed(1));
  renderRango(estadoJuego.rango);
  renderBandeja(estadoJuego);
  ticketActivoId = null;
  mostrarPantalla("hub");
}

async function mostrarMenu() {
  mostrarPantalla("menu");
  const existePartida = await invoke("existe_partida_guardada");
  btnCargarPartida.disabled = !existePartida;
}

async function iniciarPartida() {
  const estadoJuego = await invoke("iniciar_partida");
  pintarHubDesdeEstadoJuego(estadoJuego);
  await cargarPerks();
  setStatus("Partida nueva iniciada.", "ok");
}

async function cargarPartida() {
  try {
    const estadoJuego = await invoke("cargar_partida");
    pintarHubDesdeEstadoJuego(estadoJuego);
    await cargarPerks();
    setStatus("Partida cargada.", "ok");
  } catch (err) {
    setStatus(String(err), "error");
  }
}

async function cerrarDia() {
  const estadoTurno = await invoke("cerrar_dia");
  ticketActivoId = null;
  renderBandeja(estadoTurno);
  setStatus("Día cerrado. Turno nuevo.", "ok");
  sfxCierreDia();
}

async function confirmarTransicionAgencia() {
  try {
    const estadoTurno = await invoke("confirmar_transicion_agencia");
    actualizarReputacion("0.0");
    agenciaOverlay.classList.add("oculto");
    ticketActivoId = null;
    renderBandeja(estadoTurno);
    setStatus("Bienvenido a Postafeta.", "ok");
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
    return;
  }
  setStatus("Enviando ticket...", "");
  try {
    const score = await invoke("resolver_ticket", { id: ticketActivoId, sql: sqlInput.value });
    actualizarDinero(score.dinero_total);
    actualizarReputacion(score.reputacion_total.toFixed(1));
    renderRango(score.rango_actual);
    mostrarScoring(score);
    setStatus(score.mensaje, score.pass ? "ok" : "error");
    await cargarTurno();
  } catch (err) {
    setStatus(String(err), "error");
  }
}

function renderPerks({ perks, max_slots }) {
  listaPerks.innerHTML = "";
  perks.forEach((perk, indice) => {
    const li = document.createElement("li");
    li.className = `papel papel-perk papel-entrando ${perk.equipado ? "equipado" : perk.desbloqueado ? "desbloqueado" : ""}`.trim();
    li.style.animationDelay = `${indice * 60}ms`;
    li.dataset.tooltip = perk.descripcion;

    const ICONO_EQUIPADO = `<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round"><path d="M12 2l2.9 6.3 6.9.6-5.2 4.6 1.6 6.8L12 17l-6.2 3.3 1.6-6.8L2.2 8.9l6.9-.6z"/></svg>`;
    const ICONO_DESBLOQUEADO = `<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><path d="M20 6L9 17l-5-5"/></svg>`;
    const ICONO_BLOQUEADO = `<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linejoin="round"><rect x="5" y="11" width="14" height="9" rx="1"/><path d="M8 11V7a4 4 0 0 1 8 0v4"/></svg>`;

    const info = document.createElement("span");
    const iconoEstado = perk.equipado ? ICONO_EQUIPADO : perk.desbloqueado ? ICONO_DESBLOQUEADO : ICONO_BLOQUEADO;
    const textoEstado = perk.equipado ? "equipado" : perk.desbloqueado ? "desbloqueado" : "bloqueado";
    info.innerHTML = `${iconoEstado} ${perk.nombre} (${perk.categoria}) — ${textoEstado} — $${perk.costo_dinero}, ⭐${perk.reputacion_minima}`;

    const boton = document.createElement("button");
    boton.textContent = perk.equipado ? "Desequipar" : perk.desbloqueado ? "Equipar" : "Desbloquear";
    boton.addEventListener("click", () => accionPerk(perk));

    li.appendChild(info);
    li.appendChild(boton);
    listaPerks.appendChild(li);
  });

  const equipados = perks.filter((p) => p.equipado).map((p) => p.nombre);
  const resumen = `Perks equipados: ${equipados.length}/${max_slots}`;
  perksEquipadosMsg.textContent = equipados.length ? `${resumen} — ${equipados.join(", ")}` : `${resumen} — ninguno equipado.`;
}

async function cargarPerks() {
  const vista = await invoke("catalogo_perks");
  renderPerks(vista);
}

async function accionPerk(perk) {
  try {
    let vista;
    if (perk.equipado) {
      vista = await invoke("desequipar_perk", { id: perk.id });
    } else if (perk.desbloqueado) {
      vista = await invoke("equipar_perk", { id: perk.id });
    } else {
      vista = await invoke("desbloquear_perk", { id: perk.id });
      setStatus("Perk desbloqueado.", "ok");
    }
    renderPerks(vista);
  } catch (err) {
    setStatus(String(err), "error");
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

  dibujarRelaciones(esquema.relaciones);
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
  presupuestoEl = document.querySelector("#presupuesto");
  listaTickets = document.querySelector("#lista-tickets");
  ticketActivoInfo = document.querySelector("#ticket-activo-info");
  bandejaTitulo = document.querySelector("#bandeja-titulo");
  scoringOverlay = document.querySelector("#scoring-overlay");
  scoringAscenso = document.querySelector("#scoring-ascenso");
  agenciaOverlay = document.querySelector("#agencia-overlay");
  esquemaOverlay = document.querySelector("#esquema-overlay");
  esquemaLienzo = document.querySelector("#esquema-lienzo");
  esquemaSvg = document.querySelector("#esquema-svg");
  pausaOverlay = document.querySelector("#pausa-overlay");
  pantallaMenu = document.querySelector("#pantalla-menu");
  appShell = document.querySelector("#app-shell");
  pantallaHub = document.querySelector("#pantalla-hub");
  pantallaConsola = document.querySelector("#pantalla-consola");
  btnCargarPartida = document.querySelector("#btn-cargar-partida");
  ticketRetrato = document.querySelector("#ticket-retrato");
  consolaTitulo = document.querySelector("#consola-titulo");
  btnMuteMusica = document.querySelector("#btn-mute-musica");
  btnMuteEfectos = document.querySelector("#btn-mute-efectos");
  btnCerrarScoring = document.querySelector("#btn-cerrar-scoring");
  headerAppShell = document.querySelector("#header-app-shell");
  dineroHubEl = document.querySelector("#dinero-hub");
  reputacionHubEl = document.querySelector("#reputacion-hub");
  rangoPerfilEl = document.querySelector("#rango-perfil");
  progresoRangoActualTextoEl = document.querySelector("#progreso-rango-actual-texto");
  progresoRangoSiguienteEl = document.querySelector("#progreso-rango-siguiente");
  tooltipGlobal = document.querySelector("#tooltip-global");
  empresaNombreEl = document.querySelector("#empresa-nombre");
  empresaDescripcionEl = document.querySelector("#empresa-descripcion");

  await mostrarMenu();

  document.querySelector("#btn-play").addEventListener("click", runQuery);
  document.querySelector("#btn-submit").addEventListener("click", submitTicket);
  document.querySelector("#btn-cerrar-dia").addEventListener("click", cerrarDia);
  btnCerrarScoring.addEventListener("click", () => {
    scoringOverlay.classList.add("oculto");
    mostrarPantalla("hub");
  });
  document.querySelector("#btn-confirmar-agencia").addEventListener("click", confirmarTransicionAgencia);
  document.querySelector("#btn-iniciar-partida").addEventListener("click", iniciarPartida);
  document.querySelector("#btn-cargar-partida").addEventListener("click", cargarPartida);
  document.querySelector("#btn-volver-hub").addEventListener("click", () => {
    ticketActivoId = null;
    ticketActivoInfo.textContent = "Elige un ticket de la bandeja para empezar.";
    mostrarPantalla("hub");
  });

  document.addEventListener("click", (evento) => {
    if (evento.target.closest("button")) {
      iniciarAmbiente();
      sfxClick();
    }
  });

  sqlInput.addEventListener("keydown", () => sfxTecleo());

  btnMuteMusica.addEventListener("click", () => {
    const activa = alternarMusica();
    btnMuteMusica.textContent = activa ? "🔊" : "🔇";
  });

  btnMuteEfectos.addEventListener("click", () => {
    const activos = alternarEfectos();
    btnMuteEfectos.textContent = activos ? "🔊" : "🔇";
  });

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
    const hayOverlayResultado = !scoringOverlay.classList.contains("oculto");
    const hayOverlayAgencia = !agenciaOverlay.classList.contains("oculto");
    if (hayOverlayResultado || hayOverlayAgencia) return;
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
