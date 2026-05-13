import { render, screen } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { beforeEach, describe, expect, test, vi } from "vitest";

import * as api from "@/lib/api";
import { ModulePage } from "@/pages/ModulePage";

vi.mock("@/lib/api", async () => {
  const actual = await vi.importActual<typeof import("@/lib/api")>("@/lib/api");
  return {
    ...actual,
    fetchModule: vi.fn(),
    fetchModuleReleases: vi.fn(),
  };
});

function renderWithRoute(path: string) {
  return render(
    <MemoryRouter initialEntries={[path]}>
      <Routes>
        <Route path="/module/:name" element={<ModulePage />} />
      </Routes>
    </MemoryRouter>,
  );
}

describe("ModulePage", () => {
  beforeEach(() => {
    localStorage.clear();
    vi.mocked(api.fetchModuleReleases).mockReset();
  });

  test("renders module and release rows", async () => {
    const fetchModule = vi.mocked(api.fetchModule);
    const fetchModuleReleases = vi.mocked(api.fetchModuleReleases);

    fetchModule.mockResolvedValueOnce({ name: "mod-a", localizedName: "Module A" });
    fetchModuleReleases.mockResolvedValueOnce([
      {
        name: "mod-a",
        localizedName: "Module A",
        version: "1.0.0",
        dateTime: "2026-01-01 00:00:00",
        kind: "Release",
        url: "https://example.invalid/a",
      },
    ]);

    renderWithRoute("/module/mod-a");

    expect(await screen.findByText("Module A")).toBeInTheDocument();
    expect(screen.getByText("1.0.0")).toBeInTheDocument();
    expect(fetchModule).toHaveBeenCalledWith("mod-a");
    expect(fetchModuleReleases).toHaveBeenCalledWith("mod-a", false, false);
  });

  test("handles encoded module name in the route", async () => {
    const fetchModule = vi.mocked(api.fetchModule);
    const fetchModuleReleases = vi.mocked(api.fetchModuleReleases);

    fetchModule.mockResolvedValueOnce({ name: "a/b", localizedName: "A / B" });
    fetchModuleReleases.mockResolvedValueOnce([]);

    renderWithRoute("/module/a%2Fb");

    expect(await screen.findByText("A / B")).toBeInTheDocument();
    expect(fetchModule).toHaveBeenCalledWith("a/b");
  });
});
