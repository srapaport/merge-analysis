use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::sync::mpsc::channel;

use indicatif::{ProgressBar, ProgressStyle};
use rayon::ThreadPoolBuilder;
use swh_graph::graph::*;
use swh_graph::labels::EdgeLabel;
use swh_graph::mph::DynMphf;
use swh_graph::NodeType;
use crate::env;
use crate::fs;

fn get_num_merge<G: SwhLabeledForwardGraph + SwhGraphWithProperties>(graph: &G)-> usize
where
    <G as SwhGraphWithProperties>::Maps: swh_graph::properties::Maps,
    <G as SwhGraphWithProperties>::LabelNames: swh_graph::properties::LabelNames,
    <G as SwhGraphWithProperties>::Strings: swh_graph::properties::Strings,
    <G as SwhGraphWithProperties>::Persons: swh_graph::properties::Persons,
    <G as SwhGraphWithProperties>::Timestamps: swh_graph::properties::Timestamps,
{
    let bar = ProgressBar::new(graph.num_nodes() as u64);
    bar.set_style(
        ProgressStyle::with_template(
            "{msg} {wide_bar} {pos} {percent_precise}% {elapsed_precise} {eta}",
        )
        .unwrap(),
    );
    bar.set_message("Counting merge");
    let mut num_merge = 0;
    for node in 0..graph.num_nodes(){
        if graph.successors(node).into_iter().count() > 1{
            num_merge += 1;
        }
        bar.inc(1);
    }
    bar.finish();
    println!("Amount of merge in graph: {}", num_merge);
    num_merge
}

pub fn merge_analysis_multi_thread(amount_of_merge: Option<usize>){
    let (tx, rx) = channel::<
        Option<env::Changes>,
    >();
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
    let number_of_merge = amount_of_merge.unwrap_or_else(|| get_num_merge(&graph));

    let jh = thread::spawn(move || {
        let mut merge = 0;
        let mut rev = 0;
        let tx_err = Arc::new(Mutex::new(HashSet::new()));
        let workers = num_cpus::get() / 3;
        let pool = ThreadPoolBuilder::new()
            .num_threads(workers)
            .build()
            .unwrap();
        pool.install(|| {
            rayon::scope(|thread|{
                for node in 0..graph.num_nodes(){
                    if graph.properties().node_type(node) != NodeType::Revision{
                        continue;
                    }
                    rev += 1;
                    let mut parent = 0;
                    for succ in graph.successors(node){
                        if graph.properties().node_type(succ) == NodeType::Revision{
                            parent += 1;
                        }
                    }
                    if parent > 1{
                        merge += 1;
                        thread.spawn({
                            let tx = tx.clone();
                            let graph = &graph;
                            let tx_err = tx_err.clone();
                            move |_|{
                                if let Some((deleted, created)) = status_merge(node, &graph){
                                    if let Err(_) = tx.send(Some(env::Changes{
                                        commit: graph.properties().swhid(node).to_string(),
                                        created: created.into_iter().collect::<Vec<_>>().join(","),
                                        deleted: deleted.into_iter().collect::<Vec<_>>().join(","),
                                    })){
                                        tx_err.lock().unwrap().insert(node);
                                    }
                                }
                        }});
                        
                    }
                }
            });
        });
    });

    let mut amount_not_received = 0;
    let mut csv_wrt = csv::WriterBuilder::new()
        .from_path(format!("merge_res_teaser.csv"))
        .unwrap();
    let bar_wrt = ProgressBar::new(number_of_merge as u64);
    bar_wrt.set_style(
        ProgressStyle::with_template(
            "{msg} {wide_bar} {pos} {percent_precise}% {elapsed_precise} {eta}",
        )
        .unwrap(),
    );
    bar_wrt.set_message("writing data");
    for _ in 0..number_of_merge{
        if let Ok(recv) = rx.recv(){
            if let Some(changes) = recv{
                let err_msg = format!("couldn't serialize data for commit {}", changes.commit);
                csv_wrt.serialize(changes).expect(&err_msg);
            }
        }
        else{
            amount_not_received += 1;
        }
        bar_wrt.inc(1);
    }
    csv_wrt.flush().expect("couldn't flush csv");
    bar_wrt.finish();
    println!(
        "Amount of transmissions not received: {}",
        amount_not_received
    );

    jh.join().unwrap();
}

