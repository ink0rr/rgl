import { dirname, JSONC } from "../../deps.ts";

async function outputFile(path: string, text: string) {
  try {
    await Deno.writeTextFile(path, text.trim() + "\n");
  } catch (error) {
    if (error instanceof Deno.errors.NotFound) {
      await Deno.mkdir(dirname(path), { recursive: true });
      await outputFile(path, text);
    } else {
      throw error;
    }
  }
}

export async function readJson<T>(path: string) {
  const data = await Deno.readTextFile(path);
  return JSONC.parse(data) as T;
}

export async function writeJson(path: string, data: unknown, replacer?: (key: string, value: unknown) => unknown) {
  await outputFile(path, JSON.stringify(data, replacer, 2));
}
