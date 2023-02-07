Remove-Item -Recurse -Force ./pkg
& wasm-pack build --dev --scope artentus
& python.exe patch.py
