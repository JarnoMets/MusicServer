/**
 * Shared canvas/drawing helpers for waveforms, spectrograms, meters, etc.
 * These are pure utility functions with no Vue dependencies.
 */

// ─── Canvas Setup ────────────────────────────────────────────────

/**
 * Set up a canvas for high-DPI rendering and return the 2D context.
 */
export const setupHiDpiCanvas = (
  canvas: HTMLCanvasElement,
  width: number,
  height: number
): CanvasRenderingContext2D | null => {
  const dpr = window.devicePixelRatio || 1
  canvas.width = width * dpr
  canvas.height = height * dpr
  canvas.style.width = `${width}px`
  canvas.style.height = `${height}px`
  const ctx = canvas.getContext('2d')
  if (ctx) {
    ctx.scale(dpr, dpr)
  }
  return ctx
}

// ─── Waveform Drawing ────────────────────────────────────────────

export interface WaveformStyle {
  color: string
  playedColor: string
  backgroundColor: string
  centerLine: boolean
  centerLineColor: string
  barWidth: number
  barGap: number
  mirror: boolean
}

export const DEFAULT_WAVEFORM_STYLE: WaveformStyle = {
  color: '#4f46e5',
  playedColor: '#22d3ee',
  backgroundColor: 'transparent',
  centerLine: true,
  centerLineColor: 'rgba(255,255,255,0.15)',
  barWidth: 2,
  barGap: 1,
  mirror: true,
}

/**
 * Draw a waveform overview from pre-computed peak data.
 * @param ctx Canvas 2D context
 * @param peaks Array of peak values (0-1) representing the waveform
 * @param width Canvas logical width
 * @param height Canvas logical height
 * @param playProgress 0-1 fraction of how far the playhead is
 * @param style Drawing style options
 */
export const drawWaveform = (
  ctx: CanvasRenderingContext2D,
  peaks: Float32Array | number[],
  width: number,
  height: number,
  playProgress = 0,
  style: Partial<WaveformStyle> = {}
): void => {
  const s = { ...DEFAULT_WAVEFORM_STYLE, ...style }
  const midY = height / 2

  ctx.clearRect(0, 0, width, height)

  if (s.backgroundColor !== 'transparent') {
    ctx.fillStyle = s.backgroundColor
    ctx.fillRect(0, 0, width, height)
  }

  if (s.centerLine) {
    ctx.strokeStyle = s.centerLineColor
    ctx.lineWidth = 1
    ctx.beginPath()
    ctx.moveTo(0, midY)
    ctx.lineTo(width, midY)
    ctx.stroke()
  }

  const totalBars = Math.floor(width / (s.barWidth + s.barGap))
  const samplesPerBar = peaks.length / totalBars

  for (let i = 0; i < totalBars; i++) {
    // Average the samples for this bar
    const start = Math.floor(i * samplesPerBar)
    const end = Math.floor((i + 1) * samplesPerBar)
    let sum = 0
    let count = 0
    for (let j = start; j < end && j < peaks.length; j++) {
      sum += Math.abs(peaks[j])
      count++
    }
    const avg = count > 0 ? sum / count : 0
    const barHeight = avg * midY * 0.95

    const x = i * (s.barWidth + s.barGap)
    const barProgress = x / width
    ctx.fillStyle = barProgress < playProgress ? s.playedColor : s.color

    // Top half
    ctx.fillRect(x, midY - barHeight, s.barWidth, barHeight)

    // Mirror (bottom half)
    if (s.mirror) {
      ctx.globalAlpha = 0.4
      ctx.fillRect(x, midY, s.barWidth, barHeight * 0.6)
      ctx.globalAlpha = 1
    }
  }
}

// ─── Spectrogram Drawing ─────────────────────────────────────────

/**
 * Draw a spectrogram column from frequency data.
 * Called repeatedly as audio plays to build up the spectrogram display.
 * @param ctx Canvas 2D context
 * @param frequencyData Uint8Array from AnalyserNode.getByteFrequencyData
 * @param x The x-position to draw this column
 * @param height Canvas height
 * @param columnWidth Width of each column (usually 1-2px)
 * @param zoom Scaling factor for frequency range (higher = more zoomed into lows)
 */
export const drawSpectrogramColumn = (
  ctx: CanvasRenderingContext2D,
  frequencyData: Uint8Array,
  x: number,
  height: number,
  columnWidth = 1,
  zoom = 1.0
): void => {
  const totalBins = frequencyData.length
  const visibleBins = Math.max(16, Math.floor(totalBins / zoom))
  const binHeight = height / visibleBins

  for (let i = 0; i < visibleBins; i++) {
    const value = frequencyData[i] / 255
    // Map frequency: low freqs at bottom, high at top
    const y = height - (i + 1) * binHeight

    const h = (1 - i / totalBins) * 270
    const l = value * 55
    ctx.fillStyle = `hsl(${h}, ${80 + value * 20}%, ${5 + l}%)`
    ctx.fillRect(x, y, columnWidth, binHeight + 0.5)
  }
}

