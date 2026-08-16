/* epher landing page — i18n + theme.
 *
 * i18n mirrors the Localizer in crates/i18n: device locale auto-detection,
 * a stored user preference, English fallback. The landing page is static
 * (no wasm), so the negotiation is reimplemented here in ~15 lines; the
 * catalog lives in this file, keyed like the Fluent catalogs.
 *
 * Theme: light/dark, defaults to prefers-color-scheme, toggle persisted in
 * localStorage ("epher-theme"). Language preference: "epher-lang".
 */
"use strict";

const SUPPORTED = ["en", "zh-CN", "hi", "es", "fr", "ar"];

const MESSAGES = {
  en: {
    "skip-link": "Skip to content",
    "nav-label": "Main",
    guide: "User guide",
    "guide-cta": "Read the user guide",
    "source-link": "Source code",
    "theme-light": "Use light theme",
    "theme-dark": "Use dark theme",
    "lang-label": "Language",
    tagline: "A programmable, scriptable calculator",
    lede: "One calculation engine, four ways to use it. Type expressions, save functions and scripts, graph your results, and keep everything across sessions — in any of six languages.",
    builds: "Get epher",
    "one-install": "One download, every way to use epher: the command line, the REPL, the terminal UI, and the desktop app — all in the single epher executable.",
    "win-name": "Windows",
    "win-desc": "One installer. It puts epher on your PATH — use it from CMD, PowerShell, the Start menu, or a double-click.",
    "win-download": "Download the Windows installer",
    "mac-name": "macOS",
    "mac-desc": "One app. Drag it to Applications; a button inside installs the epher terminal command for you.",
    "mac-download": "Download for macOS (Apple Silicon)",
    "linux-name": "Linux",
    "linux-desc": "One install per package family: Debian/Ubuntu, Fedora/RHEL, or the AppImage for everything else (Arch included). All put epher on your PATH.",
    "linux-deb": "Download for Debian/Ubuntu (.deb)",
    "linux-rpm": "Download for Fedora/RHEL (.rpm)",
    "linux-appimage": "Download the AppImage (any distro, incl. Arch)",
    "pwa-name": "Web app",
    "pwa-desc": "Runs in your browser, is installable, and works fully offline after the first visit.",
    downloads: "Downloads",
    get: "Get it",
    "pwa-launch": "Launch the web app",
    "offline-note": "Works fully offline once loaded — install it from your browser's menu.",
    "footer-source": "Source code on GitHub",
    "footer-license": "epher is free and open source (MIT).",
    "footer-release": "Downloads come from the latest GitHub release."
  },

  "zh-CN": {
    "skip-link": "跳到主要内容",
    "nav-label": "主导航",
    guide: "用户指南",
    "guide-cta": "阅读用户指南",
    "source-link": "源代码",
    "theme-light": "使用浅色主题",
    "theme-dark": "使用深色主题",
    "lang-label": "语言",
    tagline: "可编程、可脚本化的计算器",
    lede: "一套计算引擎，四种使用方式。输入表达式、保存函数与脚本、绘制结果图表，并在会话之间保留所有内容——支持六种语言。",
    builds: "获取 epher",
    "one-install": "一次下载，囊括 epher 的所有用法：命令行、REPL、终端界面和桌面应用——全都包含在同一个 epher 可执行文件中。",
    "win-name": "Windows",
    "win-desc": "一个安装程序。安装后 epher 即可在 PATH 中使用——CMD、PowerShell、开始菜单或双击都能启动。",
    "win-download": "下载 Windows 安装程序",
    "mac-name": "macOS",
    "mac-desc": "一个应用。拖入「应用程序」文件夹即可；应用内有一个按钮可为你安装 epher 终端命令。",
    "mac-download": "下载 macOS 版（Apple 芯片）",
    "linux-name": "Linux",
    "linux-desc": "每个包系列各一个安装包：Debian/Ubuntu 用 .deb，Fedora/RHEL 用 .rpm，其他发行版（包括 Arch）用 AppImage。安装后 epher 均在 PATH 中。",
    "linux-deb": "下载 Debian/Ubuntu 版（.deb）",
    "linux-rpm": "下载 Fedora/RHEL 版（.rpm）",
    "linux-appimage": "下载 AppImage（任何发行版，含 Arch）",
    "pwa-name": "网页应用",
    "pwa-desc": "在浏览器中运行，可安装，首次访问后完全离线可用。",
    downloads: "下载",
    get: "获取",
    "pwa-launch": "打开网页应用",
    "offline-note": "加载一次后即可完全离线使用——可从浏览器菜单中安装。",
    "footer-source": "GitHub 上的源代码",
    "footer-license": "epher 是免费开源软件（MIT 许可）。",
    "footer-release": "下载来自最新的 GitHub 版本。"
  },

  hi: {
    "skip-link": "मुख्य सामग्री पर जाएँ",
    "nav-label": "मुख्य नेविगेशन",
    guide: "उपयोगकर्ता गाइड",
    "guide-cta": "उपयोगकर्ता गाइड पढ़ें",
    "source-link": "स्रोत कोड",
    "theme-light": "हल्की थीम का उपयोग करें",
    "theme-dark": "गहरी थीम का उपयोग करें",
    "lang-label": "भाषा",
    tagline: "एक प्रोग्राम करने योग्य, स्क्रिप्ट करने योग्य कैलकुलेटर",
    lede: "एक गणना इंजन, चार तरीकों से उपयोग करें। व्यंजक टाइप करें, फ़ंक्शन और स्क्रिप्ट सहेजें, परिणामों के ग्राफ़ बनाएँ — और सब कुछ सत्रों के बीच बनाए रखें, छह भाषाओं में।",
    builds: "epher प्राप्त करें",
    "one-install": "एक डाउनलोड, epher का हर रूप: कमांड लाइन, REPL, टर्मिनल इंटरफ़ेस और डेस्कटॉप ऐप — सब एक ही epher निष्पादन फ़ाइल में।",
    "win-name": "Windows",
    "win-desc": "एक इंस्टॉलर। यह epher को आपके PATH पर रखता है — CMD, PowerShell, स्टार्ट मेनू या डबल-क्लिक से चलाएँ।",
    "win-download": "Windows इंस्टॉलर डाउनलोड करें",
    "mac-name": "macOS",
    "mac-desc": "एक ऐप। इसे Applications में खींचें; अंदर एक बटन आपके लिए epher टर्मिनल कमांड इंस्टॉल करता है।",
    "mac-download": "macOS के लिए डाउनलोड करें (Apple Silicon)",
    "linux-name": "Linux",
    "linux-desc": "हर पैकेज परिवार के लिए एक इंस्टॉल: Debian/Ubuntu, Fedora/RHEL, या बाकी सबके लिए AppImage (Arch समेत)। सभी epher को आपके PATH पर रखते हैं।",
    "linux-deb": "Debian/Ubuntu के लिए डाउनलोड करें (.deb)",
    "linux-rpm": "Fedora/RHEL के लिए डाउनलोड करें (.rpm)",
    "linux-appimage": "AppImage डाउनलोड करें (कोई भी distro, Arch समेत)",
    "pwa-name": "वेब ऐप",
    "pwa-desc": "ब्राउज़र में चलता है, इंस्टॉल हो सकता है, और पहली बार खोलने के बाद पूरी तरह ऑफ़लाइन काम करता है।",
    downloads: "डाउनलोड",
    get: "प्राप्त करें",
    "pwa-launch": "वेब ऐप खोलें",
    "offline-note": "लोड होते ही पूरी तरह ऑफ़लाइन काम करता है — इसे अपने ब्राउज़र मेनू से इंस्टॉल करें।",
    "footer-source": "GitHub पर स्रोत कोड",
    "footer-license": "epher निःशुल्क और ओपन सोर्स (MIT) है।",
    "footer-release": "डाउनलोड नवीनतम GitHub रिलीज़ से आते हैं।"
  },

  es: {
    "skip-link": "Saltar al contenido",
    "nav-label": "Principal",
    guide: "Guía de usuario",
    "guide-cta": "Leer la guía de usuario",
    "source-link": "Código fuente",
    "theme-light": "Usar tema claro",
    "theme-dark": "Usar tema oscuro",
    "lang-label": "Idioma",
    tagline: "Una calculadora programable y con scripts",
    lede: "Un motor de cálculo, cuatro formas de usarlo. Escribe expresiones, guarda funciones y scripts, dibuja tus resultados y conserva todo entre sesiones, en seis idiomas.",
    builds: "Obtén epher",
    "one-install": "Una descarga, todas las formas de usar epher: línea de comandos, REPL, interfaz de terminal y aplicación de escritorio — todo en el único ejecutable epher.",
    "win-name": "Windows",
    "win-desc": "Un instalador. Pone epher en tu PATH: úsalo desde CMD, PowerShell, el menú Inicio o con un doble clic.",
    "win-download": "Descargar el instalador de Windows",
    "mac-name": "macOS",
    "mac-desc": "Una aplicación. Arrástrala a Aplicaciones; un botón dentro instala el comando de terminal epher por ti.",
    "mac-download": "Descargar para macOS (Apple Silicon)",
    "linux-name": "Linux",
    "linux-desc": "Una instalación por familia de paquetes: Debian/Ubuntu, Fedora/RHEL o el AppImage para todo lo demás (incluido Arch). Todas ponen epher en tu PATH.",
    "linux-deb": "Descargar para Debian/Ubuntu (.deb)",
    "linux-rpm": "Descargar para Fedora/RHEL (.rpm)",
    "linux-appimage": "Descargar el AppImage (cualquier distro, incl. Arch)",
    "pwa-name": "Aplicación web",
    "pwa-desc": "Se ejecuta en tu navegador, es instalable y funciona totalmente sin conexión tras la primera visita.",
    downloads: "Descargas",
    get: "Obtener",
    "pwa-launch": "Abrir la aplicación web",
    "offline-note": "Funciona totalmente sin conexión una vez cargada: instálala desde el menú de tu navegador.",
    "footer-source": "Código fuente en GitHub",
    "footer-license": "epher es software libre y de código abierto (MIT).",
    "footer-release": "Las descargas provienen de la última versión de GitHub."
  },

  fr: {
    "skip-link": "Aller au contenu",
    "nav-label": "Navigation principale",
    guide: "Guide de l'utilisateur",
    "guide-cta": "Lire le guide de l'utilisateur",
    "source-link": "Code source",
    "theme-light": "Utiliser le thème clair",
    "theme-dark": "Utiliser le thème sombre",
    "lang-label": "Langue",
    tagline: "Une calculatrice programmable et scriptable",
    lede: "Un moteur de calcul, quatre façons de l'utiliser. Saisissez des expressions, enregistrez fonctions et scripts, tracez vos résultats, et conservez tout d'une session à l'autre, dans six langues.",
    builds: "Obtenir epher",
    "one-install": "Un seul téléchargement, toutes les façons d'utiliser epher : ligne de commande, REPL, interface de terminal et application de bureau — tout dans l'unique exécutable epher.",
    "win-name": "Windows",
    "win-desc": "Un seul installateur. Il ajoute epher à votre PATH — utilisez-le depuis CMD, PowerShell, le menu Démarrer ou un double-clic.",
    "win-download": "Télécharger l'installateur Windows",
    "mac-name": "macOS",
    "mac-desc": "Une seule application. Glissez-la dans Applications ; un bouton installe la commande terminal epher pour vous.",
    "mac-download": "Télécharger pour macOS (Apple Silicon)",
    "linux-name": "Linux",
    "linux-desc": "Une installation par famille de paquets : Debian/Ubuntu, Fedora/RHEL, ou l'AppImage pour tout le reste (Arch compris). Toutes ajoutent epher à votre PATH.",
    "linux-deb": "Télécharger pour Debian/Ubuntu (.deb)",
    "linux-rpm": "Télécharger pour Fedora/RHEL (.rpm)",
    "linux-appimage": "Télécharger l'AppImage (toute distro, Arch compris)",
    "pwa-name": "Application web",
    "pwa-desc": "Fonctionne dans votre navigateur, est installable et fonctionne entièrement hors ligne après la première visite.",
    downloads: "Téléchargements",
    get: "Obtenir",
    "pwa-launch": "Ouvrir l'application web",
    "offline-note": "Fonctionne entièrement hors ligne une fois chargée — installez-la depuis le menu de votre navigateur.",
    "footer-source": "Code source sur GitHub",
    "footer-license": "epher est un logiciel libre et open source (MIT).",
    "footer-release": "Les téléchargements proviennent de la dernière version GitHub."
  },

  ar: {
    "skip-link": "تخطَّ إلى المحتوى",
    "nav-label": "التنقل الرئيسي",
    guide: "دليل المستخدم",
    "guide-cta": "اقرأ دليل المستخدم",
    "source-link": "الكود المصدري",
    "theme-light": "استخدام المظهر الفاتح",
    "theme-dark": "استخدام المظهر الداكن",
    "lang-label": "اللغة",
    tagline: "آلة حاسبة قابلة للبرمجة والكتابة النصية",
    lede: "محرك حساب واحد، وأربع طرق لاستخدامه. اكتب التعابير، واحفظ الدوال والنصوص البرمجية، وارسم النتائج، واحتفظ بكل شيء بين الجلسات — بست لغات.",
    builds: "احصل على epher",
    "one-install": "تنزيل واحد، وكل طرق استخدام epher: سطر الأوامر وREPL وواجهة الطرفية وتطبيق سطح المكتب — كلها في ملف epher التنفيذي الواحد.",
    "win-name": "Windows",
    "win-desc": "مثبِّت واحد. يضع epher في PATH — استخدمه من CMD أو PowerShell أو قائمة ابدأ أو بنقرة مزدوجة.",
    "win-download": "تنزيل مثبِّت Windows",
    "mac-name": "macOS",
    "mac-desc": "تطبيق واحد. اسحبه إلى Applications؛ وزرٌ بداخله يثبِّت أمر epher الطرفي لك.",
    "mac-download": "تنزيل لنظام macOS (Apple Silicon)",
    "linux-name": "Linux",
    "linux-desc": "تثبيت واحد لكل عائلة حزم: Debian/Ubuntu وFedora/RHEL أو AppImage لكل ما عداها (بما فيها Arch). كلها تضع epher في PATH.",
    "linux-deb": "تنزيل Debian/Ubuntu (.deb)",
    "linux-rpm": "تنزيل Fedora/RHEL (.rpm)",
    "linux-appimage": "تنزيل AppImage (أي توزيعة، بما فيها Arch)",
    "pwa-name": "تطبيق الويب",
    "pwa-desc": "يعمل في متصفحك، ويمكن تثبيته، ويعمل دون اتصال بالكامل بعد أول زيارة.",
    downloads: "التنزيلات",
    get: "احصل عليه",
    "pwa-launch": "فتح تطبيق الويب",
    "offline-note": "يعمل دون اتصال بالكامل بعد تحميله — ثبِّته من قائمة المتصفح.",
    "footer-source": "الكود المصدري على GitHub",
    "footer-license": "epher برنامج مجاني ومفتوح المصدر (MIT).",
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
  const label = (MESSAGES[currentLang] || MESSAGES.en)[key];
  const toggle = document.getElementById("theme-toggle");
  if (label) toggle.setAttribute("aria-label", label);
  const hidden = toggle.querySelector(".visually-hidden");
  if (hidden && label) hidden.textContent = label;
  // The brand mark flips tile colors with the theme (the CSS content:url
  // rule handles Chrome/Firefox; this keeps the src right for Safari).
  const brand = document.getElementById("brand-icon");
  if (brand) brand.src = theme === "dark" ? "icon-light.svg?v=2" : "icon.svg?v=2";
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
}

document.addEventListener("DOMContentLoaded", init);
