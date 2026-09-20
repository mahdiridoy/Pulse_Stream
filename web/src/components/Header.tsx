"use client";

import Link from "next/link";
import { useState, useEffect } from "react";
import {
  MagnifyingGlassIcon,
  UserIcon,
  Bars3Icon,
  XMarkIcon,
} from "@heroicons/react/24/outline";
import SearchBar from "./SearchBar";
import { useAuth } from "@/lib/auth-context";

export default function Header() {
  const { user, isAuthenticated, loading } = useAuth();
  const [scrolled, setScrolled] = useState(false);
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);
  const [searchOpen, setSearchOpen] = useState(false);

  useEffect(() => {
    const handler = () => setScrolled(window.scrollY > 20);
    window.addEventListener("scroll", handler);
    return () => window.removeEventListener("scroll", handler);
  }, []);

  const navLinks = [
    { href: "/", label: "Home" },
    { href: "/iptv", label: "IPTV" },
    { href: "/search?type=movie", label: "Movies" },
    { href: "/search?type=series", label: "Series" },
  ];

  return (
    <header
      className={`fixed top-0 left-0 right-0 z-50 transition-all duration-300 ${
        scrolled ? "glass-strong shadow-2xl" : "bg-transparent"
      }`}
    >
      <div className="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex items-center justify-between h-16 lg:h-20">
          <Link href="/" className="flex items-center gap-2 shrink-0">
            <div className="w-8 h-8 lg:w-10 lg:h-10 rounded-xl bg-gradient-brand flex items-center justify-center">
              <svg
                viewBox="0 0 24 24"
                className="w-5 h-5 lg:w-6 lg:h-6 text-white"
                fill="currentColor"
              >
                <path d="M8 5v14l11-7z" />
              </svg>
            </div>
            <span className="text-lg lg:text-xl font-bold text-white hidden sm:block">
              Pulse<span className="text-gradient">Stream</span>
            </span>
          </Link>

          <nav className="hidden lg:flex items-center gap-1">
            {navLinks.map((link) => (
              <Link
                key={link.href}
                href={link.href}
                className="px-4 py-2 text-sm font-medium text-text-muted hover:text-white transition-colors rounded-lg hover:bg-white/5"
              >
                {link.label}
              </Link>
            ))}
            {!loading && isAuthenticated && user?.role === "admin" && (
              <Link
                href="/admin"
                className="px-4 py-2 text-sm font-medium text-purple-400 hover:text-white transition-colors rounded-lg hover:bg-white/5"
              >
                Admin
              </Link>
            )}
          </nav>

          <div className="flex items-center gap-2">
            <div
              className={`transition-all duration-300 overflow-hidden ${
                searchOpen ? "w-64 lg:w-80" : "w-0 lg:w-auto"
              }`}
            >
              {searchOpen && (
                <SearchBar onClose={() => setSearchOpen(false)} autoFocus />
              )}
            </div>

            <button
              onClick={() => setSearchOpen(!searchOpen)}
              className="p-2 text-text-muted hover:text-white transition-colors rounded-lg hover:bg-white/5"
            >
              <MagnifyingGlassIcon className="w-5 h-5" />
            </button>

            {!loading && (
              <Link
                href={isAuthenticated ? "/account" : "/login"}
                className="p-2 text-text-muted hover:text-white transition-colors rounded-lg hover:bg-white/5 flex items-center gap-1.5"
              >
                {isAuthenticated ? (
                  <div className="w-6 h-6 rounded-full bg-accent flex items-center justify-center text-white text-xs font-bold">
                    {user?.username?.charAt(0).toUpperCase() || "U"}
                  </div>
                ) : (
                  <UserIcon className="w-5 h-5" />
                )}
              </Link>
            )}

            <button
              onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
              className="lg:hidden p-2 text-text-muted hover:text-white transition-colors"
            >
              {mobileMenuOpen ? (
                <XMarkIcon className="w-5 h-5" />
              ) : (
                <Bars3Icon className="w-5 h-5" />
              )}
            </button>
          </div>
        </div>
      </div>

      {mobileMenuOpen && (
        <div className="lg:hidden glass-strong border-t border-white/5 animate-fade-in">
          <nav className="px-4 py-4 space-y-1">
            {navLinks.map((link) => (
              <Link
                key={link.href}
                href={link.href}
                onClick={() => setMobileMenuOpen(false)}
                className="block px-4 py-3 text-sm font-medium text-text-muted hover:text-white hover:bg-white/5 rounded-lg transition-colors"
              >
                {link.label}
              </Link>
            ))}
            {!loading && isAuthenticated && user?.role === "admin" && (
              <Link
                href="/admin"
                onClick={() => setMobileMenuOpen(false)}
                className="block px-4 py-3 text-sm font-medium text-purple-400 hover:text-white hover:bg-white/5 rounded-lg transition-colors"
              >
                Admin
              </Link>
            )}
            {!loading && (
              <Link
                href={isAuthenticated ? "/account" : "/login"}
                onClick={() => setMobileMenuOpen(false)}
                className="block px-4 py-3 text-sm font-medium text-text-muted hover:text-white hover:bg-white/5 rounded-lg transition-colors"
              >
                {isAuthenticated ? "Account" : "Sign In"}
              </Link>
            )}
          </nav>
        </div>
      )}
    </header>
  );
}
