Dependency Graph Tools (for Rust Compiler)
===========================================

When employing the incremental features of the Rust compiler, it
generates and reuses dependency graphs.  This repository consists of
tools for processing and visualizing these graphs.

Dump a graph
-------------

Usually, one builds Rust code with cargo.  To dump a graph, one
invokes the rust compiler directly.  One can get the appropriate
`rustc` options by invoking `cargo build` with `-v`:

```
cargo build -v
```

This prints the `rustc` invocation.  For example, it prints this for
me:

```
   Compiling adapton v0.3.5 (file:///Users/hammer/repo/adapton.rust)
     Running `rustc --crate-name adapton src/lib.rs --crate-type lib --emit=dep-info,link -C debuginfo=2 -C metadata=23e1f34d61ec30bc -C extra-filename=-23e1f34d61ec30bc --out-dir /Users/hammer/repo/adapton.rust/target/debug/deps -L dependency=/Users/hammer/repo/adapton.rust/target/debug/deps`
    Finished dev [unoptimized + debuginfo] target(s) in 5.45 secs
```

To this `rustc` invocation, prepend two additional options, `-Z incremental=true`
and `-Z dump-dep-graph`:

```
rustc -Z incremental=true -Z dump-dep-graph ...stuff from above...
```

Now, you should have a file called `dep-graph.txt`.  Likely, it is
_very_ large (e.g., mine is 83M).  

That's okay!  Our visualization tool can process large files quickly,
much (_much!_) faster than graphviz!

Viz tool
----------

Follow the instructions above to generate a `dep_graph.txt` file.
Given this file, you may generate a (large) HTML file from your
dependency graph as follows:

```
cd viz
cargo run -- -i path-to-dep-graph.txt
```
