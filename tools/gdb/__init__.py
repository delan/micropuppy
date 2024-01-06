import os
from pathlib import Path
import sys

# put the current directory onto the PYTHONPATH
file = Path(os.path.realpath(__file__))
sys.path.append(str(file.parent))

# === all real imports should be below this line ===
import info_tt
import qemu

BOLD = "\033[1m"
RESET = "\033[0m"
print(f"ao!! try {BOLD}help user{RESET} :3")
