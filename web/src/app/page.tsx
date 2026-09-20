"use client";

import { useState, useEffect } from "react";
import HeroSection from "@/components/HeroSection";
import ContentRow from "@/components/ContentRow";
import { HeroSkeleton, RowSkeleton } from "@/components/Skeleton";
import { Media } from "@/types";
import { getHome } from "@/lib/api";

export default function HomePage() {
  const [featured, setFeatured] = useState<Media | null>(null);
  const [trending, setTrending] = useState<Media[]>([]);
  const [movies, setMovies] = useState<Media[]>([]);
  const [series, setSeries] = useState<Media[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const controller = new AbortController();
    setLoading(true);
    getHome(controller.signal).then((data) => {
      setFeatured(data.featured || null);
      setTrending(data.trending);
      setMovies(data.movies);
      setSeries(data.series);
      setLoading(false);
    });
    return () => controller.abort();
  }, []);

  if (loading) {
    return (
      <div>
        <HeroSkeleton />
        <RowSkeleton count={7} size="md" />
        <RowSkeleton count={7} size="md" />
        <RowSkeleton count={7} size="md" />
      </div>
    );
  }

  return (
    <div className="pb-16">
      {featured && <HeroSection media={featured} />}

      <div className="relative -mt-16 z-10">
        {trending.length > 0 && (
          <ContentRow title="Trending Now" items={trending} size="md" />
        )}
        {movies.length > 0 && (
          <ContentRow
            title="Popular Movies"
            items={movies}
            size="md"
            href="/search?type=movie"
          />
        )}
        {series.length > 0 && (
          <ContentRow
            title="Popular Series"
            items={series}
            size="md"
            href="/search?type=series"
          />
        )}
        {movies.length === 0 && series.length === 0 && trending.length === 0 && (
          <div className="text-center py-20">
            <p className="text-text-muted text-lg">
              No content available. Check your backend connection.
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
