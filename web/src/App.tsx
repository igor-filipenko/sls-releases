import { BrowserRouter, Link, Navigate, Route, Routes } from "react-router-dom";
import { useEffect, useState } from "react";
import { Moon, Sun } from "lucide-react";

import { ReleasesPage } from "@/pages/ReleasesPage";
import { ModulePage } from "@/pages/ModulePage";
import { Switch } from "@/components/ui/switch";

export default function App() {
  const [isDark, setIsDark] = useState(() => {
    const stored = localStorage.getItem("theme");
    const prefersDark = window.matchMedia?.("(prefers-color-scheme: dark)").matches ?? false;
    return stored === "dark" || (stored == null && prefersDark);
  });

  useEffect(() => {
    document.documentElement.classList.toggle("dark", isDark);
    localStorage.setItem("theme", isDark ? "dark" : "light");
  }, [isDark]);

  return (
    <BrowserRouter>
      <div className="min-h-svh bg-background">
        <header className="border-b bg-card/50 backdrop-blur-sm">
          <div className="mx-auto flex max-w-6xl items-center justify-between px-4 py-4">
            <Link to="/" className="text-lg font-semibold tracking-tight">
              SL Releases
            </Link>
            <div className="flex items-center gap-2">
              <Sun className="size-4 text-muted-foreground" aria-hidden="true" />
              <Switch checked={isDark} onCheckedChange={setIsDark} aria-label="Toggle dark mode" />
              <Moon className="size-4 text-muted-foreground" aria-hidden="true" />
            </div>
          </div>
        </header>
        <main className="mx-auto max-w-6xl px-4 py-8">
          <Routes>
            <Route path="/" element={<ReleasesPage />} />
            <Route path="/module/:name" element={<ModulePage />} />
            <Route path="*" element={<Navigate to="/" replace />} />
          </Routes>
        </main>
      </div>
    </BrowserRouter>
  );
}
