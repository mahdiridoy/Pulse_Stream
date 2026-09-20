"use client";

import { Media } from "@/types";
import Link from "next/link";
import { PlayIcon } from "@heroicons/react/24/solid";

interface ContentCardProps {
  media: Media;
  size?: "sm" | "md" | "lg";
}

export default function ContentCard({ media, size = "md" }: ContentCardProps) {
  const sizeClasses = {
    sm: "w-[130px] sm:w-[150px]",
    md: "w-[160px] sm:w-[180px] lg:w-[200px]",
    lg: "w-[200px] sm:w-[240px] lg:w-[280px]",
  };

  const heightClasses = {
    sm: "h-[195px] sm:h-[225px]",
    md: "h-[240px] sm:h-[270px] lg:h-[300px]",
    lg: "h-[300px] sm:h-[360px] lg:h-[420px]",
  };

  return (
    <Link href={`/${media.type}/${media.id}`} className="block shrink-0">
      <div className={`${sizeClasses[size]} card-hover rounded-xl overflow-hidden relative group cursor-pointer`}>
        <div
          className={`${heightClasses[size]} bg-surface-100 rounded-xl overflow-hidden relative`}
        >
          <img
            src={media.posterPath}
            alt={media.title}
            className="w-full h-full object-cover"
            loading="lazy"
          />

          <div className="absolute inset-0 bg-gradient-to-t from-black/90 via-black/20 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300" />

          {media.quality && (
            <div className="absolute top-2 left-2">
              <span className="badge-quality text-[10px]">{media.quality}</span>
            </div>
          )}

          <div className="absolute inset-0 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-all duration-300">
            <div className="w-12 h-12 rounded-full bg-white/20 backdrop-blur-sm flex items-center justify-center border border-white/20 group-hover:scale-110 transition-transform duration-300">
              <PlayIcon className="w-6 h-6 text-white fill-current ml-0.5" />
            </div>
          </div>

          <div className="absolute bottom-0 left-0 right-0 p-3 translate-y-full group-hover:translate-y-0 transition-transform duration-300">
            <h3 className="text-sm font-semibold text-white line-clamp-2 mb-1">
              {media.title}
            </h3>
            <div className="flex items-center gap-2 text-xs text-text-muted">
              <span>{media.year || media.releaseDate?.split("-")[0]}</span>
              {media.rating > 0 && (
                <span className="flex items-center gap-0.5 text-amber-400">
                  <svg className="w-3 h-3 fill-current" viewBox="0 0 20 20">
                    <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
                  </svg>
                  {media.rating.toFixed(1)}
                </span>
              )}
            </div>
          </div>
        </div>
      </div>
    </Link>
  );
}
