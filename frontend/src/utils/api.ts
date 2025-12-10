/**
 * Utility to get the API base URL based on environment
 * This is used throughout the app to handle both development and production deployments
 */

export const getAPIBaseURL = (): string => {
  // Use explicit environment variable if set (e.g., VITE_API_URL=http://api.example.com)
  if (import.meta.env.VITE_API_URL) {
    return import.meta.env.VITE_API_URL
  }

  // In production (SPA served from nginx with proxy), use relative path
  // The nginx proxy forwards /api requests to the backend container
  if (import.meta.env.MODE === 'production') {
    return '/api'
  }

  // In development, use localhost with explicit port
  return 'http://localhost:8081/api'
}

/**
 * Builds a full API URL for a given endpoint
 * @param endpoint - The API endpoint (e.g., '/music/sync/stream/123')
 * @returns The full URL to the endpoint
 */
export const buildAPIURL = (endpoint: string): string => {
  const base = getAPIBaseURL()
  // Remove leading slash from endpoint if present
  const path = endpoint.startsWith('/') ? endpoint : `/${endpoint}`
  return `${base}${path}`
}
