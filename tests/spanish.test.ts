/**
 * Spanish through the page's path: the wasm calls Speak and Sing make, and the words Speak shows
 * at the mouth.
 */
import { describe, expect, test } from "bun:test";
import { initEngine } from "./harness.ts";

await initEngine();
const { spanishPhrases, lyricsToSyllables, planPhrases } = await import("../src/wasm/engine.ts");
const { wordSpans } = await import("../src/ui/words.ts");
const male = { female: 0, child: 0 };

describe("Spanish", () => {
  test("text becomes phrases with Spanish sounds, stress and intonation", () => {
    const phrases = spanishPhrases("Hola, ¿quieres agua?");
    expect(phrases.map((p) => p.intonation)).toEqual(["continuation", "question"]);
    expect(phrases[1].segments.map((s) => s.ipa).join("")).toBe("kjeɾesɑɣwɑ");
    expect(phrases[0].segments.find((s) => s.stress === 1)?.ipa).toBe("o");
    expect(spanishPhrases("¿Dónde vives?")[0].intonation).toBe("statement");
  });

  test("each word gets its span in the plan, a continuation included", () => {
    const phrases = spanishPhrases("Compré pan, queso y huevos.");
    const plan = planPhrases(phrases, { body: male, t0: 0 });
    const words = wordSpans(phrases, plan);
    expect(words.map((w) => w.text)).toEqual(["compré", "pan", "queso", "y", "huevos"]);
    for (let i = 1; i < words.length; i++) expect(words[i].t0).toBeGreaterThanOrEqual(words[i - 1].t1 - 1e-9);
  });

  test("lyrics in ES become Spanish syllables; other scripts still go to their own front ends", () => {
    const syl = lyricsToSyllables("Pe-rro de la ciudad 빵", "es");
    expect(syl.map((s) => s.text)).toEqual(["Pe", "rro", "de", "la", "ciu", "dad", "빵"]);
    expect(syl[1].onset[0].ipa).toBe("r");
    expect(syl[2].onset[0].ipa).toBe("ð");
  });
});
