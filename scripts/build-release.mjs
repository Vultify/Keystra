// Builds the release installer with every path under the builder's home folder rewritten to "~",
// then checks the built app for that folder's name. Rust stores dependency source paths inside the
// exe, so without this the installer would carry the Windows user name.
import { execSync } from "node:child_process";
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { homedir } from "node:os";
import { basename, join } from "node:path";

const home = homedir();
const flags = [`--remap-path-prefix=${home}=~`];
if (process.env.CARGO_HOME) flags.push(`--remap-path-prefix=${process.env.CARGO_HOME}=~/.cargo`);
// Encoded flags are separated by 0x1F, so paths with spaces stay whole.
const env = { ...process.env, CARGO_ENCODED_RUSTFLAGS: flags.join("\x1f") };
delete env.RUSTFLAGS;

execSync("npx tauri build", { stdio: "inherit", env });

const target = "src-tauri/target/release";
const bundles = join(target, "bundle", "nsis");
const files = [join(target, "keystra.exe")];
if (existsSync(bundles)) {
  for (const name of readdirSync(bundles)) if (name.endsWith(".exe")) files.push(join(bundles, name));
}

// Searched as raw bytes, in both 8-bit and UTF-16 form; the name itself is never printed.
const needles = [home, basename(home)].flatMap((text) => [Buffer.from(text, "utf8"), Buffer.from(text, "utf16le")]);
let leaks = 0;
for (const file of files) {
  const bytes = readFileSync(file);
  const found = needles.filter((needle) => bytes.includes(needle)).length;
  console.log(found ? `FOUND the home folder name in ${file}` : `clean: ${file}`);
  leaks += found;
}
if (leaks > 0) {
  console.error("The build contains the home folder name. Don't publish it.");
  process.exit(1);
}
