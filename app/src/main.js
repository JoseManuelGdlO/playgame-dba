import { sfxClick, sfxTecleo, sfxCierreDia, iniciarAmbiente, alternarMusica, alternarEfectos } from "./audio.js";

const { invoke } = window.__TAURI__.core;

let sqlInput, statusMsg, resultTable, dineroEl, reputacionEl, rangoEl;
let listaPerks, perksEquipadosMsg;
let presupuestoEl, listaTickets, ticketActivoInfo, bandejaTitulo;
let scoringOverlay, scoringAscenso, agenciaOverlay;
let pantallaMenu, appShell, pantallaHub, pantallaConsola, btnCargarPartida;
let ticketRetrato, consolaTitulo;
let btnMuteMusica, btnMuteEfectos;

const RETRATOS = {
  generico: `<svg viewBox="0 0 8 8"><rect width="8" height="8" fill="#4a4a3a"/><rect x="2" y="1" width="4" height="3" fill="#8a8266"/><rect x="1" y="4" width="6" height="3" fill="#6b6b52"/><rect x="3" y="2" width="1" height="1" fill="#2a2a1f"/><rect x="5" y="2" width="1" height="1" fill="#2a2a1f"/></svg>`,
  "El Mentor": `<svg viewBox="0 0 8 8"><rect width="8" height="8" fill="#3a3a2e"/><rect x="2" y="1" width="4" height="3" fill="#9a8a7a"/><rect x="1" y="4" width="6" height="3" fill="#7a6a5a"/><rect x="2" y="2" width="4" height="1" fill="#1c1c15"/></svg>`,
  "Auditor de Cumplimiento": `<svg viewBox="0 0 8 8"><rect width="8" height="8" fill="#2a2a35"/><rect x="2" y="1" width="4" height="3" fill="#7a7a8a"/><rect x="1" y="4" width="6" height="3" fill="#5a5a6a"/><rect x="3" y="4" width="2" height="3" fill="#1c1c22"/></svg>`,
};

function retratoParaSolicitante(solicitante) {
  return RETRATOS[solicitante] || RETRATOS.generico;
}

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
}

const TITULO_FASE = {
  MiniBoss: "El Auditor de Cumplimiento quiere verte",
};

const NOMBRE_RANGO = {
  Becario: "Becario",
  AuxiliarDeSistemas: "Auxiliar de Sistemas",
};

function renderRango(rango) {
  rangoEl.textContent = NOMBRE_RANGO[rango] || rango;
}

