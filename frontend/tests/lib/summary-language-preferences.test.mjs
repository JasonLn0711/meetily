import assert from "node:assert/strict";
import { beforeEach, describe, mock, test } from "node:test";

const invokeResponses = [];
const invokeMock = mock.fn(async () => invokeResponses.shift() ?? null);

mock.module("@tauri-apps/api/core", {
  namedExports: { invoke: invokeMock },
});

const prefs = await import("../../src/lib/summary-language-preferences.ts");

function installLocalStorage() {
  const values = new Map();

  Object.defineProperty(globalThis, "window", {
    configurable: true,
    value: {
      localStorage: {
        getItem: (key) => values.get(key) ?? null,
        setItem: (key, value) => values.set(key, value),
        removeItem: (key) => values.delete(key),
        clear: () => values.clear(),
      },
    },
  });

  return values;
}

function installFailingLocalStorage() {
  Object.defineProperty(globalThis, "window", {
    configurable: true,
    value: {
      localStorage: {
        getItem: () => null,
        setItem: () => {
          throw new Error("quota exceeded");
        },
        removeItem: () => {},
        clear: () => {},
      },
    },
  });
}

describe("summary language local fallback", () => {
  let storageValues;

  beforeEach(() => {
    invokeMock.mock.resetCalls();
    invokeResponses.length = 0;
    storageValues = installLocalStorage();
  });

  test("reads summary language from local fallback when meeting has no folder", async () => {
    storageValues.set("summaryLanguageFallback:meeting-1", "fr");
    invokeResponses.push({ language: null, storage: "local_fallback" });

    assert.deepEqual(await prefs.readMeetingSummaryLanguage("meeting-1"), {
      language: "fr",
      storage: "local_fallback",
    });
  });

  test("saves summary language locally when command reports no folder", async () => {
    invokeResponses.push({ language: null, storage: "local_fallback" });

    assert.deepEqual(await prefs.saveMeetingSummaryLanguage("meeting-1", "es"), {
      language: "es",
      storage: "local_fallback",
    });
    assert.equal(storageValues.get("summaryLanguageFallback:meeting-1"), "es");
  });

  test("clears local fallback when Auto is saved for a folderless meeting", async () => {
    storageValues.set("summaryLanguageFallback:meeting-1", "de");
    invokeResponses.push({ language: null, storage: "local_fallback" });

    assert.deepEqual(await prefs.saveMeetingSummaryLanguage("meeting-1", null), {
      language: null,
      storage: "local_fallback",
    });
    assert.equal(storageValues.has("summaryLanguageFallback:meeting-1"), false);
  });

  test("caches detected language locally when meeting has no folder", async () => {
    invokeResponses.push({ language: null, storage: "local_fallback" });

    await prefs.saveCachedDetectedSummaryLanguage("meeting-1", "pt");
    assert.equal(
      storageValues.get("detectedSummaryLanguageFallback:meeting-1"),
      "pt",
    );
  });

  test("rejects when folderless summary language cannot be persisted locally", async () => {
    installFailingLocalStorage();
    invokeResponses.push({ language: null, storage: "local_fallback" });

    await assert.rejects(
      prefs.saveMeetingSummaryLanguage("meeting-1", "it"),
      /Failed to save summary language on this device/,
    );
  });
});
