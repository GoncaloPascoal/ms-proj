
import argparse
import json
import os

from statistics import statistics_figure

def main():
    parser = argparse.ArgumentParser(description="Simulation Statistics")
    parser.add_argument(dest="f", type=str, help="Name of the file containing the simulation data")
    args = parser.parse_args()

    file = args.f
    if not os.path.exists(file) or not os.path.isfile(file):
        print('File doesn\'t exist')
        return

    values = {}
    with open(file) as f:
        while True:
            raw_msg = f.readline()
            if not raw_msg:
                break

            msg = json.loads(raw_msg)
            for k, v in msg.items():
                values.setdefault(k, []).append(v)

    statistics_figure(values, animated = False)

if __name__ == "__main__":
    main()
