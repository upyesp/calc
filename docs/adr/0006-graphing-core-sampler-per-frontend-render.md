# Graphing splits into a core Sampler and per-frontend renderers

Graphing is divided into compute and render. `epher-core` owns a `Sampler` that
turns an Expression and a domain into plottable data (sampling, domain and
discontinuity handling); each frontend renders that data its own way — vector
for the GUI and PWA, ASCII/blocks for the TUI, none for the CLI.

This seam protects "logic exists once": the sampling math is shared across every
frontend, and only the pixels differ. v1 graphs 2D (`y=f(x)`, parametric,
polar); 3D is deferred.
