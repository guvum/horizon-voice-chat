#!/usr/bin/env python3
"""The mod's icons, drawn here pixel by pixel (MIT, nothing of Simple Voice Chat's).

    python tools/make_icons.py [--out DIR] [--rust FILE]

Writes `assets/voicechat/textures/gui/icons/<name>.png` (16x16 RGBA) and
`mixins/voicechat/engine/src/icons.rs`, the same pixels compiled into the mod so it
draws its icons even where the game has no content mod textures. Plain Python (zlib),
no dates in the files: the same grids always give the same bytes.
"""
import struct
import sys
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

PALETTE = {
    'K': 0xFF202020,  # outline
    'W': 0xFFF0F0F0,  # white
    'G': 0xFFB0B0B0,  # light grey
    'D': 0xFF7A7A7A,  # dark grey
    'R': 0xFFE04040,  # red
    'B': 0xFF60D0FF,  # whisper blue
    'Y': 0xFFE8B830,  # gold
    'L': 0xFF50C850,  # green
}

MIC = [
    '................',
    '......KKKK......',
    '.....KWWWGK.....',
    '.....KWGGDK.....',
    '.....KWGGDK.....',
    '..KK.KWGGDK.KK..',
    '..KWKKWGGDKKWK..',
    '..KWKKGGDDKKWK..',
    '..KWWKKKKKKWWK..',
    '...KWWWWWWWWK...',
    '....KKKWWKKK....',
    '......KWWK......',
    '......KWWK......',
    '...KKKKWWKKKK...',
    '...KWWWWWWWWK...',
    '...KKKKKKKKKK...',
]

SPEAKER_BODY = [
    '................',
    '.......K........',
    '......KK........',
    '.....KWK........',
    '....KWWK........',
    'KKKKWWWK........',
    'KWWWWWWK........',
    'KWWWWWWK........',
    'KWWWWWWK........',
    'KWWWWWWK........',
    'KKKKWWWK........',
    '....KWWK........',
    '.....KWK........',
    '......KK........',
    '.......K........',
    '................',
]

# The sound coming out of it: an inner, a middle and an outer wave, as (row, column).
WAVE_INNER = [(5, 9), (6, 10), (7, 10), (8, 10), (9, 10), (10, 9)]
WAVE_MIDDLE = [(3, 11), (4, 12), (5, 12), (6, 13), (7, 13), (8, 13), (9, 13), (10, 12), (11, 12), (12, 11)]
WAVE_OUTER = [(1, 13), (2, 14), (3, 14)] + [(r, 15) for r in range(4, 12)] + [(12, 14), (13, 14), (14, 13)]


def waves(colour, *arcs):
    grid = [['.'] * 16 for _ in range(16)]
    for arc in arcs:
        for r, c in arc:
            grid[r][c] = colour
    return [''.join(row) for row in grid]


WAVES = waves('W', WAVE_INNER, WAVE_MIDDLE, WAVE_OUTER)

CROSS = [
    '................',
    '................',
    '................',
    '................',
    '................',
    '................',
    '..........RR..RR',
    '...........RRRR.',
    '............RR..',
    '...........RRRR.',
    '..........RR..RR',
    '................',
    '................',
    '................',
    '................',
    '................',
]

DISCONNECTED = [
    '................',
    '.....R....R.....',
    '......R..R......',
    '................',
    '.KKKKK....KKKKK.',
    'KGGGGGK..KGGGGGK',
    'KGKKKGK..KGKKKGK',
    'KGK.KGK..KGK.KGK',
    'KGKKKGK..KGKKKGK',
    'KGGGGGK..KGGGGGK',
    '.KKKKK....KKKKK.',
    '................',
    '......R..R......',
    '.....R....R.....',
    '................',
    '................',
]

GROUP = [
    '................',
    '..........KKKK..',
    '.........KGGGGK.',
    '...KKKK..KGGGGK.',
    '..KWWWWK.KGGGGK.',
    '..KWWWWK.KGGGGK.',
    '..KWWWWK..KKKK..',
    '..KWWWWK.KKKKKK.',
    '...KKKK.KGGGGGGK',
    '.KKKKKKKKGGGGGGK',
    'KWWWWWWWWKGGGGGK',
    'KWWWWWWWWKGGGGGK',
    'KWWWWWWWWKKKKKKK',
    'KWWWWWWWWK......',
    'KKKKKKKKKK......',
    '................',
]

SETTINGS = [
    '................',
    '......KKKK......',
    '..KK..KGGK..KK..',
    '..KGKKKGGKKKGK..',
    '...KGGGGGGGGK...',
    '...KGGGKKGGGK...',
    'KKKKGGK..KGGKKKK',
    'KGGGGK....KGGGGK',
    'KGGGGK....KGGGGK',
    'KKKKGGK..KGGKKKK',
    '...KGGGKKGGGK...',
    '...KGGGGGGGGK...',
    '..KGKKKGGKKKGK..',
    '..KK..KGGK..KK..',
    '......KKKK......',
    '................',
]

