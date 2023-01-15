from enum import Enum, auto
from typing import List
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

    fig, plots = plt.subplots(rows, columns)
    fig.subplots_adjust(hspace=0.5)
    if animated:
        _ = animation.FuncAnimation(fig, lambda _ : update(plots), interval=1000)
    else:
        update(plots)
    plt.show()

if __name__ == '__main__':
    values = {'t': [14, 14.5, 15, 15.5, 16, 16.5, 17, 17.5, 18, 18.5, 19, 19.5], 'connected_components': [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1], 'articulation_points': [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 'graph_density': [0.002526847757422615, 0.002526847757422615, 0.002526847757422615, 0.002526847757422615, 0.002526847757422615, 0.002526847757422615, 0.002526847757422615, 0.002526847757422615, 0.002526847757422615, 0.002526847757422615, 0.002526847757422615, 0.002526847757422615], 'graph_diameter': [36447042.93069192, 36447042.93069192, 36447042.93069192, 36447042.93069192, 36447042.93069192, 36447042.93069192, 36447042.93069192, 36447042.93069192, 36447042.93069192, 36447042.93069192, 36447042.93069192, 36447042.93069192], 'average_distance': [21317166.360843584, 21317166.360843543, 21317166.360843588, 21317166.36084361, 21317166.360843603, 21317166.36084357, 21317166.360843558, 21317166.360843595, 21317166.36084358, 21317166.360843584, 21317166.360843576, 21317166.36084358], 'active_connections': [3168, 3168, 3168, 3168, 3168, 3168, 3168, 3168, 3168, 3168, 3168, 3168], 'failed_satellites': [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 'rtt_nyc': [0.0650171205081289, 0.06500890455719571, 0.06500086876651778, 0.06499301339706583, 0.0649853387090277, 0.06497784496186819, 0.06497053241438555, 0.0649634013247653, 0.06495645195063052, 0.06494968454908934, 0.06494309937677857, 0.06493669668990426], 'rtt_singapore': [0.08289624612435864, 0.08289491188711373, 0.08289371135437626, 0.08289264452988127, 0.08289171142586296, 0.08289091206308295, 0.08289024647085466, 0.08288971468706374, 0.08288931675818534, 0.08288905273929704, 0.08288892269408867, 0.0828889266948677], 'rtt_johannesburg': [0.13301739499549953, 0.13301148337378113, 0.1330057354216389, 0.13300015103027274, 0.13299473009203952, 0.13298947250064055, 0.13298437815130632, 0.13297944694097869, 0.13297467876849028, 0.1329700735347414, 0.13296563114287377, 0.13296135149844124], 'latency_nyc': [1.1672194160112723e-08, 1.167071919207734e-08, 1.1669276567299594e-08, 1.1667866332630231e-08, 1.1666488534779463e-08, 1.1665143220327684e-08, 1.1663830435735637e-08, 1.1662550227354056e-08, 1.1661302641432709e-08, 1.1660087724128916e-08, 1.1658905521515385e-08, 1.1657756079587485e-08], 'latency_singapore': [7.636545374380468e-09, 7.636422462140923e-09, 7.636311866991544e-09, 7.636213589276407e-09, 7.63612763012255e-09, 7.636053991442567e-09, 7.635992675936862e-09, 7.635943687095526e-09, 7.63590702919993e-09, 7.635882707323917e-09, 7.635870727334703e-09, 7.635871095893377e-09], 'latency_johannesburg': [1.4666085912496035e-08, 1.4665434115398317e-08, 1.4664800364001054e-08, 1.4664184646308281e-08, 1.4663586950451604e-08, 1.4663007264710858e-08, 1.466244557753448e-08, 1.46619018775596e-08, 1.4661376153631817e-08, 1.4660868394824706e-08, 1.4660378590458968e-08, 1.465990673012127e-08]}
    statistics_figure(values, animated = False)
