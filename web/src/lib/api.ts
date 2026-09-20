import {
  Media,
  Season,
  Episode,
  SearchResult,
  StreamSource,
  IptvChannel,
  IptvCategory,
} from "@/types";

const API_BASE = process.env.NEXT_PUBLIC_API_URL
  ? `${process.env.NEXT_PUBLIC_API_URL}/api/v1`
  : "/api/v1";

async function fetchAPI<T>(
  endpoint: string,
  options?: { signal?: AbortSignal; timeout?: number }
): Promise<T> {
  const controller = new AbortController();
  const timeoutId = setTimeout(
    () => controller.abort(),
    options?.timeout ?? 15000
  );

  try {
    const res = await fetch(`${API_BASE}${endpoint}`, {
      signal: options?.signal ?? controller.signal,
      headers: { Accept: "application/json" },
    });
    if (!res.ok) {
      throw new Error(`API error: ${res.status} ${res.statusText}`);
    }
    return res.json();
  } finally {
    clearTimeout(timeoutId);
  }
}

interface ApiResponse<T> {
  success: boolean;
  data: T | null;
  error?: { code: string; message: string };
}

interface RawCatalogItem {
  id: { provider: string; value: string };
  title: string;
  media_type: string;
  year?: string;
  poster_url?: string;
  season_count?: number;
}

interface RawMediaDetails {
  id: { provider: string; value: string };
  title: string;
  media_type: string;
  year?: string;
  description?: string;
  tagline?: string;
  imdb_rating?: string;
  director?: string;
  stars?: string;
  prints?: string;
  audios?: string;
  poster_url?: string;
  duration?: string;
  genres: string[];
  seasons: {
    number: number;
    episodes: {
      season: number;
      number: number;
      title?: string;
      overview?: string;
    }[];
  }[];
  dubs: { subject_id: string; language: string; label: string }[];
}

interface RawRelease {
  provider: string;
  filename: string;
  quality?: string;
  codec?: string;
  language?: string;
  season?: number;
  episode?: number;
  mirrors: {
    label: string;
    resolver_url: string;
    headers: [string, string][];
    direct_file: boolean;
  }[];
}

interface RawSearchResult {
  id: string;
  title: string;
  stype: number;
  release_year: string;
  cover_url?: string;
  season: number;
  episode: number;
  provider: string;
}

let _idCounter = 1000;
function nextId(): number {
  return _idCounter++;
}

function catalogItemToMedia(item: RawCatalogItem): Media {
  return {
    id: nextId(),
    type: item.media_type === "series" ? "series" : "movie",
    title: item.title || "Untitled",
    overview: "",
    posterPath: item.poster_url || "",
    backdropPath: "",
    releaseDate: item.year || "",
    rating: 0,
    genres: [],
    year: item.year || "",
    seasonCount: item.season_count,
  };
}

function searchResultToMedia(item: RawSearchResult): Media {
  return {
    id: nextId(),
    type: item.stype === 2 ? "series" : "movie",
    title: item.title || "Untitled",
    overview: "",
    posterPath: item.cover_url || "",
    backdropPath: "",
    releaseDate: item.release_year || "",
    rating: 0,
    genres: [],
    year: item.release_year || "",
  };
}

function detailsToMedia(details: RawMediaDetails): Media & { seasons_list?: Season[] } {
  const seasons_list: Season[] | undefined =
    details.seasons.length > 0
      ? details.seasons.map((s) => ({
          id: nextId(),
          number: s.number,
          name: `Season ${s.number}`,
          overview: "",
          posterPath: details.poster_url || "",
          episodeCount: s.episodes.length,
          airDate: "",
          episodes: s.episodes.map((ep) => ({
            id: nextId(),
            number: ep.number,
            title: ep.title || `Episode ${ep.number}`,
            overview: ep.overview || "",
            stillPath: details.poster_url || "",
            duration: 0,
            rating: 0,
            airDate: "",
          })),
        }))
      : undefined;

  return {
    id: nextId(),
    type: details.media_type === "series" ? "series" : "movie",
    title: details.title || "Untitled",
    overview: details.description || details.tagline || "",
    posterPath: details.poster_url || "",
    backdropPath: details.poster_url || "",
    releaseDate: details.year || "",
    rating: details.imdb_rating
      ? parseFloat(details.imdb_rating) || 0
      : 0,
    genres: details.genres || [],
    duration: details.duration
      ? parseInt(details.duration) || undefined
      : undefined,
    year: details.year || "",
    quality: details.prints || undefined,
    seasonCount: seasons_list?.length,
    episodeCount: seasons_list?.reduce((acc, s) => acc + s.episodeCount, 0),
    seasons_list,
  };
}

function releasesToStreams(releases: RawRelease[]): StreamSource[] {
  const sources: StreamSource[] = [];
  for (const release of releases) {
    for (const mirror of release.mirrors) {
      sources.push({
        url: mirror.resolver_url,
        quality: release.quality || "Unknown",
        label: mirror.label || release.provider,
        headers: Object.fromEntries(mirror.headers || []),
      });
    }
  }
  return sources;
}

export async function getHome(signal?: AbortSignal): Promise<{
  featured?: Media;
  trending: Media[];
  movies: Media[];
  series: Media[];
}> {
  try {
    const res = await fetchAPI<ApiResponse<RawCatalogItem[]>>("/home", {
      signal,
    });
    if (!res.success || !res.data) {
      return { trending: [], movies: [], series: [] };
    }
    const items = res.data.map(catalogItemToMedia);
    return {
      featured: items[0],
      trending: items.slice(0, 12),
      movies: items.filter((m) => m.type === "movie").slice(0, 12),
      series: items.filter((m) => m.type === "series").slice(0, 12),
    };
  } catch {
    return { trending: [], movies: [], series: [] };
  }
}

