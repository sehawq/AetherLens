"use client";

export default function GlobalError({ reset }: { error: Error & { digest?: string }; reset: () => void }) {
  return (
    <html lang="en">
      <body>
        <div className="flex min-h-screen items-center justify-center bg-[#0a0a0a] px-6 text-white">
          <div className="w-full max-w-md rounded-xl border border-white/10 bg-[#050505] p-6 text-center">
            <h2 className="text-lg font-semibold tracking-wide">Critical error</h2>
            <p className="mt-2 text-sm text-white/60">A global rendering error occurred.</p>
            <button
              onClick={reset}
              className="mt-4 rounded bg-[#0096ff] px-4 py-2 text-sm font-semibold text-white hover:opacity-90"
            >
              Reload
            </button>
          </div>
        </div>
      </body>
    </html>
  );
}
