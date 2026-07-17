@echo off
call C:\BuildTools\VC\Auxiliary\Build\vcvars64.bat
set "LIB=%LIB%;C:\Program Files (x86)\Windows Kits\10\Lib\10.0.26100.0\um\x64"
"%USERPROFILE%\.cargo\bin\cargo.exe" clippy --manifest-path src-tauri\Cargo.toml --all-targets -- -D warnings
