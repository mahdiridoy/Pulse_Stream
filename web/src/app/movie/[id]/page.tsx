"use client";

import { useState, useEffect } from "react";
import { useParams } from "next/navigation";
import Link from "next/link";
import { Media } from "@/types";
import { PlayIcon, StarIcon, ClockIcon, CalendarIcon } from "@heroicons/react/24/solid";
import { getMovieDetails } from "@/lib/api";

export default function MovieDetailPage() {
  const params = useParams();
  const id = params.id as string;
  const [media, setMedia] = useState<(Media & { seasons_list?: any[] }) | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState(false);

  useEffect(() => {
    const controller = new AbortController();
    setLoading(true);
    setError(false);
    getMovieDetails(id, "moviebox", controller.signal).then((data) => {
      if (data) {
        setMedia(data);
      } else {
        setError(true);
      }
      setLoading(false);
    });
    return () => controller.abort();
  }, [id]);

  if (loading || !media) {
    if (error) {
      return (
        <div className="min-h-screen flex items-center justify-center">
          <div className="text-center">
            <h2 className="text-xl font-semibold text-white mb-2">Movie not found</h2>
            <p className="text-text-muted mb-4">This movie could not be loaded.</p>
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

  const rating = typeof media.rating === "number" ? media.rating : 0;

  return (
    <div className="min-h-screen">
      <div className="relative w-full h-[50vh] lg:h-[60vh]">
        {media.backdropPath ? (
          <div
            className="absolute inset-0 bg-cover bg-center"
            style={{ backgroundImage: `url(${media.backdropPath})` }}
          />
        ) : (
          <div className="absolute inset-0 bg-surface-100" />
        )}
        <div className="absolute inset-0 bg-gradient-to-t from-surface via-surface/50 to-surface/20" />
        <div className="absolute inset-0 bg-gradient-to-r from-surface/80 to-transparent" />
      </div>

      <div className="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8 -mt-32 lg:-mt-40 relative z-10">
        <div className="flex flex-col lg:flex-row gap-6 lg:gap-10">
          <div className="w-40 sm:w-48 lg:w-64 shrink-0">
            {media.posterPath ? (
              <img
                src={media.posterPath}
                alt={media.title}
                className="w-full aspect-[2/3] object-cover rounded-xl shadow-2xl shadow-black/50"
              />
            ) : (
              <div className="w-full aspect-[2/3] skeleton rounded-xl" />
            )}
          </div>

          <div className="flex-1 pb-12">
            <h1 className="text-2xl sm:text-3xl lg:text-5xl font-bold text-white mb-3 leading-tight">
              {media.title}
            </h1>

            <div className="flex flex-wrap items-center gap-3 mb-4 text-sm">
              {media.quality && <span className="badge-quality">{media.quality}</span>}
              {rating > 0 && (
                <span className="flex items-center gap-1 text-amber-400">
                  <StarIcon className="w-4 h-4 fill-current" /> {rating.toFixed(1)}
                </span>
              )}
              {media.duration && (
                <span className="flex items-center gap-1 text-text-muted">
                  <ClockIcon className="w-4 h-4" /> {media.duration}m
                </span>
              )}
              {media.releaseDate && (
                <span className="flex items-center gap-1 text-text-muted">
                  <CalendarIcon className="w-4 h-4" /> {media.releaseDate}
                </span>
              )}
            </div>

            {media.genres.length > 0 && (
              <div className="flex flex-wrap gap-2 mb-5">
                {media.genres.map((g) => (
                  <span
                    key={g}
                    className="px-3 py-1 text-xs font-medium text-text-muted bg-white/5 rounded-full border border-white/5"
                  >
                    {g}
                  </span>
                ))}
              </div>
            )}

            {media.overview && (
              <p className="text-text-muted leading-relaxed mb-6 max-w-3xl">
                {media.overview}
              </p>
            )}

            <div className="flex items-center gap-3">
              <Link
                href={`/watch/movie/${id}`}
                className="btn-primary flex items-center gap-2"
              >
                <PlayIcon className="w-5 h-5 fill-current" /> Watch Now
              </Link>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
