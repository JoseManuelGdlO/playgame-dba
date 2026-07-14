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
let btnSaltarTutorial;
let headerAppShell, dineroHubEl, reputacionHubEl;
let rangoPerfilEl, progresoRangoActualTextoEl, progresoRangoSiguienteEl;
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

  rangoActual = rango;
}

function actualizarDinero(valor) {
  dineroEl.textContent = valor;
  dineroHubEl.textContent = valor;
}

function actualizarReputacion(valorFormateado) {
  reputacionEl.textContent = valorFormateado;
  reputacionHubEl.textContent = valorFormateado;
  reputacionActual = Number.parseFloat(valorFormateado) || 0;
}

let ticketActivoId = null;
let ticketActivoArquetipos = [];
let wikiArticuloActual = "select";

const UMBRAL_ASCENSO_AUXILIAR = 2.5;

/** @type {{ titulo: string, pass: boolean, deltaDinero: number, deltaRep: number, ascendio: boolean } | null} */
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

function renderResultados(resultados) {
  resultTable.innerHTML = "";
  const mostrarEtiquetas = resultados.length > 1;
  resultados.forEach((resultado, indice) => {
    const bloque = document.createElement("div");
    bloque.className = "resultado-bloque";
    if (mostrarEtiquetas) {
      const etiqueta = document.createElement("h3");
      etiqueta.className = "resultado-etiqueta";
      etiqueta.textContent = `Resultado ${indice + 1}`;
      bloque.appendChild(etiqueta);
    }
    if (resultado.error) {
      const error = document.createElement("p");
      error.className = "resultado-error";
      error.textContent = resultado.error;
      bloque.appendChild(error);
    } else {
      bloque.appendChild(crearTablaFilas(resultado.rows));
    }
    resultTable.appendChild(bloque);
  });
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

async function runQuery() {
  setStatus("Ejecutando...", "");
  try {
    const result = await invoke("run_query", { sql: textoAEjecutar() });
    setStatus(`OK — ${result.rows.length} fila(s)`, "ok");
    renderResultados([{ rows: result.rows }]);
  } catch (err) {
    setStatus(String(err), "error");
    resultTable.innerHTML = "";
  }
}

async function runAllQueries() {
  const sentencias = dividirSentencias(sqlInput.value);
  if (sentencias.length === 0) {
    setStatus("No hay ninguna consulta que ejecutar.", "error");
    return;
  }
  setStatus("Ejecutando...", "");
  const resultados = [];
  let errores = 0;
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
}

function seleccionarTicket(ticket) {
  ticketActivoId = ticket.id;
  ticketActivoMotivo = ticket.motivo || ticket.id;
  ticketActivoArquetipos = Array.isArray(ticket.arquetipos) ? [...ticket.arquetipos] : [];
  ticketActivoInfo.textContent = `Motivo: ${ticket.motivo}\nSolicitud: ${ticket.solicitud}`;
  actualizarEtiquetaIntentos(ticket.id);
  sqlInput.value = tutorialActivo() ? "" : (ticket.sql_inicial || "SELECT * FROM pacientes;");
  ticketRetrato.innerHTML = retratoParaSolicitante(ticket.solicitante);
  consolaTitulo.textContent = `query-path — ${ticket.id}`;
  mostrarPantalla("consola");
  notificarClicPrimerTicket();
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
    "Este ticket necesita un JOIN: une filas de dos tablas con una clave en común. INNER JOIN (o solo JOIN) deja solo las filas que coinciden en ambas. LEFT JOIN conserva todas las de la izquierda aunque no haya pareja a la derecha.",
    {
      permitir: ["#ticket-activo-info", "#sql-input"],
      alContinuar: () => {
        mostrarDialogo(
          retrato,
          "El Mentor",
          "Ejemplo: FROM pacientes p JOIN departamentos d ON p.departamento_id = d.id. Si ambas tablas tienen una columna nombre, usa AS paciente / AS departamento para no pisarlas. Lee la solicitud, completa el filtro y el ORDER BY, prueba con ▶ Play y envía.",
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
    "Este ticket pide agregación: COUNT, SUM, AVG… con GROUP BY. GROUP BY agrupa filas que comparten un valor (por ejemplo el tipo) y la función calcula un número por grupo.",
    {
      permitir: ["#ticket-activo-info", "#sql-input"],
      alContinuar: () => {
        mostrarDialogo(
          retrato,
          "El Mentor",
          "Ejemplo: SELECT tipo, COUNT(*) AS total FROM tratamientos GROUP BY tipo ORDER BY total DESC. Todo lo que va en el SELECT que no sea agregación debe ir también en el GROUP BY. Completa la query del ticket, pruébala y envíala.",
          {
            permitir: ["#sql-input", "#btn-play", "#btn-submit", "#btn-ver-wiki", "#btn-ver-esquema", "#wiki-overlay", "#esquema-overlay"],
            alContinuar: alTerminar,
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
      <h3>SELECT — pedir columnas</h3>
      <p>Lee filas de una tabla. Eliges qué columnas mostrar y de qué tabla.</p>
      <code class="wiki-ejemplo">SELECT nombre, fecha_ingreso
FROM pacientes;</code>
      <p><code>*</code> trae todas las columnas. Mejor listar solo lo que pide el ticket.</p>
      <p class="wiki-tip">Tip: en el hospital, empieza con SELECT y mira el resultado con ▶ Play antes de filtrar.</p>
    `,
  },
  {
    id: "where",
    titulo: "WHERE",
    html: `
      <h3>WHERE — filtrar filas</h3>
      <p>Deja solo las filas que cumplen una condición. Sin WHERE, ves toda la tabla.</p>
      <code class="wiki-ejemplo">SELECT nombre, tipo
FROM tratamientos
WHERE tipo = 'cirugia';</code>
      <ul>
        <li><code>=</code>, <code>&lt;&gt;</code>, <code>&lt;</code>, <code>&gt;</code> comparan valores</li>
        <li><code>AND</code> / <code>OR</code> combinan condiciones</li>
        <li>Textos van entre comillas simples: <code>'urgencia'</code></li>
      </ul>
    `,
  },
  {
    id: "order-by",
    titulo: "ORDER BY",
    html: `
      <h3>ORDER BY — ordenar el resultado</h3>
      <p>Ordena las filas del resultado. Muchos tickets exigen un orden concreto.</p>
      <code class="wiki-ejemplo">SELECT nombre, fecha_ingreso
FROM pacientes
ORDER BY fecha_ingreso DESC, nombre;</code>
      <ul>
        <li><code>ASC</code> = ascendente (por defecto)</li>
        <li><code>DESC</code> = descendente</li>
        <li>Puedes ordenar por varias columnas, en ese orden de prioridad</li>
      </ul>
    `,
  },
  {
    id: "join",
    titulo: "JOIN",
    html: `
      <h3>JOIN — unir tablas</h3>
      <p>Combina filas de dos tablas usando una clave en común (por ejemplo <code>departamento_id</code> ↔ <code>id</code>).</p>
      <code class="wiki-ejemplo">SELECT p.nombre AS paciente, d.nombre AS departamento
FROM pacientes p
JOIN departamentos d ON p.departamento_id = d.id
ORDER BY p.nombre;</code>
      <ul>
        <li><strong>INNER JOIN</strong> (o solo <code>JOIN</code>): solo filas con pareja en ambas tablas</li>
        <li><strong>LEFT JOIN</strong>: todas las de la izquierda, aunque no haya pareja a la derecha</li>
      </ul>
      <p class="wiki-tip">Si ambas tablas tienen <code>nombre</code>, renómbralas con <code>AS</code> para no pisar columnas.</p>
    `,
  },
  {
    id: "group-by",
    titulo: "GROUP BY",
    html: `
      <h3>GROUP BY — agrupar y contar</h3>
      <p>Agrupa filas que comparten un valor y calcula un número por grupo con <code>COUNT</code>, <code>SUM</code>, <code>AVG</code>…</p>
      <code class="wiki-ejemplo">SELECT tipo, COUNT(*) AS total
FROM tratamientos
GROUP BY tipo
ORDER BY total DESC;</code>
      <ul>
        <li>Todo lo del <code>SELECT</code> que no sea agregación debe ir también en el <code>GROUP BY</code></li>
        <li><code>COUNT(*)</code> cuenta filas del grupo; <code>COUNT(columna)</code> ignora NULLs</li>
      </ul>
    `,
  },
  {
    id: "alias",
    titulo: "Alias (AS)",
    html: `
      <h3>Alias — renombrar tablas y columnas</h3>
      <p>Un alias acorta nombres y evita choques cuando dos tablas tienen la misma columna.</p>
      <code class="wiki-ejemplo">SELECT p.nombre AS paciente, d.nombre AS departamento
FROM pacientes AS p
JOIN departamentos AS d ON p.departamento_id = d.id;</code>
      <p>Después del alias, usa ese nombre corto en el resto de la consulta (<code>p.</code>, <code>d.</code>).</p>
    `,
  },
  {
    id: "intentos",
    titulo: "Tickets e intentos",
    html: `
      <h3>Cómo entregar un ticket</h3>
      <ol style="margin:0 0 0.7rem;padding-left:1.15rem;font-size:0.82rem;line-height:1.45">
        <li>Lee el motivo y la solicitud (qué columnas y qué orden pide)</li>
        <li>Consulta el <strong>esquema</strong> si no recuerdas las tablas</li>
        <li>Escribe la query y pruébala con <strong>▶ Play</strong></li>
        <li>Cuando el resultado cuadre, <strong>✓ Enviar ticket</strong></li>
      </ol>
      <p>Cada ticket tiene intentos limitados. Un fallo gasta un intento; si se acaban, el ticket se pierde.</p>
      <p class="wiki-tip">Esta wiki y el esquema están en las pestañas del hub y también en la consola del ticket.</p>
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
  const lineaResultado = feedback.pass ? "Resuelto" : "Incorrecto";
  const partes = [`${lineaResultado} · ${feedback.titulo}`];
  if (feedback.pass) {
    const repTxt = Number(feedback.deltaRep).toFixed(1);
    partes.push(`+$${feedback.deltaDinero} · +${repTxt} rep`);
  }
  ticketToastEl.textContent = partes.join("\n");
  ticketToastEl.classList.toggle("es-fallo", !feedback.pass);
  ticketToastEl.classList.remove("oculto");
  if (toastTimer) clearTimeout(toastTimer);
  toastTimer = setTimeout(() => ticketToastEl.classList.add("oculto"), 3000);
}

function aplicarFeedbackEnHub() {
  if (!ultimoFeedback) return;
  const feedback = ultimoFeedback;
  ultimoFeedback = null;

  mostrarToastTicket(feedback);

  if (feedback.pass && feedback.deltaDinero !== 0) {
    mostrarPopBadge(dineroHubPopEl, `+$${feedback.deltaDinero}`);
  }
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
  actualizarDinero(estadoJuego.dinero);
  actualizarReputacion(estadoJuego.reputacion.toFixed(1));
  renderRango(estadoJuego.rango);
  renderBandeja(estadoJuego);
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
  pintarHubDesdeEstadoJuego(estadoJuego);
  await cargarPerks();
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
    });
  }
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
    return false;
  }
  if (enviandoTicket) {
    return false;
  }
  enviandoTicket = true;
  if (btnSubmit) btnSubmit.disabled = true;
  setStatus("Enviando ticket...", "");
  try {
    const score = await invoke("resolver_ticket", { id: ticketActivoId, sql: sqlInput.value });
    if (score.intentos_restantes) {
      setStatus(score.mensaje, "error");
      await cargarTurno();
      actualizarEtiquetaIntentos(ticketActivoId);
      return false;
    }
    ultimoFeedback = {
      titulo: ticketActivoMotivo || ticketActivoId || "Ticket",
      pass: score.pass,
      deltaDinero: score.dinero_ganado,
      deltaRep: score.reputacion_ganada,
      ascendio: score.ascendio,
    };
    actualizarDinero(score.dinero_total);
    actualizarReputacion(score.reputacion_total.toFixed(1));
    renderRango(score.rango_actual);
    mostrarScoring(score);
    setStatus(score.mensaje, score.pass ? "ok" : "error");
    await cargarTurno();
    return true;
  } catch (err) {
    setStatus(String(err), "error");
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
  reputacionHubPopEl = document.querySelector("#reputacion-hub-pop");
  ticketIntentosEl = document.querySelector("#ticket-intentos");

  await mostrarMenu();

  document.querySelector("#btn-play").addEventListener("click", () => {
    runQuery();
    notificarClicPlay();
  });
  document.querySelector("#btn-ejecutar-todas").addEventListener("click", runAllQueries);
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
    aplicarFeedbackEnHub();
    notificarCierreScoring();
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
    mostrarPantalla("hub");
  });

  document.addEventListener("click", (evento) => {
    if (evento.target.closest("button")) {
      iniciarAmbiente();
      sfxClick();
    }
  });

  sqlInput.addEventListener("keydown", () => sfxTecleo());
  sqlInput.addEventListener("input", () => notificarSqlCambiado(sqlInput.value));

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
    if (tutorialActivo()) return;
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
