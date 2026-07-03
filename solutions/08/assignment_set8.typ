// ---------- Page setup ----------
#set page(
  paper: "a4",
  margin: (top: 2.5cm, bottom: 2.5cm, left: 2.5cm, right: 2.5cm),
  header: [
    #set text(size: 9pt, fill: gray)
    #grid(
      columns: (1fr, 1fr),
      [Summer Term 2026 — Schäfer], align(right)[Computational Astrophysics — Set 8]
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
  #set text(font: "JetBrainsMono NF", size: 9pt)
  #body
]

// --------- Document begin ---------

#align(center)[
  #v(0.4cm)
  #text(size: 18pt, weight: "bold", fill: accent)[
    Computational Astrophysics Exercises
  ]
  #v(2pt)
  #text(size: 13pt)[Assignments Set 8 — Summer Term 2026]
  #v(4pt)
  #grid(
    align: left,
    columns: (.3fr, 1.2fr, 2fr),
    rows: 2,
    column-gutter: 8pt,
    row-gutter: 5pt,
    grid.cell(rowspan: 2, [*Names:*]),
    [Karl Gauck, 6754404],
    [#align(right)[*Date:* 03.07.26]],
    [Benedict Sondershaus, 6678384],
  )
  #v(0.3cm)
  #line(length: 100%, stroke: 1pt + accent)
  #v(0.2cm)
]

// =====================================================================
#exercise([1], [Linear Advection], [10])

#figure(image("img/velocities.png"), caption: "Plot of all solvers and the analytical solution. As can be seen, the FTCS solution diverges quite far with heavy oscillation")
#figure(image("img/velocities_stable.png"), caption: "Plot of only the upwind and FTCS solvers and the analytical solution")

The upwind solver is the most stable with no additional oscillation, but also does not approximate the harsh boundary at $plus.minus 1/3$ well. The Lax Wendroff solver oscillates a bit at this boundary, but also approximates the steep curve better. \
Those two solvers in this plot are comparable to a Fourier series approximation with lower and higher allowed periods. \
All around, both solvers approximate the analytical solution well. The tradeoffs lie in how well the velocity discontinuity is approximated and how much oscillation should happen at those points. \

The solver code can be run with #code("cargo run -- a8 ex1")

$part("Extra")$

We implemented a small visualizer for tracking the curve behaviour live. It plots velocity, pressure, density and divergence. Density should be a constant quantity and can be used to test the integrity of the simulation. They can be toggled on and off over the menu at the top left. \
It can be started with one of:
#code("cargo run -- a8 render-ftcs")
#code("cargo run -- a8 render-upwind")
#code("cargo run -- a8 render-lw")

We used AI for creating the line meshes of the simulation.