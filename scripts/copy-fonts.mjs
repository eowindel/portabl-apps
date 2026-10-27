// Copie les polices (embarquées dans l'exe, aucun accès réseau au runtime) vers src/fonts.
import { copyFileSync, mkdirSync } from "node:fs";
import { join } from "node:path";

const fonts = [
  ["orbitron", "orbitron-latin-700-normal.woff2"],
  ["orbitron", "orbitron-latin-900-normal.woff2"],
  ["chakra-petch", "chakra-petch-latin-400-normal.woff2"],
  ["chakra-petch", "chakra-petch-latin-600-normal.woff2"],
  ["press-start-2p", "press-start-2p-latin-400-normal.woff2"],
  ["cinzel", "cinzel-latin-700-normal.woff2"],
  ["oswald", "oswald-latin-600-normal.woff2"],
];

const out = join("src", "fonts");
mkdirSync(out, { recursive: true });
for (const [pkg, file] of fonts) {
  copyFileSync(join("node_modules", "@fontsource", pkg, "files", file), join(out, file));
  console.log("copied", file);
}
// Les polices sont sous licence OFL : leur licence voyage avec elles.
for (const pkg of new Set(fonts.map(([pkg]) => pkg))) {
  copyFileSync(join("node_modules", "@fontsource", pkg, "LICENSE"), join(out, `LICENSE-${pkg}.txt`));
}
