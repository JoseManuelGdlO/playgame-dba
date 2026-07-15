import { mostrarDialogo, ocultarDialogo, permitirSiempre } from "./dialogo.js";

const NOMBRE_MENTOR = "El Mentor";
const SELECTOR_BOTON_SALTAR = "#btn-saltar-tutorial";

function botonSaltar() {
  return document.querySelector(SELECTOR_BOTON_SALTAR);
}

function mostrarBotonSaltar() {
  const boton = botonSaltar();
  if (boton) boton.classList.remove("oculto");
}

function ocultarBotonSaltar() {
  const boton = botonSaltar();
  if (boton) boton.classList.add("oculto");
}

let activo = false;
let esperandoCierreScoring = false;
let clausulaObjetivoActual = null;
let pasoActualAlEscribir = null;
let retratoMentorSvg = "";
let callbackAlFinalizar = null;
let manejarClicPrimerTicket = null;
let manejarClicPlay = null;
let manejarCierreScoring = null;

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

// --- Tramo A: hospital_reporte_departamentos (Plan 16) ---

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

function pasoConceptoTablaA() {
  mostrarPaso(
    "Antes de escribir nada: una tabla es como una hoja de cálculo. Cada fila es un registro — en este caso, un departamento — y cada columna es un dato de ese registro, como su nombre o en qué piso está.",
    { permitir: ["#ticket-activo-info"], alContinuar: pasoLeerTicketA }
  );
}

function pasoLeerTicketA() {
  mostrarPaso(
    "Recursos Humanos quiere el directorio de áreas: el nombre y el piso de cada departamento. No piden filtrar nada ni ordenarlo — solo mostrar esos dos datos de todos los departamentos.",
    { permitir: ["#ticket-activo-info"], alContinuar: pasoClausulaSelectA }
  );
}

function pasoClausulaSelectA() {
  pasoEscribirClausula(
    "Empieza diciendo qué columnas quieres ver. Escribe: SELECT nombre, piso",
    "select nombre, piso",
    pasoClausulaFromA
  );
}

function pasoClausulaFromA() {
  pasoEscribirClausula("Ahora dile de qué tabla sacar esos datos. Agrega: FROM departamentos", "from departamentos", pasoPlayA);
}

function pasoPlayA() {
  clausulaObjetivoActual = null;
  pasoActualAlEscribir = null;
  manejarClicPlay = pasoEnviarA;
  mostrarPaso("Dale ▶ Play para probarlo contra la base de datos real.", { permitir: ["#btn-play"] });
}

function pasoEnviarA() {
  manejarCierreScoring = pasoTransicionAB;
  mostrarPaso(
    "Si el resultado se ve bien, dale ✓ Enviar ticket — así es como se resuelve cada encargo en este trabajo.",
    { permitir: ["#btn-submit"] }
  );
}

function pasoTransicionAB() {
  manejarClicPrimerTicket = pasoLeerTicketB;
  mostrarPaso(
    "Bien, ya viste tu primer reporte. El siguiente te va a pedir además un filtro — dale click para abrirlo.",
    { permitir: ["[data-primer-ticket] button"] }
  );
}

// --- Tramo B: hospital_reporte_pacientes_cardiologia (Plan 15, ampliado en Plan 16) ---

function pasoLeerTicketB() {
  mostrarPaso(
    "Contabilidad quiere un reporte de los pacientes de Cardiología. Cardiología es el departamento número 1 — vas a pedirle a la base de datos: de la tabla de pacientes, tráeme algunos datos, pero solo los del departamento 1.",
    { permitir: ["#ticket-activo-info"], alContinuar: pasoClausulaSelectB }
  );
}

function pasoClausulaSelectB() {
  pasoEscribirClausula(
    "Empieza diciendo qué columnas quieres ver. Escribe: SELECT nombre, fecha_ingreso, diagnostico",
    "select nombre, fecha_ingreso, diagnostico",
    pasoClausulaFromB
  );
}

function pasoClausulaFromB() {
  pasoEscribirClausula(
    "Ahora dile de qué tabla — cada tabla es como una hoja de cálculo, y pacientes es la hoja con un renglón por paciente. Agrega: FROM pacientes",
    "from pacientes",
    pasoComparaciones
  );
}

