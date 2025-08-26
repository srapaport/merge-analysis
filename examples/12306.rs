use std::path::PathBuf;

use merge_analysis::merge_utils;
use merge_analysis::env;
use swh_graph::graph::*;
use swh_graph::mph::DynMphf;

fn main(){
    let graph = SwhBidirectionalGraph::new(PathBuf::from(env::GRAPH_NAME_TEASER))
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
    let commit = graph.properties().node_id("swh:1:rev:141182401615f8d1b5a0ecc2ab3a5c07a3ad484a").unwrap();
    // println!("swhid node 266: {}", graph.properties().swhid(266).to_string());
    // println!("swhid node 304: {}", graph.properties().swhid(304).to_string());
    // println!("swhid node 302: {}", graph.properties().swhid(302).to_string());
    let (deleted, created, _gh, _message_status) = merge_utils::status_merge(commit, &graph).unwrap();
    
    println!("Created:");
    for file in created.into_iter(){
        println!("\t {}",file);
    }
    println!("Deleted:");
    for file in deleted.into_iter(){
        println!("\t {}", file);
    }

    // for (succ, labels) in graph.labeled_successors(302){
    //     for label in labels{
    //         let name: String;
    //         if let EdgeLabel::DirEntry(dir_entry) = label {
    //             name = String::from_utf8_lossy(
    //                 &graph.properties().label_name(dir_entry.filename_id())
    //             ).to_string();
    //         } else {
    //             continue;
    //         }
    //         println!("name: {} | id: {}", name, succ);
    //     }
    // }
    // println!("");
    // for (succ, labels) in graph.labeled_successors(304){
    //     for label in labels{
    //         let name: String;
    //         if let EdgeLabel::DirEntry(dir_entry) = label {
    //             name = String::from_utf8_lossy(
    //                 &graph.properties().label_name(dir_entry.filename_id())
    //             ).to_string();
    //         } else {
    //             continue;
    //         }
    //         println!("name: {} | id: {}", name, succ);
    //     }
    // }
}