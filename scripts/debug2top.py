#!/usr/bin/python

import networkx as nx
import matplotlib.pyplot as plt
import sys

edges = []
for e in sys.argv[1][1:-1].split("}, "):
    s = e.split(": {")
    if len(s[1]) == 0:
        continue

    for v in s[1].split(", "):
        ab = [s[0], v.replace("}", "")]
        print(ab[0] + " " + ab[1])
        edges.append([int(ab[0]), int(ab[1])])

G = nx.DiGraph()
G.add_edges_from(edges)

nx.draw_networkx(G)
plt.show()
