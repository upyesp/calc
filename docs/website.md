# Website and GitHub Pages

The project is published at **https://upyesp.github.io/calc/** via GitHub
Pages, built and deployed by the `pages` workflow (`.github/workflows/pages.yml`).

## Site layout

| Path | Content | Source |
|---|---|---|
| `/calc/` | Landing page — links to every build | `site/` (static HTML/CSS/JS, committed) |
| `/calc/pwa/` | The web app (PWA, offline-first) | `crates/web/dist` (built by trunk in CI) |
| GitHub Releases | CLI/TUI/desktop binaries | built by `.github/workflows/release.yml` |

The PWA dist is laid out by `crates/web/index.html`: `copy-file` puts the
manifest/sw/icon at the dist root (a `copy-dir` would bury them in
`dist/public/` and break installability), and `public_url = "./"` in
`Trunk.toml` keeps every asset reference relative so the app works from any
mount point. `public/sw.js` is network-first for navigations (so redeploys
reach users) and runtime-caches assets for offline use; bump its `CACHE`
constant when the strategy changes.

The landing page links to release assets via
`https://github.com/upyesp/calc/releases/latest/download/<asset>` so download
links never need a version number.

## Landing page design

- **i18n**: six locales (`en`, `zh-CN`, `hi`, `es`, `fr`, `ar`). Detection,
  stored preference, and English fallback mirror the `Localizer` in
  `crates/i18n`; the static page reimplements the ~15-line negotiation in
  `site/app.js` (there is no wasm on the landing page). `lang`/`dir` (RTL for
  Arabic) follow the active locale (WCAG 3.1.1).
- **Themes**: light/dark via `[data-theme]`; defaults to
  `prefers-color-scheme`, toggle persists to `localStorage` (`calc-theme`).
  An inline script in `<head>` applies both theme and stored language before
  first paint — no flash.
- **Accessibility**: WCAG 2.2 AA — see `docs/accessibility.md`. Contrast
  values for both themes are recorded in `site/styles.css`; keep them in
  spec when editing colors.
- The English text in `index.html` is the noscript fallback; `app.js` swaps
  in the other locales.

### Adding a string

1. Add the English text in `site/index.html` with `data-i18n="key"` (or
   `data-i18n-aria` for aria-labels).
2. Add the key to all six locale dictionaries in `site/app.js`.
3. Keep the `docs/accessibility.md` checklist in mind (labels, language).

## User guide

`site/guide/<lang>.md` holds the user guide in each of the six languages
(the master is `en.md`; translate it and keep the examples identical). The
`pages` workflow converts them to HTML with
`scripts/build-guide.mjs` (marked + a small template; heading ids, table of
contents, RTL, themes, and the WCAG patterns come from the shared
`styles.css`/`guide.css`). Output goes to `site/guide/<lang>/index.html`,
which is gitignored and generated in CI — run `npm run build:guide`
locally to preview. The landing page links to `guide/<lang>/` and the link
follows the visitor's active language.

Adding a guide language: add `<lang>.md`, add chrome strings in
`build-guide.mjs`, add the landing page strings in `site/app.js`, and add
the option to the `lang-select` in `site/index.html`.

## Releases

Push a version tag and the `release` workflow builds and attaches everything:

```
git tag v0.1.0
git push origin v0.1.0
```

Stable asset names (the landing page depends on them):

```
calc-cli-{linux-x86_64,macos-aarch64,windows-x86_64}.{tar.gz,zip}
calc-tui-{linux-x86_64,macos-aarch64,windows-x86_64}.{tar.gz,zip}
calc-desktop-linux-x86_64.{deb,rpm,AppImage}
calc-desktop-macos-aarch64.dmg
calc-desktop-windows-x86_64.exe
```

macOS and Windows desktop builds are unsigned. If the landing page's
download links need to change (e.g. a new platform), change the names here
and in `site/index.html` together.

## First-time setup (already done for this repo)

1. Enable Pages with the workflow source:
   `gh api repos/upyesp/calc/pages -f build_type=workflow`
2. Push `main` — the `pages` workflow builds and deploys.
3. If the repository ever gets recreated, redo step 1; nothing else is
   configured outside the repo.
