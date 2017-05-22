extern crate clap;
extern crate adapton_lab;

use std::cmp::Ordering;
use std::rc::Rc;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;
use std::io::BufRead;
use std::fs::File;
use std::collections::HashMap;

use adapton_lab::labviz::{Div,WriteHTML};

// Temp: Make this better at some point?
type Graph = HashMap<Rc<String>, Vec<Rc<String>>>;
type NodeSet = HashMap<Rc<String>,()>;
type NodeCount = HashMap<Rc<String>,usize>;
type ConsCount = HashMap<Rc<String>,usize>;

#[derive(Debug)]
struct Options {
    tips: bool,
    tips_visited: bool,
}

impl Options {
    fn new() -> Self {
        Options {
            tips:true,
            tips_visited:false,
        }
    }    
}

struct St {
    nodes: NodeSet,
    leaves:NodeSet,
    roots: NodeSet,
    graph: Graph,
    edges: Vec<(Rc<String>, Rc<String>)>,
    rev_graph:Graph,
    cons_count:ConsCount,
}

impl St {
    fn new() -> Self {
        St {            
            nodes: HashMap::new(),
            leaves:HashMap::new(),
            roots: HashMap::new(),
            graph: HashMap::new(),
            edges: Vec::new(),
            rev_graph:HashMap::new(),
            cons_count:HashMap::new(),
        }
    }
    fn process_nodes(self:&mut Self) {
        self.leaves=HashMap::new();
        self.roots=HashMap::new();    
        for (node,_) in self.nodes.iter() {

            let cons = Rc::new(cons_of_node_name(&node));
            let cons_count = (self.cons_count.get(&cons).unwrap_or(&0)).clone();
            self.cons_count.insert(cons, cons_count + 1);
            
            match self.graph.get(node) {
                None => { self.leaves.insert(node.clone(),()); },
                Some(_out) => {}
            };
            match self.rev_graph.get(node) {
                None => { self.roots.insert(node.clone(),()); },
                Some(_out) => {}
            };
        }
    }
    fn add_edge(self:&mut Self, src:Rc<String>, tgt:Rc<String>) {
        self.nodes.insert(src.clone(), ());
        self.nodes.insert(tgt.clone(), ());
        let edge = if false {
            (src, tgt)
        } else {
            (tgt, src)
        };
        self.edges.push(edge.clone());
        
        let inserted : bool = match self.graph.get_mut(&edge.0) {
            None => { false }
            Some(out) => {                
                out.push(edge.1.clone()); 
                true 
            },
        };
        if !inserted {
            self.graph.insert(edge.0.clone(), vec![ edge.1.clone() ]);
        };

        let inserted : bool = match self.rev_graph.get_mut(&edge.1) {
            None => { false }
            Some(out) => {                
                out.push(edge.0.clone()); 
                true 
            },
        };
        if !inserted {
            self.rev_graph.insert(edge.1, vec![ edge.0 ]);
        };                
    }
}

