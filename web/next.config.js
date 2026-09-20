/** @type {import('next').NextConfig} */
const API_URL = process.env.NEXT_PUBLIC_API_URL || "http://localhost:8080";

const nextConfig = {
  images: {
    remotePatterns: [
      { protocol: "https", hostname: "**.tmdb.org" },
      { protocol: "https", hostname: "image.tmdb.org" },
      { protocol: "http", hostname: "localhost", port: "8080" },
    ],
    unoptimized: true,
  },
  async rewrites() {
    return [
      { source: "/api/v1/:path*", destination: `${API_URL}/api/v1/:path*` },
    ];
  },
};

module.exports = nextConfig;
