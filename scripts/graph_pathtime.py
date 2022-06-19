from parse import parse
import matplotlib.pyplot as plt
import sys

TEMPLATE = "[n: {:d}, f: {:d}, c: {:d}] p {:d} | f {:d}"

f = open(sys.argv[1], "r")
lines = f.readlines()

fast_results = {}
path_results = {}

for line in lines:
    parsed = parse(TEMPLATE, line.replace("\n", ""))
    (n, f, c, pr, fr) = parsed

    if f not in fast_results:
        fast_results[f] = {}
    if n not in fast_results[f]:
        fast_results[f][n] = []
    fast_results[f][n].append(fr)

    if f not in path_results:
        path_results[f] = {}
    if n not in path_results[f]:
        path_results[f][n] = []
    path_results[f][n].append(pr)

fig, axs = plt.subplots(2, 1, frameon=False, sharey=True)

for f in fast_results:
    x = []
    y = []

    for n in sorted(fast_results[f]):
        t_list = fast_results[f][n]

        x.append(n)
        y.append(round(sum(t_list) / len(t_list)))

    axs[0].plot(x, y, label=str(fast_results[f]))

for f in path_results:
    x = []
    y = []

    for n in sorted(path_results[f]):
        t_list = path_results[f][n]

        x.append(n)
        y.append(round(sum(t_list) / len(t_list)))

    axs[1].plot(x, y, label=str(path_results[f]))

axs[0].set_title("Fast Algoritm")
axs[0].set_xlabel('Node count (n)')
axs[0].set_ylabel('Latency (ms)')
axs[0].set_yscale('log')
axs[1].set_title("Pathfind Algorithm")
axs[1].set_xlabel('Node count (n)')
axs[1].set_ylabel('Latency (ms)')
axs[1].set_yscale('log')
fig.tight_layout()

plt.savefig(sys.argv[1] + ".png", bbox_inches='tight', transparent=True)
plt.show()
