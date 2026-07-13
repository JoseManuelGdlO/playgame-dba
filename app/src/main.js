const { invoke } = window.__TAURI__.core;

let sqlInput, statusMsg, resultTable, dineroEl, reputacionEl, rangoEl;
let perksSelect, perksEquipadosMsg;
let presupuestoEl, listaTickets, ticketActivoInfo;
let scoringOverlay, scoringAscenso;

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
}

function renderBandeja(estadoTurno) {
  presupuestoEl.textContent = estadoTurno.presupuesto_restante;
  listaTickets.innerHTML = "";
  for (const ticket of estadoTurno.pendientes) {
    const li = document.createElement("li");
    const info = document.createElement("span");
    info.textContent = `[⏱️ ${ticket.costo_tiempo}] ${ticket.motivo}`;
    const boton = document.createElement("button");
    boton.textContent = ticket.id === ticketActivoId ? "En curso" : "Trabajar en este";
    boton.addEventListener("click", () => seleccionarTicket(ticket));
    li.appendChild(info);
    li.appendChild(boton);
    listaTickets.appendChild(li);
  }
  if (!estadoTurno.pendientes.some((t) => t.id === ticketActivoId)) {
    ticketActivoId = null;
    ticketActivoInfo.textContent = "Elige un ticket de la bandeja para empezar.";
  }
}

async function cargarTurno() {
  const estadoTurno = await invoke("turno_actual");
  renderBandeja(estadoTurno);
}

async function cargarRango() {
  const rango = await invoke("rango_actual");
  renderRango(rango);
}

async function cerrarDia() {
  const estadoTurno = await invoke("cerrar_dia");
  ticketActivoId = null;
  renderBandeja(estadoTurno);
  setStatus("Día cerrado. Turno nuevo.", "ok");
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
  const seleccionado = perksSelect.value;
  perksSelect.innerHTML = "";
  for (const perk of perks) {
    const opt = document.createElement("option");
    opt.value = perk.id;
    const estado = perk.equipado ? "⭐ equipado" : perk.desbloqueado ? "✅ desbloqueado" : "🔒 bloqueado";
    opt.textContent = `${perk.nombre} (${perk.categoria}) — ${estado} — $${perk.costo_dinero}, ⭐${perk.reputacion_minima}`;
    perksSelect.appendChild(opt);
  }
  if (seleccionado) perksSelect.value = seleccionado;

  const equipados = perks.filter((p) => p.equipado).map((p) => p.nombre);
  perksEquipadosMsg.textContent = equipados.length ? `Equipados: ${equipados.join(", ")}` : "Ningún perk equipado.";
}

async function cargarPerks() {
  const perks = await invoke("catalogo_perks");
  renderPerks(perks);
}

async function desbloquearPerkSeleccionado() {
  const id = perksSelect.value;
  if (!id) return;
  try {
    const perks = await invoke("desbloquear_perk", { id });
    renderPerks(perks);
    setStatus("Perk desbloqueado.", "ok");
  } catch (err) {
    setStatus(String(err), "error");
  }
}

async function equiparODesequiparPerkSeleccionado() {
  const id = perksSelect.value;
  if (!id) return;
  const actual = (await invoke("catalogo_perks")).find((p) => p.id === id);
  try {
    const perks = actual && actual.equipado
      ? await invoke("desequipar_perk", { id })
      : await invoke("equipar_perk", { id });
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
  perksSelect = document.querySelector("#perks-select");
  perksEquipadosMsg = document.querySelector("#perks-equipados-msg");
  presupuestoEl = document.querySelector("#presupuesto");
  listaTickets = document.querySelector("#lista-tickets");
  ticketActivoInfo = document.querySelector("#ticket-activo-info");
  scoringOverlay = document.querySelector("#scoring-overlay");
  scoringAscenso = document.querySelector("#scoring-ascenso");

  await cargarTurno();
  await cargarRango();
  await cargarPerks();

  document.querySelector("#btn-play").addEventListener("click", runQuery);
  document.querySelector("#btn-submit").addEventListener("click", submitTicket);
  document.querySelector("#btn-cerrar-dia").addEventListener("click", cerrarDia);
  document.querySelector("#btn-cerrar-scoring").addEventListener("click", () => scoringOverlay.classList.add("oculto"));
  document.querySelector("#btn-unlock-perk").addEventListener("click", desbloquearPerkSeleccionado);
  document.querySelector("#btn-equip-perk").addEventListener("click", equiparODesequiparPerkSeleccionado);
});
