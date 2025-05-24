# build user applications by different linker.ld which has different base_address


import os

base_address: int = 0x80400000
step: int = 0x20000  # max application size
linker: str = "src/linker.ld"

app_id: int = 0
apps: list[str] = os.listdir("src/bin")
apps.sort()

for app in apps:
    app = app[: app.find(".")]  # remove .rs
    lines: list[str] = []
    lines_before: list[str] = []

    with open(linker, "r") as f:
        for line in f.readlines():
            lines_before.append(line)
            line = line.replace(hex(base_address), hex(base_address + step * app_id))
            lines.append(line)

    # generate app specific linker.ld
    with open(linker, "w+") as f:
        f.writelines(lines)
    os.system(f"cargo build --bin {app} --release")
    print(
        f"[build.py] applications {app} start with address {hex(base_address + step * app_id)}"
    )

    # restore linker.ld
    with open(linker, "w+") as f:
        f.writelines(lines_before)

    app_id = app_id + 1