/**
 * Draw a scrolling spectrogram: shift existing content left and draw new column on the right.
 */
export const drawScrollingSpectrogram = (
  ctx: CanvasRenderingContext2D,
  frequencyData: Uint8Array,
  width: number,
  height: number,
  columnWidth = 2,
  isBeat = false,
  isBar = false,
  zoom = 1.0
): void => {
  // Shift existing image left
  const imageData = ctx.getImageData(columnWidth, 0, width - columnWidth, height)
  ctx.putImageData(imageData, 0, 0)

  // Clear the rightmost column
  ctx.clearRect(width - columnWidth, 0, columnWidth, height)

  // Draw new column
  drawSpectrogramColumn(ctx, frequencyData, width - columnWidth, height, columnWidth, zoom)

  // Draw beat marker (prominent white bars)
  if (isBeat || isBar) {
    ctx.fillStyle = isBar ? 'rgba(255, 255, 255, 1.0)' : 'rgba(255, 255, 255, 0.4)'
    // We draw as a vertical line
    ctx.fillRect(width - columnWidth, 0, columnWidth, height)
    
    // For bars, add a small pip at the top/bottom for extra visibility
    if (isBar) {
      ctx.fillStyle = '#ffffff'
      ctx.fillRect(width - columnWidth - 2, 0, columnWidth + 4, 4)
      ctx.fillRect(width - columnWidth - 2, height - 4, columnWidth + 4, 4)
    }
  }
}

// ─── Meter Drawing ───────────────────────────────────────────────

export interface MeterStyle {
  backgroundColor: string
  lowColor: string
  midColor: string
  highColor: string
  clipColor: string
  width: number
  height: number
  vertical: boolean
  segmented: boolean
  segments: number
  segmentGap: number
}

export const DEFAULT_METER_STYLE: MeterStyle = {
  backgroundColor: '#1a1a2e',
  lowColor: '#22c55e',
  midColor: '#eab308',
  highColor: '#f97316',
  clipColor: '#ef4444',
  width: 12,
  height: 200,
  vertical: true,
  segmented: true,
  segments: 30,
  segmentGap: 1,
}

/**
 * Draw a VU/level meter.
 * @param ctx Canvas 2D context  
 * @param level 0-1 linear level
 * @param peak 0-1 peak hold level (optional)
 * @param style Meter style options
 */
export const drawMeter = (
  ctx: CanvasRenderingContext2D,
  level: number,
  peak = -1,
  style: Partial<MeterStyle> = {}
): void => {
  const s = { ...DEFAULT_METER_STYLE, ...style }
  const w = s.width
  const h = s.height

  ctx.clearRect(0, 0, w, h)
  ctx.fillStyle = s.backgroundColor
  ctx.fillRect(0, 0, w, h)

  if (s.segmented) {
    const segH = (h - (s.segments - 1) * s.segmentGap) / s.segments
    const activeSegments = Math.round(level * s.segments)

    for (let i = 0; i < s.segments; i++) {
      const segIndex = s.segments - 1 - i // bottom to top
      const y = i * (segH + s.segmentGap)
      const ratio = segIndex / s.segments

      if (segIndex < activeSegments) {
        if (ratio > 0.9) ctx.fillStyle = s.clipColor
        else if (ratio > 0.75) ctx.fillStyle = s.highColor
        else if (ratio > 0.5) ctx.fillStyle = s.midColor
        else ctx.fillStyle = s.lowColor
      } else {
        ctx.fillStyle = 'rgba(255,255,255,0.05)'
      }

      ctx.fillRect(0, y, w, segH)
    }

    // Peak indicator
    if (peak >= 0) {
      const peakSeg = Math.round(peak * s.segments)
      if (peakSeg > 0) {
        const peakIndex = s.segments - peakSeg
        const peakY = peakIndex * (segH + s.segmentGap)
        ctx.fillStyle = peak > 0.9 ? s.clipColor : '#ffffff'
        ctx.fillRect(0, peakY, w, segH)
      }
    }
  }
}

// ─── Beat Grid Drawing ──────────────────────────────────────────

/**
 * Draw beat grid markers on a waveform.
 * @param ctx Canvas 2D context
 * @param bpm BPM of the track
 * @param duration Total track duration in seconds
 * @param width Canvas width
 * @param height Canvas height
 * @param scrollOffset Current scroll offset in seconds
 * @param visibleDuration How many seconds of audio are visible
 * @param beatOffset First beat offset in seconds
 * @param beatMap Optional array of individual beat timestamps in seconds
 */
