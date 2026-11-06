"""python -m unittest tools/test_make_icons.py"""
import struct
import subprocess
import sys
import tempfile
import unittest
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / 'tools' / 'make_icons.py'
NAMES = {'microphone', 'microphone_off', 'speaker', 'speaker_off', 'whisper', 'disconnected', 'group',
         'settings', 'lock', 'volumes', 'eye', 'eye_off', 'leave', 'group_hud', 'group_hud_off', 'check',
         'search'}


def run(out: Path, rust: Path):
    subprocess.run([sys.executable, str(SCRIPT), '--out', str(out), '--rust', str(rust)], check=True,
                   capture_output=True)


class MakeIcons(unittest.TestCase):
    def test_the_same_bytes_every_time(self):
        with tempfile.TemporaryDirectory() as a, tempfile.TemporaryDirectory() as b:
            run(Path(a), Path(a) / 'icons.rs')
            run(Path(b), Path(b) / 'icons.rs')
            files = sorted(p.name for p in Path(a).glob('*.png'))
            self.assertEqual({f.removesuffix('.png') for f in files}, NAMES)
            for f in files + ['icons.rs']:
                self.assertEqual((Path(a) / f).read_bytes(), (Path(b) / f).read_bytes(), f)

    def test_16x16_rgba(self):
        with tempfile.TemporaryDirectory() as d:
            run(Path(d), Path(d) / 'icons.rs')
            for p in Path(d).glob('*.png'):
                data = p.read_bytes()
                self.assertEqual(data[:8], b'\x89PNG\r\n\x1a\n')
                w, h, depth, kind = struct.unpack('>IIBB', data[16:26])
                self.assertEqual((w, h, depth, kind), (16, 16, 8, 6), p.name)
                idat = data[data.index(b'IDAT') + 4:data.index(b'IEND') - 8]
                self.assertEqual(len(zlib.decompress(idat)), 16 * (1 + 16 * 4))

    def test_the_committed_files_are_current(self):
        with tempfile.TemporaryDirectory() as d:
            run(Path(d), Path(d) / 'icons.rs')
            committed = ROOT / 'assets' / 'voicechat' / 'textures' / 'gui' / 'icons'
            for p in Path(d).glob('*.png'):
                self.assertEqual(p.read_bytes(), (committed / p.name).read_bytes(), p.name)
            rust = ROOT / 'mixins' / 'voicechat' / 'engine' / 'src' / 'icons.rs'
            self.assertEqual((Path(d) / 'icons.rs').read_bytes(), rust.read_bytes())


if __name__ == '__main__':
    unittest.main()
