import { A, type RouteSectionProps } from '@solidjs/router'

export function RootLayout(props: RouteSectionProps) {
  return (
    <div class="min-h-screen w-full grid grid-rows-[auto_1fr_auto] md:grid-rows-[auto_1fr_auto] bg-gradient-to-br from-gray-900 via-gray-800 to-gray-950">
      <header class="row-start-1 row-end-2 flex items-start justify-center bg-transparent pt-2 h-[10vh] md:pt-4 md:justify-start">
        <A href="/" class="text-emerald-100 text-xl font-semibold md:ml-4">
          Puzzle Solver
        </A>
      </header>
      <main class="row-start-2 row-end-3 w-full h-full grid grid-cols-1 md:grid-cols-12">
        <aside class="hidden md:flex md:col-span-3 xl:col-span-2 max-w-xs bg-zinc-800 place-content-center">
          <span class="text-gray-600">This is Sidebar</span>
        </aside>
        <section class="place-items-center md:col-span-9 md:place-items-start md:grid md:grid-cols-11 xl:col-span-10 bg-white/10 md:bg-white/20">
          {props.children}
        </section>
      </main>
      <footer class="row-start-3 row-end-4 items-center justify-center hidden md:flex md:h-[5vh]">
        <span class="text-zinc-200 text-xl font-semibold">Footer</span>
      </footer>
    </div>
  )
}
