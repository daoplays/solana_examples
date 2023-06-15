import numpy as np
import math
import matplotlib.pyplot as plt

def get_entropy(randoms, n_bins = 50):
	hist1 = np.histogram(randoms, bins=n_bins, range=(0,1), density=True)
	ent = 0
	for i in hist1[0]:
		if(i == 0):
			continue
		
		ent -= i * math.log(abs(i))
	return hist1, np.exp(ent)
	

n_bins = 10
d_values = np.loadtxt("seed_values")
d_diffs = np.abs(d_values[1:] - d_values[:-1])
hist, actual = get_entropy(d_values, n_bins)
hist, actual_diffs = get_entropy(d_diffs, n_bins)

e = []
e_diffs = []
for i in range(1000):
	r = np.random.uniform(0, 1, len(d_values))
	h, ent = get_entropy(r, n_bins)
	e.append(ent)
	r_diffs = np.abs(r[1:] - r[:-1])
	h, ent = get_entropy(r_diffs, n_bins)
	e_diffs.append(ent)


plt.rcParams["figure.figsize"] = (11,10)
plt.hist(e)
plt.axvline(actual, lw=4, color="black")
ax = plt.gca()
ax.axes.yaxis.set_visible(False)
plt.xticks([0.97, 0.98, 0.99, 1], fontsize=20)
plt.show()

plt.rcParams["figure.figsize"] = (11,10)
plt.hist(e_diffs)
plt.axvline(actual_diffs, lw=4,  color="black")
ax = plt.gca()
ax.axes.yaxis.set_visible(False)
plt.xticks([0.11, 0.13, 0.15, 0.17, 0.19], fontsize=20)
plt.show()
