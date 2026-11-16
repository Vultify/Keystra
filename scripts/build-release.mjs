// Builds the release installer with every path under the builder's home folder rewritten to "~",
// then checks the built app for that folder's name. Rust stores dependency source paths inside the
// exe, so without this the installer would carry the Windows user name.
import { execSync } from "node:child_process";
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { homedir } from "node:os";
import { basename, join } from "node:path";

const home = homedir();
// Windows also knows the folder by a short 8.3 name (like C:\Users\NAME~1), which TEMP paths use.
const shortHome = execSync(`cmd /c for %I in ("${home}") do @echo %~sI`, { encoding: "utf8" }).trim();
const homes = [...new Set([home, shortHome].filter(Boolean))];

const flags = homes.map((path) => `--remap-path-prefix=${path}=~`);
if (process.env.CARGO_HOME) flags.push(`--remap-path-prefix=${process.env.CARGO_HOME}=~/.cargo`);
// Encoded flags are separated by 0x1F, so paths with spaces stay whole.
const env = { ...process.env, CARGO_ENCODED_RUSTFLAGS: flags.join("\x1f") };
delete env.RUSTFLAGS;

// The base config is the dev identity ("Keystra Dev"); the release config makes it Keystra.
execSync("npx tauri build --config src-tauri/tauri.release.conf.json", { stdio: "inherit", env });

const target = "src-tauri/target/release";
const bundles = join(target, "bundle", "nsis");
const files = [join(target, "keystra.exe")];
if (existsSync(bundles)) {
  for (const name of readdirSync(bundles)) if (name.endsWith(".exe")) files.push(join(bundles, name));
}

// The folder names, long and short, searched in any letter case as 8-bit and UTF-16 text. The
// names themselves are never printed.
const names = [...new Set(homes.map((path) => basename(path).toLowerCase()))];
let leaks = 0;
for (const file of files) {
  const bytes = readFileSync(file);
  const texts = [bytes.toString("latin1").toLowerCase(), bytes.toString("utf16le").toLowerCase()];
  const found = names.filter((name) => texts.some((text) => text.includes(name))).length;
  console.log(found ? `FOUND the home folder name in ${file}` : `clean: ${file}`);
  leaks += found;
}
if (leaks > 0) {
  console.error("The build contains the home folder name. Don't publish it.");
  process.exit(1);
}
