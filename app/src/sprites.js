/** Frames de boca para sprites parlantes (perfil / diálogo). */
const FRAMES_JUGADOR = [
  "assets/sprites/jugador-habla-1.png",
  "assets/sprites/jugador-habla-2.png",
  "assets/sprites/jugador-habla-3.png",
];

const FRAMES_DELSY = [
  "assets/sprites/delsy-habla-1.png",
  "assets/sprites/delsy-habla-2.png",
  "assets/sprites/delsy-habla-3.png",
];

/**
 * Markup de un sprite de 3 frames. Frame 1 = idle; al añadir
 * `.sprite-habla--hablando` cicla la boca.
 */
export function htmlSpriteHabla(personaje, alt = "") {
  const frames = personaje === "delsy" ? FRAMES_DELSY : FRAMES_JUGADOR;
  const etiqueta = alt || (personaje === "delsy" ? "Delsy" : "Tú");
  const imgs = frames
    .map(
      (src, i) =>
        `<img src="${src}" alt="" class="sprite-habla-f sprite-habla-f--${i + 1}" draggable="false" />`
    )
    .join("");
  return `<div class="sprite-habla" data-personaje="${personaje}" role="img" aria-label="${etiqueta}">${imgs}</div>`;
}

export function retratoJugadorHabla() {
  return htmlSpriteHabla("jugador", "Tú");
}

export function retratoDelsyHabla() {
  return htmlSpriteHabla("delsy", "Delsy");
}
