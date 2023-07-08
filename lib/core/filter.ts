import { exists, join, resolve } from "../../deps.ts";
import { FilterDefinition } from "../schemas/filter_definition.ts";
import { remoteFilterSchema } from "../schemas/remote_filter.ts";
import { readJson } from "../utils/fs.ts";
import { runSubprocess } from "../utils/subprocess.ts";

type Filter = {
  getRunCommand: (script: string, args: string[]) => { command: string; args: string[] };
  installDependencies?: (cwd: string) => Promise<void>;
};

export function getFilter(type: string): Filter {
  switch (type) {
    case "deno":
      return {
        getRunCommand: (script: string, args: string[]) => ({
          command: "deno",
          args: ["run", "-A", script, ...args],
        }),
      };
    case "java":
      return {
        getRunCommand: (script: string, args: string[]) => ({
          command: "java",
          args: ["-jar", script, ...args],
        }),
      };
    case "nodejs":
      return {
        getRunCommand: (script: string, args: string[]) => ({
          command: "node",
          args: [script, ...args],
        }),
        installDependencies: async (cwd) => {
          const npm = Deno.build.os === "windows" ? "npm.cmd" : "npm";
          await new Deno.Command(npm, {
            args: ["i", "--omit=dev", "--no-audit", "--no-fund"],
            cwd,
          }).output();
        },
      };
    case "python":
      return {
        getRunCommand: (script: string, args: string[]) => ({
          command: "python",
          args: [script, ...args],
        }),
        installDependencies: async (cwd) => {
          if (!await exists(join(cwd, "requirements.txt"))) {
            return;
          }
          await new Deno.Command("pip", {
            args: ["install", "-r", "requirements.txt"],
            cwd,
          }).output();
        },
      };
    case "shell":
      return {
        getRunCommand: (script: string, args: string[]) => ({
          command: "python",
          args: [script, ...args],
        }),
      };
    default:
      throw new Error(`Filter "${type}" is not supported.`);
  }
}

export async function runFilter(name: string, runArgs: string[], def: FilterDefinition) {
  if (def.runWith) {
    const filter = getFilter(def.runWith);
    if (def.script) {
      const { command, args } = filter.getRunCommand(resolve(def.script), runArgs);
      await runSubprocess(name, command, args);
    } else if (def.command) {
      await runSubprocess(name, Deno.build.os === "windows" ? "powershell" : "bash", ["-c", def.command, ...runArgs]);
    }
  } else {
    const filterDir = join(".regolith", "cache", "filters", name);
    if (!await exists(filterDir)) {
      throw new Error(`Filter "${name}" is not installed. Run "rgl install" to install it.`);
    }
    const filterConfig = await readJson(join(filterDir, "filter.json")).then(remoteFilterSchema.parse);
    for (const filter of filterConfig.filters) {
      if (filter.script) {
        filter.script = resolve(join(filterDir, filter.script));
      }
      await runFilter(name, runArgs, filter);
    }
  }
}
