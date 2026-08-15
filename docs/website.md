# Website and GitHub Pages

The project is published at **https://upyesp.github.io/epher/** via GitHub
Pages, built and deployed by the `pages` workflow (`.github/workflows/pages.yml`).

## Site layout

| Path | Content | Source |
|---|---|---|
| `/epher/` | Landing page — links to every build | `site/` (static HTML/CSS/JS, committed) |
| `/epher/pwa/` | The web app (PWA, offline-first) | `crates/web/dist` (built by trunk in CI) |
| GitHub Releases | unified platform installers (ADR-0011) | built by `.github/workflows/release.yml` |

The PWA dist is laid out by `crates/web/index.html`: `copy-file` puts the
manifest/sw/icon at the dist root (a `copy-dir` would bury them in
`dist/public/` and break installability), and `public_url = "./"` in
`Trunk.toml` keeps every asset reference relative so the app works from any
mount point. `public/sw.js` is network-first for navigations (so redeploys
reach users) and runtime-caches assets for offline use; bump its `CACHE`
constant when the strategy changes.

The landing page links to release assets via
`https://github.com/upyesp/epher/releases/latest/download/<asset>` so download
links never need a version number.

## Landing page design

- **i18n**: six locales (`en`, `zh-CN`, `hi`, `es`, `fr`, `ar`). Detection,
  stored preference, and English fallback mirror the `Localizer` in
  `crates/i18n`; the static page reimplements the ~15-line negotiation in
  `site/app.js` (there is no wasm on the landing page). `lang`/`dir` (RTL for
  Arabic) follow the active locale (WCAG 3.1.1).
- **Themes**: light/dark via `[data-theme]`; defaults to
  `prefers-color-scheme`, toggle persists to `localStorage` (`epher-theme`).
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
git tag v0.3.0
git push origin v0.3.0
```

One download per platform (ADR-0011): every installer carries the single
unified `epher` executable — one-shot CLI, REPL (`epher repl`), piped
scripts (`epher -`), TUI (`epher tui`), and the desktop GUI (bare `epher` /
`epher gui`). The old per-frontend archives (v0.1.x–v0.2.x) are gone.

Stable asset names (the landing page depends on them):

```
epher-windows-x86_64.exe
epher-macos-aarch64.dmg
epher-linux-x86_64.{deb,rpm,AppImage}
```

- Windows: NSIS installer; `installerHooks` (`nsis-hooks.nsh`) adds the
  install dir to the user PATH so `epher` works from any terminal.
  `makensis nsis-check.nsi` compile-verifies the hook script (locally and
  as a dedicated job in the release workflow).
- macOS: unsigned dmg; the app's "Install the epher command" button
  symlinks `/usr/local/bin/epher` (osascript fallback for admin rights).
- Linux: deb/rpm install `epher` into `/usr/bin`; the AppImage covers Arch
  and every other distro.

macOS and Windows builds are unsigned. If the landing page's download
links need to change (e.g. a new platform), change the names here and in
`site/index.html` together.

## First-time setup (already done for this repo)

1. Enable Pages with the workflow source:
   `gh api repos/upyesp/epher/pages -f build_type=workflow`
2. Push `main` — the `pages` workflow builds and deploys.
3. If the repository ever gets recreated, redo step 1; nothing else is
   configured outside the repo.
