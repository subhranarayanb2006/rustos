import base64
import os

iso_path = os.path.join(os.path.dirname(__file__), "..", "huggingface", "rustos.iso")
js_path = os.path.join(os.path.dirname(__file__), "..", "huggingface", "rustos_iso.js")

if not os.path.exists(iso_path):
    iso_path = os.path.join(os.path.dirname(__file__), "..", "deliverables", "rustos.iso")

print(f"Reading ISO from {iso_path}...")
with open(iso_path, "rb") as f:
    raw_data = f.read()

print(f"ISO raw size: {len(raw_data)} bytes")
b64 = base64.b64encode(raw_data).decode("ascii")

with open(js_path, "w", encoding="utf-8") as f:
    f.write("// RustOS Bootable ISO Embedded JS Bundle\n")
    f.write('window.RUSTOS_ISO_BASE64 = "')
    f.write(b64)
    f.write('";\n')

size_js = os.path.getsize(js_path)
print(f"Generated {js_path} successfully ({size_js} bytes, {size_js / (1024*1024):.2f} MB)")