export async function searchMedia(
  query: string,
  options?: { signal?: AbortSignal; provider?: string; page?: number }
): Promise<Media[]> {
  try {
    const params = new URLSearchParams({ q: query });
    if (options?.provider) params.set("provider", options.provider);
    if (options?.page) params.set("page", String(options.page));
    const res = await fetchAPI<ApiResponse<RawSearchResult[]>>(
      `/search?${params.toString()}`,
      { signal: options?.signal }
    );
    if (!res.success || !res.data) return [];
    return res.data.map(searchResultToMedia);
  } catch {
    return [];
  }
}

export async function getMovieDetails(
  id: string,
  provider: string = "moviebox",
  signal?: AbortSignal
): Promise<(Media & { seasons_list?: Season[] }) | null> {
  try {
    const res = await fetchAPI<ApiResponse<RawMediaDetails>>(
      `/details/${provider}/${id}`,
      { signal }
    );
    if (!res.success || !res.data) return null;
    return detailsToMedia(res.data);
  } catch {
    return null;
  }
}

export async function getSeriesDetails(
  id: string,
  provider: string = "moviebox",
  signal?: AbortSignal
): Promise<(Media & { seasons_list?: Season[] }) | null> {
  try {
    const res = await fetchAPI<ApiResponse<RawMediaDetails>>(
      `/details/${provider}/${id}`,
      { signal }
    );
    if (!res.success || !res.data) return null;
    return detailsToMedia(res.data);
  } catch {
    return null;
  }
}

export async function getStreams(
  provider: string,
  id: string,
  season?: number,
  episode?: number,
  signal?: AbortSignal
): Promise<StreamSource[]> {
  try {
    const params = new URLSearchParams();
    if (season) params.set("season", String(season));
    if (episode) params.set("episode", String(episode));
    const qs = params.toString();
    const res = await fetchAPI<ApiResponse<RawRelease[]>>(
      `/streams/${provider}/${id}${qs ? `?${qs}` : ""}`,
      { signal }
    );
    if (!res.success || !res.data) return [];
    return releasesToStreams(res.data);
  } catch {
    return [];
  }
}

export async function getProviders(signal?: AbortSignal) {
  try {
    const res = await fetchAPI<ApiResponse<string[]>>("/providers", {
      signal,
    });
    return res.data || [];
  } catch {
    return [];
  }
}

function authHeaders(): Record<string, string> {
  if (typeof window === "undefined") return {};
  const token = localStorage.getItem("pulse_token");
  return token ? { Authorization: `Bearer ${token}` } : {};
}

export async function addFavorite(data: {
  content_id: string;
  provider: string;
  media_type: string;
  title?: string;
  poster_url?: string;
}) {
  try {
    const res = await fetch(`${API_BASE}/favorites`, {
      method: "POST",
      headers: { "Content-Type": "application/json", ...authHeaders() },
      body: JSON.stringify(data),
    });
    return res.json();
  } catch {
    return { success: false };
  }
}

export async function removeFavorite(id: string) {
  try {
    const res = await fetch(`${API_BASE}/favorites/${id}`, {
      method: "DELETE",
      headers: authHeaders(),
    });
    return res.json();
  } catch {
    return { success: false };
  }
}

export async function getFavorites() {
  try {
    const res = await fetch(`${API_BASE}/favorites`, {
      headers: authHeaders(),
    });
    const data = await res.json();
    return data.success ? data.data : [];
  } catch {
    return [];
  }
}

export async function updateWatchProgress(data: {
  content_id: string;
  provider: string;
  media_type: string;
  title?: string;
  poster_url?: string;
  season?: number;
  episode?: number;
  position: number;
  duration: number;
}) {
  try {
    const res = await fetch(`${API_BASE}/history`, {
      method: "POST",
      headers: { "Content-Type": "application/json", ...authHeaders() },
      body: JSON.stringify(data),
    });
    return res.json();
  } catch {
    return { success: false };
  }
}

export async function getContinueWatching() {
  try {
    const res = await fetch(`${API_BASE}/continue-watching`, {
      headers: authHeaders(),
    });
    const data = await res.json();
    return data.success ? data.data : [];
  } catch {
    return [];
  }
}

export async function getIptvChannels(signal?: AbortSignal): Promise<IptvChannel[]> {
  try {
    const res = await fetchAPI<ApiResponse<IptvChannel[]>>("/iptv/channels", {
      signal,
    });
    if (!res.success || !res.data) return [];
    return res.data;
  } catch {
    return [];
  }
}

export async function getIptvCategories(signal?: AbortSignal): Promise<IptvCategory[]> {
  try {
    const res = await fetchAPI<ApiResponse<IptvCategory[]>>("/iptv/categories", {
      signal,
    });
    if (!res.success || !res.data) return [];
    return res.data;
  } catch {
    return [];
  }
}

export async function getIptvChannelStream(
  channelId: string,
  signal?: AbortSignal
): Promise<StreamSource | null> {
  try {
    const res = await fetchAPI<ApiResponse<StreamSource>>(
      `/iptv/channels/${channelId}/stream`,
      { signal }
    );
    if (!res.success || !res.data) return null;
    return res.data;
  } catch {
    return null;
  }
}