export const drawBeatGrid = (
  ctx: CanvasRenderingContext2D,
  bpm: number,
  duration: number,
  width: number,
  height: number,
  scrollOffset: number,
  visibleDuration: number,
  beatOffset = 0,
  beatMap: number[] | null = null
): void => {
  if (duration <= 0) return

  if (beatMap && beatMap.length > 0) {
    // ─── NON-LINEAR BEAT MAP MODE ────────────────────────────
    // Find the first beat in visual range
    let startIdx = 0
    // Simple linear scan for startIdx since N is small (<1000)
    while (startIdx < beatMap.length && beatMap[startIdx] < scrollOffset) {
      startIdx++
    }

    // Draw all visible beats
    for (let i = startIdx; i < beatMap.length; i++) {
      const beatTime = beatMap[i]
      if (beatTime > scrollOffset + visibleDuration) break

      const x = ((beatTime - scrollOffset) / visibleDuration) * width
      const isBar = i % 4 === 0
      const isPhrase = i % 16 === 0

      if (isPhrase) {
        ctx.strokeStyle = 'rgba(255,255,255,0.90)'
        ctx.lineWidth = 2.5
      } else if (isBar) {
        ctx.strokeStyle = 'rgba(255,255,255,0.65)'
        ctx.lineWidth = 1.5
      } else {
        ctx.strokeStyle = 'rgba(255,255,255,0.22)'
        ctx.lineWidth = 1
      }

      ctx.beginPath()
      if (!isBar) {
        ctx.moveTo(x, height * 0.2); ctx.lineTo(x, height * 0.8)
      } else {
        ctx.moveTo(x, 0); ctx.lineTo(x, height)
      }
      ctx.stroke()

      if (isBar) {
        const barNum = Math.floor(i / 4) + 1
        ctx.fillStyle = isPhrase ? 'rgba(255,255,255,0.90)' : 'rgba(255,255,255,0.55)'
        ctx.font = isPhrase ? 'bold 10px Inter, monospace' : '9px Inter, monospace'
        ctx.fillText(`${barNum}`, x + 3, 11)
      }
    }
  } else if (bpm > 0) {
    // ─── LINEAR CALCULATION MODE ────────────────────────────────
    const beatLen = 60 / bpm
    const startBeat = Math.floor((scrollOffset - beatOffset) / beatLen)
    const endBeat = Math.ceil((scrollOffset + visibleDuration - beatOffset) / beatLen)

    for (let b = startBeat; b <= endBeat; b++) {
      const beatTime = beatOffset + b * beatLen
      const x = ((beatTime - scrollOffset) / visibleDuration) * width

      if (x < 0 || x > width) continue

      const isBar = b % 4 === 0
      const isPhrase = b % 16 === 0

      if (isPhrase) {
        ctx.strokeStyle = 'rgba(255,255,255,0.90)'
        ctx.lineWidth = 2.5
      } else if (isBar) {
        ctx.strokeStyle = 'rgba(255,255,255,0.65)'
        ctx.lineWidth = 1.5
      } else {
        ctx.strokeStyle = 'rgba(255,255,255,0.22)'
        ctx.lineWidth = 1
      }

      ctx.beginPath()
      if (!isBar) {
        ctx.moveTo(x, height * 0.2); ctx.lineTo(x, height * 0.8)
      } else {
        ctx.moveTo(x, 0); ctx.lineTo(x, height)
      }
      ctx.stroke()

      if (isBar) {
        const barNum = Math.floor(b / 4) + 1
        ctx.fillStyle = isPhrase ? 'rgba(255,255,255,0.90)' : 'rgba(255,255,255,0.55)'
        ctx.font = isPhrase ? 'bold 10px Inter, monospace' : '9px Inter, monospace'
        ctx.fillText(`${barNum}`, x + 3, 11)
      }
    }
  }
}

// ─── Scrolling Waveform Drawing ────────────────────────────────

/**
 * Draw a scrolling high-resolution waveform window.
 */
