import os, base64, requests

API_URL = "http://192.124.181.128:8045/v1/images/generations"
API_KEY = "sk-9565253724374c5db4f0bbec10720f80"
HEADERS = {"Authorization": f"Bearer {API_KEY}", "Content-Type": "application/json"}
ASSETS_DIR = "/home/m/bot tg godania/assets"

PROMPTS = {
    "tarot_spheres": "Ancient velvet tarot reading table with glowing mystic spheres of love, wealth and destiny, floating gold energy, esoteric candles, 8k cinematic",
    "card_of_the_day": "Magical single glowing golden tarot card floating above sacred crystal altar in starry galaxy nebula, glowing runes, photorealistic 8k"
}

for name, prompt in PROMPTS.items():
    out_path = os.path.join(ASSETS_DIR, f"{name}.jpg")
    print(f"Generating {name}...")
    r = requests.post(API_URL, headers=HEADERS, json={"prompt": prompt, "model": "gemini-3.1-flash-image", "n": 1}, timeout=90)
    if r.status_code == 200:
        b64 = r.json()["data"][0]["b64_json"]
        with open(out_path, "wb") as f:
            f.write(base64.b64decode(b64))
        print(f"Saved {name}.jpg")
    else:
        print(f"Err {name}: {r.status_code}")
