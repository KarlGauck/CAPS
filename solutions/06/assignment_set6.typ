// ---------- Page setup ----------
#set page(
  paper: "a4",
  margin: (top: 2.5cm, bottom: 2.5cm, left: 2.5cm, right: 2.5cm),
  header: [
    #set text(size: 9pt, fill: gray)
    #grid(
      columns: (1fr, 1fr),
      [Summer Term 2026 — Schäfer], align(right)[Computational Astrophysics — Set 6]
    )
    #line(length: 100%, stroke: 0.4pt + gray)
  ],
  footer: [
    #line(length: 100%, stroke: 0.4pt + gray)
    #set text(size: 9pt, fill: gray)
  ]
)

// ---------- Misc settings ----------
#set text(font: "New Computer Modern", size: 11pt, lang: "en")
#set par(justify: true, leading: 0.65em)
#set heading(numbering: none)
#set math.equation(numbering: "(1)")

// ---------- Colours ----------
#let accent = rgb("#003366")
#let codebg = rgb("#f5f5f5")

// ---------- Custom components ----------

#let exercise(number, title, points) = block(
  width: 100%,
  inset: (x: 10pt, y: 8pt),
  fill: accent,
  radius: 4pt,
  below: 10pt,
)[
  #set text(fill: white, weight: "bold", size: 12pt)
  Exercise #number: #title #h(1fr) #points pts
]

#let part(label) = [
  #v(6pt)
  #text(weight: "bold")[(#label)]
  #h(4pt)
]

#let code(body) = block(
  width: 100%,
  inset: 10pt,
  fill: codebg,
  radius: 4pt,
  stroke: 0.4pt + gray,
)[
  #set text(font: "New Computer Modern Mono", size: 9.5pt)
  #body
]

// --------- Document begin ---------

#align(center)[
  #v(0.4cm)
  #text(size: 18pt, weight: "bold", fill: accent)[
    Computational Astrophysics Exercises
  ]
  #v(2pt)
  #text(size: 13pt)[Assignments Set 6 — Summer Term 2026]
  #v(4pt)
  #grid(
    align: left,
    columns: (.3fr, 1.2fr, 2fr),
    rows: 2,
    column-gutter: 8pt,
    row-gutter: 5pt,
    grid.cell(rowspan: 2, [*Names:*]),
    [Karl Gauck, 6754404],
    [#align(right)[*Date:* 19.06.26]],
    [Benedict Sondershaus, 6678384],
  )
  #v(0.3cm)
  #line(length: 100%, stroke: 1pt + accent)
  #v(0.2cm)
]

// =====================================================================
#exercise([1], [The Restricted Circular Three Body Problem], [10])

#part("2a.")

We implemented an RK4 integrator for the RC3BP equations of motion with $mu_2 = 10^(-3)$, $C_J = 3.03$, $Delta t = 10^(-3)$, and initial conditions $dot(x)_0 = 0$, $y_0 = 0$. The following figures show particle positions sampled at every 1st, 10th, and 500th step, with increasing total step counts.

#figure(
  image("img/paths_10000_1.png", width: 90%),
  caption: [Particle paths for 10\,000 steps with every point plotted. Trajectories are still short, the structure of individual orbits is visible.]
)

#figure(
  image("img/paths_50000_1.png", width: 90%),
  caption: [50\,000 steps, every point. Orbits begin to fill out their allowed regions. An hollow inner exclusion zone and outer boundary become apparent.]
)

#figure(
  image("img/paths_100000_10.png", width: 90%),
  caption: [100\,000 steps, every 10th point. The torus-like structure becomes clear. Most initial conditions show bounded curves, indicating periodicity.]
)

#figure(
  image("img/paths_500000_10.png", width: 90%),
  caption: [500\,000 steps, every 10th point. The grey trajectory ($x_0 = 0.5$) has escaped to large radii. All other initial conditions remain bounded.]
)

#figure(
  image("img/paths_1000000_500.png", width: 90%),
  caption: [1\,000\,000 steps, every 500th point. The escaping grey orbit now completely diverges.]
)

#figure(
  image("img/paths_5000000_500.png", width: 90%),
  caption: [5\,000\,000 steps, every 500th point. The grey ($x_0 = 0.5$) orbit has completely escaped, confirming it is chaotic and unbound on long timescales.]
)

*Symmetry:* The trajectories show a reflection symmetry about the $x$-axis. This follows from the equations of motion: The $y$-equation is odd in $y$, so if $(x(t), y(t))$ is a solution with $dot(y_0)>0$, $(-x(t), y(t))$ with $dot(y_0)<0$ is also a solution.

*Stability:* Initial conditions $x_0 in {0.21, 0.24, 0.26, 0.27, 0.4, 0.6, 0.8}$ yield bounded, periodic trajectories. The initial condition $x_0 = 0.5$ produces an orbit that escapes to large distances for large step counts, thus indicating chaotic or unstable behaviour.

#part("2b.")

Poincaré sections were constructed by recording $(x, dot(x))$ each time the particle crosses $y = 0$ with $dot(y) > 0$.

#figure(
  image("img/poincare.png", width: 90%),
  caption: [Poincaré section for steps until $5 dot 10^5$. Stable orbits trace closed invariant curves around $dot(x) approx 0.3$-$0.5$. Loops can be seen for _red_, _blue_, _green_, _purple_, _yellow_, _pink_ and _black_ is concentrated into a point. The _gray_ ($x_0=0.5$) section is chaotic, this becomes even more apparent in #ref(<poincare_unstable>)]
)<poincare>
#figure(
  image("img/poincare_unstable.png", width: 90%),
  caption: [Poincaré section for larger step counts, until $5 dot 10^6$. This clearly shows the chaos of the initial value $x_0=0.5$]
)<poincare_unstable>

This analysis shows the stability more clearly:

- *Stable orbits* are $x_0 in \{ 0.21, 0.24, 0.26, 0.27, 0.4, 0.6, 0.8 \}$), as the crossings lie on closed curves in the range $dot(x) approx 0.2$-$0.6$. The looping curves (resonant islands) can be seen and traced, although with gaps in between. The clearest islands are _red_ ($x_0=0.21$) and _purple_ ($x_0=0.6$).

- *The unstable orbit* of $x_0 = 0.27$ is scattered broadly across the plane, first bounded in #ref(<poincare>) and then completely divergent in #ref(<poincare_unstable>).

$part("Extra")$

We implemented a small visualizer for the three bodies. This shows the two large masses in fixed position, while the small mass orbits around them.
