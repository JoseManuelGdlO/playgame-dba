let contexto = null;
let musicaSilenciada = false;
let efectosSilenciados = false;
let ambienteIniciado = false;
let busAmbiente = null;
let indicePatronAmbiente = 0;

const PATRON_AMBIENTE_HZ = [130.81, 164.81, 196.0, 164.81];
const DURACION_NOTA_AMBIENTE_MS = 700;
const FRECUENCIA_PAD_HZ = 65.41;

const PATRON_BOSS_HZ = [110, 110, 130.81, 98, 110, 82.41];
const DURACION_NOTA_BOSS_MS = 420;
const FRECUENCIA_PAD_BOSS_HZ = 55;

let modoMusica = "ambiente"; // "ambiente" | "boss"

function obtenerContexto() {
  if (!contexto) {
    contexto = new (window.AudioContext || window.webkitAudioContext)();
  }
  if (contexto.state === "suspended") {
    contexto.resume();
  }
  return contexto;
}

function obtenerBusAmbiente() {
  const ctx = obtenerContexto();
  if (!busAmbiente) {
    busAmbiente = ctx.createGain();
    busAmbiente.gain.value = musicaSilenciada ? 0 : 1;
    busAmbiente.connect(ctx.destination);
  }
  return busAmbiente;
}

function tono(frecuenciaHz, duracionMs, tipo, volumen) {
  if (efectosSilenciados) return;
  const ctx = obtenerContexto();
  const osc = ctx.createOscillator();
  const ganancia = ctx.createGain();
  osc.type = tipo;
  osc.frequency.value = frecuenciaHz;
  ganancia.gain.setValueAtTime(volumen, ctx.currentTime);
  ganancia.gain.exponentialRampToValueAtTime(0.0001, ctx.currentTime + duracionMs / 1000);
  osc.connect(ganancia);
  ganancia.connect(ctx.destination);
  osc.start();
  osc.stop(ctx.currentTime + duracionMs / 1000);
}

function secuenciaTonos(notasHz, duracionNotaMs, tipo, volumen, separacionMs) {
  notasHz.forEach((frecuenciaHz, indice) => {
    setTimeout(() => tono(frecuenciaHz, duracionNotaMs, tipo, volumen), indice * separacionMs);
  });
}

export function sfxClick() {
  tono(600, 60, "square", 0.05);
}

export function sfxTecleo() {
  const variacion = 0.9 + Math.random() * 0.2;
  tono(1200 * variacion, 30, "square", 0.04);
}

export function sfxBlip() {
  const variacion = 0.85 + Math.random() * 0.5;
  tono(300 * variacion, 45, "square", 0.05);
}

export function sfxTick() {
  tono(880, 80, "sine", 0.08);
}

export function sfxExito() {
  secuenciaTonos([523.25, 659.25, 783.99], 150, "triangle", 0.12, 90);
}

export function sfxError() {
  tono(160, 350, "sawtooth", 0.12);
}

export function sfxCierreDia() {
  secuenciaTonos([300, 220], 220, "sine", 0.08, 100);
}

export function sfxAscenso() {
  secuenciaTonos([523.25, 659.25, 783.99, 1046.5], 200, "triangle", 0.14, 110);
}

function reproducirNotaAmbiente(frecuenciaHz) {
  const ctx = obtenerContexto();
  const bus = obtenerBusAmbiente();
  const osc = ctx.createOscillator();
  const ganancia = ctx.createGain();
  osc.type = modoMusica === "boss" ? "sawtooth" : "triangle";
  osc.frequency.value = frecuenciaHz;
  const pico = modoMusica === "boss" ? 0.04 : 0.05;
  const duracionMs = modoMusica === "boss" ? DURACION_NOTA_BOSS_MS : DURACION_NOTA_AMBIENTE_MS;
  ganancia.gain.setValueAtTime(0.0001, ctx.currentTime);
  ganancia.gain.exponentialRampToValueAtTime(pico, ctx.currentTime + 0.05);
  ganancia.gain.exponentialRampToValueAtTime(0.0001, ctx.currentTime + duracionMs / 1000);
  osc.connect(ganancia);
  ganancia.connect(bus);
  osc.start();
  osc.stop(ctx.currentTime + duracionMs / 1000);
}

function reproducirPadAmbiente(frecuenciaHz, duracionMs) {
  const ctx = obtenerContexto();
  const bus = obtenerBusAmbiente();
  const osc = ctx.createOscillator();
  const ganancia = ctx.createGain();
  osc.type = "sine";
  osc.frequency.value = frecuenciaHz;
  ganancia.gain.setValueAtTime(0.0001, ctx.currentTime);
  ganancia.gain.linearRampToValueAtTime(0.03, ctx.currentTime + 0.4);
  ganancia.gain.linearRampToValueAtTime(0.0001, ctx.currentTime + duracionMs / 1000);
  osc.connect(ganancia);
  ganancia.connect(bus);
  osc.start();
  osc.stop(ctx.currentTime + duracionMs / 1000);
}

function agendarSiguienteNotaAmbiente() {
  const patron = modoMusica === "boss" ? PATRON_BOSS_HZ : PATRON_AMBIENTE_HZ;
  const duracionNota = modoMusica === "boss" ? DURACION_NOTA_BOSS_MS : DURACION_NOTA_AMBIENTE_MS;
  const padHz = modoMusica === "boss" ? FRECUENCIA_PAD_BOSS_HZ : FRECUENCIA_PAD_HZ;

  if (indicePatronAmbiente % patron.length === 0) {
    reproducirPadAmbiente(padHz, patron.length * duracionNota);
  }
  reproducirNotaAmbiente(patron[indicePatronAmbiente % patron.length]);
  indicePatronAmbiente += 1;
  setTimeout(agendarSiguienteNotaAmbiente, duracionNota);
}

export function establecerModoMusica(modo) {
  if (modo !== "ambiente" && modo !== "boss") return;
  if (modoMusica === modo) return;
  modoMusica = modo;
  indicePatronAmbiente = 0;
}

export function iniciarAmbiente() {
  if (ambienteIniciado) return;
  ambienteIniciado = true;
  obtenerBusAmbiente();
  agendarSiguienteNotaAmbiente();
}

export function alternarMusica() {
  musicaSilenciada = !musicaSilenciada;
  if (busAmbiente) {
    busAmbiente.gain.value = musicaSilenciada ? 0 : 1;
  }
  return !musicaSilenciada;
}

export function alternarEfectos() {
  efectosSilenciados = !efectosSilenciados;
  return !efectosSilenciados;
}
