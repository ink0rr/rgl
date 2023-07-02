import { logger } from "./logger.ts";

export async function runSubprocess(name: string, command: string, args: string[]) {
  const process = new Deno.Command(command, {
    args,
    cwd: "./.regolith/tmp",
    stderr: "piped",
    stdout: "piped",
    stdin: "null",
    env: {
      ROOT_DIR: Deno.cwd(),
    },
  }).spawn();

  const decoder = new TextDecoder();
  await process.stdout.pipeTo(
    new WritableStream({
      write(chunk) {
        logger.info(`[${name}] ${decoder.decode(chunk).replace(/\n$/, "")}`);
      },
    }),
  );
  await process.stderr.pipeTo(
    new WritableStream({
      write(chunk) {
        logger.error(`[${name}] ${decoder.decode(chunk).replace(/\n$/, "")}`);
      },
    }),
  );
  await process.status;
}
