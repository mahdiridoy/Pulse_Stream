"use client";

import { useState, useEffect } from "react";
import { useParams } from "next/navigation";
import Link from "next/link";
import { Media, Season, Episode } from "@/types";
import { PlayIcon, StarIcon, CalendarIcon, ChevronDownIcon } from "@heroicons/react/24/solid";
import { getSeriesDetails } from "@/lib/api";

export default function SeriesDetailPage() {
  const params = useParams();
  const id = params.id as string;
  const [series, setSeries] = useState<(Media & { seasons_list: Season[] }) | null>(null);
  const [selectedSeason, setSelectedSeason] = useState(1);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);

  useEffect(() => {
    const controller = new AbortController();
    setLoading(true);
    setError(false);
    getSeriesDetails(id, "moviebox", controller.signal).then((data) => {
      if (data && data.seasons_list && data.seasons_list.length > 0) {
        setSeries(data as Media & { seasons_list: Season[] });
      } else {
        setError(true);
      }
      setLoading(false);
    });
    return () => controller.abort();
  }, [id]);

  if (loading || !series) {
    if (error) {
      return (
        <div className="min-h-screen flex items-center justify-center">
          <div className="text-center">
            <h2 className="text-xl font-semibold text-white mb-2">Series not found</h2>
            <p className="text-text-muted mb-4">This series could not be loaded.</p>
            <Link href="/" className="text-accent hover:text-accent-light">
              Go Home
            </Link>
          </div>
        </div>
      );
    }
    return (
      <div className="min-h-screen">
        <div className="w-full h-[50vh] skeleton" />
        <div className="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8 -mt-24 relative z-10">
          <div className="flex flex-col lg:flex-row gap-8">
            <div className="w-48 lg:w-64 shrink-0">
              <div className="aspect-[2/3] skeleton rounded-xl" />
            </div>
            <div className="flex-1 space-y-4 pt-16 lg:pt-0">
              <div className="skeleton w-3/4 h-10 rounded-lg" />
              <div className="skeleton w-1/2 h-6 rounded-md" />
              <div className="skeleton w-full h-20 rounded-lg" />
            </div>
          </div>
        </div>
      </div>
    );
  }

  const currentSeason =
    series.seasons_list.find((s) => s.number === selectedSeason) ||
    series.seasons_list[0];

  const rating = typeof series.rating === "number" ? series.rating : 0;

  return (
    <div className="min-h-screen pb-16">
      <div className="relative w-full h-[50vh] lg:h-[60vh]">
        {series.backdropPath ? (
          <div
            className="absolute inset-0 bg-cover bg-center"
            style={{ backgroundImage: `url(${series.backdropPath})` }}
          />
        ) : (
          <div className="absolute inset-0 bg-surface-100" />
        )}
        <div className="absolute inset-0 bg-gradient-to-t from-surface via-surface/50 to-surface/20" />
        <div className="absolute inset-0 bg-gradient-to-r from-surface/80 to-transparent" />
      </div>

      <div className="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8 -mt-32 lg:-mt-40 relative z-10">
        <div className="flex flex-col lg:flex-row gap-6 lg:gap-10 mb-10">
          <div className="w-40 sm:w-48 lg:w-64 shrink-0">
            {series.posterPath ? (
              <img
                src={series.posterPath}
                alt={series.title}
                className="w-full aspect-[2/3] object-cover rounded-xl shadow-2xl shadow-black/50"
              />
            ) : (
              <div className="w-full aspect-[2/3] skeleton rounded-xl" />
            )}
          </div>
          <div className="flex-1">
            <h1 className="text-2xl sm:text-3xl lg:text-5xl font-bold text-white mb-3">
              {series.title}
            </h1>
            <div className="flex flex-wrap items-center gap-3 mb-4 text-sm">
              {series.quality && (
                <span className="badge-quality">{series.quality}</span>
              )}
              {rating > 0 && (
                <span className="flex items-center gap-1 text-amber-400">
                  <StarIcon className="w-4 h-4 fill-current" /> {rating.toFixed(1)}
                </span>
              )}
              {series.seasonCount && (
                <span className="text-text-muted">
                  {series.seasonCount} Season{series.seasonCount > 1 ? "s" : ""}
                </span>
              )}
              {series.releaseDate && (
                <span className="flex items-center gap-1 text-text-muted">
                  <CalendarIcon className="w-4 h-4" /> {series.releaseDate}
                </span>
              )}
            </div>
            {series.genres.length > 0 && (
              <div className="flex flex-wrap gap-2 mb-5">
                {series.genres.map((g) => (
                  <span
                    key={g}
                    className="px-3 py-1 text-xs font-medium text-text-muted bg-white/5 rounded-full border border-white/5"
                  >
                    {g}
                  </span>
                ))}
              </div>
            )}
            {series.overview && (
              <p className="text-text-muted leading-relaxed mb-6 max-w-3xl">
                {series.overview}
              </p>
            )}
            <Link
              href={`/watch/series/${id}?season=1&episode=1`}
              className="btn-primary inline-flex items-center gap-2"
            >
              <PlayIcon className="w-5 h-5 fill-current" /> S1 E1 - Play
            </Link>
          </div>
        </div>

        <div className="mb-6">
          <div className="flex items-center gap-2 mb-4">
            <h2 className="text-lg font-bold text-white">Episodes</h2>
            <div className="relative">
              <select
                value={selectedSeason}
                onChange={(e) => setSelectedSeason(Number(e.target.value))}
                className="appearance-none pl-3 pr-8 py-1.5 bg-surface-100 border border-white/10 rounded-lg text-sm text-white focus:outline-none focus:border-accent/50 cursor-pointer"
              >
                {series.seasons_list.map((s) => (
                  <option key={s.number} value={s.number}>
                    {s.name}
                  </option>
                ))}
              </select>
              <ChevronDownIcon className="absolute right-2 top-1/2 -translate-y-1/2 w-4 h-4 text-text-dim pointer-events-none" />
            </div>
          </div>

          <div className="space-y-2">
            {currentSeason.episodes?.map((ep) => (
              <Link
                key={ep.id}
                href={`/watch/series/${id}?season=${selectedSeason}&episode=${ep.number}`}
                className="flex gap-4 p-3 bg-surface-50 rounded-xl border border-white/5 hover:bg-surface-100 hover:border-white/10 transition-all duration-200 group cursor-pointer"
              >
                <div className="w-32 sm:w-40 lg:w-48 aspect-video bg-surface-100 rounded-lg overflow-hidden shrink-0 relative">
                  {ep.stillPath ? (
                    <img
                      src={ep.stillPath}
                      alt={ep.title}
                      className="w-full h-full object-cover"
                    />
                  ) : (
                    <div className="w-full h-full bg-surface-200" />
                  )}
                  <div className="absolute inset-0 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity bg-black/40">
                    <PlayIcon className="w-8 h-8 text-white fill-current" />
                  </div>
                </div>
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 mb-1">
                    <span className="text-xs text-text-dim font-medium">
                      E{ep.number}
                    </span>
                    <h3 className="text-sm font-semibold text-white truncate">
                      {ep.title}
                    </h3>
                  </div>
                  {ep.overview && (
                    <p className="text-xs text-text-muted line-clamp-2">
                      {ep.overview}
                    </p>
                  )}
                </div>
              </Link>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
