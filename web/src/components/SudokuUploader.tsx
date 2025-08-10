import { createSignal, onMount, onCleanup, createEffect } from 'solid-js'

export interface SudokuUploaderProps {
  onImage?: (image: Blob) => void
}

/** Max upload size (hard stop). Adjust as needed. */
const MAX_BYTES = 10 * 1024 * 1024 // 10 MiB

/**
Optional: downscale huge images before sending to WASM.
Uses createImageBitmap + OffscreenCanvas when available, falls back to <canvas>.
Keeps aspect ratio, constrains largest side to maxSide.
*/
async function downscaleImage(
  file: File,
  maxSide = 1536,
  mime = 'image/webp',
  quality = 0.92
): Promise<Blob> {
  const url = URL.createObjectURL(file)
  try {
    const bitmap = await createImageBitmap(
      await fetch(url).then((r) => r.blob())
    )
    const { width, height } = bitmap
    const scale = Math.min(1, maxSide / Math.max(width, height))
    if (scale === 1) {
      // No downscale needed: return original file
      return file
    }
    const targetW = Math.max(1, Math.round(width * scale))
    const targetH = Math.max(1, Math.round(height * scale))

    // Prefer OffscreenCanvas if present
    if (typeof OffscreenCanvas !== 'undefined') {
      const c = new OffscreenCanvas(targetW, targetH)
      const ctx = c.getContext('2d')!
      ctx.imageSmoothingQuality = 'high'
      ctx.drawImage(bitmap, 0, 0, targetW, targetH)
      const blob = await c.convertToBlob({ type: mime, quality })
      bitmap.close()
      return blob
    }

    // Fallback to DOM canvas
    const canvas = document.createElement('canvas')
    canvas.width = targetW
    canvas.height = targetH
    const ctx = canvas.getContext('2d')!
    ctx.imageSmoothingQuality = 'high'
    ctx.drawImage(bitmap, 0, 0, targetW, targetH)
    bitmap.close()
    return await new Promise<Blob>((resolve) => {
      canvas.toBlob((b) => resolve(b || file), mime, quality)
    })
  } finally {
    URL.revokeObjectURL(url)
  }
}

export default function SudokuUploader({ onImage }: SudokuUploaderProps) {
  // UI state
  const [imagePreviewUrl, setImagePreviewUrl] = createSignal<string | null>(
    null
  )
  const [isDragging, setIsDragging] = createSignal(false)
  // Error state is handled in parent, not here

  // Refs
  let fileInputRef: HTMLInputElement | undefined
  let dropZoneRef: HTMLDivElement | undefined

  // Track drag depth to avoid flicker on child enter/leave
  let dragDepth = 0

  // Manage object URL lifecycle to avoid leaks
  createEffect<string | null>((prev = null) => {
    const current = imagePreviewUrl()
    if (prev && prev !== current) URL.revokeObjectURL(prev)
    return current
  })
  onCleanup(() => {
    const current = imagePreviewUrl()
    if (current) URL.revokeObjectURL(current)
  })

  async function handleFiles(files: FileList | null | undefined) {
    const file = files?.[0]
    if (!file) return

    // Error state is handled in parent, not here

    if (!file.type.startsWith('image/')) {
      // Error state is handled in parent, not here
      return
    }
    if (file.size > MAX_BYTES) {
      // Error state is handled in parent, not here
      return
    }

    // Show preview early for UX
    const objectUrl = URL.createObjectURL(file)
    setImagePreviewUrl(objectUrl)

    // Prepare data for WASM: optionally downscale
    let toProcess: Blob = file
    try {
      toProcess = await downscaleImage(file)
    } catch (e) {
      console.warn('Downscale failed, using original file', e)
    }

    // Pass the image blob to the parent
    if (onImage) onImage(toProcess)
  }

  function onFileInputChange(e: Event) {
    handleFiles((e.currentTarget as HTMLInputElement).files)
  }

  function onDrop(e: DragEvent) {
    e.preventDefault()
    dragDepth = 0
    setIsDragging(false)
    handleFiles(e.dataTransfer?.files)
  }

  function onDragOver(e: DragEvent) {
    e.preventDefault()
  }

  function onDragEnter(e: DragEvent) {
    e.preventDefault()
    if (dragDepth++ === 0) setIsDragging(true)
  }

  function onDragLeave(e: DragEvent) {
    e.preventDefault()
    if (--dragDepth <= 0) {
      dragDepth = 0
      setIsDragging(false)
    }
  }

  function onKeyActivate(e: KeyboardEvent) {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault()
      fileInputRef?.click()
    }
  }

  // Clipboard paste (global). Only accept first image item.
  function onPaste(e: ClipboardEvent) {
    const items = e.clipboardData?.items
    if (!items) return
    for (const it of items) {
      if (it.kind === 'file' && it.type.startsWith('image/')) {
        const f = it.getAsFile()
        if (f)
          handleFiles({ 0: f, length: 1, item: () => f } as unknown as FileList)
        break
      }
    }
  }

  onMount(() => {
    document.addEventListener('paste', onPaste)
  })
  onCleanup(() => {
    document.removeEventListener('paste', onPaste)
  })

  return (
    <>
      <div
        ref={dropZoneRef}
        class={`border-2 border-dashed rounded-xl p-2 text-center transition-colors duration-200 outline-none select-none max-w-xl ${
          isDragging() ? 'border-blue-600 bg-blue-50' : 'border-gray-300 '
        }`}
        tabindex={0}
        role="button"
        aria-label="Drag an image, paste from clipboard, or select a file"
        onKeyDown={onKeyActivate}
        onDrop={onDrop}
        onDragOver={onDragOver}
        onDragEnter={onDragEnter}
        onDragLeave={onDragLeave}
      >
        <p
          class={`text-lg ${isDragging() ? 'text-zinc-800' : 'text-zinc-100'}`}
        >
          Drag an image here, paste (Ctrl/⌘+V), or click to select a file
        </p>
        <input
          ref={fileInputRef}
          type="file"
          accept="image/png,image/jpeg,image/webp"
          class="hidden"
          onChange={onFileInputChange}
        />
        <div class="mt-4">
          <button
            type="button"
            class="px-4 py-2 rounded-lg bg-blue-600 text-white hover:bg-blue-700 disabled:opacity-50"
            onClick={() => fileInputRef?.click()}
          >
            Select File
          </button>
        </div>
      </div>

      {imagePreviewUrl() && (
        <div class="mt-6">
          <img
            src={imagePreviewUrl()!}
            alt="Uploaded Sudoku image"
            class="max-w-full rounded-xl shadow-sm"
          />
        </div>
      )}
    </>
  )
}
