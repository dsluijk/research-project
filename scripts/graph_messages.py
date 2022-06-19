from parse import parse
import matplotlib.pyplot as plt
import sys

TEMPLATE = "[n: {:d}, f: {:d}, c: {:d}, i: {:d}] f: d {:d}%, m {:d}, t: {:d} | r: d {:d}%, m {:d}, t: {:d}"

f = open(sys.argv[1], "r")
lines = f.readlines()

flood_results = {}
routed_results = {}

for line in lines:
    parsed = parse(TEMPLATE, line.replace("\n", ""))
    (n, f, c, i, fd, fm, ft, rd, rm, rt) = parsed

    if f not in flood_results:
        flood_results[f] = {}
    if n not in flood_results[f]:
        flood_results[f][n] = []
    flood_results[f][n].append(fm)

    if f not in routed_results:
        routed_results[f] = {}
    if n not in routed_results[f]:
        routed_results[f][n] = []
    routed_results[f][n].append(rm)

fig, axs = plt.subplots(2, 1, frameon=False)

for f in flood_results:
    x = []
    y = []

    for n in sorted(flood_results[f]):
        m_list = flood_results[f][n]

        x.append(n)
        y.append(round(sum(m_list) / len(m_list)))

    axs[0].plot(x, y, label=str(flood_results[f]))

for f in routed_results:
    x = []
    y = []

    for n in sorted(routed_results[f]):
        m_list = routed_results[f][n]

        x.append(n)
        y.append(round(sum(m_list) / len(m_list)))

    axs[1].plot(x, y, label=str(routed_results[f]))

axs[0].set_xticks([1, 4, 8, 12, 16, 20])
axs[0].set_yticks([1, 20, 40, 60, 80, 100, 120, 140, 160])
axs[0].set_title("Flooding")
axs[0].set_xlabel('Node count (n)')
axs[0].set_ylabel('Messages Per Broadcast (avg)')
axs[1].set_xticks([1, 4, 8, 12, 16, 20])
axs[1].set_yticks([1, 20, 40, 60, 80, 100, 120, 140, 160])
axs[1].set_title("Routing")
axs[1].set_xlabel('Node count (n)')
axs[1].set_ylabel('Messages Per Broadcast (avg)')
fig.tight_layout()

plt.savefig(sys.argv[1] + ".png", bbox_inches='tight', transparent=True)
plt.show()
