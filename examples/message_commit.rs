use std::path::PathBuf;
use merge_analysis::env;
use swh_graph::graph::*;
use swh_graph::mph::DynMphf;
use counter::Counter;

fn main(){
    let _graph = SwhBidirectionalGraph::new(PathBuf::from(env::GRAPH_NAME_TEASER))
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

    let mut amount_err = 0;
    let mut amount_read = 0;
    let mut poisoned: Counter<String, usize> = Counter::new();
    let mut csv_rdr = csv::Reader::from_path("./results/merge_res_full.csv").unwrap();
    for result in csv_rdr.deserialize::<env::Changes>(){
        if let Ok(changes) = result{
            amount_read += 1;
            poisoned[&changes.poisoned.to_string()] += 1;
        }
        else{
            amount_err += 1;
        }
    }
    println!("Poisoned: {:?}", poisoned);
    println!("Amount read: {}", amount_read);
    println!("Amount err: {}", amount_err);
}