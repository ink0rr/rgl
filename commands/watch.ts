import { Command } from "../deps.ts";
import { runOrWatch } from "../lib/core/runner.ts";

export const watch = new Command()
  .name("watch")
  .description("Runs Regolith using specified profile and watches for changes")
  .arguments("[profile:string]")
  .action(async (_, profile = "default") => {
    await runOrWatch(profile, true);
  });
