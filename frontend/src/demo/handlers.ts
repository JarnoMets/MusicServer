import { demoGet, demoSet, newId } from './db'
import { parseBody, type DemoHandleResult, type DemoRequestHandler } from './adapter'

export interface MusicDemoState {
  tracks: Record<string, unknown>[]
  playlists: Record<string, unknown>[]
  playlistDetails: Record<string, Record<string, unknown>>
  artists: unknown
  genres: unknown
  stats: unknown
  me: unknown
}

const STATE_KEY = 'state'

type AnyRec = Record<string, unknown>

async function state(): Promise<MusicDemoState> {
  const s = await demoGet<MusicDemoState>(STATE_KEY)
  if (!s) throw new Error('Music demo state missing')
  return s
}

async function save(s: MusicDemoState): Promise<void> {
  await demoSet(STATE_KEY, s)
}

function ok(data: unknown, status = 200): DemoHandleResult {
  return { status, data }
}

function nowIso(): string {
  return new Date().toISOString()
}

export async function saveMusicDemoState(s: MusicDemoState): Promise<void> {
  await demoSet(STATE_KEY, s)
}

export const musicDemoHandler: DemoRequestHandler = async ({ method, path, data, params }) => {
  const body = parseBody(data) as AnyRec

  if (path === '/me' && method === 'get') {
    return ok((await state()).me)
  }
  if (path === '/health' || path === '/health/db') {
    return ok({ status: 'ok', demo_local: true })
  }

  if (path === '/music/all-cached' && method === 'get') {
    return ok((await state()).tracks)
  }
  if (path === '/music' && method === 'get') {
    let tracks = [...(await state()).tracks]
    const search = String(params?.search || '').toLowerCase()
    if (search) {
      tracks = tracks.filter((t) =>
        JSON.stringify(t).toLowerCase().includes(search)
      )
    }
    const limit = Number(params?.limit ?? tracks.length)
    const offset = Number(params?.offset ?? 0)
    return ok(tracks.slice(offset, offset + limit))
  }
  if (path === '/music/stats' && method === 'get') {
    return ok((await state()).stats ?? { total: (await state()).tracks.length })
  }
  if (path === '/music' && method === 'post') {
    const s = await state()
    const track = { id: newId(), ...body, created_at: nowIso(), updated_at: nowIso() }
    s.tracks.push(track)
    await save(s)
    return ok(track, 201)
  }

  const musicMatch = path.match(/^\/music\/([^/]+)(.*)$/)
  if (musicMatch) {
    const id = decodeURIComponent(musicMatch[1])
    const rest = musicMatch[2]
    // streams are allowlisted — never handled here
    if (rest === '/stream') return null

    const s = await state()
    const idx = s.tracks.findIndex((t) => String(t.id) === id)

    if (rest === '' && method === 'get') {
      return idx < 0 ? { status: 404, data: { error: 'Not found' } } : ok(s.tracks[idx])
    }
    if (rest === '' && (method === 'patch' || method === 'put')) {
      if (idx < 0) return { status: 404, data: { error: 'Not found' } }
      s.tracks[idx] = { ...s.tracks[idx], ...body, id, updated_at: nowIso() }
      await save(s)
      return ok(s.tracks[idx])
    }
    if (rest === '' && method === 'delete') {
      s.tracks = s.tracks.filter((t) => String(t.id) !== id)
      await save(s)
      return ok({ ok: true })
    }
    if (rest === '/playlists' && method === 'get') {
      return ok(
        s.playlists.filter((p) => {
          const detail = s.playlistDetails[String(p.id)]
          const items = (detail?.tracks || detail?.items || []) as AnyRec[]
          return items.some((it) => String(it.music_file_id || it.id) === id)
        })
      )
    }
    // BPM detect / cut etc. — soft stub
    if (method !== 'get') return ok({ ok: true, demo_local: true, id })
  }

  if (path === '/playlists' && method === 'get') {
    return ok((await state()).playlists)
  }
  if (path === '/playlists' && method === 'post') {
    const s = await state()
    const id = newId()
    const pl = {
      id,
      name: body.name ?? 'Playlist',
      description: body.description ?? null,
      created_at: nowIso(),
      updated_at: nowIso()
    }
    s.playlists.push(pl)
    s.playlistDetails[id] = { ...pl, tracks: [], items: [] }
    await save(s)
    return ok(pl, 201)
  }

  const plMatch = path.match(/^\/playlists\/([^/]+)(.*)$/)
  if (plMatch) {
    const id = decodeURIComponent(plMatch[1])
    const rest = plMatch[2]
    const s = await state()

    if (rest === '' && method === 'get') {
      return ok(s.playlistDetails[id] ?? { id, tracks: [], items: [] })
    }
    if (rest === '' && (method === 'patch' || method === 'put')) {
      const idx = s.playlists.findIndex((p) => String(p.id) === id)
      if (idx >= 0) s.playlists[idx] = { ...s.playlists[idx], ...body, id }
      s.playlistDetails[id] = { ...(s.playlistDetails[id] || { id }), ...body, id }
      await save(s)
      return ok(s.playlistDetails[id])
    }
    if (rest === '' && method === 'delete') {
      s.playlists = s.playlists.filter((p) => String(p.id) !== id)
      delete s.playlistDetails[id]
      await save(s)
      return ok({ ok: true })
    }
    if (rest === '/tracks' && method === 'post') {
      const detail = s.playlistDetails[id] || { id, tracks: [], items: [] }
      const items = ([...(detail.tracks as AnyRec[] || []), ...(detail.items as AnyRec[] || [])])
      const entry = {
        id: newId(),
        music_file_id: body.music_file_id,
        position: body.position ?? items.length,
        ...body
      }
      const tracks = [...(detail.tracks as AnyRec[] || []), entry]
      s.playlistDetails[id] = { ...detail, tracks, items: tracks }
      await save(s)
      return ok(entry, 201)
    }
    if (rest.match(/^\/tracks\//) && method === 'delete') {
      const trackId = rest.split('/')[2]
      const detail = s.playlistDetails[id] || { id, tracks: [] }
      const tracks = ((detail.tracks as AnyRec[]) || []).filter(
        (t) => String(t.id) !== trackId && String(t.music_file_id) !== trackId
      )
      s.playlistDetails[id] = { ...detail, tracks, items: tracks }
      await save(s)
      return ok({ ok: true })
    }
    if (rest === '/reorder' && method === 'post') {
      return ok({ ok: true })
    }
    if (rest.startsWith('/export')) {
      return ok({ ok: true, demo_local: true })
    }
  }

  if (path.startsWith('/artists')) {
    if (method === 'get') return ok((await state()).artists ?? [])
    return ok({ ok: true, demo_local: true })
  }
  if (path.startsWith('/genres')) {
    if (method === 'get') return ok((await state()).genres ?? [])
    return ok({ ok: true, demo_local: true })
  }
  if (path.startsWith('/streams') && method === 'get') return ok([])
  if (path.startsWith('/tokens')) {
    if (method === 'get') return ok([])
    return ok({ ok: true, demo_local: true })
  }
  if (path.startsWith('/admin') || path.startsWith('/youtube')) {
    if (method === 'get') return ok({})
    return ok({ ok: true, demo_local: true })
  }

  return null
}
