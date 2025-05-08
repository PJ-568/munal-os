import argparse
from pathlib import Path
import json
from math import ceil
from PIL import Image, ImageDraw, ImageFont

FONTS_FOLDER_PATH = Path("applib/fonts/")
CHAR_RANGE = (32, 127)


def main():

    parser = argparse.ArgumentParser()
    parser.add_argument("--font", type=str, required=True)
    parser.add_argument("--name", type=str, required=True)
    parser.add_argument("--size", type=int, required=True)
    args = parser.parse_args()

    chars = [chr(i) for i in range(*CHAR_RANGE)]

    font = ImageFont.truetype(args.font, args.size)
    (asc, desc) = font.getmetrics()
    char_h = asc + desc

    char_w_set = set(font.getlength(c) for c in chars)
    assert len(char_w_set) == 1
    char_w = int(ceil(char_w_set.pop()))

    image = Image.new("L", (char_w * len(chars), char_h))

    draw = ImageDraw.Draw(image)

    for i, c in enumerate(chars):
        draw.text((i * char_w, 0), c, font=font, fill=255)

    output_path = FONTS_FOLDER_PATH / args.name / f"{args.size}"
    output_path.mkdir(exist_ok=True, parents=True)

    image.save(output_path / "bitmap.png")

    spec = {
        "nb_chars": len(chars),
        "char_h": char_h,
        "char_w": char_w,
        "base_y": asc,
    }

    with open(output_path / "spec.json", "w") as f:
        json.dump(spec, f, indent=2)

if __name__ == "__main__":
    main()
