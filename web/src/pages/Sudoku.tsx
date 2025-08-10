import { createSignal } from 'solid-js'
import SudokuUploader from '../components/SudokuUploader'

/**
Stub for a WASM-backed image processing function (OCR + solver pre-step)
Replace with your wasm-bindgen/wasm-pack exported function.
Supports cancellation via AbortSignal and ignores stale results via a jobId.
*/
async function processImageInWasm(
  imageBlob: Blob,
  opts: { signal?: AbortSignal; jobId: number }
): Promise<string> {
  const { signal, jobId } = opts
  // Simulate async work with cancel support
  await new Promise<void>((resolve, reject) => {
    const t = setTimeout(() => resolve(), 900)
    const onAbort = () => {
      clearTimeout(t)
      reject(new DOMException('Aborted', 'AbortError'))
    }
    if (signal) {
      if (signal.aborted) onAbort()
      signal.addEventListener('abort', onAbort, { once: true })
    }
  })
  return `WASM stub ✓ (job #${jobId}). Bytes: ${imageBlob.size}`
}

export default function Sudoku() {
  const [result, setResult] = createSignal<string>('')
  const [error, setError] = createSignal<string>('')
  const [isProcessing, setIsProcessing] = createSignal(false)
  let currentJob = 0
  let currentAbort: AbortController | null = null

  async function handleImage(image: Blob) {
    setError('')
    setResult('')
    currentAbort?.abort()
    const jobId = ++currentJob
    const ac = new AbortController()
    currentAbort = ac
    setIsProcessing(true)
    try {
      const res = await processImageInWasm(image, { signal: ac.signal, jobId })
      if (jobId !== currentJob) return
      setResult(res)
    } catch (e: any) {
      if (e?.name === 'AbortError') return
      setError(e?.message || 'Image processing error')
    } finally {
      if (jobId === currentJob) setIsProcessing(false)
    }
  }

  return (
    <div class="md:col-start-2 xl:col-start-3 col-span-5 mx-auto p-4">
      <SudokuUploader onImage={handleImage} />
      {isProcessing() && (
        <div class="mt-6 p-4 rounded-xl bg-blue-50 text-blue-900">
          Processing image in WASM…
        </div>
      )}
      {error() && (
        <div class="mt-6 p-4 rounded-xl bg-red-50 text-red-900">
          <strong>Error:</strong> {error()}
        </div>
      )}
      {result() && (
        <div class="mt-6 p-4 rounded-xl bg-emerald-50 text-emerald-900">
          <h2 class="text-xl font-medium mb-2">Result</h2>
          <pre class="whitespace-pre-wrap break-words">{result()}</pre>
        </div>
      )}
    </div>
  )
}
