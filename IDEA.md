
## Situation

I have now: 

1. Forked https://github.com/orhanbalci/rough-rs into https://github.com/vexyart/vruffr which is this main repo
2. I’ve moved the older project called 'skesvg' into the 'extra' folder of this repo. 
3. We have more reference Rust code in @./extra/ext/ 

## Objective 

Your goal is to: 

1. Functionally decouple this 'vruffr' repo from the upstream https://github.com/orhanbalci/rough-rs — we don’t care about comitting our changes back to them

2. Integrate the 'skesvg' functionality into the 'vruffr' repo. 

3. Rename and refactor everything so that the new project is 'vruffr'

4. Drop the backends 'rough_plotters_svg' and 'rough_iced' — we only want to keep the backends 
- 'rough_piet' which uses @./extra/ext/piet/ 
- 'rough_tiny_skia' which uses @./extra/ext/tiny-skia/
- 'rough_vello' which uses @./extra/ext/vello/ and possibly @./extra/ext/vello_svg/

5. Our vruffr project should be a set of crates, a Rust library, a CLI tool in Rust, a Python package (bindings), a Python Fire-based CLI tool, and wasm-web code. 

6. The CLIs / libraries need to be able to select the backend + backend surface + output format combos.

Backend/surface: 

- @./extra/ext/piet/tree/main/piet-cairo
- @./extra/ext/piet/tree/main/piet-coregraphics
- @./extra/ext/piet/tree/main/piet-direct2d
- @./extra/ext/piet/tree/main/piet-svg
- @./extra/ext/piet/tree/main/piet-web
- @./extra/ext/vello 
- @./extra/ext/tiny-skia

Format: 

- png
- svg (drawn from scratch)
- svgpatch (tries to patch the input svg using the output svg, maintaining as much of the input svg structure as possible)

Note: Not all backend / surface / format combos are possible, and not all needs to be supported in the CLIs.  

7. The CLIs & libraries need to expose as much as possible of creative / roughening parameters, including things like densities and types of hachure fills, stroke widths, stroke colors, fill colors, randomness, etc.

8. The CLIs & libraries need to expose additional sensible parameters, especially for sizing, filtering of certain SVG types of objects, also the font/font-size replacement that we have in our old 'skesvg' project.

9. Ultimately we want to get rid of the old 'skesvg' code, and use the new 'vruffr' project for all our needs. No backwards compatibility needed! 

## Tasks

### Think big, design big, write clearly

- Analyze all existing code

- Take a step back

- See the big picture

- Ultrathink 

- Into @./SPEC.md write a detailed specification of the new project

- Fully rewrite all the writing, docs etc. — we treat it as an independent project, not a fork of the old one. Of course we acknowledge the old/used projects, but the README should be completely rewritten to focus on the new project: what it does, how it does it, why it’s done this way, how to set it up 

- Into README adapt development guidelines from ./CLAUDE.md 

- Adapt @./extra/test-data/ @./extra/test-data.sh @./extra/build.sh into the new codebase. 

### FIXME: Panics in rough_tiny_skia

We have these 'patched panics' in @./rough_tiny_skia/ and I want these systematically fixed. 

```
$ ./extra/target/release/skesvg ./extra/test-data/tigr.svg -o ./extra/test-data/tigr1.svg

thread 'main' (1074005) panicked at /Users/adam/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rough_tiny_skia-0.12.0/src/skia_generator.rs:281:19:
called `Option::unwrap()` on a `None` value
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

thread 'main' (1074005) panicked at /Users/adam/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rough_tiny_skia-0.12.0/src/skia_generator.rs:281:19:
called `Option::unwrap()` on a `None` value

thread 'main' (1074005) panicked at /Users/adam/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rough_tiny_skia-0.12.0/src/skia_generator.rs:281:19:
called `Option::unwrap()` on a `None` value

thread 'main' (1074005) panicked at /Users/adam/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rough_tiny_skia-0.12.0/src/skia_generator.rs:281:19:
called `Option::unwrap()` on a `None` value

thread 'main' (1074005) panicked at /Users/adam/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rough_tiny_skia-0.12.0/src/skia_generator.rs:281:19:
called `Option::unwrap()` on a `None` value

thread 'main' (1074005) panicked at /Users/adam/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rough_tiny_skia-0.12.0/src/skia_generator.rs:281:19:
called `Option::unwrap()` on a `None` value
Rendered ./test-data/tigr.svg -> ./test-data/tigr1.svg [SVG plain]
```

### Iterate 

Analyze, ultrathink, iterate, improve. Look at the rendered results. 

### Add more features

- Add color manipulation (especially making everything monochrome in several different ways)

- Add more realistic filters when rendering to bitmap. Basically the "rough" vector output is OK, but it’s still vector. The rendered images could be probably made even more to look like sketches but applying some additional manipulation. Ultrathink and research how to do this. Then plan it and implement it.

---

