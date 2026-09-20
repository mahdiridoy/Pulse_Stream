"use client";

import { useState, useEffect, useCallback, Suspense, useRef } from "react";
import { useSearchParams, useRouter } from "next/navigation";
import ContentCard from "@/components/ContentCard";
import { CardSkeleton } from "@/components/Skeleton";
import { Media } from "@/types";
import { MagnifyingGlassIcon } from "@heroicons/react/24/outline";
import { searchMedia } from "@/lib/api";

function SearchContent() {
  const searchParams = useSearchParams();
  const router = useRouter();
  const query = searchParams.get("q") || "";
  const typeFilter = searchParams.get("type") || "all";

  const [results, setResults] = useState<Media[]>([]);
  const [loading, setLoading] = useState(false);
  const [searchInput, setSearchInput] = useState(query);
  const abortRef = useRef<AbortController | null>(null);

  const doSearch = useCallback(
    (q: string, type: string) => {
      if (abortRef.current) abortRef.current.abort();
      if (!q.trim()) {
        setResults([]);
        setLoading(false);
        return;
      }

      const controller = new AbortController();
      abortRef.current = controller;
      setLoading(true);

      searchMedia(q, { signal: controller.signal }).then((items) => {
        let filtered = items;
        if (type === "movie") filtered = items.filter((m) => m.type === "movie");
        else if (type === "series")
          filtered = items.filter((m) => m.type === "series");
        setResults(filtered);
        setLoading(false);
      });
    },
    []
  );

  useEffect(() => {
    const timer = setTimeout(() => {
      if (query.trim()) doSearch(query, typeFilter);
      else {
        setResults([]);
        setLoading(false);
      }
    }, 400);
    return () => clearTimeout(timer);
  }, [query, typeFilter, doSearch]);

  useEffect(() => {
    return () => {
      if (abortRef.current) abortRef.current.abort();
    };
  }, []);

  const handleSearchSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    const params = new URLSearchParams();
    if (searchInput.trim()) params.set("q", searchInput.trim());
    if (typeFilter !== "all") params.set("type", typeFilter);
    router.push(`/search?${params.toString()}`);
  };

  const tabs = [
    { key: "all", label: "All" },
    { key: "movie", label: "Movies" },
    { key: "series", label: "Series" },
  ];

  return (
    <div className="min-h-screen pt-24 pb-16">
      <div className="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
        <div className="max-w-2xl mx-auto mb-8">
          <form onSubmit={handleSearchSubmit} className="relative">
            <MagnifyingGlassIcon className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-text-dim" />
            <input
              type="text"
              value={searchInput}
              onChange={(e) => setSearchInput(e.target.value)}
              placeholder="Search for movies, series..."
              className="w-full pl-12 pr-4 py-4 bg-surface-100 border border-white/10 rounded-2xl
                         text-base text-white placeholder:text-text-dim
                         focus:outline-none focus:border-accent/50 focus:ring-2 focus:ring-accent/25
                         transition-all duration-200"
              autoFocus
            />
          </form>

          <div className="flex gap-1 mt-4 p-1 bg-surface-100 rounded-xl w-fit">
            {tabs.map((tab) => (
              <button
                key={tab.key}
                onClick={() => {
                  const params = new URLSearchParams();
                  if (query) params.set("q", query);
                  if (tab.key !== "all") params.set("type", tab.key);
                  router.push(`/search?${params.toString()}`);
                }}
                className={`px-4 py-2 text-sm font-medium rounded-lg transition-all duration-200 ${
                  typeFilter === tab.key
                    ? "bg-accent text-white shadow-lg shadow-accent/25"
                    : "text-text-muted hover:text-white hover:bg-white/5"
                }`}
              >
                {tab.label}
              </button>
            ))}
          </div>
        </div>

        {query && (
          <p className="text-text-muted text-sm mb-6">
            {loading ? "Searching..." : `${results.length} results for "${query}"`}
          </p>
        )}

        {loading ? (
          <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-3 lg:gap-4">
            {Array.from({ length: 12 }).map((_, i) => (
              <CardSkeleton key={i} size="md" />
            ))}
          </div>
        ) : results.length > 0 ? (
          <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-3 lg:gap-4">
            {results.map((item, idx) => (
              <ContentCard key={`${item.type}-${item.id}-${idx}`} media={item} size="md" />
            ))}
          </div>
        ) : query && !loading ? (
          <div className="text-center py-20">
            <MagnifyingGlassIcon className="w-16 h-16 text-surface-300 mx-auto mb-4" />
            <h3 className="text-xl font-semibold text-white mb-2">No results found</h3>
            <p className="text-text-muted">Try adjusting your search or filters</p>
          </div>
        ) : (
          <div className="text-center py-20">
            <MagnifyingGlassIcon className="w-16 h-16 text-surface-300 mx-auto mb-4" />
            <h3 className="text-xl font-semibold text-white mb-2">Search for content</h3>
            <p className="text-text-muted">Type a movie or series name to get started</p>
          </div>
        )}
      </div>
    </div>
  );
}

export default function SearchPage() {
  return (
    <Suspense
      fallback={
        <div className="min-h-screen pt-24 pb-16">
          <div className="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
            <div className="max-w-2xl mx-auto mb-8">
              <div className="skeleton w-full h-14 rounded-2xl" />
              <div className="flex gap-1 mt-4 p-1 bg-surface-100 rounded-xl w-fit">
                <div className="skeleton w-16 h-9 rounded-lg" />
                <div className="skeleton w-16 h-9 rounded-lg" />
                <div className="skeleton w-16 h-9 rounded-lg" />
              </div>
            </div>
            <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-3 lg:gap-4">
              {Array.from({ length: 12 }).map((_, i) => (
                <CardSkeleton key={i} size="md" />
              ))}
            </div>
          </div>
        </div>
      }
    >
      <SearchContent />
    </Suspense>
  );
}
