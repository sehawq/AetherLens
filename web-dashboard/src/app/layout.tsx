import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "AetherLens | Network Traffic Analysis",
  description: "Real-time network traffic analysis engine - Public Showcase",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}
