#!/usr/bin/env python3
"""下载固定版本的公开硬件测试 ROM，供 Rust 集成测试直接使用。"""
import argparse
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
import urllib.request


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('rom_directory', type=Path)
    args = parser.parse_args()
    manifest = Path(__file__).resolve().parents[1] / 'nes-test/tests/hardware_roms.txt'
    header, *roms = manifest.read_text().splitlines()
    revision = header.removeprefix('# revision: ')

    def fetch(relative):
        url = f'https://raw.githubusercontent.com/christopherpow/nes-test-roms/{revision}/{relative}'
        with urllib.request.urlopen(url, timeout=30) as response:
            data = response.read()
        path = args.rom_directory / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)

    with ThreadPoolExecutor(max_workers=4) as pool:
        list(pool.map(fetch, roms))
    print(f'已下载 {len(roms)} 项硬件测试 ROM，版本 {revision}')


if __name__ == '__main__':
    main()
