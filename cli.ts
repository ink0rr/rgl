import { clean } from "./commands/clean.ts";
import { run } from "./commands/run.ts";
import { Command, fromZodError, z } from "./deps.ts";
import { logger } from "./lib/utils/logger.ts";

if (import.meta.main) {
  try {
    await new Command()
      .name("rgl")
      .description("Oversimplified Regolith runner")
      .arguments("<command>")
      .command("clean", clean)
      .command("run", run)
      .parse(Deno.args);
  } catch (e) {
    if (e instanceof z.ZodError) {
      logger.error(fromZodError(e).message);
    } else {
      logger.error(e.message);
    }
  }
}
