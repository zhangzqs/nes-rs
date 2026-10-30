#!/usr/bin/env python3
"""Run a pinned, bounded NES hardware regression suite; no game ROMs are fetched."""
import argparse
import concurrent.futures
import json
from pathlib import Path
import subprocess
import tempfile
import urllib.request

REVISION = '95d8f621ae55cee0d09b91519a8989ae0e64753b'
SUITES = {
    'instr_test-v5/rom_singles': [
        '01-basics', '02-implied', '03-immediate', '04-zero_page', '05-zp_xy',
        '06-absolute', '07-abs_xy', '08-ind_x', '09-ind_y', '10-branches',
        '11-stack', '12-jmp_jsr', '13-rts', '14-rti', '15-brk', '16-special'],
    'instr_timing/rom_singles': ['1-instr_timing', '2-branch_timing'],
    'apu_test/rom_singles': ['1-len_ctr', '2-len_table', '3-irq_flag',
        '4-jitter', '5-len_timing', '6-irq_flag_timing', '7-dmc_basics', '8-dmc_rates'],
    'ppu_vbl_nmi/rom_singles': ['01-vbl_basics', '02-vbl_set_time', '03-vbl_clear_time',
        '04-nmi_control', '05-nmi_timing', '06-suppression', '07-nmi_on_timing',
        '08-nmi_off_timing', '09-even_odd_frames', '10-even_odd_timing'],
    # The alternate (old NEC) MMC3 test asserts the opposite of Sharp MMC3 behavior.
    'mmc3_test_2/rom_singles': ['1-clocking', '2-details', '3-A12_clocking',
        '4-scanline_timing', '5-MMC3'],
    'sprite_hit_tests_2005.10.05': ['01.basics', '02.alignment', '03.corners',
        '04.flip', '05.left_clip', '06.right_edge', '07.screen_bottom',
        '08.double_height', '09.timing_basics', '10.timing_order', '11.edge_timing'],
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('rom_directory', type=Path)
    parser.add_argument('--fetch', action='store_true', help='Fetch the selected tests at the pinned revision')
    parser.add_argument('--binary', type=Path, default=Path('target/release/nes-emulator'))
    parser.add_argument('--jobs', type=int, default=4)
    parser.add_argument('--report', type=Path)
    args = parser.parse_args()
    if args.jobs < 1:
        parser.error('--jobs must be positive')
    binary = args.binary.resolve()
    tests = [f'{suite}/{name}.nes' for suite, names in SUITES.items() for name in names]

    def run(relative):
        try:
            rom = args.rom_directory / relative
            if args.fetch:
                url = f'https://raw.githubusercontent.com/christopherpow/nes-test-roms/{REVISION}/{relative}'
                data = urllib.request.urlopen(url, timeout=30).read()
                rom.parent.mkdir(parents=True, exist_ok=True)
                rom.write_bytes(data)
            if not rom.is_file():
                raise FileNotFoundError(rom)
            sprite = relative.startswith('sprite_hit_')
            with tempfile.TemporaryDirectory(prefix='nes-test-') as temporary:
                ram = Path(temporary) / 'ram.bin'
                command = [str(binary), str(rom), '--headless', '--frames', '300' if sprite else '3000']
                command += ['--ram-dump', str(ram)] if sprite else ['--test-rom']
                result = subprocess.run(command, capture_output=True, text=True, timeout=180)
                passed = result.returncode == 0
                if sprite:
                    passed = passed and ram.read_bytes()[0xf8] == 1
                detail = (result.stdout + result.stderr).strip()
                return {'rom': relative, 'passed': passed, 'output': detail}
        except (OSError, subprocess.TimeoutExpired) as error:
            return {'rom': relative, 'passed': False, 'output': str(error)}

    with concurrent.futures.ThreadPoolExecutor(max_workers=args.jobs) as pool:
        results = list(pool.map(run, tests))
    for result in results:
        print(('PASS ' if result['passed'] else 'FAIL ') + result['rom'])
        if not result['passed']:
            print(result['output'])
    passed = sum(result['passed'] for result in results)
    print(f'{passed}/{len(results)} passed; test revision {REVISION}')
    if args.report:
        args.report.write_text(json.dumps({'revision': REVISION, 'results': results}, indent=2) + '\n')
    return 0 if passed == len(results) else 1


if __name__ == '__main__':
    raise SystemExit(main())
