#!/usr/bin/python

import networkx as nx
import matplotlib.pyplot as plt
import sys

f = open(sys.argv[1], "r")
lines = f.readlines()
edges = []

for line in lines:
    items = line.split(" ")
    edges.append([int(items[0]), int(items[1])])

G = nx.Graph()
G.add_edges_from(edges)

print("Connectivity: " + str(nx.approximation.node_connectivity(G)))

nx.draw_networkx(G)
plt.show()
