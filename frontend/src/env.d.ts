declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  // Use safer, non-empty object types and avoid `any` to reduce lint warnings
  const component: DefineComponent<Record<string, unknown>, Record<string, unknown>, unknown>
  export default component
}

interface ImportMetaEnv {
  readonly VITE_API_URL?: string
  readonly MODE?: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}
