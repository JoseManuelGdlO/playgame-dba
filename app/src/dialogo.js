import { sfxBlip } from "./audio.js";

let capaBloqueo = null;
let tarjeta = null;
let elementoTexto = null;
let selectoresPermitidos = [];
let siempreVisibles = [];
let callbackContinuar = null;
let intervaloRevelado = null;
let manejadorRedimension = null;
let textoCompleto = "";
let revelando = false;

const VELOCIDAD_REVELADO_MS = 30;

// Deliberadamente solo bloquea "click" — no mousedown/keydown. El tutorial
// (Plan 17) depende de que un elemento no permitido, como #sql-input durante
// el beat de Enviar, siga siendo enfocable y editable con el teclado para
// poder corregir un intento fallido; bloquear esos eventos también dejaría
// al jugador sin forma de arreglar su query mientras el diálogo sigue abierto.
function manejarClickDocumento(evento) {
  const enTarjeta = evento.target.closest(".dialogo-tarjeta");
  const enPermitido = selectoresPermitidos.some((selector) => evento.target.closest(selector));
  const enSiempreVisible = siempreVisibles.some((selector) => evento.target.closest(selector));
  if (enTarjeta || enPermitido || enSiempreVisible) return;
  evento.stopPropagation();
  evento.preventDefault();
}

function detenerRevelado() {
  if (intervaloRevelado) {
    clearInterval(intervaloRevelado);
    intervaloRevelado = null;
  }
  revelando = false;
}

function completarRevelado() {
  detenerRevelado();
  elementoTexto.textContent = textoCompleto;
}

function iniciarRevelado() {
  let indice = 0;
  revelando = true;
  elementoTexto.textContent = "";
  intervaloRevelado = setInterval(() => {
    indice += 1;
    elementoTexto.textContent = textoCompleto.slice(0, indice);
    if (indice % 3 === 0) {
      sfxBlip();
    }
    if (indice >= textoCompleto.length) {
      detenerRevelado();
      const sprite = tarjeta?.querySelector(".sprite-habla");
      if (sprite) sprite.classList.remove("sprite-habla--hablando");
    }
  }, VELOCIDAD_REVELADO_MS);
}

function actualizarSpotlight(spotlight, selector) {
  const elemento = selector && document.querySelector(selector);
  if (!elemento) {
    spotlight.style.display = "none";
    return;
  }
  const rect = elemento.getBoundingClientRect();
  spotlight.style.display = "block";
  spotlight.style.left = `${rect.left - 6}px`;
  spotlight.style.top = `${rect.top - 6}px`;
  spotlight.style.width = `${rect.width + 12}px`;
  spotlight.style.height = `${rect.height + 12}px`;
}

export function permitirSiempre(selectores) {
  siempreVisibles = selectores;
}

export function mostrarDialogo(retratoSvg, nombre, texto, opciones = {}) {
  ocultarDialogo();

  selectoresPermitidos = opciones.permitir || [];
  callbackContinuar = opciones.alContinuar || null;
  textoCompleto = texto;

  capaBloqueo = document.createElement("div");
  capaBloqueo.className = "dialogo-bloqueo";

  const objetivoSpotlight = selectoresPermitidos[0];
  if (objetivoSpotlight) {
    const spotlight = document.createElement("div");
    spotlight.className = "dialogo-spotlight";
    capaBloqueo.appendChild(spotlight);
    actualizarSpotlight(spotlight, objetivoSpotlight);
    manejadorRedimension = () => actualizarSpotlight(spotlight, objetivoSpotlight);
    window.addEventListener("resize", manejadorRedimension);
  } else {
    const dim = document.createElement("div");
    dim.className = "dialogo-dim";
    capaBloqueo.appendChild(dim);
  }

  tarjeta = document.createElement("div");
  tarjeta.className = "dialogo-tarjeta";
  tarjeta.innerHTML = `
    <div class="retrato">${retratoSvg}</div>
    <div class="dialogo-cuerpo">
      <div class="dialogo-nombre">${nombre}</div>
      <div class="dialogo-texto"></div>
    </div>
  `;
  elementoTexto = tarjeta.querySelector(".dialogo-texto");
  const spriteHabla = tarjeta.querySelector(".sprite-habla");
  if (opciones.hablando !== false && spriteHabla) {
    spriteHabla.classList.add("sprite-habla--hablando");
  }
  tarjeta.addEventListener("click", () => {
    if (revelando) {
      completarRevelado();
      if (spriteHabla) spriteHabla.classList.remove("sprite-habla--hablando");
    } else if (callbackContinuar) {
      callbackContinuar();
    }
  });
  capaBloqueo.appendChild(tarjeta);

  document.body.appendChild(capaBloqueo);
  document.addEventListener("click", manejarClickDocumento, true);

  iniciarRevelado();
  if (spriteHabla && opciones.hablando !== false) {
    const detenerBoca = () => spriteHabla.classList.remove("sprite-habla--hablando");
    const checkFin = setInterval(() => {
      if (!revelando) {
        clearInterval(checkFin);
        detenerBoca();
      }
    }, 40);
  }
}

export function ocultarDialogo() {
  detenerRevelado();
  document.removeEventListener("click", manejarClickDocumento, true);
  if (manejadorRedimension) {
    window.removeEventListener("resize", manejadorRedimension);
    manejadorRedimension = null;
  }
  if (capaBloqueo) {
    capaBloqueo.remove();
    capaBloqueo = null;
  }
  tarjeta = null;
  elementoTexto = null;
  selectoresPermitidos = [];
  callbackContinuar = null;
}
