"""Write the BMP that `measure_preview.py` reads from one of this record's lossless PNGs.

    python3 -I png_to_bmp.py top.pixels.png top.bmp

Standard library only.  It decodes an 8-bit RGB, non-interlaced PNG and writes
a 24-bit top-down BMP with the 54-byte header `sips -s format bmp` wrote for
the measured files, so the output's SHA-256 can be compared with the one in
README.md.  It prints that SHA-256.
"""
import hashlib
import struct
import sys
import zlib


def decode(data):
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        raise SystemExit("not a PNG")
    pos, idat, head = 8, [], None
    while pos < len(data):
        size, kind = struct.unpack_from(">I4s", data, pos)
        body = data[pos + 8:pos + 8 + size]
        if kind == b"IHDR":
            head = struct.unpack(">IIBBBBB", body)
        elif kind == b"IDAT":
            idat.append(body)
        pos += 12 + size
    width, height, depth, colour, _, _, interlace = head
    if (depth, colour, interlace) != (8, 2, 0):
        raise SystemExit("only 8-bit RGB, non-interlaced PNGs are handled")
    raw = zlib.decompress(b"".join(idat))
    stride = width * 3
    prev, rows = bytearray(stride), []
    for y in range(height):
        kind = raw[y * (stride + 1)]
        row = bytearray(raw[y * (stride + 1) + 1:(y + 1) * (stride + 1)])
        if kind == 1:
            for i in range(3, stride):
                row[i] = (row[i] + row[i - 3]) & 255
        elif kind == 2:
            for i in range(stride):
                row[i] = (row[i] + prev[i]) & 255
        elif kind == 3:
            for i in range(stride):
                row[i] = (row[i] + (((row[i - 3] if i >= 3 else 0) + prev[i]) >> 1)) & 255
        elif kind == 4:
            for i in range(stride):
                a = row[i - 3] if i >= 3 else 0
                b = prev[i]
                c = prev[i - 3] if i >= 3 else 0
                pa, pb, pc = abs(b - c), abs(a - c), abs(a + b - 2 * c)
                row[i] = (row[i] + (a if pa <= pb and pa <= pc else b if pb <= pc else c)) & 255
        elif kind != 0:
            raise SystemExit(f"row {y}: unknown filter {kind}")
        rows.append(row)
        prev = row
    return width, height, rows


def main(png, bmp):
    with open(png, "rb") as f:
        width, height, rows = decode(f.read())
    pad = b"\0" * (-(width * 3) % 4)
    pixels = bytearray()
    for row in rows:
        row[0::3], row[2::3] = row[2::3], row[0::3]  # RGB to BGR
        pixels += row + pad
    header = struct.pack("<2sIHHIIiiHHIIiiII", b"BM", 54 + len(pixels), 0, 0, 54,
                         40, width, -height, 1, 24, 0, len(pixels), 0, 0, 0, 0)
    out = header + bytes(pixels)
    with open(bmp, "wb") as f:
        f.write(out)
    print(hashlib.sha256(out).hexdigest(), bmp)


if __name__ == "__main__":
    main(*sys.argv[1:])