fn main() {
    let mut options = Options::new();
    let args = clap::App::new("dep-viz")
        .version("0.1")
        .author("Matthew Hammer <matthew.hammer@colorado.edu>")
        .about("Consumes Rustc dependency information; Produces visualizations as HTML via a depth-first traversal.")
        .args_from_usage("\
                --no-tips            'smaller HTML files: do not show tips for nodes on hover'
                --tips-visited       'even larger HTML files: show tips even for visited nodes'
            -i, --infile=[infile]    'name for input file'
            -o, --outfile=[outfile]  'name for output file'
            -c, --countfile=[countfile]  'name for output file with DepNode constuctor counts'
"
        )
    .get_matches();
    let infile_name = args.value_of("infile").unwrap_or("dep_graph.txt");
    let outfile_name = args.value_of("outfile").unwrap_or("dep_graph.html");
    
    options.tips = !(args.is_present("no-tips"));
    options.tips_visited = args.is_present("tips-visited");

    println!("{:?}\n", options);
    
    println!("Reading input file: {}", infile_name);    
    let f = File::open(infile_name).unwrap();
    let file = BufReader::new(&f);    
    let mut st = St::new();
    for (_num, line) in file.lines().enumerate() {
        let line = line.unwrap();
        let v: Vec<&str> = line.split(" -> ").map(|s|s.trim()).collect();
        assert_eq!(v.len(), 2);
        st.add_edge(Rc::new(v[0].to_string()), 
                    Rc::new(v[1].to_string()));
    }
    println!("...Read input file: {}", infile_name);
    st.process_nodes();
    println!("  Found {} edges", st.edges.len());
    println!("  Found {} nodes", st.nodes.len());
    println!("    Found {} root nodes", st.roots.len());
    println!("    Found {} leaf nodes", st.leaves.len());

    println!("\nCounting DepNode constructor frequencies...");
    let count_outfile_name = args.value_of("countfile").unwrap_or("dep_graph.counts.txt");
    let count_outfile = File::create(count_outfile_name).unwrap();
    let mut buf_writer = BufWriter::new(count_outfile);
    let mut counts = vec![];
    for (cons,cons_count) in st.cons_count { counts.push((cons, cons_count)) }
    counts.sort_by(|&(_,c1),&(_,c2)| if c1 <= c2 { Ordering::Less } else { Ordering::Greater } );
    let counts_len = counts.len();
    for (cons,cons_count) in counts {
        let _ = write!(&mut buf_writer, "{:8} {}\n", cons_count, cons);
    }
    let _ = buf_writer.flush();    
    println!("...Wrote DepNode constructor frequencies: {}", count_outfile_name);
    println!("  Found {} distinct node constructors; Each becomes a CSS class in HTML output.", counts_len);

    println!("\nTraversing graph (DFS)...");
    let mut stack = vec![];
    for (node,_) in st.roots.iter() {
        stack.push(node.clone());
    }

    let (div, visited) = dfs(&options, &st.graph, stack);
    println!("...Traversed graph.");
    println!("  Visited {} nodes, {:.1}% of total", visited.len(), 
             100f32 * (visited.len() as f32) / (st.nodes.len() as f32));

    if false {
        println!("\nFinding unvisited graph nodes...");
        let mut unvisited : NodeSet = HashMap::new();
        for (node, _) in st.nodes.iter() {
            if visited.get(node) == None {
                unvisited.insert(node.clone(), ());
            }
        }
        assert_eq!(visited.len() + unvisited.len(), 
                   st.nodes.len());
        println!("...Found {} unvisited graph nodes, {:.1}% of total nodes.", 
                 unvisited.len(),              
                 100f32 * (unvisited.len() as f32) / (st.nodes.len() as f32));
    }

    println!("\nWriting HTML output: {}", outfile_name);
    let outfile = File::create(outfile_name).unwrap();    
    let mut buf_writer = BufWriter::new(outfile);
    let _ = buf_writer.write_all(style_string().as_bytes());
    div.write_html(&mut buf_writer);
    let _ = buf_writer.flush();
    println!("..Wrote HTML output: {}", outfile_name);   
}

fn dfs (options:&Options, graph: &Graph, stack:Vec<Rc<String>>) -> (Div, NodeCount) {
    let mut visited:NodeCount = HashMap::new();
    let mut divs:Vec<Div> = Vec::new();
    for n in stack.iter() {
        divs.push(dfs_rec(options, graph, &mut visited, 0, n));
        //if visited.len() > 10000 { break } else { continue }
    }
    (Div{tag:"dfs".to_string(),
         classes:vec![],         
         extent:Box::new(divs),
         text: None}, visited)
}

fn cons_of_node_name(node:&Rc<String>) -> String {
    let cons : Vec<&str> = node.trim().split("(").collect();
    assert!(cons.len() > 0 && cons[0] != "");
    let cons : Vec<&str> = cons[0].trim().split("{").collect();
    assert!(cons.len() > 0 && cons[0] != "");
    cons[0].to_string()
}

fn append_classes_of_node(node:&Rc<String>, mut classes:Vec<String>) -> Vec<String> {
    classes.push(cons_of_node_name(node));
    classes
}

fn append_tooltip(options:&Options, node:&Rc<String>, mut extent:Vec<Div>, visited:bool, depth: usize) -> Vec<Div> {
    if options.tips && (!visited || options.tips_visited) {
        let show_full_details_depth = 2;
        extent.insert(0, Div{tag:"tooltip".to_string(), 
                             classes:vec![], 
                             extent:Box::new(vec![]),
                             text: Some(
                                 format!("{}: {}", depth, 
                                         if !visited && depth < show_full_details_depth
                                         { (**node).clone() } 
                                         else 
                                         { cons_of_node_name(node) }))
        });
        extent
    }
    else {
        extent 
    }
}

