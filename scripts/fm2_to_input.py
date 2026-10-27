#!/usr/bin/env python3
"""Convert an FM2 power-on, two-standard-controller movie into nes-emulator inputs."""
import argparse
from pathlib import Path


def convert(text, offset):
    frame = 0
    previous = None
    output = ['# FRAME P1_HEX P2_HEX; converted from FM2']
    for line in text.splitlines():
        if line.startswith(('savestate ', 'binary 1', 'fourscore 1', 'palFlag 1')):
            raise ValueError('Only text, power-on FM2 movies with two standard controllers are supported')
        if line.startswith(('port0 ', 'port1 ')) and line.split()[1] != '1':
            raise ValueError('Only standard controllers are supported')
        if not line.startswith('|'):
            continue
        columns = line.split('|')
        if len(columns) < 4 or (int(columns[1]) != 0 and not (frame == 0 and int(columns[1]) in (1, 2))):
            raise ValueError(f'Unsupported FM2 command at frame {frame}')
        buttons = []
        for column in columns[2:4]:
            if len(column) != 8:
                raise ValueError(f'Invalid controller input at frame {frame}')
            buttons.append(sum(1 << (7 - bit) for bit, value in enumerate(column) if value not in '. '))
        at = frame + offset
        if at >= 0 and buttons != previous:
            output.append(f'{at} {buttons[0]:02x} {buttons[1]:02x}')
            previous = buttons
        frame += 1
    if frame == 0:
        raise ValueError('No movie frames found')
    return '\n'.join(output) + '\n'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('movie', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--offset', type=int, default=0, help='Adjust input sampling boundary, in frames')
    args = parser.parse_args()
    try:
        args.output.write_text(convert(args.movie.read_text(), args.offset))
    except (OSError, ValueError) as error:
        parser.exit(1, f'{error}\n')


if __name__ == '__main__':
    main()