LOCK = [
    '................',
    '................',
    '.....KKKKKK.....',
    '....KDKKKKDK....',
    '....KDK..KDK....',
    '....KDK..KDK....',
    '...KKKKKKKKKK...',
    '...KYYYYYYYYK...',
    '...KYYYYYYYYK...',
    '...KYYYKKYYYK...',
    '...KYYYKKYYYK...',
    '...KYYYYYYYYK...',
    '...KKKKKKKKKK...',
    '................',
    '................',
    '................',
]

VOLUMES = [
    '................',
    '....KKK.........',
    '.KKKKWKKKKKKKKK.',
    '.KDDKWKDDDDDDDK.',
    '.KKKKWKKKKKKKKK.',
    '....KKK.........',
    '..........KKK...',
    '.KKKKKKKKKKWKKK.',
    '.KDDDDDDDDKWKDK.',
    '.KKKKKKKKKKWKKK.',
    '..........KKK...',
    '.......KKK......',
    '.KKKKKKKWKKKKKK.',
    '.KDDDDDKWKDDDDK.',
    '.KKKKKKKWKKKKKK.',
    '.......KKK......',
]

EYE = [
    '................',
    '................',
    '................',
    '.....KKKKKK.....',
    '...KKWWWWWWKK...',
    '..KWWWKKKKWWWK..',
    '.KWWWKDDDDKWWWK.',
    'KWWWWKDKKDKWWWWK',
    'KWWWWKDKKDKWWWWK',
    '.KWWWKDDDDKWWWK.',
    '..KWWWKKKKWWWK..',
    '...KKWWWWWWKK...',
    '.....KKKKKK.....',
    '................',
    '................',
    '................',
]

LEAVE = [
    '................',
    '.KKKKKKKK.......',
    '.KGGGGGGK.......',
    '.KGDDDDGK.......',
    '.KGD....K..K....',
    '.KGD.......KK...',
    '.KGD..KKKKKKRK..',
    '.KGD..KRRRRRRRK.',
    '.KGD..KRRRRRRRK.',
    '.KGD..KKKKKKRK..',
    '.KGD.......KK...',
    '.KGD....K..K....',
    '.KGDDDDGK.......',
    '.KGGGGGGK.......',
    '.KKKKKKKK.......',
    '................',
]

HUD = [
    '................',
    'KKKKKKKKKKKKKKKK',
    'KDDDDDDDDDDDDDDK',
    'KDWWDWWWWDDDDDDK',
    'KDDDDDDDDDDDDDDK',
    'KDWWDWWWDDDDDDDK',
    'KDDDDDDDDDDDDDDK',
    'KDWWDWWWWWDDDDDK',
    'KDDDDDDDDDDDDDDK',
    'KDDDDDDDDDDDDDDK',
    'KDDDDDDDDDDDDDDK',
    'KDDDDDDDDDDDDDDK',
    'KKKKKKKKKKKKKKKK',
    '......KKKK......',
    '....KKKKKKKK....',
    '................',
]

CHECK = [
    '................',
    '................',
    '.............KK.',
    '............KLLK',
    '...........KLLK.',
    '..........KLLK..',
    '.KK......KLLK...',
    'KLLK....KLLK....',
    '.KLLK..KLLK.....',
    '..KLLKKLLK......',
    '...KLLLLK.......',
    '....KLLK........',
    '.....KK.........',
    '................',
    '................',
    '................',
]

SEARCH = [
    '................',
    '...KKKKK........',
    '..KGGGGGK.......',
    '.KGWWGGGGK......',
    '.KGWGGGGGK......',
    '.KGGGGGGGK......',
    '.KGGGGGGGK......',
    '.KGGGGGGGK......',
    '..KGGGGGK.......',
    '...KKKKKKK......',
    '.........KKK....',
    '..........KDK...',
    '...........KDK..',
    '............KDK.',
    '.............KK.',
    '................',
]


def over(*layers):
    """Layers drawn over each other: a later layer's pixels win, '.' lets through."""
    out = [list(row) for row in layers[0]]
    for layer in layers[1:]:
        for y, row in enumerate(layer):
            for x, c in enumerate(row):
                if c != '.':
                    out[y][x] = c
    return [''.join(r) for r in out]


def slash(grid):
    """A red bar from the top right to the bottom left, as over a muted microphone."""
    bar = [['.'] * 16 for _ in range(16)]
    for y in range(1, 15):
        for x in (14 - y, 15 - y):
            bar[y][x] = 'R'
    return over(grid, [''.join(r) for r in bar])


def whisper_waves():
    """The inner and middle waves only, in blue: a quieter voice."""
    return waves('B', WAVE_INNER, WAVE_MIDDLE)