#[allow(dead_code)]
pub fn merge_analysis(){
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

    let bar = ProgressBar::new(graph.num_nodes() as u64);
    bar.set_style(
        ProgressStyle::with_template(
            "{wide_bar} {pos} {percent_precise}% {elapsed_precise} {eta}",
        )
        .unwrap(),
    );
    let mut merge = 0;
    let mut rev = 0;
    let mut res = vec![];
    for node in 0..graph.num_nodes(){
        bar.inc(1);
        if graph.properties().node_type(node) != NodeType::Revision{
            continue;
        }
        rev += 1;
        let mut parent = 0;
        for succ in graph.successors(node){
            if graph.properties().node_type(succ) == NodeType::Revision{
                parent += 1;
            }
        }
        if parent > 1{
            merge += 1;
            if let Some((deleted, created)) = status_merge(node, &graph){
                res.push(env::Changes{
                    commit: graph.properties().swhid(node).to_string(),
                    created: created.into_iter().collect::<Vec<_>>().join(","),
                    deleted: deleted.into_iter().collect::<Vec<_>>().join(","),
                });
            }
        }
    }
    bar.finish();
    let mut csv_wrt = csv::WriterBuilder::new()
        .from_path(format!("merge_res_teaser.csv"))
        .unwrap();
    res.into_iter().for_each(|change|{
        let err_msg = format!("couldn't serialize changes for {}", change.commit);
        csv_wrt.serialize(change).expect(&err_msg);
    });
    csv_wrt.flush().expect("couldn't flush");
    println!("amount of commits: {}", rev);
    println!("amount of merge commits: {}", merge);
    let percentage: f32 = ((merge as f32)/(rev as f32))*100.0;
    println!("percentage of merge: {}%", percentage);

}

fn status_merge<G: SwhLabeledForwardGraph + SwhGraphWithProperties + SwhLabeledBackwardGraph>(
    commit: usize,
    graph: &G,
) -> Option<(HashSet<String>, HashSet<String>)>
where
    <G as SwhGraphWithProperties>::Maps: swh_graph::properties::Maps,
    <G as SwhGraphWithProperties>::LabelNames: swh_graph::properties::LabelNames,
    <G as SwhGraphWithProperties>::Strings: swh_graph::properties::Strings,
    <G as SwhGraphWithProperties>::Persons: swh_graph::properties::Persons,
    <G as SwhGraphWithProperties>::Timestamps: swh_graph::properties::Timestamps,
{
    let mut created_files = HashSet::new();
    let Some((mut fs_parents, visited_dir)) = fs::get_list_of_content_parents(commit, graph) else{
        return None;
    };
    let root_dir = fs::get_dir(commit, graph).expect(&format!("No root directory for commit {}", commit));
    
    let mut path_node: HashMap<usize, String> = HashMap::new();
    path_node.insert(root_dir, ".".to_string());

    let mut to_visit = VecDeque::new();
    to_visit.push_back(root_dir);
    let mut visited = HashSet::new();
    while let Some(node) = to_visit.pop_front(){
        if visited.contains(&node){
            continue;
        }
        visited.insert(node);

        let current_path = path_node.get(&node).expect("couldn't find path in path_node").clone();
        for (succ, labels) in graph.labeled_successors(node){
            for label in labels{
                let name: String;
                if let EdgeLabel::DirEntry(dir_entry) = label {
                    name = String::from_utf8_lossy(
                        &graph.properties().label_name(dir_entry.filename_id())
                    ).to_string();
                } else {
                    continue;
                }
                let path = if current_path == "." {
                    name.clone()
                } else {
                    format!("{}/{}", current_path, name)
                };
                match graph.properties().node_type(succ) {
                    NodeType::Content => {
                        if !fs_parents.remove(&path){
                            created_files.insert(path);
                        }
                    }
                    NodeType::Directory => {    
                        path_node.insert(succ, path.clone());                        
                        if visited_dir.contains(&succ){
                            fs_parents.retain(|filename| !filename.starts_with(&path));
                        } else {
                            to_visit.push_back(succ);
                        }
                    }
                    _ => continue,
                }
            }
        }
    }
    if fs_parents.len() > 0 || created_files.len() > 0{
        return Some((fs_parents, created_files));
    }
    None
}