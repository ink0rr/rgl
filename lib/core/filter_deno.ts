import { resolve } from "../../deps.ts";
import { runSubprocess } from "../utils/subprocess.ts";
import { Filter } from "./filter.ts";

export class DenoFilter extends Filter {
  constructor(
    private name: string,
    private script: string,
  ) {
    super();
  }

  async run(args?: string[], settings?: Record<string, unknown>) {
    args ??= [];
    if (settings) {
      args.unshift(JSON.stringify(settings));
    }
    await runSubprocess(this.name, "deno", ["run", "-A", resolve(this.script)].concat(args));
  }
}
