const { invoke } = window.__TAURI__.core;

let sqlInput, statusMsg, resultTable, dineroEl, perkIndicator, ticketEnunciado;

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
    setStatus(score.mensaje, score.pass ? "ok" : "error");
  } catch (err) {
    setStatus(String(err), "error");
  }
}

async function unlockPerk() {
  try {
    const status = await invoke("unlock_perk");
    dineroEl.textContent = status.dinero_total;
    if (status.unlocked) {
      perkIndicator.textContent = "✅ Perk: Café Cargado";
      perkIndicator.className = "perk-unlocked";
    }
  } catch (err) {
    setStatus(String(err), "error");
  }
}

window.addEventListener("DOMContentLoaded", async () => {
  sqlInput = document.querySelector("#sql-input");
  statusMsg = document.querySelector("#status-msg");
  resultTable = document.querySelector("#result-table");
  dineroEl = document.querySelector("#dinero");
  perkIndicator = document.querySelector("#perk-indicator");
  ticketEnunciado = document.querySelector("#ticket-enunciado");

  const ticket = await invoke("ticket_actual");
  ticketEnunciado.textContent = `Motivo: ${ticket.motivo}\nSolicitud: ${ticket.solicitud}`;
  sqlInput.value = ticket.sql_inicial || "SELECT * FROM pacientes;";

  document.querySelector("#btn-play").addEventListener("click", runQuery);
  document.querySelector("#btn-submit").addEventListener("click", submitTicket);
  document.querySelector("#btn-unlock-perk").addEventListener("click", unlockPerk);
});
