import type { RouteSectionProps } from '@solidjs/router'

export function RootLayout(props: RouteSectionProps) {
  return (
    <div class="grid min-h-screen grid-rows-layout bg-gradient-to-br from-gray-900 via-gray-800 to-gray-950">
      <header class="row-start-1"></header>
      <main class="row-start-2">{props.children}</main>
      <footer class="row-start-3"></footer>
    </div>
  )
}
