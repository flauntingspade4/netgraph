use petgraph::{
	dot::{Config, Dot},
	graph::NodeIndex,
};

use linkify::{LinkFinder, LinkKind};

use clap::{App, Arg};

use simple_stopwatch::Stopwatch;

type Graph = petgraph::graph::Graph<String, ()>;

fn main() -> Result<(), reqwest::Error> {
	let link_graph = App::new("Link graph")
		.version("0.0.1")
		.author("Elliot .W <elliotwhybrow@gmail.com>")
		.about("Makes a list of websites from a given site's links.")
		.arg(
			Arg::with_name("limit")
				.short("l")
				.long("limit")
				.help("Sets the level of links to follow. Default of 3")
				.takes_value(true),
		)
		.arg(
			Arg::with_name("INPUT")
				.help("Sets the input url to use")
				.required(true)
				.index(1),
		)
		.get_matches();

	let mut link_finder = LinkFinderFor::new(
		link_graph
			.value_of("limit")
			.unwrap_or("3")
			.parse()
			.expect("Failed to parse the limit to a number"),
		String::from(link_graph.value_of("INPUT").unwrap()),
	);
	link_finder.run();

	Ok(())
}

struct LinkFinderFor {
	limit: u64,
	graph: Graph,
}

impl LinkFinderFor {
	fn new(limit: u64, link: String) -> Self {
		let mut graph = Graph::new();
		graph.add_node(String::from(link));
		Self { limit, graph }
	}
	fn find(&mut self, previous_node: NodeIndex, enumeration: u64) {
		if self.limit >= enumeration {
			let request = reqwest::blocking::get(self.graph.node_weight(previous_node).unwrap());
			match &request {
				Ok(_) => {}
				Err(e) => {
					println!("Error: {}", e);
					return;
				}
			}
			let body = request
				.unwrap()
				.text()
				.expect("Failed to parse webpage to text");

			let mut finder = LinkFinder::new();
			finder.kinds(&[LinkKind::Url]);
			let links: Vec<_> = finder.links(&body).collect();
			for link in links {
				let link = link.as_str();
				if link.contains("www") && !link.contains("gif") && !link.contains("dtd") && !link.contains("svg") {
					let new_node = self.graph.add_node(String::from(link));
					self.graph.add_edge(previous_node, new_node, ());
				}
			}
			for node_index in self.graph.clone().neighbors(previous_node) {
				self.find(node_index, enumeration + 1);
			}
		}
	}
	fn run(&mut self) {
		let sw = Stopwatch::start_new();
		self.find(NodeIndex::new(0), 1);
		let elapsed_ms = sw.ms();
		println!(
			"{:?}",
			Dot::with_config(&self.graph, &[Config::EdgeNoLabel])
		);
		println!("Time taken: {}ms", elapsed_ms);
	}
}