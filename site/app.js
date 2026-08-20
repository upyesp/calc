/* epher website — i18n + theme + disclosure nav.
 *
 * i18n mirrors the Localizer in crates/i18n: device locale auto-detection,
 * a stored user preference, English fallback. The catalogs live in
 * i18n/<lang>.js (loaded before this file as plain scripts — no fetch, so
 * the page also works offline from any mount point); this file applies
 * them by data-i18n / data-i18n-aria / data-i18n-href attributes.
 *
 * Theme: light/dark, defaults to prefers-color-scheme, toggle persisted in
 * localStorage ("epher-theme"). Language preference: "epher-lang".
 *
 * Nav: below 880px the header links collapse into a disclosure menu
 * (WAI-ARIA APG pattern): the button carries aria-expanded, Escape closes
 * it and restores focus, a click outside closes it, and the links are
 * removed from the tab order while closed (the `hidden` attribute).
 */
"use strict";

const SUPPORTED = ["en", "zh-CN", "hi", "es", "fr", "ar", "de", "pt"];

const MESSAGES = window.EPHER_I18N || { en: {} };

function normalize(code) {
  return code.replace("_", "-").toLowerCase();
}

/** Negotiate a supported locale from the device's languages — the static
 *  twin of `Localizer::resolve` in crates/i18n: exact match first, then
 *  language-prefix match, English fallback. */
function detect() {
  const wanted = (navigator.languages || [navigator.language || "en"]).map(normalize);
  for (const w of wanted) {
    const hit = SUPPORTED.find((s) => s.toLowerCase() === w);
    if (hit) return hit;
  }
  for (const w of wanted) {
    const prefix = w.split("-")[0];
    const hit = SUPPORTED.find((s) => s.toLowerCase() === prefix);
    if (hit) return hit;
  }
  return "en";
}

let currentLang = "en";

function applyLang(lang) {
  currentLang = lang;
  const dict = MESSAGES[lang] || MESSAGES.en || {};
  document.querySelectorAll("[data-i18n]").forEach((el) => {
    const key = el.getAttribute("data-i18n");
    if (dict[key]) el.textContent = dict[key];
  });
  document.querySelectorAll("[data-i18n-aria]").forEach((el) => {
    const key = el.getAttribute("data-i18n-aria");
    if (dict[key]) el.setAttribute("aria-label", dict[key]);
  });
  // links whose target depends on the active locale (the user guide)
  document.querySelectorAll("[data-i18n-href]").forEach((el) => {
    el.href = `guide/${lang}/`;
  });
  // WCAG 3.1.1: lang (and dir for Arabic) must track the active locale.
  document.documentElement.lang = lang;
  document.documentElement.dir = lang === "ar" ? "rtl" : "ltr";
  document.getElementById("lang-select").value = lang;
}

function setTheme(theme) {
  document.documentElement.dataset.theme = theme;
  try {
    localStorage.setItem("epher-theme", theme);
  } catch (e) {
    /* private mode: ignore */
  }
  // The toggle's label names the theme it switches TO.
  const next = theme === "dark" ? "light" : "dark";
  const key = next === "dark" ? "theme-dark" : "theme-light";
  const label = (MESSAGES[currentLang] || MESSAGES.en || {})[key];
  const toggle = document.getElementById("theme-toggle");
  if (label) toggle.setAttribute("aria-label", label);
  const hidden = toggle.querySelector(".visually-hidden");
  if (hidden && label) hidden.textContent = label;
  // The brand mark flips tile colors with the theme (the CSS content:url
  // rule handles Chrome/Firefox; this keeps the src right for Safari).
  const brand = document.getElementById("brand-icon");
  if (brand) brand.src = theme === "dark" ? "icon-light.svg?v=3" : "icon.svg?v=3";
}

/** Disclosure nav (mobile): open/close the collapsed header links. */
function initMenu() {
  const button = document.getElementById("menu-toggle");
  const nav = document.getElementById("site-nav");
  if (!button || !nav) return;

  const setMenu = (open) => {
    button.setAttribute("aria-expanded", String(open));
    // `hidden` takes the links out of the tab order and the a11y tree;
    // the desktop stylesheet overrides it back to visible (author rules
    // beat the UA's [hidden] { display: none }).
    nav.hidden = !open;
  };

  button.addEventListener("click", () => {
    setMenu(button.getAttribute("aria-expanded") !== "true");
  });

  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape" && button.getAttribute("aria-expanded") === "true") {
      setMenu(false);
      button.focus();
    }
  });

  document.addEventListener("click", (e) => {
    if (
      button.getAttribute("aria-expanded") === "true" &&
      !nav.contains(e.target) &&
      !button.contains(e.target)
    ) {
      setMenu(false);
    }
  });

  // following a link closes the menu (same-page anchors included)
  nav.addEventListener("click", (e) => {
    if (e.target.closest("a")) setMenu(false);
  });
}

function init() {
  let stored = null;
  try {
    stored = localStorage.getItem("epher-lang");
  } catch (e) {
    /* ignore */
  }
  applyLang(stored && SUPPORTED.includes(stored) ? stored : detect());

  const theme =
    document.documentElement.dataset.theme === "dark" ? "dark" : "light";
  setTheme(theme);

  document.getElementById("theme-toggle").addEventListener("click", () => {
    const current = document.documentElement.dataset.theme === "dark" ? "light" : "dark";
    setTheme(current);
  });

  document.getElementById("lang-select").addEventListener("change", (e) => {
    applyLang(e.target.value);
    try {
      localStorage.setItem("epher-lang", e.target.value);
    } catch (err) {
      /* ignore */
    }
    setTheme(document.documentElement.dataset.theme); // refresh toggle label
  });

  initMenu();
}

document.addEventListener("DOMContentLoaded", init);
