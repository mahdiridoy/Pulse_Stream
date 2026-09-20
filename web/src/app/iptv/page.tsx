"use client";

import { useState, useEffect, useCallback, useMemo } from "react";
import { IptvChannel, IptvCategory } from "@/types";
import { getIptvChannels, getIptvCategories } from "@/lib/api";
import IptvPlayer from "@/components/IptvPlayer";
import {
  MagnifyingGlassIcon,
  TvIcon,
  SignalIcon,
} from "@heroicons/react/24/outline";

export default function IptvPage() {
  const [channels, setChannels] = useState<IptvChannel[]>([]);
  const [categories, setCategories] = useState<IptvCategory[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedCategory, setSelectedCategory] = useState("all");
  const [playingChannel, setPlayingChannel] = useState<IptvChannel | null>(null);

  useEffect(() => {
    const controller = new AbortController();
    setLoading(true);

    Promise.all([
      getIptvChannels(controller.signal),
      getIptvCategories(controller.signal),
    ]).then(([channelData, categoryData]) => {
      setChannels(channelData);
      setCategories(categoryData);
      setLoading(false);
    });

    return () => controller.abort();
  }, []);

  const filteredChannels = useMemo(() => {
    let result = channels;

    if (selectedCategory !== "all") {
      result = result.filter(
        (ch) =>
          ch.category === selectedCategory || ch.group === selectedCategory
      );
    }

    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      result = result.filter(
        (ch) =>
          ch.name.toLowerCase().includes(q) ||
          ch.category.toLowerCase().includes(q) ||
          ch.country.toLowerCase().includes(q) ||
          ch.language.toLowerCase().includes(q)
      );
    }

    return result;
  }, [channels, searchQuery, selectedCategory]);

  const channelsByCategory = useMemo(() => {
    const map = new Map<string, IptvChannel[]>();
    for (const ch of filteredChannels) {
      const key = ch.category || "Uncategorized";
      if (!map.has(key)) map.set(key, []);
      map.get(key)!.push(ch);
    }
    return map;
  }, [filteredChannels]);

  const handleChannelClick = useCallback((channel: IptvChannel) => {
    setPlayingChannel(channel);
  }, []);

  const liveCount = channels.filter((ch) => ch.isLive).length;

  if (loading) {
    return (
      <div className="min-h-screen pt-24 pb-16">
        <div className="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div className="mb-8">
            <div className="skeleton w-64 h-10 rounded-xl mb-4" />
            <div className="skeleton w-96 h-5 rounded-lg mb-6" />
            <div className="skeleton w-full max-w-xl h-12 rounded-2xl" />
          </div>
          <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-3 lg:gap-4">
            {Array.from({ length: 18 }).map((_, i) => (
              <div
                key={i}
                className="bg-surface-100 rounded-xl overflow-hidden animate-pulse"
              >
                <div className="aspect-video bg-surface-200" />
                <div className="p-3">
                  <div className="skeleton w-3/4 h-4 rounded mb-2" />
                  <div className="skeleton w-1/2 h-3 rounded" />
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen pt-24 pb-16">
      <div className="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
        <div className="mb-8">
          <div className="flex items-center gap-3 mb-2">
            <TvIcon className="w-8 h-8 text-accent" />
            <h1 className="text-3xl font-bold text-white">IPTV Channels</h1>
          </div>
          <p className="text-text-muted mb-6">
            {channels.length > 0
              ? `${channels.length} channels available \u2022 ${liveCount} live now`
              : "Browse live TV channels"}
          </p>

          <div className="flex flex-col sm:flex-row gap-4">
            <div className="relative flex-1 max-w-xl">
              <MagnifyingGlassIcon className="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-text-dim" />
              <input
                type="text"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder="Search channels, categories, countries..."
                className="w-full pl-12 pr-4 py-3 bg-surface-100 border border-white/10 rounded-2xl
                           text-base text-white placeholder:text-text-dim
                           focus:outline-none focus:border-accent/50 focus:ring-2 focus:ring-accent/25
                           transition-all duration-200"
              />
            </div>

            <div className="flex gap-2 overflow-x-auto pb-1 scrollbar-hide">
              <button
                onClick={() => setSelectedCategory("all")}
                className={`px-4 py-2 text-sm font-medium rounded-xl whitespace-nowrap transition-all duration-200 ${
                  selectedCategory === "all"
                    ? "bg-accent text-white shadow-lg shadow-accent/25"
                    : "bg-surface-100 text-text-muted hover:text-white hover:bg-white/10"
                }`}
              >
                All ({channels.length})
              </button>
              {categories.map((cat) => (
                <button
                  key={cat.id}
                  onClick={() => setSelectedCategory(cat.name)}
                  className={`px-4 py-2 text-sm font-medium rounded-xl whitespace-nowrap transition-all duration-200 ${
                    selectedCategory === cat.name
                      ? "bg-accent text-white shadow-lg shadow-accent/25"
                      : "bg-surface-100 text-text-muted hover:text-white hover:bg-white/10"
                  }`}
                >
                  {cat.name} ({cat.count})
                </button>
              ))}
            </div>
          </div>
        </div>

        {filteredChannels.length === 0 ? (
          <div className="text-center py-20">
            <TvIcon className="w-16 h-16 text-surface-300 mx-auto mb-4" />
            <h3 className="text-xl font-semibold text-white mb-2">
              No channels found
            </h3>
            <p className="text-text-muted">
              {searchQuery
                ? "Try adjusting your search"
                : "No IPTV channels available yet"}
            </p>
          </div>
        ) : (
          <div className="space-y-10">
            {selectedCategory === "all" && searchQuery === "" ? (
              Array.from(channelsByCategory.entries()).map(
                ([category, categoryChannels]) => (
                  <div key={category}>
                    <div className="flex items-center gap-2 mb-4">
                      <h2 className="text-xl font-semibold text-white">
                        {category}
                      </h2>
                      <span className="px-2 py-0.5 bg-white/10 text-text-muted text-xs rounded-full">
                        {categoryChannels.length}
                      </span>
                    </div>
                    <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-3 lg:gap-4">
                      {categoryChannels.map((channel) => (
                        <ChannelCard
                          key={channel.id}
                          channel={channel}
                          onClick={handleChannelClick}
                        />
                      ))}
                    </div>
                  </div>
                )
              )
            ) : (
              <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-3 lg:gap-4">
                {filteredChannels.map((channel) => (
                  <ChannelCard
                    key={channel.id}
                    channel={channel}
                    onClick={handleChannelClick}
                  />
                ))}
              </div>
            )}
          </div>
        )}
      </div>

      {playingChannel && (
        <IptvPlayer
          streamUrl={playingChannel.streamUrl}
          title={playingChannel.name}
          onClose={() => setPlayingChannel(null)}
          onError={(err) => console.error("Stream error:", err)}
        />
      )}
    </div>
  );
}

function ChannelCard({
  channel,
  onClick,
}: {
  channel: IptvChannel;
  onClick: (ch: IptvChannel) => void;
}) {
  return (
    <button
      onClick={() => onClick(channel)}
      className="bg-surface-100 hover:bg-surface-200 border border-white/5 hover:border-accent/30
                 rounded-xl overflow-hidden transition-all duration-200 group text-left"
    >
      <div className="aspect-video relative overflow-hidden bg-surface-200">
        {channel.logo ? (
          <img
            src={channel.logo}
            alt={channel.name}
            className="w-full h-full object-contain p-2 group-hover:scale-105 transition-transform duration-300"
            loading="lazy"
          />
        ) : (
          <div className="w-full h-full flex items-center justify-center">
            <TvIcon className="w-10 h-10 text-surface-300" />
          </div>
        )}

        {channel.isLive && (
          <div className="absolute top-2 right-2 flex items-center gap-1 px-1.5 py-0.5 bg-red-500 rounded text-[10px] font-bold text-white">
            <SignalIcon className="w-3 h-3" />
            LIVE
          </div>
        )}

        <div className="absolute inset-0 bg-black/0 group-hover:bg-black/40 transition-colors flex items-center justify-center opacity-0 group-hover:opacity-100">
          <div className="w-12 h-12 rounded-full bg-accent/90 flex items-center justify-center shadow-lg">
            <svg
              className="w-5 h-5 text-white ml-0.5"
              fill="currentColor"
              viewBox="0 0 24 24"
            >
              <path d="M8 5v14l11-7z" />
            </svg>
          </div>
        </div>
      </div>

      <div className="p-3">
        <h3 className="text-sm font-medium text-white truncate group-hover:text-accent transition-colors">
          {channel.name}
        </h3>
        <div className="flex items-center gap-2 mt-1">
          {channel.country && (
            <span className="text-[11px] text-text-dim truncate">
              {channel.country}
            </span>
          )}
          {channel.language && channel.language !== channel.country && (
            <>
              <span className="text-text-dim text-[11px]">\u00b7</span>
              <span className="text-[11px] text-text-dim truncate">
                {channel.language}
              </span>
            </>
          )}
        </div>
      </div>
    </button>
  );
}
