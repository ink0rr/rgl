import { exists, join } from "../../deps.ts";
import { Filter } from "./filter.ts";

export class RemoteFilter extends Filter {
  constructor(private name: string) {
    super();
  }

  async run(args?: string[], settings?: Record<string, unknown>) {
    const filterPath = join(".regolith", "tmp", "filters", this.name);
    if (!await exists(filterPath)) {
      throw Error(`Filter "${this.name}" is not installed.`);
    }
    
  }
}
