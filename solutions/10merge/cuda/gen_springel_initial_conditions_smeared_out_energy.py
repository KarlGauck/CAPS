#!/usr/bin/env python

"""
Generate initial conditions for a 3D Sedov-Taylor blast wave simulation.

Difference to the classic Springel & Hernquist (2002) setup:
Instead of depositing the whole explosion energy E in a *single* central
particle, the energy is smeared over a compact SPH cubic-spline kernel of
radius h_inject centred on the origin. This gives a much smoother, less
noisy injection while conserving the total energy *exactly*:

    e_i = E * W(|r_i|, h_inject) / ( m * sum_j W(|r_j|, h_inject) )

so that   sum_i m_i e_i = E   holds by construction.

The kernel and its support-radius convention (support = sml, not 2*sml)
match miluph's internal kernel in rhs.c and seagenSedov.py.
"""

import numpy as np

# ---------------------------------------------------------------------------
# settings: energy E deposited around the origin (0,0,0) on a cubic lattice
# with unit box length and unit density
# ---------------------------------------------------------------------------
xmin = -0.5
xmax = 0.5
explosion_energy = 1.0     # total injected thermal energy E
efloor = 1e-6              # ambient (background) specific internal energy
N = 64                     # particles per dimension -> N**3 particles total
material_type = 0

rho = 1.0

# cubic lattice
x, dx = np.linspace(xmin, xmax, N, retstep=True)
m = rho * dx**3

# smoothing length used in the simulation (must match materialscenario.data)
sml = 2.51 * dx

# width over which the explosion energy is spread.
# h_inject = sml  -> energy smeared over exactly one SPH kernel support
#                    (~ a few dozen particles, physically the resolution scale)
# increase the factor (e.g. 1.5, 2.0) for an even smoother / broader blast.
h_inject = 1.0 * sml

print("set smoothing length to %.17lf" % sml)


def cubic_spline_W(r, h):
    """
    3D cubic spline kernel, compact support r < h.
    Same convention as miluph (rhs.c) and seagenSedov.py:
    support radius = h, normalisation f = 8 / (pi * h**3).
    """
    q = np.asarray(r) / h
    f = 8.0 / np.pi / h**3
    W = np.zeros_like(q)
    m1 = q <= 0.5
    m2 = (q > 0.5) & (q <= 1.0)
    W[m1] = f * (6.0 * q[m1]**3 - 6.0 * q[m1]**2 + 1.0)
    W[m2] = 2.0 * f * (1.0 - q[m2])**3
    return W


# ---------------------------------------------------------------------------
# build the full particle grid (same i,j,k ordering as the original loops)
# ---------------------------------------------------------------------------
X, Y, Z = np.meshgrid(x, x, x, indexing='ij')
X = X.ravel()
Y = Y.ravel()
Z = Z.ravel()
r = np.sqrt(X**2 + Y**2 + Z**2)

# kernel weights centred on the origin
W = cubic_spline_W(r, h_inject)

Wsum = W.sum()
if Wsum <= 0.0:
    raise RuntimeError(
        "No particle falls inside the injection kernel. "
        "Increase h_inject (currently %.5f = %.2f dx)." % (h_inject, h_inject / dx))

# normalise so that the *total* injected energy equals explosion_energy:
#   sum_i m_i e_inject_i = E   =>   e_inject_i = E * W_i / (m * sum_j W_j)
e_inject = explosion_energy * W / (m * Wsum)

# ambient floor everywhere, explosion on top
e = np.maximum(e_inject, efloor)

# ---------------------------------------------------------------------------
# diagnostics
# ---------------------------------------------------------------------------
n_hot = int((e_inject > efloor).sum())
E_check = float((m * e_inject).sum())
print("injection radius h_inject = %.6f  (= %.2f dx)" % (h_inject, h_inject / dx))
print("number of particles receiving energy: %d" % n_hot)
print("peak specific energy e_max = %.6e  (single-particle setup: %.6e)"
      % (e_inject.max(), explosion_energy / m))
print("total injected energy (check, should be 1.0) = %.17lf" % E_check)

# ---------------------------------------------------------------------------
# write file: columns  x y z  vx vy vz  m  e  material_type
# (matches miluph ASCII reader with INTEGRATE_ENERGY enabled,
#  INTEGRATE_DENSITY disabled)
# ---------------------------------------------------------------------------
with open("springel_sedov_smeared.0000", "w") as output:
    for xi, yi, zi, ei in zip(X, Y, Z, e):
        print("%.17lf %.17lf %.17lf 0.0 0.0 0.0 %.17lf %.17lf %d"
              % (xi, yi, zi, m, ei, material_type), file=output)