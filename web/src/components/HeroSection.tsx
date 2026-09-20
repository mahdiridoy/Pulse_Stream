"use client";

import { Media } from "@/types";
import { PlayIcon, InformationCircleIcon } from "@heroicons/react/24/solid";
import Link from "next/link";

interface HeroSectionProps {
  media: Media;
}

export default function HeroSection({ media }: HeroSectionProps) {
  return (
    <section className="relative w-full h-[70vh] min-h-[500px] max-h-[800px]">
      <div
        className="absolute inset-0 bg-cover bg-center bg-no-repeat"
        style={{ backgroundImage: `url(${media.backdropPath})` }}
      >
        <div className="absolute inset-0 bg-gradient-to-r from-surface via-surface/80 to-transparent" />
        <div className="absolute inset-0 bg-gradient-to-t from-surface via-surface/20 to-transparent" />
        <div className="absolute inset-0 bg-gradient-to-b from-surface/60 via-transparent to-surface" />
      </div>

      <div className="absolute inset-0 flex items-end pb-16 lg:pb-24">
        <div className="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8 w-full">
          <div className="max-w-2xl animate-slide-up">
            <div className="flex items-center gap-3 mb-4">
              {media.quality && <span className="badge-quality">{media.quality}</span>}
              <span className="text-sm text-text-muted font-medium">{media.year || media.releaseDate?.split("-")[0]}</span>
              {media.rating > 0 && (
                <span className="flex items-center gap-1 text-sm text-amber-400">
                  <svg className="w-4 h-4 fill-current" viewBox="0 0 20 20">
                    <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
                  </svg>
                  {media.rating.toFixed(1)}
                </span>
              )}
            </div>

            <h1 className="text-3xl sm:text-4xl lg:text-6xl font-bold text-white mb-4 leading-tight tracking-tight">
              {media.title}
            </h1>

            <div className="flex flex-wrap gap-2 mb-4">
              {media.genres?.slice(0, 3).map((genre) => (
                <span
                  key={genre}
                  className="px-3 py-1 text-xs font-medium text-text-muted bg-white/5 rounded-full border border-white/5"
                >
                  {genre}
                </span>
              ))}
            </div>

            <p className="text-sm sm:text-base text-text-muted leading-relaxed mb-8 line-clamp-3">
              {media.overview}
            </p>

            <div className="flex items-center gap-3">
              <Link
                href={`/watch/${media.type}/${media.id}`}
                className="btn-primary flex items-center gap-2 text-sm sm:text-base"
              >
                <PlayIcon className="w-5 h-5 fill-current" />
                Watch Now
              </Link>
              <Link
                href={`/${media.type}/${media.id}`}
                className="btn-secondary flex items-center gap-2 text-sm sm:text-base"
              >
                <InformationCircleIcon className="w-5 h-5" />
                Details
              </Link>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}
