"""Preview board 10 manifest; see zener_boards.py.

Usage: python3 -I tools/frame_boards/preview_board_10.py OUT.json
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import zener_boards

if __name__ == "__main__":
    zener_boards.main("preview_board_10", sys.argv)
