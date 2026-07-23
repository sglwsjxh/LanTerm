<script setup lang="ts">
import { onMounted, onUnmounted, shallowRef, ref } from 'vue'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import '@xterm/xterm/css/xterm.css'

const connected = ref(false)
const term = shallowRef<Terminal | null>(null)
const fitAddon = shallowRef<FitAddon | null>(null)
let ws: WebSocket | null = null
let resizeTimer: number | null = null
const containerRef = shallowRef<HTMLDivElement | null>(null)
const hiddenInputRef = shallowRef<HTMLTextAreaElement | null>(null)

onMounted(() => {
  if (!containerRef.value) return
  const t = new Terminal({
    cursorBlink: true,
    fontSize: 14,
    fontFamily: 'Menlo, Monaco, "Courier New", monospace',
    theme: { background: '#1e1e2e', foreground: '#cdd6f4', cursor: '#f38ba8' },
  })
  t.open(containerRef.value)
  const fit = new FitAddon()
  t.loadAddon(fit)
  fit.fit()
  term.value = t
  fitAddon.value = fit

  const proto = location.protocol === 'https:' ? 'wss:' : 'ws:'
  ws = new WebSocket(`${proto}//${location.host}/ws`)
  ws.binaryType = 'arraybuffer'
  ws.onopen = () => { connected.value = true; sendResize() }
  ws.onmessage = (e) => {
    if (typeof e.data === 'string') {
      try {
        const ctrl = JSON.parse(e.data)
        if (ctrl.type === 'error') { connected.value = false; return }
      } catch { t.write(e.data) }
    } else {
      t.write(new Uint8Array(e.data))
    }
  }
  ws.onclose = () => { connected.value = false }
  ws.onerror = () => { connected.value = false }

  t.onData((data) => {
    if (ws?.readyState === WebSocket.OPEN) ws.send(new TextEncoder().encode(data))
  })
  t.onResize(({ cols, rows }) => {
    if (ws?.readyState === WebSocket.OPEN)
      ws.send(JSON.stringify({ type: 'resize', cols, rows }))
  })
  window.addEventListener('resize', onWindowResize)

  containerRef.value.addEventListener('click', () => hiddenInputRef.value?.focus())
})

function onWindowResize() {
  if (resizeTimer) clearTimeout(resizeTimer)
  resizeTimer = window.setTimeout(() => { fitAddon.value?.fit() }, 150)
}

function sendResize() {
  const d = fitAddon.value?.proposeDimensions()
  if (d && ws?.readyState === WebSocket.OPEN)
    ws.send(JSON.stringify({ type: 'resize', cols: d.cols, rows: d.rows }))
}

onUnmounted(() => {
  window.removeEventListener('resize', onWindowResize)
  if (resizeTimer) clearTimeout(resizeTimer)
  if (ws) { ws.close(); ws = null }
  if (term.value) { term.value.dispose(); term.value = null }
})
</script>

<template>
  <div class="terminal-wrapper">
    <div ref="containerRef" class="terminal-container"></div>
    <textarea ref="hiddenInputRef" class="hidden-input" autocomplete="off" autocapitalize="off" autocorrect="off"></textarea>
  </div>
</template>

<style scoped>
.terminal-wrapper { width: 100%; height: 100%; position: relative; overflow: hidden; }
.terminal-container { width: 100%; height: 100%; overflow: hidden; }
.hidden-input {
  position: absolute; top: -9999px; left: -9999px;
  width: 1px; height: 1px; opacity: 0; border: none;
}
</style>