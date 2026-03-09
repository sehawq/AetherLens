const configuredApiBase = process.env.NEXT_PUBLIC_API_URL?.trim();
const configuredHubUrl = process.env.NEXT_PUBLIC_HUB_URL?.trim();

function resolveHostPortUrl(port: number): string {
  if (typeof window === "undefined") {
    return `http://localhost:${port}`;
  }
  const protocol = window.location.protocol === "https:" ? "https:" : "http:";
  return `${protocol}//${window.location.hostname}:${port}`;
}

export function getApiBaseUrl(): string {
  if (configuredApiBase) return configuredApiBase;
  return resolveHostPortUrl(5000);
}

export function getHubUrl(): string {
  if (configuredHubUrl) return configuredHubUrl;
  return `${getApiBaseUrl()}/hubs/aether-lens`;
}