# Name (the PNG's and `skin::Icon::file`), Rust variant, pixels.
ICONS = [
    ('microphone', 'Mic', MIC),
    ('microphone_off', 'MicOff', slash(MIC)),
    ('speaker', 'Speaker', over(SPEAKER_BODY, WAVES)),
    ('speaker_off', 'SpeakerOff', over(SPEAKER_BODY, CROSS)),
    ('whisper', 'Whisper', over(SPEAKER_BODY, whisper_waves())),
    ('disconnected', 'Disconnected', DISCONNECTED),
    ('group', 'Group', GROUP),
    ('settings', 'Settings', SETTINGS),
    ('lock', 'Lock', LOCK),
    ('volumes', 'Volumes', VOLUMES),
    ('eye', 'Eye', EYE),
    ('eye_off', 'EyeOff', slash(EYE)),
    ('leave', 'Leave', LEAVE),
    ('group_hud', 'GroupHud', HUD),
    ('group_hud_off', 'GroupHudOff', slash(HUD)),
    ('check', 'Check', CHECK),
    ('search', 'Search', SEARCH),
]


def check(name, grid):
    if len(grid) != 16 or any(len(r) != 16 for r in grid):
        raise SystemExit(f'{name}: not 16x16 ({[len(r) for r in grid]})')
    bad = {c for r in grid for c in r} - set(PALETTE) - {'.'}
    if bad:
        raise SystemExit(f'{name}: unknown colours {bad}')


def png(grid) -> bytes:
    raw = b''
    for row in grid:
        raw += b'\x00'  # filter: none
        for c in row:
            argb = PALETTE.get(c, 0)
            raw += struct.pack('>BBBB', (argb >> 16) & 255, (argb >> 8) & 255, argb & 255, (argb >> 24) & 255)

    def chunk(kind, data):
        return struct.pack('>I', len(data)) + kind + data + struct.pack('>I', zlib.crc32(kind + data) & 0xFFFFFFFF)

    ihdr = struct.pack('>IIBBBBB', 16, 16, 8, 6, 0, 0, 0)
    return b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', ihdr) + chunk(b'IDAT', zlib.compress(raw, 9)) + chunk(b'IEND', b'')


RUST_HEAD = '''// Generated by tools/make_icons.py (the same pixels as the PNGs in
// assets/voicechat/textures/gui/icons/): edit the grids there, not here.
//
// The icons pixel by pixel, for drawing them with fills where the game has no
// sprite for them (no content mod textures).
use crate::skin::Icon;
use crate::ui::Draw;

fn colour(c: u8) -> u32 {
    match c {
{palette}
        _ => 0,
    }
}

fn rows(i: Icon) -> [&'static str; 16] {
    match i {
{icons}
    }
}

/// Part `src` of icon `i` (in its 16x16 pixels) drawn into `dst` with
/// fills, one per run of a colour along a row.
pub fn fills(i: Icon, src: [f32; 4], dst: [f32; 4]) -> Vec<Draw> {
    let (sx, sy) = (dst[2] / src[2].max(1.0), dst[3] / src[3].max(1.0));
    let (x0, y0) = (src[0].max(0.0) as usize, src[1].max(0.0) as usize);
    let (x1, y1) = (((src[0] + src[2]) as usize).min(16), ((src[1] + src[3]) as usize).min(16));
    let mut out = Vec::new();
    for (y, row) in rows(i).iter().enumerate().take(y1).skip(y0) {
        let row = row.as_bytes();
        let mut x = x0;
        while x < x1 {
            let c = row[x];
            let mut end = x + 1;
            while end < x1 && row[end] == c {
                end += 1;
            }
            let argb = colour(c);
            if argb != 0 {
                let px = dst[0] + (x - x0) as f32 * sx;
                let py = dst[1] + (y - y0) as f32 * sy;
                out.push(Draw::Fill([px, py, (end - x) as f32 * sx, sy], argb));
            }
            x = end;
        }
    }
    out
}
'''


def rust() -> str:
    palette = '\n'.join(f"        b'{k}' => 0x{v:08X}," for k, v in PALETTE.items())
    icons = []
    for _, variant, grid in ICONS:
        lines = ''.join(f'            "{row}",\n' for row in grid)
        icons.append(f'        Icon::{variant} => [\n{lines}        ],')
    return RUST_HEAD.replace('{palette}', palette).replace('{icons}', '\n'.join(icons))


def main(argv):
    out = ROOT / 'assets' / 'voicechat' / 'textures' / 'gui' / 'icons'
    rust_file = ROOT / 'mixins' / 'voicechat' / 'engine' / 'src' / 'icons.rs'
    args = list(argv)
    if '--out' in args:
        out = Path(args[args.index('--out') + 1])
        rust_file = None
    if '--rust' in args:
        rust_file = Path(args[args.index('--rust') + 1])
    out.mkdir(parents=True, exist_ok=True)
    for name, _, grid in ICONS:
        check(name, grid)
        (out / f'{name}.png').write_bytes(png(grid))
    if rust_file:
        rust_file.write_bytes(rust().encode())
    print(f'{len(ICONS)} icons in {out}' + (f', pixels in {rust_file}' if rust_file else ''))


if __name__ == '__main__':
    main(sys.argv[1:])
