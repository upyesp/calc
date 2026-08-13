// Build the user guide: site/guide/<lang>.md → site/guide/<lang>/index.html
// (one static, localized, theme-capable HTML page per language).
//
// Run: npm run build:guide  (CI: pages.yml runs this before assembling _site)
//
// The pages share the landing page's styles.css + guide.css and follow the
// same accessibility requirements (WCAG 2.2 AA — skip link, labels, contrast,
// lang/dir per locale, focus-visible via the shared stylesheet).
import { marked } from "marked";
import { readFileSync, mkdirSync, writeFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const GUIDE = join(ROOT, "site", "guide");

const LANGS = ["en", "zh-CN", "hi", "es", "fr", "ar"];

// Per-language chrome strings (mirror the landing page dictionaries in
// site/app.js; the guide page itself is single-language so no runtime i18n).
const CHROME = {
  en: { title: "Calc — User guide", back: "Back to home", contents: "Contents", themeDark: "Use dark theme", themeLight: "Use light theme", footer: "Calc user guide" },
  "zh-CN": { title: "Calc — 用户指南", back: "返回主页", contents: "目录", themeDark: "使用深色主题", themeLight: "使用浅色主题", footer: "Calc 用户指南" },
  hi: { title: "Calc — उपयोगकर्ता गाइड", back: "मुख्य पृष्ठ पर वापस जाएँ", contents: "विषय-सूची", themeDark: "गहरी थीम का उपयोग करें", themeLight: "हल्की थीम का उपयोग करें", footer: "Calc उपयोगकर्ता गाइड" },
  es: { title: "Calc — Guía de usuario", back: "Volver al inicio", contents: "Contenido", themeDark: "Usar tema oscuro", themeLight: "Usar tema claro", footer: "Guía de usuario de Calc" },
  fr: { title: "Calc — Guide de l'utilisateur", back: "Retour à l'accueil", contents: "Sommaire", themeDark: "Utiliser le thème sombre", themeLight: "Utiliser le thème clair", footer: "Guide de l'utilisateur de Calc" },
  ar: { title: "Calc — دليل المستخدم", back: "العودة إلى الصفحة الرئيسية", contents: "المحتويات", themeDark: "استخدام المظهر الداكن", themeLight: "استخدام المظهر الفاتح", footer: "دليل مستخدم Calc" },
};

const usedIds = new Set();
function slugify(text) {
  let slug = text
    .toLowerCase()
    .replace(/[^a-z0-9\u0600-\u06FF\u4E00-\u9FFF\u0900-\u097F\s-]/g, "")
    .trim()
    .replace(/\s+/g, "-");
  if (!slug) slug = "section";
  let id = slug;
  let n = 2;
  while (usedIds.has(id)) id = `${slug}-${n++}`;
  usedIds.add(id);
  return id;
}

function postprocess(html) {
  // wrap tables for horizontal scroll (mobile + 200% zoom)
  html = html.replace(
    /<table>/g,
    '<div class="table-wrap"><table>'
  ).replace(
    /<\/table>/g,
    "</table></div>"
  );

  // heading ids + TOC entries
  const toc = [];
  html = html.replace(/<h([1-4])>(.*?)<\/h\1>/g, (_, level, inner) => {
    const text = inner.replace(/<[^>]*>/g, "");
    const id = slugify(text);
    if (level >= 2 && level <= 3) {
      toc.push(`<li class="toc-l${level}"><a href="#${id}">${text}</a></li>`);
    }
    return `<h${level} id="${id}">${inner}</h${level}>`;
  });
  return { html, toc };
}

function themeScript() {
  return `<script>
  (function () {
    try {
      var theme = localStorage.getItem("calc-theme");
      if (theme !== "light" && theme !== "dark") {
        theme = window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
      }
      document.documentElement.dataset.theme = theme;
    } catch (e) {
      document.documentElement.dataset.theme = "light";
    }
  })();
</script>`;
}

function page(lang, body, toc) {
  const c = CHROME[lang];
  const dir = lang === "ar" ? ' dir="rtl"' : "";
  return `<!DOCTYPE html>
<html lang="${lang}"${dir}>
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>${c.title}</title>
  <meta name="theme-color" media="(prefers-color-scheme: light)" content="#ffffff" />
  <meta name="theme-color" media="(prefers-color-scheme: dark)" content="#1c1c1e" />
  <link rel="icon" href="../../icon.svg" type="image/svg+xml" />
  <link rel="stylesheet" href="../../styles.css" />
  <link rel="stylesheet" href="../../guide.css" />
  ${themeScript()}
</head>
<body>
  <a class="skip-link" href="#main">${c.back}</a>
  <header class="site-header guide-header">
    <a class="brand" href="../../">
      <img class="brand-icon" src="../../icon.svg" alt="" width="32" height="32" />
      <span>Calc</span>
    </a>
    <nav class="header-actions" aria-label="${c.contents}">
      <a class="gh-link" href="../../">${c.back}</a>
      <button type="button" id="theme-toggle" class="icon-btn" aria-pressed="false" aria-label="${c.themeDark}">
        <svg class="icon-moon" aria-hidden="true" viewBox="0 0 24 24"><path d="M21 12.8A9 9 0 1 1 11.2 3 7 7 0 0 0 21 12.8Z" /></svg>
        <svg class="icon-sun" aria-hidden="true" viewBox="0 0 24 24"><circle cx="12" cy="12" r="4.5" /><path d="M12 2v2.5M12 19.5V22M2 12h2.5M19.5 12H22M4.9 4.9l1.8 1.8M17.3 17.3l1.8 1.8M19.1 4.9l-1.8 1.8M6.7 17.3l-1.8 1.8" /></svg>
        <span class="visually-hidden">${c.themeDark}</span>
      </button>
    </nav>
  </header>

  <main id="main" class="guide">
    <nav class="toc" aria-label="${c.contents}">
      <h2 class="toc-title">${c.contents}</h2>
      <ul>${toc.join("")}</ul>
    </nav>
    <div class="guide-body">
      ${body}
    </div>
  </main>

  <footer class="site-footer">
    <p class="muted">${c.footer}</p>
  </footer>

  <script>
    // theme toggle (guide pages are single-language; no i18n needed here)
    (function () {
      var toggle = document.getElementById("theme-toggle");
      var labels = ${JSON.stringify({ dark: CHROME[lang].themeDark, light: CHROME[lang].themeLight })};
      function setTheme(t) {
        document.documentElement.dataset.theme = t;
        try { localStorage.setItem("calc-theme", t); } catch (e) {}
        var next = t === "dark" ? "light" : "dark";
        toggle.setAttribute("aria-label", labels[next]);
      }
      toggle.addEventListener("click", function () {
        setTheme(document.documentElement.dataset.theme === "dark" ? "light" : "dark");
      });
    })();
  </script>
</body>
</html>`;
}

let built = 0;
for (const lang of LANGS) {
  usedIds.clear();
  const md = readFileSync(join(GUIDE, `${lang}.md`), "utf8");
  const { html, toc } = postprocess(marked.parse(md, { gfm: true, breaks: false }));
  const outDir = join(GUIDE, lang);
  mkdirSync(outDir, { recursive: true });
  writeFileSync(join(outDir, "index.html"), page(lang, html, toc));
  built++;
}
console.log(`built ${built} guide pages (${LANGS.join(", ")})`);
