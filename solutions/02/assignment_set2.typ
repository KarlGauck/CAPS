// ---------- Page setup ----------
#set page(
  paper: "a4",
  margin: (top: 2.5cm, bottom: 2.5cm, left: 2.5cm, right: 2.5cm),
  header: [
    #set text(size: 9pt, fill: gray)
    #grid(
      columns: (1fr, 1fr),
      [Summer Term 2026 — Schäfer], align(right)[Computational Astrophysics — Set 2]
    )
    #line(length: 100%, stroke: 0.4pt + gray)
  ],
  footer: [
    #line(length: 100%, stroke: 0.4pt + gray)
    #set text(size: 9pt, fill: gray)
    // #align(center)[#counter(page).display("1 of 1", both: true)]
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

// Exercise heading
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

// Sub-question label  e.g.  part("a")
#let part(label) = [
  #v(6pt)
  #text(weight: "bold")[(#label)]
  #h(4pt)
]

// Code block
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

// Title block
#align(center)[
  #v(0.4cm)
  #text(size: 18pt, weight: "bold", fill: accent)[
    Computational Astrophysics Exercises
  ]
  #v(2pt)
  #text(size: 13pt)[Assignments Set 2 — Summer Term 2026]
  #v(4pt)
  #grid(
    align: left,
    columns: (.3fr, 1.2fr, 2fr),
    rows: 2,
    column-gutter: 8pt,
    row-gutter: 5pt,
    grid.cell(rowspan: 2, [*Names:*]),
    [Karl Gauck, 6754404],
    [#align(right)[*Date:* 08.05.26]],
    [Benedict Sondershaus, 6678384],
  )
  #v(0.3cm)
  #line(length: 100%, stroke: 1pt + accent)
  #v(0.2cm)
]

// =====================================================================
#exercise([1], [The vis-viva equation], [1])

The specific energy $epsilon$ in the two body problem is constant throughout the orbit. At any point:
$ epsilon = 1/2 dot(r)^2 - mu/r = "const" $ <energy>

At periapsis ($r_p$) and apoapsis ($r_a$), the velocity vector is perpendicular to $vec(r)$. By conservation of specific angular momentum $h$:
$ h = r_p v_p = r_a v_a arrow.r v_p = h/r_p, v_a = h/r_a $

Equating energy at these two points:
$ 1/2 (h/r_p)^2 - mu/r_p = 1/2 (h/r_a)^2 - mu/r_a $
$ h^2/2 (1/r_p^2 - 1/r_a^2) = mu (1/r_p - 1/r_a) $
$ h^2/2 ( (r_a + r_p) / (r_p r_a) ) = mu $

Using the geometric relation for an ellipse $2a = r_a + r_p$:
$ h^2/2 ( (2a) / (r_p r_a) ) = mu arrow.r.squiggly h^2 = (mu r_p r_a) / a $

Now we can substitute $h^2$ back into @energy at periapsis to find the specific energy:
$ epsilon = h^2 / (2 r_p^2) - mu/r_p = (mu r_p r_a) / (2 a r_p^2) - mu/r_p = (mu r_a) / (2 a r_p) - mu/r_p $
$ =^(r_a=2a-r_p) (mu (2a - r_p)) / (2 a r_p) - mu/r_p = -mu/(2a) $

Setting equal to @energy:
$ 1/2 dot(r)^2 - mu/r = -mu/(2a) $
Or:
$ v^2 = mu (2/r - 1/a) $

// =====================================================================
#exercise([2], [Kepler's equation and root finding], [7])

#part("i")

We used the initial values from the hint: $E_0 = M$ for $e < 0.8$ and $E_0 = pi$ for $e > 0.8$.

The orbits of Mercury ($e=0.205$, $a=0.39$ audefault_result) and Halley's comet ($e=0.967$, $a=17.8$ au) are plotted below.

#figure(
  image("img/orbit_mercury.png"),
  caption: [Orbit of Mercury. Both methods agree, but Newton–Raphson converges in fewer iterations.]
)

#figure(
  image("img/orbit_halley.png"),
  caption: [Orbit of Halley's comet. The high eccentricity makes the fixed-point iteration converge more slowly; Newton–Raphson remains efficient.]
)

Newton–Raphson converges quadratically and therefore requires significantly fewer iterations than the linearly-convergent fixed-point scheme, especially for the more eccentric Halley orbit.

#part("ii")
Using the J2000 orbital elements, we determine the mean anomaly of Earth and Mars on 1 January 1985 via

$ M_0("planet") = lambda - phi_0 - 15 "yr" dot frac(2pi, P), $

then propagate forward day by day with

$ M(t) = frac(2pi t, P) + M_0, $

solving Kepler's equation at each step with Newton–Raphson. The Euclidean distance between the two positions is plotted below.

#figure(
  image("img/distance.png"),
  caption: [Earth–Mars distance from 1 January 1985 to 29 May 2024. The roughly 26-month period where the planets are closest is visible as repeating minima]
)

// =====================================================================
#exercise([3], [The Lagrange point], [2])

The L1 Lagrange point satisfies

$ frac(G M, r^2) - frac(G m, (R-r)^2) = omega^2 r $

or equivalently $f(r) = 0$ with

$ f(r) = frac(G M, r^2) - frac(G m, (R-r)^2) - omega^2 r. $

We have implemented Newton–Raphson directly in Rust with

$ f'(r) = -frac(2 G M, r^3) - frac(2 G m, (R-r)^3) - omega^2. $

Using the given parameters with starting value $r_0 = R/2$, Newton–Raphson converges to a distance from the earth of

$ r_(L 1) approx 3.260 times 10^8 "m" $

or roughly $5.84 times 10^7$ m in front of the Moon.
