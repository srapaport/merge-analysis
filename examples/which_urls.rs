use std::collections::HashSet;
use std::path::PathBuf;
use indicatif::ProgressBar;
use merge_analysis::env;
use serde::{Deserialize, Serialize};
use swh_graph::{graph::*, NodeType};
use swh_graph::mph::DynMphf;
use std::time::Instant;

fn main(){
    let graph = SwhBidirectionalGraph::new(PathBuf::from(env::GRAPH_NAME))
        .expect("Could not load graph")
        .init_properties()
        .load_properties(|properties| properties.load_maps::<DynMphf>())
        .expect("Could not load maps")
        .load_properties(|properties| properties.load_timestamps())
        .expect("Could not load timestamps")
        .load_properties(|properties| properties.load_persons())
        .expect("Could not load persons")
        .load_properties(|properties| properties.load_strings())
        .expect("Could not load strings")
        .load_properties(|properties| properties.load_label_names())
        .expect("Could no load label names")
        .load_labels()
        .expect("Could not load labels");

    let start = Instant::now();

    multi_input("/home/infres/rapaport/data_privacy/notebooks/results_with_sentiment.csv", &graph);

    println!("work complete | time elapsed: {:.2?}", start.elapsed());

    // urls.into_iter().for_each(|url_node|{
    //     let url = String::from_utf8(
    //         graph.properties().message(url_node).unwrap()
    //     ).unwrap();
    //     println!("url reached: {}", url);
    // });
}

fn single_input<G: SwhLabeledForwardGraph + SwhGraphWithProperties + SwhLabeledBackwardGraph>(rev: &str, graph: &G)->HashSet<usize>
where
    <G as SwhGraphWithProperties>::Maps: swh_graph::properties::Maps,
    <G as SwhGraphWithProperties>::LabelNames: swh_graph::properties::LabelNames,
    <G as SwhGraphWithProperties>::Strings: swh_graph::properties::Strings,
    <G as SwhGraphWithProperties>::Persons: swh_graph::properties::Persons,
    <G as SwhGraphWithProperties>::Timestamps: swh_graph::properties::Timestamps,
{
    let mut urls = HashSet::new();

    let mut to_visit = vec![graph.properties().node_id(rev).unwrap()];
    let mut visited = HashSet::new();
    while let Some(visiting) = to_visit.pop(){
        if visited.contains(&visiting){
            continue;
        }
        visited.insert(visiting);

        for pred in graph.predecessors(visiting){
            match graph.properties().node_type(pred){
                NodeType::Origin => {
                    urls.insert(pred);
                },
                _ => to_visit.push(pred),
            }
        }
    }
    urls
}

fn multi_input<G: SwhLabeledForwardGraph + SwhGraphWithProperties + SwhLabeledBackwardGraph>(csv_path: &str, graph: &G)
where
    <G as SwhGraphWithProperties>::Maps: swh_graph::properties::Maps,
    <G as SwhGraphWithProperties>::LabelNames: swh_graph::properties::LabelNames,
    <G as SwhGraphWithProperties>::Strings: swh_graph::properties::Strings,
    <G as SwhGraphWithProperties>::Persons: swh_graph::properties::Persons,
    <G as SwhGraphWithProperties>::Timestamps: swh_graph::properties::Timestamps,
{
    #[derive(Serialize, Deserialize)]
    struct RevisionResults {
        id: String,
        message: String,
        author: String,
        date: i64,
        date_offset: i16,
        committer: String,
        committer_date: i64,
        committer_offset: i16,
        directory: String,
        keyword: String,
    }
    #[derive(Serialize, Deserialize)]
    struct RevisionResultsNew {
        id: String,
        message: String,
        author: String,
        date: i64,
        date_offset: i16,
        committer: String,
        committer_date: i64,
        committer_offset: i16,
        directory: String,
        keyword: String,
        urls: String
    }
    let mut csv_rdr = csv::Reader::from_path(csv_path).unwrap();
    let mut csv_wrt = csv::Writer::from_path(&format!("{}.new.csv", csv_path)).unwrap();
    let bar = ProgressBar::new(csv_rdr.records().count() as u64);
    for result in csv_rdr.deserialize(){
        bar.inc(1);
        if let Ok(rev) = result{
            let rev: RevisionResults = rev;
            let urls = single_input(&rev.id, graph);
            let urls_string = urls.iter()
                .map(|&url_node| {
                    String::from_utf8(
                        graph.properties().message(url_node).unwrap()
                    ).unwrap()
                })
                .collect::<Vec<_>>()
                .join(", ");
            
            csv_wrt.serialize(RevisionResultsNew{
                id: rev.id,
                message: rev.message,
                author: rev.author,
                date: rev.date,
                date_offset: rev.date_offset,
                committer: rev.committer,
                committer_date: rev.committer_date,
                committer_offset: rev.committer_offset,
                directory: rev.directory,
                keyword: rev.keyword,
                urls: urls_string,
            }).unwrap();
        }
    }
    csv_wrt.flush().unwrap();
    bar.finish();
}