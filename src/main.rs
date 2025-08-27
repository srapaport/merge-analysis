use std::path::PathBuf;

use merge_analysis::env;
use swh_graph::{graph::SwhBidirectionalGraph, mph::DynMphf};

fn main() {
    let opts = env::Options{
        graph: String::from(env::GRAPH_NAME_TEASER),
        results: format!("results/merge_res_teaser.csv"),
        amount_merge: Some(env::AMOUNT_MERGE_TEASER),
    };
    // let _graph = SwhBidirectionalGraph::new(PathBuf::from(&opts.graph))
    //         .expect("Could not load graph")
    //         .init_properties()
    //         .load_properties(|properties| properties.load_maps::<DynMphf>())
    //         .expect("Could not load maps")
    //         .load_properties(|properties| properties.load_timestamps())
    //         .expect("Could not load timestamps")
    //         .load_properties(|properties| properties.load_persons())
    //         .expect("Could not load persons")
    //         .load_properties(|properties| properties.load_strings())
    //         .expect("Could not load strings")
    //         .load_properties(|properties| properties.load_label_names())
    //         .expect("Could no load label names")
    //         .load_labels()
    //         .expect("Could not load labels");
    merge_analysis::merge_analysis_call(&opts);
    //merge_analysis::merge_utils::get_num_merge(&graph);
}
