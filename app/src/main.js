const { invoke } = window.__TAURI__.core;

let sqlInput, statusMsg, resultTable, dineroEl, reputacionEl, ticketEnunciado;
let perksSelect, perksEquipadosMsg;

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

async function runQuery() {
  setStatus("Ejecutando...", "");
  try {
    const result = await invoke("run_query", { sql: sqlInput.value });
    setStatus(`OK — ${result.rows.length} fila(s)`, "ok");
    renderRows(result.rows);
  } catch (err) {
    setStatus(String(err), "error");
    resultTable.innerHTML = "";
  }
}

async function submitTicket() {
  setStatus("Enviando ticket...", "");
  try {
    const score = await invoke("submit_ticket", { sql: sqlInput.value });
    dineroEl.textContent = score.dinero_total;
    reputacionEl.textContent = score.reputacion_total.toFixed(1);
    setStatus(score.mensaje, score.pass ? "ok" : "error");
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
  ticketEnunciado = document.querySelector("#ticket-enunciado");
  perksSelect = document.querySelector("#perks-select");
  perksEquipadosMsg = document.querySelector("#perks-equipados-msg");

  const ticket = await invoke("ticket_actual");
  ticketEnunciado.textContent = `Motivo: ${ticket.motivo}\nSolicitud: ${ticket.solicitud}`;
  sqlInput.value = ticket.sql_inicial || "SELECT * FROM pacientes;";

  await cargarPerks();

  document.querySelector("#btn-play").addEventListener("click", runQuery);
  document.querySelector("#btn-submit").addEventListener("click", submitTicket);
  document.querySelector("#btn-unlock-perk").addEventListener("click", desbloquearPerkSeleccionado);
  document.querySelector("#btn-equip-perk").addEventListener("click", equiparODesequiparPerkSeleccionado);
});
