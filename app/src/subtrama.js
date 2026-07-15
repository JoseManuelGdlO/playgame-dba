import { mostrarDialogo, ocultarDialogo } from "./dialogo.js";
import { retratoDelsyHabla, retratoJugadorHabla } from "./sprites.js";

const STORAGE_KEY = "querypath-subtrama-vistas";

/** Capítulos = cambios de empleo / fatiga visible del jugador. Lenguaje sencillo. */
export const CAPITULOS_EMPLEO = [
  {
    id: "primer_dia",
    titulo: "Primer día",
    pasos: [
      {
        quien: "delsy",
        texto:
          "¡Hola! Soy Delsy, de recepción… bueno, de recepción y de “lo que nadie más quiere hacer”. ¿Tú eres el becario nuevo?",
      },
      {
        quien: "jugador",
        texto:
          "Sí. Me dijeron que voy a ayudar con reportes y pedidos de otras áreas. Suena… intenso.",
      },
      {
        quien: "delsy",
        texto:
          "El anterior se fue llorando al cuarto de limpieza. Dejó un Post-it que solo decía “ayuda”. Nadie sabe si pedía café o un abrazo.",
      },
      {
        quien: "delsy",
        texto:
          "Si necesitas algo, avísame. Si necesitas café bueno… tampoco avises, porque no hay. Pero puedo fingir que sí.",
      },
    ],
  },
  {
    id: "ya_rodado",
    titulo: "Ya no eres el nuevo",
    pasos: [
      {
        quien: "delsy",
        texto:
          "Oye… se te están notando las ojeras. Aquí eso cuenta como “experiencia laboral”.",
      },
      {
        quien: "jugador",
        texto:
          "Solo estoy cerrando pedidos de la bandeja. Y releerlos. Y fingir que me alcanza el tiempo del día.",
      },
      {
        quien: "delsy",
        texto:
          "Clásico. Ayer alguien se llevó la taza del “mejor jefe del mundo”. RRHH abrió un expediente. El expediente… era un meme impreso.",
      },
      {
        quien: "delsy",
        texto:
          "Entre nos: si te dicen que “vas por buen camino al Auditor”, no es un favor. Es un reality show… pero sin premio.",
      },
    ],
  },
  {
    id: "ascenso_auxiliar",
    titulo: "Ascenso (con pastel triste)",
    pasos: [
      {
        quien: "delsy",
        texto:
          "¡Felicidades, Auxiliar! Te compramos un pastel. Bueno, lo “compramos” en el sentido de que alguien lo encontró en la cocina.",
      },
      {
        quien: "jugador",
        texto:
          "¿Tiene mi nombre? El glaseado parece decir… “Feliz jueves a quien sea”.",
      },
      {
        quien: "delsy",
        texto:
          "Rehacer gafetes cuesta. Imprimieron “Auxiliar de Sistmeas”. La “e” se perdió en un pedido viejo.",
      },
      {
        quien: "delsy",
        texto:
          "Ahora te toca el Auditor. Si sales bien, te invito a un pretzel imaginario. El Día del Pretzel sigue cancelado, pero la imaginación es gratis.",
      },
    ],
  },
  {
    id: "traslado_kevin",
    titulo: "Traslado (firmado: Kevin)",
    pasos: [
      {
        quien: "delsy",
        texto:
          "Llegó un fax. O un papel raro. O un sueño colectivo. Dice que te pasan a Postafeta. Firma: “- Kevin”.",
      },
      {
        quien: "jugador",
        texto:
          "¿Kevin es real? Nadie lo ha visto. Solo aparece firmando mensajes en el chat interno.",
      },
      {
        quien: "delsy",
        texto:
          "Si preguntas tres veces, la gerencia te mira raro y cambia de tema. Mejor no.",
      },
      {
        quien: "delsy",
        texto:
          "Vas a estar bien. O cansado. O las dos. Mándame un mensaje si allá el internet también “funciona cuando quiere”. …Cuídate.",
      },
      {
        quien: "jugador",
        texto:
          "Gracias, Delsy. Si vuelvo, prometo no calentar pescado en el microondas.",
      },
    ],
  },
];

function cargarVistas() {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return new Set();
    const arr = JSON.parse(raw);
    return new Set(Array.isArray(arr) ? arr : []);
  } catch {
    return new Set();
  }
}

function guardarVistas(vistas) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify([...vistas]));
}

let vistas = cargarVistas();
let reproduciendo = false;

function retratoDe(quien) {
  return quien === "delsy" ? retratoDelsyHabla() : retratoJugadorHabla();
}

function nombreDe(quien) {
  return quien === "delsy" ? "Delsy" : "Tú";
}

/**
 * Reproduce un capítulo (dialogo encadenado). Resuelve al terminar.
 */
export function reproducirCapitulo(capitulo) {
  return new Promise((resolve) => {
    if (!capitulo || !capitulo.pasos?.length) {
      resolve();
      return;
    }
    reproduciendo = true;
    let i = 0;
    const siguiente = () => {
      if (i >= capitulo.pasos.length) {
        ocultarDialogo();
        reproduciendo = false;
        resolve();
        return;
      }
      const paso = capitulo.pasos[i];
      i += 1;
      mostrarDialogo(retratoDe(paso.quien), nombreDe(paso.quien), paso.texto, {
        hablando: true,
        alContinuar: siguiente,
      });
    };
    siguiente();
  });
}

export function subtramaActiva() {
  return reproduciendo;
}

export function reiniciarSubtramaDebug() {
  vistas = new Set();
  guardarVistas(vistas);
}

/**
 * Muestra el capítulo de ese índice de empleo (0–3) si aún no se vio.
 * `indiceEmpleo` alinea con las 4 etapas de retrato/cansancio.
 */
export async function intentarCapituloEmpleo(indiceEmpleo) {
  const capitulo = CAPITULOS_EMPLEO[indiceEmpleo];
  if (!capitulo || vistas.has(capitulo.id) || reproduciendo) return false;
  vistas.add(capitulo.id);
  guardarVistas(vistas);
  await reproducirCapitulo(capitulo);
  return true;
}

/**
 * Encola el primer capítulo pendiente hasta `indiceEmpleo` inclusive
 * (así no te saltas la bienvenida de Delsy si asciendes muy rápido).
 */
export async function intentarCapitulosHasta(indiceEmpleo) {
  if (reproduciendo) return false;
  const tope = Math.max(0, Math.min(CAPITULOS_EMPLEO.length - 1, indiceEmpleo));
  for (let i = 0; i <= tope; i++) {
    const mostro = await intentarCapituloEmpleo(i);
    if (mostro) return true;
  }
  return false;
}

/** Fuerza el siguiente capítulo no visto (debug / playtest). */
export async function forzarSiguienteCapituloSubtrama() {
  let pendiente = CAPITULOS_EMPLEO.find((c) => !vistas.has(c.id));
  if (!pendiente) {
    reiniciarSubtramaDebug();
    pendiente = CAPITULOS_EMPLEO[0];
  }
  vistas.add(pendiente.id);
  guardarVistas(vistas);
  await reproducirCapitulo(pendiente);
  return pendiente;
}
