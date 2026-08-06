// Colour rules shared by nodes, edges and the sidebar.

export const railColor = (r: string) =>
  r.startsWith('GND') ? '#111827' : /^V/.test(r) ? '#dc2626' : '#0e7490'

export const netWireColor = (net: string, dark: boolean): string => {
  let h = 0
  for (let i = 0; i < net.length; i++) h = (h * 31 + net.charCodeAt(i)) >>> 0
  const hue = h % 360
  return dark ? `hsl(${hue} 75% 62%)` : `hsl(${hue} 70% 40%)`
}

export const selectNet = (net: string) =>
  window.dispatchEvent(new CustomEvent('explorer-select-net', { detail: net }))
