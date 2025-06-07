import argparse
import shutil
from pathlib import Path
from collections import Counter
import json
from math import ceil
from PIL import Image, ImageDraw, ImageFont

FONTS_FOLDER_PATH = Path("applib/fonts/")
CHAR_RANGE = (32, 127)


def main():

    parser = argparse.ArgumentParser()
    parser.add_argument("--font", type=str, required=True)
    parser.add_argument("--name", type=str, required=True)
    parser.add_argument("--sizes", type=str, required=True)
    args = parser.parse_args()

    sizes_list = [int(s) for s in args.sizes.split(",")]

    chars = [chr(i) for i in range(*CHAR_RANGE)]

    family_path = FONTS_FOLDER_PATH / args.name
    shutil.rmtree(family_path, ignore_errors=True)

    for size in sizes_list:

        font = ImageFont.truetype(args.font, size)
        (asc, desc) = font.getmetrics()
        char_h = asc + desc

        char_w_set = set(font.getlength(c) for c in chars)
        assert len(char_w_set) == 1
        char_w = int(ceil(char_w_set.pop()))

        image = Image.new("L", (char_w * len(chars), char_h))

        draw = ImageDraw.Draw(image)

        for i, c in enumerate(chars):
            draw.text((i * char_w, 0), c, font=font, fill=255)

        output_path = family_path / f"{size}"
        output_path.mkdir(exist_ok=True, parents=True)

        image.save(output_path / "bitmap.png")

        spec = {
            "size": size,
            "nb_chars": len(chars),
            "char_h": char_h,
            "char_w": char_w,
            "base_y": asc,
        }

        with open(output_path / "spec.json", "w") as f:
            json.dump(spec, f, indent=2)

if __name__ == "__main__":
    main()
