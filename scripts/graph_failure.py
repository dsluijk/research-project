from parse import parse
import matplotlib.pyplot as plt
import sys

TEMPLATE = "n: {:d}, f: {:d}, c: {:d}, a: {}"

f = open(sys.argv[1], "r")
lines = f.readlines()

fast_x = []
fast_y = []
path_x = []
path_y = []

for line in lines:
    parsed = parse(TEMPLATE, line.replace("\n", ""))
    (n, f, c, a) = parsed

    if a == "f":
        fast_x.append(n)
        fast_y.append(f)
    else:
        path_x.append(n)
        path_y.append(f)

plt.scatter(fast_x, fast_y, label="Fast Algorithm")
plt.scatter(path_x, path_y, label="Pathfind Algorithm")
plt.xlabel("Amount of Nodes (n)")
plt.ylabel("Faulty Nodes (f)")
plt.xticks([1, 4, 8, 12, 16, 20])
plt.yticks([1, 4, 8, 12, 16, 20])
plt.legend()
plt.savefig(sys.argv[1] + ".png", bbox_inches='tight', transparent=True)
plt.show()