export const drawScrollingWaveform = (
  ctx: CanvasRenderingContext2D,
  peaks: Float32Array | number[],
  currentTime: number,
  duration: number,
  width: number,
  height: number,
  visibleDuration = 10, // seconds visible
  color = '#4f46e5',
  beatGrid: { bpm: number; offset: number } | null = null
): void => {
  ctx.clearRect(0, 0, width, height)
  if (duration <= 0 || peaks.length === 0) return

  const midY = height / 2
  const samplesPerSecond = peaks.length / duration
  const startVisible = currentTime - visibleDuration / 2
  const endVisible = currentTime + visibleDuration / 2

  // Draw Beat Grid Background
  if (beatGrid && beatGrid.bpm > 0) {
    drawBeatGrid(
      ctx,
      beatGrid.bpm,
      duration,
      width,
      height,
      startVisible,
      visibleDuration,
      beatGrid.offset
    )
  }

  // Draw Waveform Window
  const startSample = Math.max(0, Math.floor(startVisible * samplesPerSecond))
  const endSample = Math.min(peaks.length, Math.ceil(endVisible * samplesPerSecond))
  
  ctx.beginPath()
  ctx.strokeStyle = color
  ctx.lineWidth = 1.5

  for (let s = startSample; s < endSample; s++) {
    const time = s / samplesPerSecond
    const x = ((time - startVisible) / visibleDuration) * width
    const amp = peaks[s]
    const barH = amp * midY * 0.9

    ctx.moveTo(x, midY - barH)
    ctx.lineTo(x, midY + barH)
  }
  ctx.stroke()

  // Center Line
  ctx.strokeStyle = 'rgba(255,255,255,0.1)'
  ctx.beginPath()
  ctx.moveTo(0, midY)
  ctx.lineTo(width, midY)
  ctx.stroke()
}

// ─── Playhead Drawing ────────────────────────────────────────────

/**
 * Draw a playhead line on a waveform/spectrogram canvas.
 */
export const drawPlayhead = (
  ctx: CanvasRenderingContext2D,
  x: number,
  height: number,
  color = '#ffffff',
  lineWidth = 2
): void => {
  ctx.strokeStyle = color
  ctx.lineWidth = lineWidth
  ctx.shadowColor = color
  ctx.shadowBlur = 6
  ctx.beginPath()
  ctx.moveTo(x, 0)
  ctx.lineTo(x, height)
  ctx.stroke()
  ctx.shadowBlur = 0

  // Triangle marker at top
  ctx.fillStyle = color
  ctx.beginPath()
  ctx.moveTo(x - 5, 0)
  ctx.lineTo(x + 5, 0)
  ctx.lineTo(x, 6)
  ctx.closePath()
  ctx.fill()
}

// ─── Jog Wheel Drawing ──────────────────────────────────────────

/**
 * Draw a DJ jog wheel.
 * @param ctx Canvas 2D context
 * @param cx Center X
 * @param cy Center Y
 * @param radius Radius
 * @param rotation Current rotation angle in radians
 * @param isPlaying Whether the deck is playing
 * @param primaryColor Theme color
 */
export const drawJogWheel = (
  ctx: CanvasRenderingContext2D,
  cx: number,
  cy: number,
  radius: number,
  rotation: number,
  isPlaying: boolean,
  primaryColor = '#4f46e5'
): void => {
  // Outer ring
  ctx.beginPath()
  ctx.arc(cx, cy, radius, 0, Math.PI * 2)
  ctx.fillStyle = '#1a1a2e'
  ctx.fill()
  ctx.strokeStyle = isPlaying ? primaryColor : '#333'
  ctx.lineWidth = 2
  ctx.stroke()

  // Inner platter
  ctx.beginPath()
  ctx.arc(cx, cy, radius * 0.75, 0, Math.PI * 2)
  ctx.fillStyle = '#0f0f1e'
  ctx.fill()
  ctx.strokeStyle = '#2a2a3e'
  ctx.lineWidth = 1
  ctx.stroke()

  // Grooves (rotating)
  ctx.save()
  ctx.translate(cx, cy)
  ctx.rotate(rotation)
  for (let i = 0; i < 8; i++) {
    const angle = (i / 8) * Math.PI * 2
    ctx.beginPath()
    ctx.arc(0, 0, radius * 0.5, angle, angle + 0.15)
    ctx.strokeStyle = 'rgba(255,255,255,0.08)'
    ctx.lineWidth = 1
    ctx.stroke()
  }

  // Position marker (dot)
  const markerAngle = 0 // Always at top in rotated context
  const markerX = Math.cos(markerAngle) * radius * 0.55
  const markerY = Math.sin(markerAngle) * radius * 0.55
  ctx.beginPath()
  ctx.arc(markerX, markerY, 4, 0, Math.PI * 2)
  ctx.fillStyle = isPlaying ? primaryColor : '#666'
  ctx.fill()

  ctx.restore()

  // Center label area
  ctx.beginPath()
  ctx.arc(cx, cy, radius * 0.25, 0, Math.PI * 2)
  ctx.fillStyle = '#0a0a1a'
  ctx.fill()
  ctx.strokeStyle = '#2a2a3e'
  ctx.lineWidth = 1
  ctx.stroke()
}
