/** Efectos de UI de perks (Instinto, Rayos X, Piloto, Red de Seguridad). */

let perksEquipadosIds = new Set();
/** @type {string[]} */
let sqlHistorial = [];
let silenciarHistorial = false;
/** @type {string[]} */
let vocabularioSql = [
  "SELECT",
  "FROM",
  "WHERE",
  "JOIN",
  "INNER",
  "LEFT",
  "ON",
  "GROUP",
  "BY",
  "ORDER",
  "ASC",
  "DESC",
  "COUNT",
  "SUM",
  "AVG",
  "AS",
  "AND",
  "OR",
  "LIMIT",
];
/** @type {string[]} */
let tablasInstinto = [];

export function perkEquipado(id) {
  return perksEquipadosIds.has(id);
}

export function sincronizarPerksEquipados(vista) {
  perksEquipadosIds = new Set(
    (vista?.perks || []).filter((p) => p.equipado).map((p) => p.id)
  );
  document.body.classList.toggle("perk-rayos-x", perkEquipado("rayos_x"));
  document.body.classList.toggle("perk-piloto", perkEquipado("piloto_automatico"));
  document.body.classList.toggle("perk-red", perkEquipado("red_de_seguridad"));
  const btnUndo = document.querySelector("#btn-deshacer-sql");
  if (btnUndo) btnUndo.classList.toggle("oculto", !perkEquipado("red_de_seguridad"));
}

export function registrarVocabularioEsquema(tablas) {
  const extras = [];
  for (const tabla of tablas || []) {
    if (tabla.nombre) extras.push(tabla.nombre);
    for (const col of tabla.columnas || []) {
      if (col.nombre) extras.push(col.nombre);
    }
  }
  const base = [
    "SELECT",
    "FROM",
    "WHERE",
    "JOIN",
    "INNER",
    "LEFT",
    "ON",
    "GROUP",
    "BY",
    "ORDER",
    "ASC",
    "DESC",
    "COUNT",
    "SUM",
    "AVG",
    "AS",
    "AND",
    "OR",
    "LIMIT",
  ];
  vocabularioSql = [...new Set([...base, ...extras])];
}

export function pintarPistaInstinto(tablas) {
  tablasInstinto = Array.isArray(tablas) ? tablas : [];
  const el = document.querySelector("#pista-instinto");
  if (!el) return;
  if (!perkEquipado("instinto") || tablasInstinto.length === 0) {
    el.classList.add("oculto");
    el.innerHTML = "";
    return;
  }
  el.classList.remove("oculto");
  el.innerHTML = `
    <span class="pista-instinto-etiqueta">Instinto · mira estas tablas</span>
    <div class="pista-instinto-tags">
      ${tablasInstinto.map((t) => `<span class="pista-instinto-tag">${t}</span>`).join("")}
    </div>
  `;
}

export function aplicarResaltadoInstintoEnEsquema(lienzo) {
  if (!lienzo) return;
  lienzo.querySelectorAll(".caja-tabla").forEach((caja) => {
    const nombre = (caja.dataset.tabla || "").toLowerCase();
    const hit = tablasInstinto.some((t) => t.toLowerCase() === nombre);
    caja.classList.toggle("es-pista-instinto", hit && perkEquipado("instinto"));
  });
}

export function empujarHistorialSql(valor) {
  if (!perkEquipado("red_de_seguridad") || silenciarHistorial) return;
  if (sqlHistorial.length === 0 || sqlHistorial[sqlHistorial.length - 1] !== valor) {
    sqlHistorial.push(valor);
    if (sqlHistorial.length > 80) sqlHistorial.shift();
  }
}

export function deshacerSql(sqlInput) {
  if (!perkEquipado("red_de_seguridad") || !sqlInput) return false;
  if (sqlHistorial.length < 2) return false;
  sqlHistorial.pop();
  const anterior = sqlHistorial[sqlHistorial.length - 1] ?? "";
  silenciarHistorial = true;
  sqlInput.value = anterior;
  silenciarHistorial = false;
  return true;
}

export function reiniciarHistorialSql(valorInicial) {
  sqlHistorial = [valorInicial ?? ""];
}

/** Aviso en español sencillo si el SQL parece peligroso o sin sentido. */
export function avisoSqlSospechoso(sql) {
  if (!perkEquipado("red_de_seguridad")) return null;
  const s = (sql || "").trim();
  if (!s) return "No hay nada escrito. ¿Seguro?";
  const upper = s.toUpperCase();
  if (/\b(DROP|DELETE|TRUNCATE|ALTER|UPDATE)\b/.test(upper)) {
    return "Eso puede borrar o cambiar datos. En este juego casi nunca hace falta. ¿Lo ejecutas igual?";
  }
  if (/\b(INSERT|UPDATE)\b/.test(upper) && !/\bWHERE\b/.test(upper) && /\bUPDATE\b/.test(upper)) {
    return "Un cambio sin filtro puede afectar muchas filas. ¿Seguro?";
  }
  if (!/\bSELECT\b/.test(upper) && !/\bWITH\b/.test(upper)) {
    return "No parece una consulta de lectura (SELECT). ¿Lo ejecutas igual?";
  }
  return null;
}

export function actualizarSugerenciasSql(sqlInput, listaEl) {
  if (!perkEquipado("piloto_automatico") || !sqlInput || !listaEl) {
    if (listaEl) listaEl.classList.add("oculto");
    return;
  }
  const valor = sqlInput.value;
  const pos = sqlInput.selectionStart ?? valor.length;
  const antes = valor.slice(0, pos);
  const match = antes.match(/([A-Za-z_][A-Za-z0-9_]*)$/);
  if (!match || match[1].length < 1) {
    listaEl.classList.add("oculto");
    listaEl.innerHTML = "";
    return;
  }
  const prefijo = match[1].toLowerCase();
  const hits = vocabularioSql
    .filter((w) => w.toLowerCase().startsWith(prefijo) && w.toLowerCase() !== prefijo)
    .slice(0, 8);
  if (hits.length === 0) {
    listaEl.classList.add("oculto");
    listaEl.innerHTML = "";
    return;
  }
  listaEl.innerHTML = hits.map((h, i) => `<li data-sug="${h}" class="${i === 0 ? "es-activa" : ""}">${h}</li>`).join("");
  listaEl.classList.remove("oculto");
}

export function aplicarSugerenciaSql(sqlInput, palabra) {
  if (!sqlInput || !palabra) return;
  const valor = sqlInput.value;
  const pos = sqlInput.selectionStart ?? valor.length;
  const antes = valor.slice(0, pos);
  const despues = valor.slice(pos);
  const match = antes.match(/([A-Za-z_][A-Za-z0-9_]*)$/);
  if (!match) return;
  const inicio = pos - match[1].length;
  const nuevo = valor.slice(0, inicio) + palabra + despues;
  empujarHistorialSql(valor);
  silenciarHistorial = true;
  sqlInput.value = nuevo;
  const caret = inicio + palabra.length;
  sqlInput.setSelectionRange(caret, caret);
  silenciarHistorial = false;
  empujarHistorialSql(nuevo);
}
