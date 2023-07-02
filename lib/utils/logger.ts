import { colors } from "../../deps.ts";

export const logger = Object.freeze({
  info: (...args: unknown[]) => console.info(` ${colors.cyan("i")} `, ...args),
  warn: (...args: unknown[]) => console.warn(` ${colors.yellow("!")} `, ...args),
  error: (...args: unknown[]) => console.error(` ${colors.red("X")} `, ...args),
  success: (...args: unknown[]) => console.info(` ${colors.green("✓")} `, ...args),
});
