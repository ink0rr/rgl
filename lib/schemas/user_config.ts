import { z } from "../../deps.ts";

export const userConfigSchema = z.object({
  mojang_dir: z.string().optional(),
}).passthrough();
