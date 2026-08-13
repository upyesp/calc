/* Calc landing page — i18n + theme.
 *
 * i18n mirrors the Localizer in crates/i18n: device locale auto-detection,
 * a stored user preference, English fallback. The landing page is static
 * (no wasm), so the negotiation is reimplemented here in ~15 lines; the
 * catalog lives in this file, keyed like the Fluent catalogs.
 *
 * Theme: light/dark, defaults to prefers-color-scheme, toggle persisted in
 * localStorage ("calc-theme"). Language preference: "calc-lang".
 */
"use strict";

const SUPPORTED = ["en", "zh-CN", "hi", "es", "fr", "ar"];

const MESSAGES = {
  en: {
    "skip-link": "Skip to content",
    "nav-label": "Main",
    "source-link": "Source code",
    "theme-light": "Use light theme",
    "theme-dark": "Use dark theme",
    "lang-label": "Language",
    tagline: "A programmable, scriptable calculator",
    lede: "One calculation engine, four ways to use it. Type expressions, save functions and scripts, graph your results, and keep everything across sessions — in any of six languages.",
    builds: "Get Calc",
    "cli-name": "Command line",
    "cli-desc": "One-shot expressions and an interactive REPL with saved functions, scripts, and settings.",
    "tui-name": "Terminal UI",
    "tui-desc": "A full-screen terminal app with persistent history, saved functions, and ASCII graphing.",
    "desktop-name": "Desktop app",
    "desktop-desc": "A native window around the same engine — installs like a regular application.",
    "pwa-name": "Web app",
    "pwa-desc": "Runs in your browser, is installable, and works fully offline after the first visit.",
    downloads: "Downloads",
    get: "Get it",
    "cli-linux": "Download Calc CLI for Linux",
    "cli-macos": "Download Calc CLI for macOS",
    "cli-windows": "Download Calc CLI for Windows",
    "tui-linux": "Download Calc TUI for Linux",
    "tui-macos": "Download Calc TUI for macOS",
    "tui-windows": "Download Calc TUI for Windows",
    "desktop-deb": "Download Calc Desktop for Linux (.deb)",
    "desktop-rpm": "Download Calc Desktop for Linux (.rpm)",
    "desktop-appimage": "Download Calc Desktop for Linux (AppImage)",
    "desktop-macos": "Download Calc Desktop for macOS",
    "desktop-windows": "Download Calc Desktop for Windows",
    "pwa-launch": "Launch the web app",
    "offline-note": "Works fully offline once loaded — install it from your browser's menu.",
    "footer-source": "Source code on GitHub",
    "footer-license": "Calc is free and open source (MIT).",
    "footer-release": "Downloads come from the latest GitHub release."
  },

  "zh-CN": {
    "skip-link": "跳到主要内容",
    "nav-label": "主导航",
    "source-link": "源代码",
    "theme-light": "使用浅色主题",
    "theme-dark": "使用深色主题",
    "lang-label": "语言",
    tagline: "可编程、可脚本化的计算器",
    lede: "一套计算引擎，四种使用方式。输入表达式、保存函数与脚本、绘制结果图表，并在会话之间保留所有内容——支持六种语言。",
    builds: "获取 Calc",
    "cli-name": "命令行",
    "cli-desc": "单次表达式计算与交互式 REPL，支持保存函数、脚本和设置。",
    "tui-name": "终端界面",
    "tui-desc": "全屏终端应用，带持久历史、已保存函数和 ASCII 图表。",
    "desktop-name": "桌面应用",
    "desktop-desc": "同一引擎的原生窗口，像普通应用程序一样安装。",
    "pwa-name": "网页应用",
    "pwa-desc": "在浏览器中运行，可安装，首次访问后完全离线可用。",
    downloads: "下载",
    get: "获取",
    "cli-linux": "下载 Linux 版 Calc 命令行",
    "cli-macos": "下载 macOS 版 Calc 命令行",
    "cli-windows": "下载 Windows 版 Calc 命令行",
    "tui-linux": "下载 Linux 版 Calc 终端界面",
    "tui-macos": "下载 macOS 版 Calc 终端界面",
    "tui-windows": "下载 Windows 版 Calc 终端界面",
    "desktop-deb": "下载 Linux 版 Calc 桌面应用（.deb）",
    "desktop-rpm": "下载 Linux 版 Calc 桌面应用（.rpm）",
    "desktop-appimage": "下载 Linux 版 Calc 桌面应用（AppImage）",
    "desktop-macos": "下载 macOS 版 Calc 桌面应用",
    "desktop-windows": "下载 Windows 版 Calc 桌面应用",
    "pwa-launch": "打开网页应用",
    "offline-note": "加载一次后即可完全离线使用——可从浏览器菜单中安装。",
    "footer-source": "GitHub 上的源代码",
    "footer-license": "Calc 是免费开源软件（MIT 许可）。",
    "footer-release": "下载来自最新的 GitHub 版本。"
  },

  hi: {
    "skip-link": "मुख्य सामग्री पर जाएँ",
    "nav-label": "मुख्य नेविगेशन",
    "source-link": "स्रोत कोड",
    "theme-light": "हल्की थीम का उपयोग करें",
    "theme-dark": "गहरी थीम का उपयोग करें",
    "lang-label": "भाषा",
    tagline: "एक प्रोग्राम करने योग्य, स्क्रिप्ट करने योग्य कैलकुलेटर",
    lede: "एक गणना इंजन, चार तरीकों से उपयोग करें। व्यंजक टाइप करें, फ़ंक्शन और स्क्रिप्ट सहेजें, परिणामों के ग्राफ़ बनाएँ — और सब कुछ सत्रों के बीच बनाए रखें, छह भाषाओं में।",
    builds: "Calc प्राप्त करें",
    "cli-name": "कमांड लाइन",
    "cli-desc": "एकल व्यंजक और इंटरैक्टिव REPL, सहेजे गए फ़ंक्शन, स्क्रिप्ट और सेटिंग्स के साथ।",
    "tui-name": "टर्मिनल इंटरफ़ेस",
    "tui-desc": "पूर्ण-स्क्रीन टर्मिनल ऐप — स्थायी इतिहास, सहेजे गए फ़ंक्शन और ASCII ग्राफ़िंग।",
    "desktop-name": "डेस्कटॉप ऐप",
    "desktop-desc": "उसी इंजन के चारों ओर एक मूल विंडो — सामान्य एप्लिकेशन की तरह इंस्टॉल होती है।",
    "pwa-name": "वेब ऐप",
    "pwa-desc": "ब्राउज़र में चलता है, इंस्टॉल हो सकता है, और पहली बार खोलने के बाद पूरी तरह ऑफ़लाइन काम करता है।",
    downloads: "डाउनलोड",
    get: "प्राप्त करें",
    "cli-linux": "Linux के लिए Calc CLI डाउनलोड करें",
    "cli-macos": "macOS के लिए Calc CLI डाउनलोड करें",
    "cli-windows": "Windows के लिए Calc CLI डाउनलोड करें",
    "tui-linux": "Linux के लिए Calc TUI डाउनलोड करें",
    "tui-macos": "macOS के लिए Calc TUI डाउनलोड करें",
    "tui-windows": "Windows के लिए Calc TUI डाउनलोड करें",
    "desktop-deb": "Linux के लिए Calc डेस्कटॉप डाउनलोड करें (.deb)",
    "desktop-rpm": "Linux के लिए Calc डेस्कटॉप डाउनलोड करें (.rpm)",
    "desktop-appimage": "Linux के लिए Calc डेस्कटॉप डाउनलोड करें (AppImage)",
    "desktop-macos": "macOS के लिए Calc डेस्कटॉप डाउनलोड करें",
    "desktop-windows": "Windows के लिए Calc डेस्कटॉप डाउनलोड करें",
    "pwa-launch": "वेब ऐप खोलें",
    "offline-note": "लोड होते ही पूरी तरह ऑफ़लाइन काम करता है — इसे अपने ब्राउज़र मेनू से इंस्टॉल करें।",
    "footer-source": "GitHub पर स्रोत कोड",
    "footer-license": "Calc निःशुल्क और ओपन सोर्स (MIT) है।",
    "footer-release": "डाउनलोड नवीनतम GitHub रिलीज़ से आते हैं।"
  },

  es: {
    "skip-link": "Saltar al contenido",
    "nav-label": "Principal",
    "source-link": "Código fuente",
    "theme-light": "Usar tema claro",
    "theme-dark": "Usar tema oscuro",
    "lang-label": "Idioma",
    tagline: "Una calculadora programable y con scripts",
    lede: "Un motor de cálculo, cuatro formas de usarlo. Escribe expresiones, guarda funciones y scripts, dibuja tus resultados y conserva todo entre sesiones, en seis idiomas.",
    builds: "Obtén Calc",
    "cli-name": "Línea de comandos",
    "cli-desc": "Expresiones de una sola vez y REPL interactivo con funciones, scripts y ajustes guardados.",
    "tui-name": "Interfaz de terminal",
    "tui-desc": "Una aplicación de terminal a pantalla completa con historial persistente, funciones guardadas y gráficos ASCII.",
    "desktop-name": "Aplicación de escritorio",
    "desktop-desc": "Una ventana nativa alrededor del mismo motor: se instala como una aplicación normal.",
    "pwa-name": "Aplicación web",
    "pwa-desc": "Se ejecuta en tu navegador, es instalable y funciona totalmente sin conexión tras la primera visita.",
    downloads: "Descargas",
    get: "Obtener",
    "cli-linux": "Descargar Calc CLI para Linux",
    "cli-macos": "Descargar Calc CLI para macOS",
    "cli-windows": "Descargar Calc CLI para Windows",
    "tui-linux": "Descargar Calc TUI para Linux",
    "tui-macos": "Descargar Calc TUI para macOS",
    "tui-windows": "Descargar Calc TUI para Windows",
    "desktop-deb": "Descargar Calc de escritorio para Linux (.deb)",
    "desktop-rpm": "Descargar Calc de escritorio para Linux (.rpm)",
    "desktop-appimage": "Descargar Calc de escritorio para Linux (AppImage)",
    "desktop-macos": "Descargar Calc de escritorio para macOS",
    "desktop-windows": "Descargar Calc de escritorio para Windows",
    "pwa-launch": "Abrir la aplicación web",
    "offline-note": "Funciona totalmente sin conexión una vez cargada: instálala desde el menú de tu navegador.",
    "footer-source": "Código fuente en GitHub",
    "footer-license": "Calc es software libre y de código abierto (MIT).",
    "footer-release": "Las descargas provienen de la última versión de GitHub."
  },

  fr: {
    "skip-link": "Aller au contenu",
    "nav-label": "Navigation principale",
    "source-link": "Code source",
    "theme-light": "Utiliser le thème clair",
    "theme-dark": "Utiliser le thème sombre",
    "lang-label": "Langue",
    tagline: "Une calculatrice programmable et scriptable",
    lede: "Un moteur de calcul, quatre façons de l'utiliser. Saisissez des expressions, enregistrez fonctions et scripts, tracez vos résultats, et conservez tout d'une session à l'autre, dans six langues.",
    builds: "Obtenir Calc",
    "cli-name": "Ligne de commande",
    "cli-desc": "Expressions ponctuelles et REPL interactif avec fonctions, scripts et réglages enregistrés.",
    "tui-name": "Interface de terminal",
    "tui-desc": "Une application terminal plein écran avec historique persistant, fonctions enregistrées et graphiques ASCII.",
    "desktop-name": "Application de bureau",
    "desktop-desc": "Une fenêtre native autour du même moteur — s'installe comme une application classique.",
    "pwa-name": "Application web",
    "pwa-desc": "Fonctionne dans votre navigateur, est installable et fonctionne entièrement hors ligne après la première visite.",
    downloads: "Téléchargements",
    get: "Obtenir",
    "cli-linux": "Télécharger Calc CLI pour Linux",
    "cli-macos": "Télécharger Calc CLI pour macOS",
    "cli-windows": "Télécharger Calc CLI pour Windows",
    "tui-linux": "Télécharger Calc TUI pour Linux",
    "tui-macos": "Télécharger Calc TUI pour macOS",
    "tui-windows": "Télécharger Calc TUI pour Windows",
    "desktop-deb": "Télécharger Calc bureau pour Linux (.deb)",
    "desktop-rpm": "Télécharger Calc bureau pour Linux (.rpm)",
    "desktop-appimage": "Télécharger Calc bureau pour Linux (AppImage)",
    "desktop-macos": "Télécharger Calc bureau pour macOS",
    "desktop-windows": "Télécharger Calc bureau pour Windows",
    "pwa-launch": "Ouvrir l'application web",
    "offline-note": "Fonctionne entièrement hors ligne une fois chargée — installez-la depuis le menu de votre navigateur.",
    "footer-source": "Code source sur GitHub",
    "footer-license": "Calc est un logiciel libre et open source (MIT).",
    "footer-release": "Les téléchargements proviennent de la dernière version GitHub."
  },

  ar: {
    "skip-link": "تخطَّ إلى المحتوى",
    "nav-label": "التنقل الرئيسي",
    "source-link": "الكود المصدري",
    "theme-light": "استخدام المظهر الفاتح",
    "theme-dark": "استخدام المظهر الداكن",
    "lang-label": "اللغة",
    tagline: "آلة حاسبة قابلة للبرمجة والكتابة النصية",
    lede: "محرك حساب واحد، وأربع طرق لاستخدامه. اكتب التعابير، واحفظ الدوال والنصوص البرمجية، وارسم النتائج، واحتفظ بكل شيء بين الجلسات — بست لغات.",
    builds: "احصل على Calc",
    "cli-name": "سطر الأوامر",
    "cli-desc": "تعابير لمرة واحدة وواجهة REPL تفاعلية مع دوال ونصوص برمجية وإعدادات محفوظة.",
    "tui-name": "واجهة الطرفية",
    "tui-desc": "تطبيق طرفية بملء الشاشة مع سجل دائم ودوال محفوظة ورسوم بيانية ASCII.",
    "desktop-name": "تطبيق سطح المكتب",
    "desktop-desc": "نافذة أصلية حول المحرك نفسه — يُثبَّت مثل أي تطبيق عادي.",
    "pwa-name": "تطبيق الويب",
    "pwa-desc": "يعمل في متصفحك، ويمكن تثبيته، ويعمل دون اتصال بالكامل بعد أول زيارة.",
    downloads: "التنزيلات",
    get: "احصل عليه",
    "cli-linux": "تنزيل Calc CLI لنظام Linux",
    "cli-macos": "تنزيل Calc CLI لنظام macOS",
    "cli-windows": "تنزيل Calc CLI لنظام Windows",
    "tui-linux": "تنزيل Calc TUI لنظام Linux",
    "tui-macos": "تنزيل Calc TUI لنظام macOS",
    "tui-windows": "تنزيل Calc TUI لنظام Windows",
    "desktop-deb": "تنزيل Calc لسطح المكتب لنظام Linux (.deb)",
    "desktop-rpm": "تنزيل Calc لسطح المكتب لنظام Linux (.rpm)",
    "desktop-appimage": "تنزيل Calc لسطح المكتب لنظام Linux (AppImage)",
    "desktop-macos": "تنزيل Calc لسطح المكتب لنظام macOS",
    "desktop-windows": "تنزيل Calc لسطح المكتب لنظام Windows",
    "pwa-launch": "فتح تطبيق الويب",
    "offline-note": "يعمل دون اتصال بالكامل بعد تحميله — ثبِّته من قائمة المتصفح.",
    "footer-source": "الكود المصدري على GitHub",
    "footer-license": "Calc برنامج مجاني ومفتوح المصدر (MIT).",
    "footer-release": "التنزيلات من أحدث إصدار على GitHub."
  }
};

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
  const dict = MESSAGES[lang] || MESSAGES.en;
  document.querySelectorAll("[data-i18n]").forEach((el) => {
    const key = el.getAttribute("data-i18n");
    if (dict[key]) el.textContent = dict[key];
  });
  document.querySelectorAll("[data-i18n-aria]").forEach((el) => {
    const key = el.getAttribute("data-i18n-aria");
    if (dict[key]) el.setAttribute("aria-label", dict[key]);
  });
  // WCAG 3.1.1: lang (and dir for Arabic) must track the active locale.
  document.documentElement.lang = lang;
  document.documentElement.dir = lang === "ar" ? "rtl" : "ltr";
  document.getElementById("lang-select").value = lang;
}

function setTheme(theme) {
  document.documentElement.dataset.theme = theme;
  try {
    localStorage.setItem("calc-theme", theme);
  } catch (e) {
    /* private mode: ignore */
  }
  // The toggle's label names the theme it switches TO.
  const next = theme === "dark" ? "light" : "dark";
  const key = next === "dark" ? "theme-dark" : "theme-light";
  const label = (MESSAGES[currentLang] || MESSAGES.en)[key];
  const toggle = document.getElementById("theme-toggle");
  if (label) toggle.setAttribute("aria-label", label);
  const hidden = toggle.querySelector(".visually-hidden");
  if (hidden && label) hidden.textContent = label;
}

function init() {
  let stored = null;
  try {
    stored = localStorage.getItem("calc-lang");
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
      localStorage.setItem("calc-lang", e.target.value);
    } catch (err) {
      /* ignore */
    }
    setTheme(document.documentElement.dataset.theme); // refresh toggle label
  });
}

document.addEventListener("DOMContentLoaded", init);
