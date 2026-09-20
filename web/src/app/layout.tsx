import type { Metadata } from "next";
import "./globals.css";
import Header from "@/components/Header";
import { AuthProvider } from "@/lib/auth-context";

export const metadata: Metadata = {
  title: "Pulse Stream - Watch Movies & Series",
  description:
    "Stream thousands of movies and TV series in stunning quality. Premium entertainment at your fingertips.",
  keywords: "streaming, movies, series, anime, watch online",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" className="dark">
      <body className="min-h-screen bg-surface antialiased">
        <AuthProvider>
          <Header />
          <main className="relative">{children}</main>
        </AuthProvider>
      </body>
    </html>
  );
}
