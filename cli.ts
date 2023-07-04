import { config } from "./commands/config.ts";
import { install } from "./commands/install.ts";
import { run } from "./commands/run.ts";
import { watch } from "./commands/watch.ts";
import { Command, fromZodError, z } from "./deps.ts";
import { logger } from "./lib/utils/logger.ts";

if (import.meta.main) {
  try {
    await new Command()
      .name("rgl")
      .description("Oversimplified Regolith runner")
      .arguments("<command>")
      .command("config", config)
      .command("install", install)
      .command("run", run)
      .command("watch", watch)
      .parse(Deno.args);
  } catch (e) {
    if (e instanceof z.ZodError) {
      logger.error(fromZodError(e).message);
    } else {
      logger.error(e.message);
    }
  }
}
