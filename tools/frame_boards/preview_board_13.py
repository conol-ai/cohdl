"""Preview board 13 manifest; see w3_boards_13_14.py.

Usage: python3 -I tools/frame_boards/preview_board_13.py OUT.json
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import w3_boards_13_14

if __name__ == "__main__":
    w3_boards_13_14.main("preview_board_13", sys.argv)
