import { mostrarDialogo, ocultarDialogo, permitirSiempre } from "./dialogo.js";

const NOMBRE_MENTOR = "El Mentor";
const SELECTOR_BOTON_SALTAR = "#btn-saltar-tutorial";

let activo = false;
let esperandoCierreScoring = false;
let clausulaObjetivoActual = null;
let pasoActualAlEscribir = null;
let retratoMentorSvg = "";
let callbackAlFinalizar = null;

function normalizar(texto) {
  return texto.toLowerCase().replace(/\s+/g, " ").trim();
}

function mostrarPaso(texto, opciones) {
  mostrarDialogo(retratoMentorSvg, NOMBRE_MENTOR, texto, opciones);
}

function pasoEscribirClausula(texto, clausulaObjetivo, siguientePaso) {
  clausulaObjetivoActual = clausulaObjetivo;
  pasoActualAlEscribir = siguientePaso;
  mostrarPaso(texto, { permitir: ["#sql-input"] });
}

function paso0Bienvenida() {
  mostrarPaso(
    "Bienvenido a tu primer día en Hospital Arcángel. Aquí vas a recibir pedidos reales de otros equipos — tickets — y tu trabajo es resolverlos escribiendo SQL de verdad contra la base de datos de la empresa.",
    { alContinuar: paso1Bandeja }
  );
}

function paso1Bandeja() {
  mostrarPaso("Esa es tu bandeja. Ahí llegan tus pendientes. Dale click al primero para abrir ese ticket.", {
    permitir: ["[data-primer-ticket] button"],
  });
}

function paso2LeerTicket() {
  mostrarPaso(
    "Contabilidad quiere un reporte de los pacientes de Cardiología. Cardiología es el departamento número 1 — vas a pedirle a la base de datos: de la tabla de pacientes, tráeme algunos datos, pero solo los del departamento 1.",
    { permitir: ["#ticket-activo-info"], alContinuar: paso3ClausulaSelect }
  );
}

function paso3ClausulaSelect() {
  pasoEscribirClausula(
    "Empieza diciendo qué columnas quieres ver. Escribe: SELECT nombre, fecha_ingreso, diagnostico",
    "select nombre, fecha_ingreso, diagnostico",
    paso4ClausulaFrom
  );
}

function paso4ClausulaFrom() {
  pasoEscribirClausula(
    "Ahora dile de qué tabla — cada tabla es como una hoja de cálculo, y pacientes es la hoja con un renglón por paciente. Agrega: FROM pacientes",
    "from pacientes",
    paso5ClausulaWhere
  );
}

function paso5ClausulaWhere() {
  pasoEscribirClausula(
    "Contabilidad solo quiere Cardiología, que es el departamento número 1. Agrega: WHERE departamento_id = 1",
    "where departamento_id = 1",
    paso6ClausulaOrderBy
  );
}

function paso6ClausulaOrderBy() {
  pasoEscribirClausula(
    "Y lo quieren del ingreso más reciente al más antiguo. Agrega: ORDER BY fecha_ingreso DESC",
    "order by fecha_ingreso desc",
    paso7Play
  );
}

function paso7Play() {
  clausulaObjetivoActual = null;
  pasoActualAlEscribir = null;
  mostrarPaso("Dale ▶ Play para probarlo contra la base de datos real.", { permitir: ["#btn-play"] });
}

function paso8Enviar() {
  mostrarPaso("Si el resultado se ve bien, dale ✓ Enviar ticket — así es como se resuelve cada encargo en este trabajo.", {
    permitir: ["#btn-submit"],
  });
}

function pasoCierre() {
  mostrarPaso(
    "Bien hecho — ese es tu primer ticket resuelto. El resto de tu bandeja funciona igual: lee lo que piden, escribe la query, pruébala, y envíala. Ahí te dejo.",
    { alContinuar: finalizarTutorial }
  );
}

function finalizarTutorial() {
  activo = false;
  esperandoCierreScoring = false;
  clausulaObjetivoActual = null;
  pasoActualAlEscribir = null;
  permitirSiempre([]);
  ocultarDialogo();
  if (callbackAlFinalizar) callbackAlFinalizar();
}

export function iniciarTutorial(retratoSvg, alFinalizar) {
  activo = true;
  esperandoCierreScoring = false;
  clausulaObjetivoActual = null;
  pasoActualAlEscribir = null;
  retratoMentorSvg = retratoSvg;
  callbackAlFinalizar = alFinalizar || null;
  permitirSiempre([SELECTOR_BOTON_SALTAR]);
  paso0Bienvenida();
}

export function tutorialActivo() {
  return activo;
}

export function saltarTutorial() {
  if (!activo) return;
  finalizarTutorial();
}

export function notificarClicPrimerTicket() {
  if (!activo) return;
  paso2LeerTicket();
}

export function notificarSqlCambiado(valorSql) {
  if (!activo || !clausulaObjetivoActual) return;
  if (normalizar(valorSql).includes(clausulaObjetivoActual)) {
    const siguiente = pasoActualAlEscribir;
    clausulaObjetivoActual = null;
    pasoActualAlEscribir = null;
    siguiente();
    notificarSqlCambiado(valorSql);
  }
}

export function notificarClicPlay() {
  if (!activo) return;
  paso8Enviar();
}

export function notificarClicEnviar() {
  if (!activo) return;
  ocultarDialogo();
  esperandoCierreScoring = true;
}

export function notificarCierreScoring() {
  if (!activo || !esperandoCierreScoring) return;
  esperandoCierreScoring = false;
  pasoCierre();
}
