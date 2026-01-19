import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  output: 'standalone',
  transpilePackages: ["d3", "lucide-react"],
  experimental: {
    reactCompiler: true,
  },
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
