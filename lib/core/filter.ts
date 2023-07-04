import { exists } from "https://deno.land/std@0.190.0/fs/exists.ts";
import { join, resolve } from "../../deps.ts";
import { FilterDefinition } from "../schemas/filter_definition.ts";
import { readJson } from "../utils/fs.ts";
import { runSubprocess } from "../utils/subprocess.ts";

type Filter = {
  getRunCommand: (script: string, args: string[]) => { command: string; args: string[] };
  installDependencies?: () => void;
};

const filters: Record<string, Filter> = {
  deno: {
    getRunCommand(script, args) {
      return {
        command: "deno",
        args: ["run", "-A", script, ...args],
      };
    },
  },
  nodejs: {
    getRunCommand(script, args) {
      return {
        command: "node",
        args: [script, ...args],
      };
    },
  },
  python: {
    getRunCommand(script, args) {
      return {
        command: "python",
        args: [script, ...args],
      };
    },
  },
};

export async function runFilter(name: string, runArgs: string[], def: FilterDefinition) {
  if (def.runWith === undefined) {
    const filterDir = join(".regolith", "cache", "filters", name);
    if (!await exists(filterDir)) {
      throw new Error(`Filter "${name}" is not installed. Run "rgl install" to install it.`);
    }
    try {
      const filterJson = await readJson<{ filters: FilterDefinition[] }>(join(filterDir, "filter.json"));
      for (const filter of filterJson.filters) {
        if (filter.runWith) {
          if (filter.script) {
            filter.script = resolve(join(filterDir, filter.script));
            await runFilter(name, runArgs, filterJson.filters[0]);
          } else if (filter.command) {
            // TODO: run command
          }
        }
      }
    } catch {
      throw new Error(`Invalid filter "${name}"`);
    }
  } else {
    const filter = filters[def.runWith];
    if (!filter) {
      throw new Error(`Filter "${def.runWith}" is not supported.`);
    }

    const { command, args } = filter.getRunCommand(resolve(def.script!), runArgs);
    await runSubprocess(name, command, args);
  }
}
