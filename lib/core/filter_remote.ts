import { exists } from "../../deps.ts";
import { Filter } from "./filter.ts";

export class RemoteFilter extends Filter {
  constructor() {
    super();
  }

  async run(args?: string[], settings?: Record<string, unknown>) {
    if (await exists("")) {
    }
  }
}
