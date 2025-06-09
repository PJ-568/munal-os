import json

def main():

    with open("credits.json", "r") as f:
        icon_credits = json.load(f)

    f = open("README.md", "w")
    f.write("| Icon | Link | Author |\n")
    f.write("| --- | --- | --- |\n")
    for filename, source in icon_credits.items():
        txt = "|".join([
            f"![{filename}](png/{filename})",
            f"[{source['link']}]({source['link']})",
            source["author"],
        ])
        f.write(f"|{txt}|\n")

    f.close()

if __name__ == "__main__":
    main()
