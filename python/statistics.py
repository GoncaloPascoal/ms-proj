
from enum import Enum, auto
from typing import List, Iterable
from matplotlib import animation, pyplot as plt
from matplotlib.axes import Axes

import argparse, json, os
import numpy as np

values = {}

class PlotType(Enum):
    DENSITY = auto()
    CONNECTIVITY = auto()
    CONNECTIONS = auto()
    FAILURES = auto()
    RTT = auto()
    LATENCY_DISTANCE_RATIO = auto()

possible_plots = {
    PlotType.DENSITY: {
        'title': 'Density of the Satellite Graph',
        'y_values': {'graph_density': None},
        'y_label': 'Density',
    },
    PlotType.CONNECTIVITY: {
        'title': 'Graph Connectivity',
        'y_values': {'connected_components': 'Connected Components', 'articulation_points': 'Articulation Points'},
        'y_label': None,
    },
    PlotType.CONNECTIONS: {
        'title': 'Active Connections between Satellites',
        'y_values': {'active_connections': 'Active Connections'},
        'y_label': 'Active Connections (units)',
    },
    PlotType.FAILURES: {
        'title': 'Evolution of Satellite Failures',
        'y_values': {'failure_ratio': None},
        'y_label': 'Satellite Failures (%)',
    },
    PlotType.RTT: {
        'title': 'Round Trip Time (from London to Other Cities)',
        'y_values': {'rtt_nyc': 'New York', 'rtt_singapore': 'Singapore', 'rtt_johannesburg': 'Johannesburg'},
        'y_label': 'Round Trip Time (ms)',
    },
    PlotType.LATENCY_DISTANCE_RATIO: {
        'title': 'Latency to Distance Ratio (from London to Other Cities)',
        'y_values': {'latency_nyc': 'New York', 'latency_singapore': 'Singapore', 'latency_johannesburg': 'Johannesburg'},
        'y_label': 'Latency to Distance Ratio (s/m)',
    },
}

def plot_line(ax: Axes, x: str, y: str, label: str) -> None:
    ax.plot(values.get(x, []), values.get(y, []), marker='.', label=label)

def plot(ax: Axes, p: PlotType) -> None:
    plot = possible_plots[p]
    title = plot['title']
    y_label = plot['y_label']
    y_values = plot['y_values']

    ax.clear()
    ax.set_title(title)
    ax.set_xlabel('Time (s)')
    ax.set_ylabel(y_label)

    legend = True
    for y, label in y_values.items():
        plot_line(ax, 't', y, label)
        if label == None:
            legend = False

    if y.startswith('rtt'):
        ax.set_ylim(0, 200)
    elif y.startswith('latency'):
        ax.set_ylim(0)

    if legend:
        ax.legend()

def filter_average(l: Iterable[float | None]) -> float | None:
    filtered = list(filter(lambda x: x is not None, l))
    if filtered:
        return sum(filtered) / len(filtered)
    return None

def statistics_figure(v: dict,
                      plot_types: List[PlotType] = [
                          PlotType.RTT                   , PlotType.CONNECTIVITY,
                          PlotType.LATENCY_DISTANCE_RATIO, PlotType.FAILURES    ,
                      ], rows: int = 2, columns: int = 2, animated: bool = True) -> None:
    global values
    values = v

    def update(figure_plots):
        i = 0
        for row in figure_plots:
            for ax in row:
                plot(ax, plot_types[i])
                i += 1

    if animated:
        fig, plots = plt.subplots(rows, columns)
        fig.subplots_adjust(hspace=0.5)
        _ = animation.FuncAnimation(fig, lambda _: update(plots), interval=1000)
        plt.show()
    else:
        for plot_type in plot_types:
            _, plots = plt.subplots(figsize=(10, 4))
            plot(plots, plot_type)
            plt.tight_layout()
            plt.show()

def main():
    parser = argparse.ArgumentParser(description='Interactive Satellite Megaconstellation Simulation - Statistics Component')
    parser.add_argument(dest='path', type=str,
                        help='Path to the file containing the simulation data')
    args = parser.parse_args()
    path = args.path

    if not os.path.exists(path) or not os.path.isfile(path):
        print('File doesn\'t exist')
        return

    values = {}
    with open(path) as f:
        msgs = json.loads(f.read())
        for msg in msgs:
            for k, v in msg.items():
                values.setdefault(k, []).append(v)

    for k, v in values.items():
        if k.startswith('rtt') or k.startswith('latency'):
            print(f'{k} -> {filter_average(v)}')

    plot_order = [
        PlotType.RTT, PlotType.LATENCY_DISTANCE_RATIO,
        PlotType.CONNECTIVITY, PlotType.FAILURES,
    ]
    statistics_figure(values, animated=False, plot_types=plot_order)

if __name__ == '__main__':
    main()
