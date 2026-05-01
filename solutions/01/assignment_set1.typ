// ---------- Page setup ----------
#set page(
  paper: "a4",
  margin: (top: 2.5cm, bottom: 2.5cm, left: 2.5cm, right: 2.5cm),
  header: [
    #set text(size: 9pt, fill: gray)
    #grid(
      columns: (1fr, 1fr),
      [Summer Term 2026 — Schäfer], align(right)[Computational Astrophysics — Set 1]
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
  #text(size: 13pt)[Assignments Set 1 — Summer Term 2026]
  #v(4pt)
  #grid(
    align: left,
    columns: (.3fr, 1.2fr, 2fr),
    rows: 2,
    column-gutter: 8pt,
    row-gutter: 5pt,
    grid.cell(rowspan: 2, [*Names:*]),
    [Karl Gauck, 6754404],
    [#align(right)[*Date:* 27.04.26]],
    [Benedict Sondershaus, 6678384],
  )
  #v(0.3cm)
  #line(length: 100%, stroke: 1pt + accent)
  #v(0.2cm)
]

TODO: use original exercise descriptions

#exercise([1], [Floating Point Arithmetic], [3])

Calculate the partial sum
$ s_k = sum_(n=1)^(k) 1/n^2 $
for $k = 10^3, 10^6, 10^7, 10^8$ in both single and double precision, and analyse the convergence to the analytical limit $s_infinity = pi^2 / 6$.

#part("a") *Forward summation.* Compute $s_k$ for $k = 10^3, 10^6, 10^7, 10^8$. Plot $|s_k - s_infinity| \/ s_infinity$ and report the runtime of your program. Compare single vs. double precision.

Single precision (*f32 forwards* in the plots) is not as precise as double precision for this particular problem. The summation for single precision attains a wrong value, while the double precision calculation approaches the correct one for $k -> infinity$.

#part("b") *Backward summation.* Repeat part (a) starting from the smallest summand. Compare with (a) and explain your findings.

With backwards summation, both single and double precision approximate the correct assymptotic value correctly for $k -> infinity$. This probably stems from the fact that floating point representations are more precise the nearer the value they are representing is at $0$. Thus, if the calculations start with smaller values, the higher precision in the vicinity of $0$ is preserved longer.

The plots are combined for *(a)* and *(b)*:

#grid(
  columns: 2,
  column-gutter: 10pt,
  [#figure(image("img/Prec-relError.png"), caption: [The relative error gets smaller for $k -> infinity$ for all methods aside from *f32 with forwards summation*. There the error stays the same for our sets of $k$.])],
  [#figure(image("img/Prec-abs.png"), caption: [The error behaviour can also be seen in the real values, as *f32 forwards* computes a slightly smaller value than the other methods which are around the same value.])],
  [#figure(image("img/Prec-durations.png"), caption: [The duration of the summations is roughly the same for all methods, the fine deviations can't be considered as we only have sample size $n=1$. But generally speaking, f64 reversed is the slowest and f32 reversed is the fastest.])],
)

#exercise([2], [Machine Epsilon / Unit Roundoff], [2])

The machine epsilon $epsilon$ is the smallest value satisfying $1 + epsilon > 1$.

Write a program that computes $epsilon$ for both single and double precision. Compare your result with the values reported by `np.finfo()` (Python) or `limits.h` (C).

#table(
  columns: 3,
  table.header([], [Single Precision (```rust f32```)], [Double Precision (```rust f64```)]),
  [measured],
  `0.00000005960465`,
  `0.00000000000000011102230246251568`,
  [in `rust`],
  `0.00000011920929`,
  `0.0000000000000002220446049250313`
)
Our measured values are smaller than the pre-defined values of Rust. Those given epsilons are exactly double our values.

#exercise([3], [Nifty Tricks — Rescaling], [2])

Consider the computation of
$ c = sqrt(a^2 + b^2). $

Using single precision with $a = 10^(30)$ and $b = 1$, the naive implementation returns `inf`:

#code[
```c
/* snippet.c */
float a = 1e30, b = 1.0, res = 0.0;
res = sqrt(a*a + b*b);
fprintf(stdout, "standard method res: %e\n", res);
// output: inf
```
]

#part("a") *Explain* why `inf` is returned for these inputs.

`inf` is returned, because when `1e30` is squared, the result would be `1e60` which is larger than $(2 - 2^(-23)) times 2^127 approx 3.4028235 times 10^38$, which is the largest single precision number smaller than `inf`. Thus the calculation spills into infinity.

#part("b") *Nifty trick.* Describe and implement an algebraically equivalent rescaling that avoids overflow while remaining in single precision.

The *nifty trick*#sym.trademark is to simply scale down the original value, so the overflow does not happen. We used $10^20$.

#exercise([4], [Old Greek Guy Got Pie], [3])

Archimedes (287–212 BC) approximated $pi$ using inscribed regular $n$-polygons in a unit circle of radius $1/2$. The perimeter of the $n$-polygon is
$ U_n = n sin(pi/n), quad lim_(n -> infinity) U_n = pi. $

Starting from the square ($A_2 = U_4 = 2sqrt(2)$), the recursion is
$ A_(n+1) = 2^n sqrt(2 lr((1 - sqrt(1 - lr((A_n / 2^n))^2)))), $ <eq4>

#part("1") Implement the recursion @eq4 and reproduce the error plot (|$A_n - pi$| vs.\ number of iterations). You should observe divergence after iteration 15.

TODO: answer

#part("2") Explain why the error grows after iteration 15, even though double precision nominally offers $~10^(-15)$ accuracy.

TODO: answer

#part("3") Kahan's stable reformulation replaces @eq4 with
$ Z_n = frac(2 lr((A_n / 2^(n+1)))^2, 1 + sqrt(1 - lr((A_n / 2^n))^2)), quad A_(n+1) = 2^n sqrt(4 Z_n). $
Implement this alternative and explain the improvement.

TODO: answer

// End line
#v(0.5cm)
#line(length: 100%, stroke: 0.6pt + accent)