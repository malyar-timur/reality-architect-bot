import os
import json
import base64
import requests

API_URL = "http://192.124.181.128:8045/v1/images/generations"
API_KEY = "sk-9565253724374c5db4f0bbec10720f80"
HEADERS = {
    "Authorization": f"Bearer {API_KEY}",
    "Content-Type": "application/json"
}

ASSETS_DIR = "/home/m/bot tg godania/assets"
os.makedirs(ASSETS_DIR, exist_ok=True)

PROMPTS = {
    "tariffs": (
        "Mystical crystal prism pyramid with glowing violet, emerald and golden energy beams, "
        "sacred geometric circles, glowing tarot talismans and golden coins floating in deep velvet cosmic space, "
        "premium luxury aesthetic, dark fantasy, highly detailed, photorealistic 8k"
    ),
    "main_menu": (
        "Ancient occult library with a glowing celestial sphere and sacred altar, "
        "floating golden constellation runes, soft ethereal violet and gold candlelight, "
        "mystical reality architect temple, deep atmosphere, cinematic lighting, 8k"
    ),
    "astrology": (
        "Celestial cosmic zodiac wheel glowing in starry nebula space, 12 constellations radiating golden light, "
        "astrology celestial compass, sacred geometry, ultra high quality cinematic 8k"
    ),
    "leela": (
        "Ancient sacred game board of Leela Gyan Chauper, glowing mystical pathways, "
        "cosmic snakes and divine ladders made of starlight, golden dice floating with magical aura, 8k"
    )
}

def generate_image(name, prompt):
    out_path = os.path.join(ASSETS_DIR, f"{name}.jpg")
    print(f"Generating {name}...")
    payload = {
        "prompt": prompt,
        "model": "gemini-3.1-flash-image",
        "n": 1
    }
    try:
        r = requests.post(API_URL, headers=HEADERS, json=payload, timeout=90)
        if r.status_code == 200:
            data = r.json()
            b64 = data["data"][0]["b64_json"]
            img_bytes = base64.b64decode(b64)
            with open(out_path, "wb") as f:
                f.write(img_bytes)
            print(f"Saved {name}.jpg ({len(img_bytes)} bytes)")
        else:
            print(f"Error {name}: {r.status_code} {r.text}")
    except Exception as e:
        print(f"Failed {name}: {e}")

if __name__ == "__main__":
    for name, prompt in PROMPTS.items():
        generate_image(name, prompt)
