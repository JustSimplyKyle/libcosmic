# WebAssembly support branch

This branch is based on
[`pop-os/libcosmic`](https://github.com/pop-os/libcosmic) revision
`43a50cb6f5af18fa6cbea535fb46460a3c5cd573`.

Its `iced` submodule points to
[`JustSimplyKyle/iced`](https://github.com/JustSimplyKyle/iced) revision
`d213c65897181738137232da7dbe854f43ecead5`, based on
[`pop-os/iced`](https://github.com/pop-os/iced) revision
`f5aa5a6f8cc2db6e2adbbc140c310a5a4af2f8d2`. Keeping the whole iced family at
one revision is required because iced's internal types are not interchangeable
between revisions.

The iced runner uses the libcosmic `cosmic-0.14` winit fork at revision
`261cda54017f98a12dc55569c864430fe6770366`, as locked by the root
`Cargo.lock`.

## WebAssembly patches

- Added a `web` feature that enables libcosmic's `winit` and `wgpu` backends
  together with iced WebGL.
- Added a no-system-config fallback for targets that are neither Unix nor
  Windows. Browser builds therefore use COSMIC's defaults.
- Updated the vendored iced winit runner to the current winit web extension
  API, initialize its browser boot state, run the browser event loop, and keep
  the compositor alive for the event loop's static lifetime. The mounted canvas
  is sized to the viewport and browser-default body margins are removed.
- Wake the browser event loop when asynchronous iced actions arrive, and keep
  Vulkan environment-variable cleanup on its Wayland-only code path so browser
  compositor creation cannot call unsupported process-environment APIs.
- Restrict WASM compositor selection to iced's enabled WebGL backend instead of
  preferring browser WebGPU, improving support across browsers and software
  rendering environments.
These changes deliberately patch the coherent libcosmic/iced stack instead of
replacing only `iced_winit`, which would introduce incompatible duplicate iced
types. Browser consumers may also need to patch libcosmic's git `atomicwrites`
dependency with a target-compatible implementation; `real-blog` supplies a
small workspace-local adapter for that purpose.
