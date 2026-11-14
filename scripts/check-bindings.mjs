// Regenerates src/lib/bindings from a clean folder and fails if the result differs from what's
// committed, including new files that were never added and leftovers from removed types.
import { execSync } from "node:child_process";
import { rmSync } from "node:fs";

const folder = "src/lib/bindings";

rmSync(folder, { recursive: true, force: true });
execSync("npm test", { stdio: "inherit" });

const changes = execSync(`git status --porcelain --untracked-files=all -- ${folder}`, { encoding: "utf8" }).trim();
if (changes) {
  console.error(`\nThe generated bindings don't match what's committed:\n${changes}`);
  console.error("Commit the files above so the screens and the Rust side agree.");
  process.exit(1);
}
console.log("Bindings match the Rust types.");