fn dfs_rec (options:&Options, graph: &Graph, 
            visited: &mut NodeCount, 
            depth:usize, node:&Rc<String>) -> Div 
{
    if visited.get(node) == None {
        visited.insert(node.clone(),1);
        match graph.get(node) {
            None => { 
                /* Has no children */
                Div{tag:if false {(**node).clone()} else { "".to_string() },
                    classes:append_classes_of_node(node, vec![
                        "node".to_string(),
                        "no-extent".to_string()
                    ]),
                    extent:Box::new(append_tooltip(options, node, vec![], false, depth)),
                    text:None}
            }
            Some(out) => {
                /* Has children */
                let mut divs = vec![];
                for n in out {
                    divs.push(dfs_rec(options, graph, visited, depth+1, n))
                }
                Div{tag:if false {(**node).clone()} else { "".to_string() },
                    classes:append_classes_of_node(node, vec![
                        "node".to_string()
                    ]),
                    extent:Box::new(append_tooltip(options, node, divs, false, depth)),
                    text:None}
            }
        }
    } 
    else {
        // increment counter
        let node_count = visited.get(node).unwrap() + 1;
        visited.insert(node.clone(), node_count);
        // Already visited
        Div{tag:if false {(**node).clone()} else { "".to_string() },
            classes:append_classes_of_node(node, vec![
                "node".to_string(),
                "visited".to_string()                    
            ]),
            extent:Box::new(append_tooltip(options, node, vec![], true, depth)),
            //extent:Box::new(vec![]),
            text:None}
    }
}

pub fn style_string() -> &'static str {
"
<html>
<head>
<script src=\"https://ajax.googleapis.com/ajax/libs/jquery/3.1.1/jquery.min.js\"></script>

<style>
div { 
  display: inline
}
body {
  display: inline;
  color: #aa88cc;
  background: #313;
  font-family: sans-serif;
  text-decoration: none;
  padding: 0px;
  margin: 0px;
}
body:visited {
  color: #aa88cc;
}
a {
  text-decoration: none;
}
a:hover {
  text-decoration: underline;
}
hr {
  display: block;
  float: left;
  clear: both;
  width: 0px;
  border: none;
}

.node {
  color: black;
  display: inline-block;
  border-style: solid;
  border-color: #221133;
  border-width: 1px;
  background: #ffccff;
  font-size: 0px;
  padding: 0px;
  margin: 1px;
  border-radius: 5px;
}

.visited {
  border-width: 2px;
}

.no-extent {
  padding: 3px;
}
.tooltip {
  visibility: hidden;
  background-color: #324;
  border-style: solid;
  border-color: #eeeeee;
  border-width: 1px;
  color: #eae;
  text-align: left;
  padding: 5px 0;
  border-radius: 2px;
  font-size: 12px; 
  position: absolute;
  z-index: 1;
}
.node:hover {
  border-color: white;
}
.node:hover > .tooltip {
  visibility: visible;
}

/* Very unique things
Krate
CoherenceOverlapCheck
TypeckBodiesKrate
LateLintCheck
Reachability
Coherence
CrateVariances
PrivacyAccessLevels
AllLocalTraitImpls
CoherenceCheckTrait
ItemVarianceConstraints
*/

/* Less common things. */

.WorkProduct {
  border-width: 3px;
  border-color: blue;
  background: #999;
}
.Hir {
  border-width: 3px;
  border-color: #005555;
  background: #00aaff;
}
.Mir {
  border-width: 3px;
  border-color: #005555;
  background: #00aaff;
}

/* Extremely common things */

.ItemSignature {
  border-color: #550000;
  background: #ff0000;
}
.ItemAttrs {
  border-color: #555500;
  background: #ffff00;
}
.MetaData {
  border-color: #000055;
  background: #0000ff;
}
.DefSpan {
  border-color: #550055;
  background: #ff00ff;
}
.SymbolName {
  border-color: #555555;
  background: #ffffff;
}

</style>
</head>
<body>
"
}
