import { BrowserRouter, Link, Navigate, Route, Routes } from "react-router-dom";

import { ReleasesPage } from "@/pages/ReleasesPage";
import { ModulePage } from "@/pages/ModulePage";

export default function App() {
  return (
    <BrowserRouter>
      <div className="min-h-svh bg-background">
        <header className="border-b bg-card/50 backdrop-blur-sm">
          <div className="mx-auto flex max-w-6xl items-center justify-between px-4 py-4">
            <Link to="/" className="text-lg font-semibold tracking-tight">
              SLS Releases
            </Link>
            <span className="text-xs text-muted-foreground">
              Bun · Vite · React · shadcn/ui
            </span>
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