function pasoComparaciones() {
  mostrarPaso(
    "Antes de filtrar: un filtro compara cada fila contra una condición. El signo = significa 'igual a' — también existen > y < para comparar números o fechas, aunque este ticket solo necesita =.",
    { permitir: ["#sql-input"], alContinuar: pasoClausulaWhereB }
  );
}

function pasoClausulaWhereB() {
  pasoEscribirClausula(
    "Contabilidad solo quiere Cardiología, que es el departamento número 1. Agrega: WHERE departamento_id = 1",
    "where departamento_id = 1",
    pasoClausulaOrderByB
  );
}

function pasoClausulaOrderByB() {
  pasoEscribirClausula(
    "Y lo quieren del ingreso más reciente al más antiguo. Agrega: ORDER BY fecha_ingreso DESC",
    "order by fecha_ingreso desc",
    pasoPlayB
  );
}

function pasoPlayB() {
  clausulaObjetivoActual = null;
  pasoActualAlEscribir = null;
  manejarClicPlay = pasoEnviarB;
  mostrarPaso("Dale ▶ Play para probarlo contra la base de datos real.", { permitir: ["#btn-play"] });
}

function pasoEnviarB() {
  manejarCierreScoring = pasoTourHub1;
  mostrarPaso(
    "Si el resultado se ve bien, dale ✓ Enviar ticket — así es como se resuelve cada encargo en este trabajo.",
    { permitir: ["#btn-submit"] }
  );
}

// --- Tramo C: tour del Hub (Plan 16) ---

function pasoTourHub1() {
  mostrarPaso(
    "El sueldo de los tickets se cobra al cerrar el día; con ese dinero desbloqueas perks. La reputación además determina qué perks y qué rango puedes alcanzar.",
    { permitir: [".hub-topbar"], alContinuar: pasoTourHub2 }
  );
}

function pasoTourHub2() {
  mostrarPaso(
    "Los perks son bonos permanentes — algunos te dan más dinero o reputación por ticket resuelto. Cada uno cuesta dinero y pide una reputación mínima para desbloquearse.",
    { permitir: [".hub-columna-perks"], alContinuar: pasoTourHub3 }
  );
}

function pasoTourHub3() {
  mostrarPaso(
    "Cada ticket bien resuelto suma reputación. Al llegar al umbral necesario subes de rango — de Becario a Auxiliar de Sistemas, por ejemplo — lo que desbloquea tickets nuevos y un slot más de perk.",
    { permitir: [".tarjeta-progreso-carrera"], alContinuar: pasoCierre }
  );
}

function pasoCierre() {
  mostrarPaso(
    "Bien hecho — ya resolviste tus dos primeros tickets y ya conoces lo esencial. El resto de tu bandeja funciona igual: lee lo que piden, escribe la query, pruébala, y envíala. Ahí te dejo.",
    { alContinuar: finalizarTutorial }
  );
}

function finalizarTutorial() {
  activo = false;
  esperandoCierreScoring = false;
  clausulaObjetivoActual = null;
  pasoActualAlEscribir = null;
  manejarClicPrimerTicket = null;
  manejarClicPlay = null;
  manejarCierreScoring = null;
  permitirSiempre([]);
  ocultarBotonSaltar();
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
  manejarClicPrimerTicket = pasoConceptoTablaA;
  manejarClicPlay = null;
  manejarCierreScoring = null;
  permitirSiempre([SELECTOR_BOTON_SALTAR]);
  mostrarBotonSaltar();
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
  if (!activo || !manejarClicPrimerTicket) return;
  manejarClicPrimerTicket();
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
  if (!activo || !manejarClicPlay) return;
  manejarClicPlay();
}

export function notificarClicEnviar() {
  if (!activo) return;
  ocultarDialogo();
  esperandoCierreScoring = true;
}

export function notificarCierreScoring() {
  if (!activo || !esperandoCierreScoring || !manejarCierreScoring) return;
  esperandoCierreScoring = false;
  manejarCierreScoring();
}
