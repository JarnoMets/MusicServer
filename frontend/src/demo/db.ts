const DB_NAME = 'music-demo-local-v1'
const STORE = 'kv'
const META_KEY = '__meta__'

export interface DemoMeta {
  hydrated: boolean
  hydratedAt: string | null
  version: number
}

const DEFAULT_META: DemoMeta = { hydrated: false, hydratedAt: null, version: 1 }

function openDb(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(DB_NAME, 1)
    req.onupgradeneeded = () => {
      if (!req.result.objectStoreNames.contains(STORE)) req.result.createObjectStore(STORE)
    }
    req.onsuccess = () => resolve(req.result)
    req.onerror = () => reject(req.error)
  })
}

export async function demoGet<T>(key: string): Promise<T | undefined> {
  const db = await openDb()
  return new Promise((resolve, reject) => {
    const req = db.transaction(STORE, 'readonly').objectStore(STORE).get(key)
    req.onsuccess = () => resolve(req.result as T | undefined)
    req.onerror = () => reject(req.error)
  })
}

export async function demoSet(key: string, value: unknown): Promise<void> {
  const db = await openDb()
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE, 'readwrite')
    tx.objectStore(STORE).put(value, key)
    tx.oncomplete = () => resolve()
    tx.onerror = () => reject(tx.error)
  })
}

export async function getDemoMeta(): Promise<DemoMeta> {
  return (await demoGet<DemoMeta>(META_KEY)) ?? { ...DEFAULT_META }
}

export async function setDemoMeta(patch: Partial<DemoMeta>): Promise<DemoMeta> {
  const next = { ...(await getDemoMeta()), ...patch }
  await demoSet(META_KEY, next)
  return next
}

export async function isDemoHydrated(): Promise<boolean> {
  return (await getDemoMeta()).hydrated
}

export function newId(): string {
  return crypto.randomUUID()
}
