from enum import Enum, auto
import matplotlib.pyplot as plt
import matplotlib.animation as animation

from matplotlib.axes import Axes

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
        'y_values': {'graph_density': 'Density'},
    },
    PlotType.CONNECTIVITY: {
        'title': 'Graph Connectivity',
        'y_values': {'connected_components': 'Connected Components', 'articulation_points': 'Articulation Points'},
    },
    PlotType.CONNECTIONS: {
        'title': 'Active Connections between Satellites',
        'y_values': {'active_connections': 'Active Connections'},
    },
    PlotType.FAILURES: {
        'title': 'Evolution of Satellite Failures',
        'y_values': {'failed_satellites': 'Satellite Failures'},
    },
    PlotType.RTT: {
        'title': 'Round Trip Time (from London to Other Cities)',
        'y_values': {'rtt_nyc': 'New York', 'rtt_singapore' : 'Singapore', 'rtt_johannesburg': 'Johannesburg'},
    },
    PlotType.LATENCY_DISTANCE_RATIO: {
        'title': 'Latency to Distance Ratio (from London to Other Cities)',
        'y_values': {'latency_nyc': 'New York', 'latency_singapore': 'Singapore', 'latency_johannesburg': 'Johannesburg'},
    },
}

def statistics_figure(v: dict) -> None:
    global values
    values = v

    plot_types = [
        PlotType.RTT                   , PlotType.CONNECTIVITY,
        PlotType.LATENCY_DISTANCE_RATIO, PlotType.FAILURES    ,
    ]
    rows, columns = 2, 2

    def update(figure_plots):
        i = 0
        for row in figure_plots:
            for ax in row:
                plot(ax, plot_types[i])
                i += 1

    fig, plots = plt.subplots(rows, columns)
    fig.subplots_adjust(hspace=0.5)
    _ = animation.FuncAnimation(fig, lambda _ : update(plots), interval=1000)
    plt.show()

def plot_line(ax: Axes, x: str, y: str, ylabel: str) -> None:
    ax.plot(values.get(x, []), values.get(y, []), marker='.', label=ylabel)

def plot(ax: Axes, p: PlotType) -> None:
    y_values = possible_plots[p]['y_values']
    title = possible_plots[p]['title']
    ax.clear()
    ax.set_title(title)
    for y, ylabel in y_values.items():
        plot_line(ax, 't', y, ylabel)
    ax.legend()

if __name__ == '__main__':
    statistics_figure(values)
