import { describe, expect, test } from "bun:test";

import {
  parseModuleReleaseLine,
  parseReleaseLine,
  parseReleasesCsv,
} from "./csv";

describe("parseReleaseLine", () => {
  test("parses simple row", () => {
    const row = parseReleaseLine("a, A, 1.0.0, https://example.com/r");
    expect(row).toEqual({
      name: "a",
      localizedName: "A",
      version: "1.0.0",
      url: "https://example.com/r",
    });
  });

  test("tolerates commas in localized name", () => {
    const row = parseReleaseLine(
      "mod, Hello, world, title, 2.1.0, https://gh/x"
    );
    expect(row).toEqual({
      name: "mod",
      localizedName: "Hello, world, title",
      version: "2.1.0",
      url: "https://gh/x",
    });
  });
});

describe("parseModuleReleaseLine", () => {
  test("parses date with commas", () => {
    const row = parseModuleReleaseLine(
      "1.0.0-RC1, Jan 15, 2026 at 3:45 PM, https://github.com/x/y"
    );
    expect(row).toEqual({
      version: "1.0.0-RC1",
      dateTime: "Jan 15, 2026 at 3:45 PM",
      url: "https://github.com/x/y",
    });
  });
});

describe("parseReleasesCsv", () => {
  test("skips blank lines", () => {
    const rows = parseReleasesCsv("a, b, 1, https://x\n\n");
    expect(rows).toHaveLength(1);
  });
});
