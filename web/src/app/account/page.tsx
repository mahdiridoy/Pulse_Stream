"use client";

import { useState, useEffect } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { useAuth } from "@/lib/auth-context";

export default function AccountPage() {
  const { user, isAuthenticated, loading, logout } = useAuth();
  const router = useRouter();
  const [favorites, setFavorites] = useState<any[]>([]);
  const [history, setHistory] = useState<any[]>([]);
  const [tab, setTab] = useState<"overview" | "favorites" | "history">("overview");

  useEffect(() => {
    if (!loading && !isAuthenticated) {
      router.push("/login");
    }
  }, [loading, isAuthenticated, router]);

  useEffect(() => {
    if (!isAuthenticated) return;
    const token = localStorage.getItem("pulse_token");
    const headers = { Authorization: `Bearer ${token}` };

    fetch("/api/v1/favorites", { headers })
      .then((r) => r.json())
      .then((d) => {
        if (d.success) setFavorites(d.data || []);
      })
      .catch(() => {});

    fetch("/api/v1/history", { headers })
      .then((r) => r.json())
      .then((d) => {
        if (d.success) setHistory(d.data || []);
      })
      .catch(() => {});
  }, [isAuthenticated]);

  if (loading || !isAuthenticated || !user) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="w-12 h-12 border-4 border-white/20 border-t-accent rounded-full animate-spin" />
      </div>
    );
  }

  return (
    <div className="min-h-screen pt-24 pb-16">
      <div className="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-white mb-2">
            {user.username}
          </h1>
          <p className="text-text-muted">{user.email}</p>
        </div>

        <div className="flex gap-1 p-1 bg-surface-100 rounded-xl w-fit mb-8">
          {[
            { key: "overview", label: "Overview" },
            { key: "favorites", label: `My List (${favorites.length})` },
            { key: "history", label: `History (${history.length})` },
          ].map((t) => (
            <button
              key={t.key}
              onClick={() => setTab(t.key as typeof tab)}
              className={`px-4 py-2 text-sm font-medium rounded-lg transition-all ${
                tab === t.key
                  ? "bg-accent text-white"
                  : "text-text-muted hover:text-white hover:bg-white/5"
              }`}
            >
              {t.label}
            </button>
          ))}
        </div>

        {tab === "overview" && (
          <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
            <div className="bg-surface-50 border border-white/10 rounded-xl p-6">
              <p className="text-text-muted text-sm mb-1">Role</p>
              <p className="text-white font-semibold capitalize">{user.role}</p>
            </div>
            <div className="bg-surface-50 border border-white/10 rounded-xl p-6">
              <p className="text-text-muted text-sm mb-1">My List</p>
              <p className="text-white font-semibold">{favorites.length} items</p>
            </div>
            <div className="bg-surface-50 border border-white/10 rounded-xl p-6">
              <p className="text-text-muted text-sm mb-1">Watch History</p>
              <p className="text-white font-semibold">{history.length} items</p>
            </div>
            <div className="sm:col-span-3 mt-4">
              <button
                onClick={() => {
                  logout();
                  router.push("/");
                }}
                className="px-6 py-2.5 bg-red-500/10 border border-red-500/20 text-red-400 rounded-xl hover:bg-red-500/20 transition-colors"
              >
                Sign Out
              </button>
            </div>
          </div>
        )}

        {tab === "favorites" && (
          <div>
            {favorites.length === 0 ? (
              <div className="text-center py-20">
                <p className="text-text-muted text-lg mb-2">Your list is empty</p>
                <Link href="/" className="text-accent hover:text-accent-light">
                  Browse content
                </Link>
              </div>
            ) : (
              <div className="space-y-2">
                {favorites.map((fav: any) => (
                  <div
                    key={fav.id}
                    className="flex items-center gap-4 p-3 bg-surface-50 rounded-xl border border-white/5"
                  >
                    {fav.poster_url && (
                      <img
                        src={fav.poster_url}
                        alt={fav.title || ""}
                        className="w-12 h-16 object-cover rounded-lg"
                      />
                    )}
                    <div className="flex-1">
                      <p className="text-white font-medium">{fav.title || fav.content_id}</p>
                      <p className="text-text-muted text-xs">
                        {fav.media_type} · {fav.provider}
                      </p>
                    </div>
                    <button
                      onClick={async () => {
                        const token = localStorage.getItem("pulse_token");
                        await fetch(`/api/v1/favorites/${fav.id}`, {
                          method: "DELETE",
                          headers: { Authorization: `Bearer ${token}` },
                        });
                        setFavorites((prev) => prev.filter((f: any) => f.id !== fav.id));
                      }}
                      className="text-text-dim hover:text-red-400 text-sm transition-colors"
                    >
                      Remove
                    </button>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}

        {tab === "history" && (
          <div>
            {history.length === 0 ? (
              <div className="text-center py-20">
                <p className="text-text-muted text-lg">No watch history yet</p>
              </div>
            ) : (
              <div className="space-y-2">
                {history.map((h: any) => (
                  <div
                    key={h.id}
                    className="flex items-center gap-4 p-3 bg-surface-50 rounded-xl border border-white/5"
                  >
                    {h.poster_url && (
                      <img
                        src={h.poster_url}
                        alt={h.title || ""}
                        className="w-12 h-16 object-cover rounded-lg"
                      />
                    )}
                    <div className="flex-1">
                      <p className="text-white font-medium">{h.title || h.content_id}</p>
                      <p className="text-text-muted text-xs">
                        {h.media_type} · {h.season ? `S${h.season}E${h.episode}` : "Movie"}
                      </p>
                      {h.duration > 0 && (
                        <div className="mt-1 w-32 h-1 bg-white/10 rounded-full">
                          <div
                            className="h-full bg-accent rounded-full"
                            style={{
                              width: `${Math.min(100, (h.position / h.duration) * 100)}%`,
                            }}
                          />
                        </div>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
