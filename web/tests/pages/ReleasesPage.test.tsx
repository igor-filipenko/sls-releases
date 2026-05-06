import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import { describe, expect, test, vi } from "vitest";

import * as api from "@/lib/api";
import { ReleasesPage } from "@/pages/ReleasesPage";

vi.mock("@/lib/api", async () => {
  const actual = await vi.importActual<typeof import("@/lib/api")>("@/lib/api");
  return {
    ...actual,
    fetchReleases: vi.fn(),
  };
});

describe("ReleasesPage", () => {
  test("renders rows and reload triggers another load", async () => {
    const fetchReleases = vi.mocked(api.fetchReleases);
    fetchReleases.mockResolvedValueOnce([
      {
        name: "mod-a",
        localizedName: "Module A",
        version: "1.2.3",
        dateTime: "2026-01-01 00:00:00",
        kind: "Release",
        url: "https://example.invalid/a",
      },
    ]);
    fetchReleases.mockResolvedValueOnce([
      {
        name: "mod-a",
        localizedName: "Module A",
        version: "1.2.4",
        dateTime: "2026-01-02 00:00:00",
        kind: "Release",
        url: "https://example.invalid/b",
      },
    ]);

    const user = userEvent.setup();
    render(
      <MemoryRouter>
        <ReleasesPage />
      </MemoryRouter>,
    );

    expect(await screen.findByText("mod-a")).toBeInTheDocument();
    expect(screen.getByText("1.2.3")).toBeInTheDocument();
    expect(fetchReleases).toHaveBeenCalledTimes(1);

    await user.click(screen.getByRole("button", { name: "Reload releases" }));

    expect(await screen.findByText("1.2.4")).toBeInTheDocument();
    expect(fetchReleases).toHaveBeenCalledTimes(2);
  });

  test("shows error and allows retry", async () => {
    const fetchReleases = vi.mocked(api.fetchReleases);
    fetchReleases.mockRejectedValueOnce(new Error("boom"));
    fetchReleases.mockResolvedValueOnce([
      {
        name: "mod-b",
        localizedName: "Module B",
        version: "2.0.0",
        dateTime: "2026-01-01 00:00:00",
        kind: "Release",
        url: "https://example.invalid/c",
      },
    ]);

    const user = userEvent.setup();
    render(
      <MemoryRouter>
        <ReleasesPage />
      </MemoryRouter>,
    );

    expect(await screen.findByText("boom")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Retry" }));

    expect(await screen.findByText("mod-b")).toBeInTheDocument();

    const row = screen.getByText("mod-b").closest("tr");
    expect(row).not.toBeNull();
    expect(within(row as HTMLElement).getByText("2.0.0")).toBeInTheDocument();
  });
});
