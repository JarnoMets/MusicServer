import type { AxiosInstance, AxiosResponse, InternalAxiosRequestConfig } from 'axios'
import { isDemoHost } from '../composables/useDemoReadonly'
import { isDemoHydrated } from './db'

export type DemoHandleResult = { status: number; data: unknown }
export type DemoRequestHandler = (ctx: {
  method: string
  path: string
  data: unknown
  params: Record<string, unknown> | undefined
}) => Promise<DemoHandleResult | null>

function pathFromConfig(config: InternalAxiosRequestConfig): string {
  const base = String(config.baseURL || '').replace(/\/$/, '')
  let url = String(config.url || '')
  if (url.startsWith('http')) {
    try {
      url = new URL(url).pathname
    } catch {
      /* keep */
    }
  }
  const full = url.startsWith('/') ? url : `${base}/${url}`
  const stripped = full.replace(/^\/music\/api/, '').replace(/^\/api/, '')
  return stripped.startsWith('/') ? stripped : `/${stripped}`
}

export function installDemoAdapter(
  api: AxiosInstance,
  opts: {
    handler: DemoRequestHandler
    isAllowlisted?: (method: string, path: string) => boolean
  }
): void {
  api.interceptors.request.use(async (config) => {
    if (!isDemoHost()) return config
    const method = String(config.method || 'get').toLowerCase()
    const path = pathFromConfig(config)
    if (!(await isDemoHydrated())) return config
    if (opts.isAllowlisted?.(method, path)) return config

    const handled = await opts.handler({
      method,
      path,
      data: config.data,
      params: config.params as Record<string, unknown> | undefined
    })

    const result =
      handled ??
      (method === 'get' || method === 'head' || method === 'options'
        ? { status: 200, data: method === 'head' ? null : [] }
        : { status: 200, data: { ok: true, demo_local: true } })

    config.adapter = async () =>
      ({
        data: result.data,
        status: result.status,
        statusText: 'OK',
        headers: {},
        config,
        request: {}
      }) as AxiosResponse
    return config
  })
}

export function parseBody(data: unknown): unknown {
  if (typeof data === 'string') {
    try {
      return JSON.parse(data)
    } catch {
      return data
    }
  }
  return data
}
