# Jiji: Yet another i3-bar alternative

Jiji is an i3-bar alternative. Written in Rust using gtk3-rs.

## Motivation

I wanted to have more controls on PulseAudio on the i3-bar. Existing
alternatives do not provide much support. Fine, I'll write one.

I wanted to have richer controls on the bars. This excludes text-based
implementations. I wanted to use languages with a package manager in order to
utilize libraries easily. This excludes C/C++ based implementations (one can
disagree on this). If there's an implementation that meets these conditions, I
wanted to just utilize that, but I couldn't find one.

Hence, write my own one. In order to have rich controls, GTK seems a natural
choice. Rust seems to have a good GTK binding. Here, i3-bar alternative written
in Rust using GTK.
