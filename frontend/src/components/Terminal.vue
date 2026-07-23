<script setup lang="ts">
/*
 * LanTerm - Lightweight LAN web terminal sharing
 *
 * Copyright (C) 2026 清木殇
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

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

  // 给终端初始焦点，让光标闪烁
  t.focus()

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
  <div ref="containerRef" class="terminal-container"></div>
</template>

<style scoped>
.terminal-container { width: 100%; height: 100%; overflow: hidden; }
</style>