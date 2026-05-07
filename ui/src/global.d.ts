interface WgTerminal {
  open(elementId: string, wsUrl: string): void
  dispose(elementId: string): void
}

interface WgRemoteDesktop {
  open(canvasId: string, wsUrl: string, width: number, height: number): void
  dispose(canvasId: string): void
  sendPli(canvasId: string): void
}

interface Window {
  wgTerminal: WgTerminal
  wgRemoteDesktop: WgRemoteDesktop
}
