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
        'y_values': {'failed_satellites': None},
        'y_label': 'Satellite Failures (%)',
    },
    PlotType.RTT: {
        'title': 'Round Trip Time (from London to Other Cities)',
        'y_values': {'rtt_nyc': 'New York', 'rtt_singapore' : 'Singapore', 'rtt_johannesburg': 'Johannesburg'},
        'y_label': 'Round Trip Time (s)',
    },
    PlotType.LATENCY_DISTANCE_RATIO: {
        'title': 'Latency to Distance Ratio (from London to Other Cities)',
        'y_values': {'latency_nyc': 'New York', 'latency_singapore': 'Singapore', 'latency_johannesburg': 'Johannesburg'},
        'y_label': 'Latency to Distance Ratio (s/m)',
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
    plot = possible_plots[p]
    title = plot['title']
    y_label = plot['y_label']
    y_values = plot['y_values']

    ax.clear()
    ax.set_title(title)
    ax.set_xlabel('Time (s)')
    ax.set_ylabel(y_label)

    legend = True
    for y, ylabel in y_values.items():
        plot_line(ax, 't', y, ylabel)
        if ylabel == None:
            legend = False
    
    if legend:
        ax.legend()

if __name__ == '__main__':
    statistics_figure(values)
