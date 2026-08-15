; nsis-check.nsi — compile-only harness for nsis-hooks.nsh (ADR-0011).
;
; Verified with `makensis nsis-check.nsi` (Linux ships makensis; no wine
; needed — this checks syntax, macros, labels, and includes). The string
; logic itself is hand-traced in the hooks file; keep this harness in sync
; so the hook file always compiles against the same NSIS feature set the
; Tauri installer uses (LogicLib, no plugins).
;
; Run from the repo: makensis crates/tauri-app/src-tauri/nsis-check.nsi

!include "LogicLib.nsh"
OutFile "nsis-check.exe"
Name "epher nsis check"

!include "nsis-hooks.nsh"

Section
  ; Exercise both hook macros the installer would expand (they reference
  ; $INSTDIR, the HKCU Environment key, and both helpers).
  !insertmacro NSIS_HOOK_POSTINSTALL
  !insertmacro NSIS_HOOK_POSTUNINSTALL

  ; Direct helper calls with representative stack usage.
  Push "C:\tools;C:\Program Files\epher;C:\bin"
  Push "C:\Program Files\epher"
  Call epher_path_contains
  Pop $0

  Push "C:\tools;C:\Program Files\epher;C:\bin"
  Push "C:\Program Files\epher"
  Call epher_remove_from_path
  Pop $1
SectionEnd
