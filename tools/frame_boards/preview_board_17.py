"""Preview board 17 manifest; see sot23_boards.py.

Usage: python3 -I tools/frame_boards/preview_board_17.py OUT.json
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import sot23_boards

if __name__ == "__main__":
    sot23_boards.main("preview_board_17", sys.argv)
