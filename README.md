Dependency Graph Tools (for Rust Compiler)
===========================================

When employing the incremental features of the Rust compiler, it
generates and reuses dependency graphs.  This repository consists of
tools for processing and visualizing these graphs.

Dump a graph with `cargo rustc`
--------------------------------

Use the `rustc` subcommand of `cargo` to pass some additional
debugging flags, through `cargo`, to its `rustc` invocations.  In
particular, use two additional options, `-Z incremental=true` and `-Z
dump-dep-graph`, as follows:

```
cargo rustc -- -Z incremental=true -Z dump-dep-graph
```

**Verbose mode**: Optionally, if you want to see how cargo invokes
`rustc`, just include `-v` before the `--`:

```
cargo rustc -v -- -Z incremental=true -Z dump-dep-graph
```

Now, you should have a file called `dep-graph.txt`.  Likely, it is
_very_ large; the one that I generated for the `adapton` crate is 83M,
for example.

That's okay!  Our visualization tool can process large files quickly,
much (_much!_) faster than graphviz!

Viz tool
----------

**Quick how-to**:
Follow the instructions above to generate a `dep_graph.txt` file.
Given this file, you may generate a (large) HTML file from your
dependency graph as follows:

```
cd viz
cargo run --release -- -i path-to-dep-graph.txt
```

**Explanation of HTML output**:
To visualize the dependency graph, we first transform this graph into
a tree via a depth-first search, and we render this traversal as an
HTML file.  This DFS-tree-based visualization approach has several
benefits:

1. Web browsers can render trees of `div`s very efficiently (cf, the
   time required to run `graphviz`) 

2. Tree-based visualizations are more visually compact than
   graph-based visualizations (cf, the utility of running `graphviz`
   on the output and opening in an image viewer; I do not recommend
   actually trying this).

However, there are downsides too:

1. When the DFS traversal encounters a node that it has already
visited, it saves space in the HTML file and the on-screen rendering
by _not_ revisiting it.  Instead, it emits an empty `div` tag,
rendered as a dot via CSS.  Hence, the _order_ of the DFS determines
where nodes are rendered, in relation to their parent node in the
output HTML.

2. The DFS traversal is determined by the order of the `dep_graph.txt`
file, which is currently not ordered in any meaningful way (its based
on traversing the entries of a hash table, whose order is determined
by hashes and hash buckets).

