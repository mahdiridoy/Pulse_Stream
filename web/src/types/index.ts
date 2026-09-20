export interface Media {
  id: number;
  type: "movie" | "series";
  title: string;
  originalTitle?: string;
  overview: string;
  posterPath: string;
  backdropPath: string;
  releaseDate: string;
  rating: number;
  quality?: string;
  genres: string[];
  duration?: number;
  seasonCount?: number;
  episodeCount?: number;
  trailerUrl?: string;
  streamUrl?: string;
  year?: string;
}

export interface Episode {
  id: number;
  number: number;
  title: string;
  overview: string;
  stillPath: string;
  duration: number;
  rating: number;
  airDate: string;
  streamUrl?: string;
}

export interface Season {
  id: number;
  number: number;
  name: string;
  overview: string;
  posterPath: string;
  episodeCount: number;
  airDate: string;
  episodes?: Episode[];
}

export interface SearchResult {
  id: number;
  type: "movie" | "series";
  title: string;
  posterPath: string;
  year: string;
  rating: number;
  overview: string;
  provider?: string;
}

export interface UserProfile {
  id: number;
  username: string;
  avatar: string;
  plan: "free" | "premium";
}

export interface WatchProgress {
  mediaId: number;
  type: "movie" | "series";
  season?: number;
  episode?: number;
  progress: number;
  duration: number;
  lastWatched: string;
}

export interface StreamSource {
  url: string;
  quality: string;
  label: string;
  headers?: Record<string, string>;
}

export interface ApiError {
  code: string;
  message: string;
}

export interface IptvChannel {
  id: string;
  name: string;
  logo: string;
  streamUrl: string;
  category: string;
  group: string;
  country: string;
  language: string;
  isLive: boolean;
}

export interface IptvCategory {
  id: string;
  name: string;
  count: number;
}
