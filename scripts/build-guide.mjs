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
// copy/copied label the example-block copy button (and its announcement).
const CHROME = {
  en: { title: "epher — User guide", back: "Back to home", contents: "Contents", themeDark: "Use dark theme", themeLight: "Use light theme", footer: "epher user guide", copy: "Copy", copied: "Copied" },
  "zh-CN": { title: "epher — 用户指南", back: "返回主页", contents: "目录", themeDark: "使用深色主题", themeLight: "使用浅色主题", footer: "epher 用户指南", copy: "复制", copied: "已复制" },
  hi: { title: "epher — उपयोगकर्ता गाइड", back: "मुख्य पृष्ठ पर वापस जाएँ", contents: "विषय-सूची", themeDark: "गहरी थीम का उपयोग करें", themeLight: "हल्की थीम का उपयोग करें", footer: "epher उपयोगकर्ता गाइड", copy: "कॉपी करें", copied: "कॉपी हो गया" },
  es: { title: "epher — Guía de usuario", back: "Volver al inicio", contents: "Contenido", themeDark: "Usar tema oscuro", themeLight: "Usar tema claro", footer: "Guía de usuario de epher", copy: "Copiar", copied: "Copiado" },
  fr: { title: "epher — Guide de l'utilisateur", back: "Retour à l'accueil", contents: "Sommaire", themeDark: "Utiliser le thème sombre", themeLight: "Utiliser le thème clair", footer: "Guide de l'utilisateur de epher", copy: "Copier", copied: "Copié" },
  ar: { title: "epher — دليل المستخدم", back: "العودة إلى الصفحة الرئيسية", contents: "المحتويات", themeDark: "استخدام المظهر الداكن", themeLight: "استخدام المظهر الفاتح", footer: "دليل مستخدم epher", copy: "نسخ", copied: "تم النسخ" },
};

// --- example code blocks ------------------------------------------------
//
// Guide fenced blocks come in two kinds (see docs/website.md):
//   ```epher / ```sh → what the reader types: a code block with lightweight
//                      syntax highlighting and a copy button (below)
//   ```text          → what epher answers, REPL transcripts, URLs, paths:
//                      the plain box, unchanged
// The highlighter is a tiny epher tokenizer (keywords, constants, numbers,
// strings, function calls); epher has no comment syntax.

const KEYWORDS = new Set([
  "def", "if", "then", "else", "while", "do", "and", "or", "not",
  "graph", "save", "language", "quit",
]);
const CONSTANTS = new Set(["pi", "e", "tau", "phi", "true", "false"]);

// chrome strings of the language currently being rendered
let currentChrome = CHROME.en;

function escapeHtml(s) {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

const TOKEN = /(\d+(?:\.\d*)?(?:[eE][+-]?\d+)?|\.\d+)|([A-Za-z_][A-Za-z0-9_]*)|("[^"\n]*"|'[^'\n]*')|([\s\S])/g;

function highlightEpher(code) {
  let out = "";
  for (const m of code.matchAll(TOKEN)) {
    const [text, num, ident, str, ch] = m;
    if (num !== undefined) {
      out += `<span class="tok-num">${escapeHtml(text)}</span>`;
    } else if (str !== undefined) {
      out += `<span class="tok-str">${escapeHtml(text)}</span>`;
    } else if (ident !== undefined) {
      const after = code[m.index + text.length]; // function call: ident directly before (
      const cls = KEYWORDS.has(text)
        ? "tok-kw"
        : CONSTANTS.has(text)
          ? "tok-num"
          : after === "("
            ? "tok-fn"
            : null;
      out += cls ? `<span class="${cls}">${escapeHtml(text)}</span>` : escapeHtml(text);
    } else {
      out += escapeHtml(ch);
    }
  }
  return out;
}

function exampleBlock(code, info) {
  return `<div class="example">
<pre><code class="language-${info}">${highlightEpher(code)}</code></pre>
<div class="example-bar">
<button type="button" class="copy-btn">
<svg class="icon-copy" aria-hidden="true" viewBox="0 0 24 24"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
<svg class="icon-check" aria-hidden="true" viewBox="0 0 24 24"><path d="M20 6 9 17l-5-5"/></svg>
<span class="copy-label">${currentChrome.copy}</span>
</button>
</div>
</div>`;
}

marked.use({
  renderer: {
    code(code, infostring) {
      const info = (infostring || "").trim().split(/\s+/)[0];
      if (info === "epher" || info === "sh") return exampleBlock(code, info);
      return `<pre><code class="language-text">${escapeHtml(code)}\n</code></pre>`;
    },
  },
});

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
      var theme = localStorage.getItem("epher-theme");
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
      <span>epher</span>
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

  <p class="visually-hidden" id="copy-status" role="status"></p>

  <script>
    // theme toggle (guide pages are single-language; no i18n needed here)
    (function () {
      var toggle = document.getElementById("theme-toggle");
      var labels = ${JSON.stringify({ dark: CHROME[lang].themeDark, light: CHROME[lang].themeLight })};
      function setTheme(t) {
        document.documentElement.dataset.theme = t;
        try { localStorage.setItem("epher-theme", t); } catch (e) {}
        var next = t === "dark" ? "light" : "dark";
        toggle.setAttribute("aria-label", labels[next]);
      }
      toggle.addEventListener("click", function () {
        setTheme(document.documentElement.dataset.theme === "dark" ? "light" : "dark");
      });
    })();
  </script>

  <script>
    // copy buttons on example code blocks (epher/sh fenced blocks in the md)
    (function () {
      var strings = ${JSON.stringify({ copy: CHROME[lang].copy, copied: CHROME[lang].copied })};
      var live = document.getElementById("copy-status");

      function copyToClipboard(text) {
        if (navigator.clipboard && navigator.clipboard.writeText) {
          return navigator.clipboard.writeText(text).then(
            function () { return true; },
            function () { return fallback(text); }
          );
        }
        return Promise.resolve(fallback(text));
      }

      // older browsers / non-secure contexts
      function fallback(text) {
        try {
          var ta = document.createElement("textarea");
          ta.value = text;
          ta.setAttribute("readonly", "");
          ta.style.position = "fixed";
          ta.style.opacity = "0";
          document.body.appendChild(ta);
          ta.select();
          var ok = document.execCommand("copy");
          ta.remove();
          return ok;
        } catch (e) {
          return false;
        }
      }

      document.querySelectorAll(".copy-btn").forEach(function (btn) {
        var timer = null;
        btn.addEventListener("click", function () {
          var example = btn.closest(".example");
          var code = example && example.querySelector("code");
          if (!code) return;
          copyToClipboard(code.textContent).then(function (ok) {
            if (!ok) return;
            var label = btn.querySelector(".copy-label");
            btn.classList.add("copied");
            label.textContent = strings.copied;
            live.textContent = strings.copied; // announce (role=status)
            clearTimeout(timer);
            timer = setTimeout(function () {
              btn.classList.remove("copied");
              label.textContent = strings.copy;
              live.textContent = "";
            }, 2000);
          });
        });
      });
    })();
  </script>
</body>
</html>`;
}

let built = 0;
for (const lang of LANGS) {
  usedIds.clear();
  currentChrome = CHROME[lang];
  const md = readFileSync(join(GUIDE, `${lang}.md`), "utf8");
  const { html, toc } = postprocess(marked.parse(md, { gfm: true, breaks: false }));
  const outDir = join(GUIDE, lang);
  mkdirSync(outDir, { recursive: true });
  writeFileSync(join(outDir, "index.html"), page(lang, html, toc));
  built++;
}
console.log(`built ${built} guide pages (${LANGS.join(", ")})`);
