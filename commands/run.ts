import { Command } from "../deps.ts";
import { runProfile } from "../lib/core/profile.ts";

export const run = new Command()
  .name("run")
  .description("Runs Regolith using specified profile")
  .arguments("[profile:string]")
  .action(async (_, profile = "default") => {
    await runProfile(profile);
  });
