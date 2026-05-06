import { describe, expect, test } from "vitest";

import { buildModuleReleasesPath, buildModulesPath, buildReleasesPath } from "@/lib/api";

describe("api path builders", () => {
  test("buildModulesPath without name", () => {
    expect(buildModulesPath()).toBe("/sls/modules");
  });

  test("buildModulesPath with name", () => {
    expect(buildModulesPath("foo")).toBe("/sls/modules?name=foo");
  });

  test("buildReleasesPath adds rc and ms params", () => {
    expect(buildReleasesPath(false, false)).toBe("/sls/releases");
    expect(buildReleasesPath(true, false)).toBe("/sls/releases?rc=true");
    expect(buildReleasesPath(false, true)).toBe("/sls/releases?ms=true");
    expect(buildReleasesPath(true, true)).toBe("/sls/releases?rc=true&ms=true");
  });

  test("buildModuleReleasesPath encodes module name and adds params", () => {
    expect(buildModuleReleasesPath("a/b", false, false)).toBe("/sls/releases/a%2Fb");
    expect(buildModuleReleasesPath("a/b", true, false)).toBe("/sls/releases/a%2Fb?rc=true");
    expect(buildModuleReleasesPath("a/b", false, true)).toBe("/sls/releases/a%2Fb?ms=true");
    expect(buildModuleReleasesPath("a/b", true, true)).toBe("/sls/releases/a%2Fb?rc=true&ms=true");
  });
});
