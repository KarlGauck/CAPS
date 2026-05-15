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
  #text(size: 13pt)[Assignments Set 3 — Summer Term 2026]
  #v(4pt)
  #grid(
    align: left,
    columns: (.3fr, 1.2fr, 2fr),
    rows: 2,
    column-gutter: 8pt,
    row-gutter: 5pt,
    grid.cell(rowspan: 2, [*Names:*]),
    [Karl Gauck, 6754404],
    [#align(right)[*Date:* 15.05.26]],
    [Benedict Sondershaus, 6678384],
  )
  #v(0.3cm)
  #line(length: 100%, stroke: 1pt + accent)
  #v(0.2cm)
]

// =====================================================================
#exercise([1], [Catastrophic cancellation], [3])

#part("a")
The naive solution is implemented in the method ```rust classic_solution(a, b, c)```. It returns both roots given by
$
r_(1,2) = (-b plus.minus sqrt(b^2 - 4a c))/(2a)
$
as an optional value as tuple. If the square root is not real, an empty option will be returned instead.

We are only interested in the first value with $-b + sqrt(...)$ .

#part("b")
The following plot shows the errors for finding the roots with parameters $a=1, b=1, c=10^(-n), n=1..25$. It shows the errors for single (f32) and double (f64) precision, together with the respective machine epsilon. The error is calculated against the "stable" method, calculated with a float with 512 bits (16x precision) to make sure the values are accurate.

#figure(
  image("img/quadratic_eq_error.png", width: 86%),
  caption: [Error plots of the roots for $a=1,b=1,c=10^(-n),n=1..25$, $c$ is on the x-axis, the error is on the y-axis. Both axes are in log scale]
)

We could not observe a real "explosive" increase of error at a certain point, only a plateau, where the error lines diverge from the machine epsilon to another greater value, let's call it $e'(c)$. This $e(c)$ is the same for f32 and f64 and again decreases for decreasing $c$. The machine epsilon values or values in its surrounding would be expected as absolute error, but the difference between the machine epsilon and $e(c)$ is of a magnitude of $~16$.

So this plateau is probably caused by the catastrophic cancellation as in this range $b^2 approx sqrt(b^2 - 4a c)$ holds and the difference cannot be described by the floating point precision. For even smaller values of $c$ (around $10^(-17)$ for double precision), the precision of the floating point values increases again, and thus the error decreases.

#part("c")
The stable solution is implemented with *Vieta's formulas*. They state that for a polynomial of degree $2$ in the representation $P(x)=a x^2 + b x + c$ the following holds about its roots $r_1, r_2$:

$ r_1+r_2 = -b/a $
$ r_1 r_2 = c/a $ <vieta2>

Our problem is if $b >> 4 a c$ as the first root is calculated via:
$ r_1 = (-b + sqrt(b^2 - 4 a c))/(2a) $
and thus $b$ and the square root will cancel without the required precision. But the second root can be calculated without problems:
$ r_2 = -(b + sqrt(b^2 - 4 a c))/(2a) $
We can use this together with @vieta2 to calculate $r_1$:
$ r_1 &= c / (r_2 a) \
      &= (-2 c) / (b + sqrt(b^2 - 4 a c)) $

Thus the potential for cancellation is eliminated.

This stable solution is implemented in ```rust other_stable_solution(a,b,c)``` for 512-bit floats.

#part("d")
The stable solution from *(c)* is again susceptible to catastrophic cancellation when \ $|b| >> 4 a c$ and $b < 0$, as then $b + sqrt(b^2 - 4 a c) approx 0$. Thus we simply check if $b$ is negative and if it is, we return the naive solution for $r_1$ and a similar solution like the stable one with Vieta for $r_2$:
$ r_2 = (2 c) / (-b + sqrt(b^2 - 4 a c)) $

The implementation can be found in ```rust final_stable_solution(a,b,c)```

#pagebreak()

// ------------------------------------------
// Ex 2
// ------------------------------------------
#exercise([2], [Interpolation], [7])

#part("a")
The Runge function $f(x) = 1/(1+x^2)$ is interpolated on $[-5, 5]$ using Newton interpolation for polynomial degrees $n = 12$ and $n = 20$. The divided difference coefficients are computed in the method ```rust make_pol_line(n)``` via the given pseudocode.

With equidistant nodes the *Runge phenomenon* is visible: Both polynomials diverge near the endpoints $x = plus.minus 5$, with $n = 20$ being surprisingly worse than $n = 12$ (so higher degree doesn't always increase precision). 

#grid(
  columns: 2,
  column-gutter: 10pt,
  [#figure(image("img/pol_range4.png"), caption: [Interpolation polynomials $P_(12)$ and $P_(20)$ with equidistant nodes on $[-4,4]$ for better visibility of the interpolation. Runge phenomenon is already visible at the edges.])],
  [#figure(image("img/pol_range5.png"), caption: [Interpolation polynomials $P_(12)$ and $P_(20)$ with equidistant nodes on $[-5,5]$. Boundary divergence is strongly amplified compared to $[-4,4]$.])],
)

#part("b")
$W(x) = product_(i=0)^n (x - x_i)$ is computed in ```rust make_W_line(n)``` and plotted alongside all other plots. With equidistant nodes it grows to values on the order of $10^(12)$ near the interval edges for $n=12$, and the $n = 20$ $W$-function oscillates with even larger amplitude across the whole interval.

#part("c")
With Chebyshev nodes the $W(x)$ function is orders of magnitude smaller and the error is spread across the interval rather than spiking at the edges.

#grid(
  columns: 2,
  column-gutter: 10pt,
  [#figure(image("img/pol_error_w.png"), caption: [Interpolation error and $W(x)$ with equidistant nodes. The scale reaches $~10^(12)$, dominated by boundary blow-up.])],
  [#figure(image("img/pol_error_w_ch.png"), caption: [Interpolation error and $W(x)$ with Chebyshev nodes. The boundary blow-up is not visible anymore and $W(x)$ is spread along the interval.])],
  [#figure(image("img/pol_error.png"), caption: [Absolute interpolation error with equidistant nodes. The error is largest near $x = plus.minus 5$ and increases with degree.])],
  [#figure(image("img/pol_error_ch.png"), caption: [Absolute interpolation error with Chebyshev nodes. The error is small and evenly distributed across the interval for both degrees.])],
)
