import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  transpilePackages: ["d3", "lucide-react"],
  async rewrites() {
    return [
      {
        source: "/api/:path*",
        destination: "http://bot:8080/api/:path*",
      },
    ];
  },
};


export default nextConfig;