let ticketActivoId = null;

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
  listaTickets.innerHTML = "";
  estadoTurno.pendientes.forEach((ticket, indice) => {
    const li = document.createElement("li");
    li.className = "papel papel-entrando";
    li.style.animationDelay = `${indice * 60}ms`;
    const info = document.createElement("span");
    info.textContent = `[⏱️ ${ticket.costo_tiempo}] ${ticket.motivo}`;
    const boton = document.createElement("button");
    boton.textContent = ticket.id === ticketActivoId ? "En curso" : "Trabajar en este";
    boton.addEventListener("click", () => seleccionarTicket(ticket));
    li.appendChild(info);
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
  dineroEl.textContent = estadoJuego.dinero;
  reputacionEl.textContent = estadoJuego.reputacion.toFixed(1);
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
    reputacionEl.textContent = "0.0";
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

function mostrarScoring(score) {
  document.querySelector("#scoring-titulo").textContent = score.pass ? "✅ Resuelto" : "❌ Incorrecto";
  animarNumero(document.querySelector("#scoring-correctitud"), score.puntaje_correctitud, 0);
  animarNumero(document.querySelector("#scoring-velocidad"), score.puntaje_velocidad, 0);
  animarNumero(document.querySelector("#scoring-practicas"), score.puntaje_practicas, 0);
  animarNumero(document.querySelector("#scoring-dinero"), score.dinero_ganado, 0);
  animarNumero(document.querySelector("#scoring-reputacion"), score.reputacion_ganada, 1);
  document.querySelector("#scoring-mentor").textContent = score.comentario_mentor || "";
  scoringAscenso.textContent = score.ascendio
    ? `¡Ascendiste a ${NOMBRE_RANGO[score.rango_actual] || score.rango_actual}! +1 slot de perk. Nuevos tickets disponibles.`
    : "";
  scoringOverlay.classList.remove("oculto");
}

async function submitTicket() {
  if (!ticketActivoId) {
    setStatus("Elige un ticket de la bandeja primero.", "error");
    return;
  }
  setStatus("Enviando ticket...", "");
  try {
    const score = await invoke("resolver_ticket", { id: ticketActivoId, sql: sqlInput.value });
    dineroEl.textContent = score.dinero_total;
    reputacionEl.textContent = score.reputacion_total.toFixed(1);
    renderRango(score.rango_actual);
    mostrarScoring(score);
    setStatus(score.mensaje, score.pass ? "ok" : "error");
    await cargarTurno();
  } catch (err) {
    setStatus(String(err), "error");
  }
}

function renderPerks(perks) {
  listaPerks.innerHTML = "";
  perks.forEach((perk, indice) => {
    const li = document.createElement("li");
    li.className = `papel papel-perk papel-entrando ${perk.equipado ? "equipado" : perk.desbloqueado ? "desbloqueado" : ""}`.trim();
    li.style.animationDelay = `${indice * 60}ms`;

    const info = document.createElement("span");
    const estado = perk.equipado ? "⭐ equipado" : perk.desbloqueado ? "✅ desbloqueado" : "🔒 bloqueado";
    info.textContent = `${perk.nombre} (${perk.categoria}) — ${estado} — $${perk.costo_dinero}, ⭐${perk.reputacion_minima}`;

    const boton = document.createElement("button");
    boton.textContent = perk.equipado ? "Desequipar" : perk.desbloqueado ? "Equipar" : "Desbloquear";
    boton.addEventListener("click", () => accionPerk(perk));

    li.appendChild(info);
    li.appendChild(boton);
    listaPerks.appendChild(li);
  });

  const equipados = perks.filter((p) => p.equipado).map((p) => p.nombre);
  perksEquipadosMsg.textContent = equipados.length ? `Equipados: ${equipados.join(", ")}` : "Ningún perk equipado.";
}

async function cargarPerks() {
  const perks = await invoke("catalogo_perks");
  renderPerks(perks);
}

async function accionPerk(perk) {
  try {
    let perks;
    if (perk.equipado) {
      perks = await invoke("desequipar_perk", { id: perk.id });
    } else if (perk.desbloqueado) {
      perks = await invoke("equipar_perk", { id: perk.id });
    } else {
      perks = await invoke("desbloquear_perk", { id: perk.id });
      setStatus("Perk desbloqueado.", "ok");
    }
    renderPerks(perks);
  } catch (err) {
    setStatus(String(err), "error");
  }
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
  pantallaMenu = document.querySelector("#pantalla-menu");
  appShell = document.querySelector("#app-shell");
  pantallaHub = document.querySelector("#pantalla-hub");
  pantallaConsola = document.querySelector("#pantalla-consola");
  btnCargarPartida = document.querySelector("#btn-cargar-partida");
  ticketRetrato = document.querySelector("#ticket-retrato");
  consolaTitulo = document.querySelector("#consola-titulo");
  btnMuteMusica = document.querySelector("#btn-mute-musica");
  btnMuteEfectos = document.querySelector("#btn-mute-efectos");

  await mostrarMenu();

  document.querySelector("#btn-play").addEventListener("click", runQuery);
  document.querySelector("#btn-submit").addEventListener("click", submitTicket);
  document.querySelector("#btn-cerrar-dia").addEventListener("click", cerrarDia);
  document.querySelector("#btn-cerrar-scoring").addEventListener("click", () => {
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
});
