"use client";

import { useState, useEffect, useCallback, Suspense, useRef } from "react";
import { useParams, useSearchParams, useRouter } from "next/navigation";
import Link from "next/link";
import VideoPlayer from "@/components/VideoPlayer";
import { Media, StreamSource } from "@/types";
import {
  ArrowLeftIcon,
  ChevronLeftIcon,
  ChevronRightIcon,
} from "@heroicons/react/24/outline";
import { getMovieDetails, getSeriesDetails, getStreams } from "@/lib/api";

function WatchContent() {
  const params = useParams();
  const searchParams = useSearchParams();
  const router = useRouter();

  const type = params.type as "movie" | "series";
  const id = params.id as string;
  const season = Number(searchParams.get("season") || 1);
  const episode = Number(searchParams.get("episode") || 1);

  const [media, setMedia] = useState<Media | null>(null);
  const [sources, setSources] = useState<StreamSource[]>([]);
  const [currentSourceIdx, setCurrentSourceIdx] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [provider, setProvider] = useState("moviebox");

  useEffect(() => {
    const controller = new AbortController();
    setLoading(true);
    setError(null);

    const load = async () => {
      try {
        if (type === "movie") {
          const data = await getMovieDetails(id, provider, controller.signal);
          if (data) setMedia(data);
        } else {
          const data = await getSeriesDetails(id, provider, controller.signal);
          if (data) setMedia(data);
        }

        const streams = await getStreams(
          provider,
          id,
          type === "series" ? season : undefined,
          type === "series" ? episode : undefined,
          controller.signal
        );
        if (streams.length > 0) {
          setSources(streams);
          setCurrentSourceIdx(0);
        } else {
          setError("No streams available for this content.");
        }
      } catch {
        if (!controller.signal.aborted) {
          setError("Failed to load content.");
        }
      }
      setLoading(false);
    };

    load();
    return () => controller.abort();
  }, [type, id, season, episode, provider]);

  const goNext = useCallback(() => {
    if (type === "series" && media) {
      router.push(
        `/watch/series/${id}?season=${season}&episode=${episode + 1}`
      );
    }
  }, [type, media, id, season, episode, router]);

  const goPrev = useCallback(() => {
    if (type === "series" && episode > 1) {
      router.push(
        `/watch/series/${id}?season=${season}&episode=${episode - 1}`
      );
    }
  }, [type, episode, season, id, router]);

  const tryNextSource = useCallback(() => {
    if (currentSourceIdx < sources.length - 1) {
      setCurrentSourceIdx((i) => i + 1);
    }
  }, [currentSourceIdx, sources.length]);

  if (loading) {
    return (
      <div className="min-h-screen bg-black flex items-center justify-center">
        <div className="w-12 h-12 border-4 border-white/20 border-t-accent rounded-full animate-spin" />
      </div>
    );
  }

  if (error && !media) {
    return (
      <div className="min-h-screen bg-black flex items-center justify-center">
        <div className="text-center">
          <h2 className="text-xl font-semibold text-white mb-2">
            Unable to load content
          </h2>
          <p className="text-text-muted mb-4">{error}</p>
          <Link href="/" className="text-accent hover:text-accent-light">
            Go Home
          </Link>
        </div>
      </div>
    );
  }

  const currentSource = sources[currentSourceIdx];
  const videoTitle =
    type === "movie"
      ? media?.title || "Unknown"
      : `${media?.title || "Unknown"} - S${season}E${episode}`;

  return (
    <div className="min-h-screen bg-black">
      <div className="fixed top-0 left-0 right-0 z-50 bg-gradient-to-b from-black/80 to-transparent p-3 sm:p-4">
        <div className="max-w-[1440px] mx-auto flex items-center justify-between">
          <div className="flex items-center gap-3">
            <button
              onClick={() => router.back()}
              className="p-2 text-white/70 hover:text-white transition-colors rounded-lg hover:bg-white/10"
            >
              <ArrowLeftIcon className="w-5 h-5" />
            </button>
            <div>
              <h1 className="text-white font-medium text-sm sm:text-base">
                {media?.title || "Unknown"}
              </h1>
              {type === "series" && (
                <p className="text-text-muted text-xs">
                  S{season} E{episode}
                </p>
              )}
            </div>
          </div>
          <Link
            href={`/${type}/${id}`}
            className="text-xs text-accent-light hover:text-accent transition-colors"
          >
            Details
          </Link>
        </div>
      </div>

      <div className="pt-14">
        {currentSource ? (
          <VideoPlayer
            key={currentSource.url}
            src={currentSource.url}
            poster={media?.backdropPath || undefined}
            title={videoTitle}
            onEnded={type === "series" ? goNext : undefined}
          />
        ) : (
          <div className="w-full aspect-video bg-black flex items-center justify-center">
            <div className="text-center">
              <p className="text-white mb-4">
                {error || "No playable source found."}
              </p>
              {sources.length > 1 && currentSourceIdx < sources.length - 1 && (
                <button
                  onClick={tryNextSource}
                  className="btn-primary"
                >
                  Try Another Source
                </button>
              )}
            </div>
          </div>
        )}
      </div>

      {sources.length > 1 && (
        <div className="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8 py-4">
          <p className="text-text-muted text-xs mb-2">
            Source: {currentSource?.label} ({currentSource?.quality})
          </p>
          <div className="flex gap-2 flex-wrap">
            {sources.map((s, idx) => (
              <button
                key={idx}
                onClick={() => setCurrentSourceIdx(idx)}
                className={`px-3 py-1 text-xs rounded-lg border transition-all ${
                  idx === currentSourceIdx
                    ? "bg-accent/20 border-accent/50 text-white"
                    : "bg-surface-50 border-white/5 text-text-muted hover:bg-surface-100"
                }`}
              >
                {s.label} - {s.quality}
              </button>
            ))}
          </div>
        </div>
      )}

      {error && media && (
        <div className="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8 py-4">
          <div className="bg-red-500/10 border border-red-500/20 rounded-xl p-4">
            <p className="text-red-400 text-sm">{error}</p>
            <div className="flex gap-2 mt-3">
              {currentSourceIdx < sources.length - 1 && (
                <button
                  onClick={tryNextSource}
                  className="text-sm text-accent hover:text-accent-light"
                >
                  Try Another Source
                </button>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default function WatchPage() {
  return (
    <Suspense
      fallback={
        <div className="min-h-screen bg-black flex items-center justify-center">
          <div className="w-12 h-12 border-4 border-white/20 border-t-accent rounded-full animate-spin" />
        </div>
      }
    >
      <WatchContent />
    </Suspense>
  );
}
