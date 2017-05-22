#[macro_use] extern crate clap;
extern crate adapton_lab;

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

struct Options {
    tooltips: bool,
    tooltips_visited: bool,
}

impl Options {
    fn new() -> Self {
        Options {
            tooltips:true,
            tooltips_visited:false,
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
}

impl St {
    fn new() -> Self {
        St {            
            nodes: HashMap::new(),
            leaves:HashMap::new(),
            roots: HashMap::new(),
            graph: HashMap::new(),
            edges: Vec::new(),
            rev_graph:HashMap::new()
        }
    }
    fn process_nodes(self:&mut Self) {
        self.leaves=HashMap::new();
        self.roots=HashMap::new();        
        for (node,_) in self.nodes.iter() {
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
            //(Rc::new(v[0].to_string()), Rc::new(v[1].to_string()))
            (src, tgt)
        } else {
            //(Rc::new(v[1].to_string()), Rc::new(v[0].to_string()))
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
        .about("Consumes Rustc dependency information; Produces visualizations")
        .args_from_usage("\
                --tooltips-visited    'show tooltips for visited nodes'
            -i, --infile=[infile]     'name for input file'
            -o, --outfile=[outfile]   'name for output file'")
    .get_matches();
    let infile_name = args.value_of("infile").unwrap_or("dep_graph.txt");
    let outfile_name = args.value_of("outfile").unwrap_or("dep_graph.html");
    options.tooltips_visited = value_t!(args, "tooltips-visited", bool).unwrap_or(false);
    let f = File::open(infile_name).unwrap();
    let file = BufReader::new(&f);    
    let mut st = St::new();

    println!("Reading input file: {}", infile_name);
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

    println!("\nPerforming DFS on graph...");
    let mut stack = vec![];
    for (node,_) in st.roots.iter() {
        stack.push(node.clone());
        //break;
    }

    let (div, visited) = dfs(&options, &st.graph, stack);
    println!("...Performed DFS on graph.");
    println!("  Visited {} nodes, {:.1}% of total", visited.len(), 
             100f32 * (visited.len() as f32) / (st.nodes.len() as f32));

    println!("\nFinding unvisited graph nodes...");
    let mut unvisited : NodeSet = HashMap::new();
    for (node, _) in st.nodes.iter() {
        if visited.get(node) == None {
            unvisited.insert(node.clone(), ());
            //println!("unvisited: {}", node);
        }
    }
    assert_eq!(visited.len() + unvisited.len(), 
               st.nodes.len());
    //println!("...Found graph nodes.");
    println!("...Found {} unvisited graph nodes, {:.1}% of total.", 
             unvisited.len(),              
             100f32 * (unvisited.len() as f32) / (st.nodes.len() as f32));

    println!("Writing HTML output: {}", outfile_name);
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
    cons[0].to_string()
}

fn append_classes_of_node(node:&Rc<String>, mut classes:Vec<String>) -> Vec<String> {
    classes.push(cons_of_node_name(node));
    classes
}

fn append_tooltip(options:&Options, node:&Rc<String>, mut extent:Vec<Div>, visited:bool, depth: usize) -> Vec<Div> {
    if options.tooltips && (!visited || options.tooltips_visited) {
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

fn dfs_rec (options:&Options, graph: &Graph, visited: &mut NodeCount, depth:usize, node:&Rc<String>) -> Div {
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
            classes:vec![
                "node".to_string(),
                "visited".to_string()
            ],
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
  border-color: green;
}
.node:hover > .tooltip {
  visibility: visible;
}

.WorkProduct {
  border-width: 2px;
  border-color: blue;
  background: #999;
}

.MetaData {
  border-color: #000055;
  background: #0000ff;
}

.Hir {
  border-color: #005555;
  background: #00aaff;
}

</style>
</head>
<body>
"
}
