import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  reactStrictMode: true,
  // Allow accessing the dev server from local network IPs
  // experimental: {
  //   // This is needed to prevent "Cross origin request detected" warnings when accessing via IP
  //   allowedDevOrigins: [
  //     "localhost:3000", 
  //     "127.0.0.1:3000", 
  //     "192.168.1.107:3000", // Your current IP
  //     "192.168.0.1:3000",
  //     "192.168.1.1:3000"
  //   ],
  // },
};

export default nextConfig;
