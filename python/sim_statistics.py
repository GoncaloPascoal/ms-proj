from enum import Enum, auto
import matplotlib.pyplot as plt
import matplotlib.animation as animation

from matplotlib.axes import Axes

values = {}

class PlotType(Enum):
    AVERAGE_DISTANCE = auto()
    DIAMETER = auto()
    DENSITY = auto()
    CONNECTIVITY = auto()
    CONNECTIONS = auto()
    FAILURES = auto()
    RTT = auto()

possible_plots = {
    PlotType.AVERAGE_DISTANCE: {
        'title': 'Average Distance between Satellites',
        'y_values': ['average_distance'],
    },
    PlotType.DIAMETER: {
        'title': 'Diameter of the Satellite Graph',
        'y_values': ['graph_diameter'],
    },
    PlotType.DENSITY: {
        'title': 'Density of the Satellite Graph',
        'y_values': ['graph_density'],
    },
    PlotType.CONNECTIVITY: {
        'title': 'Graph Connectivity',
        'y_values': ['connected_components', 'articulation_points'],
    },
    PlotType.CONNECTIONS: {
        'title': 'Active Connections between Satellites',
        'y_values': ['active_connections'],
    },
    PlotType.FAILURES: {
        'title': 'Evolution of Satellite Failures',
        'y_values': ['active_satellites', 'failed_satellites'],
    },
    PlotType.RTT: {
        'title': 'Round Trip Time from London to Other Cities',
        'y_values': ['nyc', 'singapore', 'johannesburg'],
    },
}

def statistics_figure(v: dict):
    global values
    values = v

    plot_types = [
        PlotType.AVERAGE_DISTANCE, PlotType.DIAMETER   , PlotType.RTT     ,
        PlotType.CONNECTIVITY    , PlotType.CONNECTIONS, PlotType.FAILURES,
    ]
    rows, columns = 2, 3

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

def plot_line(ax: Axes, x: str, y: str):
    ax.plot(values.get(x, []), values.get(y, []), marker='.', label=y)

def plot(ax: Axes, p: PlotType):
    y_values = possible_plots[p]['y_values']
    title = possible_plots[p]['title']
    ax.clear()
    ax.set_title(title)
    for y in y_values:
        plot_line(ax, 't', y)
    ax.legend()

if __name__ == '__main__':
    statistics_figure(values)
