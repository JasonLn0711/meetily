import assert from "node:assert/strict";
import { afterEach, describe, mock, test } from "node:test";

import { blocksToMarkdownSafely } from "../../src/lib/blocknote-markdown.ts";

describe("blocksToMarkdownSafely", () => {
  afterEach(() => mock.restoreAll());

  test("returns markdown when conversion succeeds", async () => {
    const convert = mock.fn(async () => "# Summary");
    const result = await blocksToMarkdownSafely(
      { blocksToMarkdownLossy: convert },
      [],
      { source: "test-success" },
    );

    assert.deepEqual(result, { markdown: "# Summary", ok: true });
    assert.equal(convert.mock.callCount(), 1);
  });

  test("returns fallback markdown when conversion throws", async () => {
    const error = new Error("conversion failed");
    const consoleError = mock.method(console, "error", () => {});
    const result = await blocksToMarkdownSafely(
      { blocksToMarkdownLossy: async () => { throw error; } },
      [{ id: "block-1" }],
      { source: "test-fallback", fallbackMarkdown: "existing markdown" },
    );

    assert.deepEqual(result, { markdown: "existing markdown", ok: false });
    assert.equal(consoleError.mock.callCount(), 1);
    assert.deepEqual(consoleError.mock.calls[0].arguments, [
      "Failed to convert BlockNote blocks to markdown",
      { source: "test-fallback", blocksCount: 1, error },
    ]);
  });

  test("omits markdown when conversion throws without fallback", async () => {
    mock.method(console, "error", () => {});
    const result = await blocksToMarkdownSafely(
      { blocksToMarkdownLossy: async () => { throw new Error("conversion failed"); } },
      [],
      { source: "test-empty-fallback" },
    );

    assert.deepEqual(result, { markdown: undefined, ok: false });
  });
});
