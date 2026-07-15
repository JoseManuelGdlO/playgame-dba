let contexto = null;
let musicaSilenciada = false;
let efectosSilenciados = false;
let ambienteIniciado = false;
let modoMusica = "ambiente"; // "ambiente" | "boss"
let volumenMusica = 0.7;
let indicePistaAmbiente = 0;
/** @type {HTMLAudioElement | null} */
let audioAmbiente = null;
/** @type {HTMLAudioElement | null} */
let audioBoss = null;

/** Microwave primero; el resto rota en loop como base ambiental. */
const PISTAS_AMBIENTE = [
  "assets/music/microwave-dance.mp3",
  "assets/music/super-duper.mp3",
  "assets/music/tech-no-ledge.mp3",
  "assets/music/froggy-fraud-adventure.mp3",
];
const PISTA_BOSS = "assets/music/ashes.mp3";

function obtenerContexto() {
  if (!contexto) {
    contexto = new (window.AudioContext || window.webkitAudioContext)();
  }
  if (contexto.state === "suspended") {
    contexto.resume();
  }
  return contexto;
}

function volumenMusicaEfectivo() {
  return musicaSilenciada ? 0 : volumenMusica;
}

function aplicarVolumenPistas() {
  const v = volumenMusicaEfectivo();
  if (audioAmbiente) audioAmbiente.volume = v;
  if (audioBoss) audioBoss.volume = v;
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

function crearAudio(src, { loop = false } = {}) {
  const el = new Audio(src);
  el.loop = loop;
  el.preload = "auto";
  el.volume = volumenMusicaEfectivo();
  return el;
}

function asegurarAudioAmbiente() {
  if (audioAmbiente) return audioAmbiente;
  audioAmbiente = crearAudio(PISTAS_AMBIENTE[indicePistaAmbiente], { loop: false });
  audioAmbiente.addEventListener("ended", () => {
    if (modoMusica !== "ambiente") return;
    indicePistaAmbiente = (indicePistaAmbiente + 1) % PISTAS_AMBIENTE.length;
    audioAmbiente.src = PISTAS_AMBIENTE[indicePistaAmbiente];
    audioAmbiente.volume = volumenMusicaEfectivo();
    void audioAmbiente.play().catch(() => {});
  });
  return audioAmbiente;
}

function asegurarAudioBoss() {
  if (audioBoss) return audioBoss;
  audioBoss = crearAudio(PISTA_BOSS, { loop: true });
  return audioBoss;
}

function pausarAmbiente() {
  if (!audioAmbiente) return;
  audioAmbiente.pause();
}

function pausarBoss() {
  if (!audioBoss) return;
  audioBoss.pause();
  audioBoss.currentTime = 0;
}

function reproducirAmbiente() {
  const el = asegurarAudioAmbiente();
  el.volume = volumenMusicaEfectivo();
  void el.play().catch(() => {});
}

function reproducirBoss() {
  const el = asegurarAudioBoss();
  el.volume = volumenMusicaEfectivo();
  el.currentTime = 0;
  void el.play().catch(() => {});
}

export function establecerModoMusica(modo) {
  if (modo !== "ambiente" && modo !== "boss") return;
  if (modoMusica === modo) return;
  modoMusica = modo;
  if (!ambienteIniciado) return;
  if (modo === "boss") {
    pausarAmbiente();
    reproducirBoss();
  } else {
    pausarBoss();
    reproducirAmbiente();
  }
}

export function iniciarAmbiente() {
  obtenerContexto();
  if (ambienteIniciado) {
    // Reanuda si quedó pausado por política/autoplay o mute parcial.
    if (modoMusica === "boss") {
      if (audioBoss?.paused) reproducirBoss();
    } else if (audioAmbiente?.paused) {
      reproducirAmbiente();
    }
    return;
  }
  ambienteIniciado = true;
  indicePistaAmbiente = 0;
  if (modoMusica === "boss") {
    reproducirBoss();
  } else {
    reproducirAmbiente();
  }
}

/** @param {number} valor 0–1 */
export function establecerVolumenMusica(valor) {
  const n = Number(valor);
  volumenMusica = Number.isFinite(n) ? Math.min(1, Math.max(0, n)) : volumenMusica;
  aplicarVolumenPistas();
  return volumenMusica;
}

export function obtenerVolumenMusica() {
  return volumenMusica;
}

export function alternarMusica() {
  musicaSilenciada = !musicaSilenciada;
  aplicarVolumenPistas();
  return !musicaSilenciada;
}

export function alternarEfectos() {
  efectosSilenciados = !efectosSilenciados;
  return !efectosSilenciados;
}
