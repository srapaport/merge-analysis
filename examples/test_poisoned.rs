use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::path::PathBuf;

use merge_analysis::fs;
use merge_analysis::env;
use swh_graph::graph::*;
use swh_graph::labels::EdgeLabel;
use swh_graph::mph::DynMphf;
use swh_graph::NodeType;

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

    // let (deleted, created) = status_merge(
    //     graph.properties().node_id("swh:1:rev:7b02f31fe6b6a52461c3dd6936c158f0bf0c1eeb").unwrap(),
    //     &graph
    // ).unwrap();
    let msg = String::from_utf8(
            graph.properties().message(
                graph.properties().node_id("swh:1:rev:7b02f31fe6b6a52461c3dd6936c158f0bf0c1eeb").unwrap()
            ).unwrap()
        ).unwrap();
    println!("commit msg: {}",
        msg
    );
    if msg.contains("[ghstack-poisoned]"){
        println!("FOUND!");
    }
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
    let msg = String::from_utf8(
            graph.properties().message(
                graph.properties().node_id("swh:1:rev:7b02f31fe6b6a52461c3dd6936c158f0bf0c1eeb").unwrap()
            ).unwrap()
        ).unwrap();
    let ghstack_poisoned = msg.contains("[ghstack-poisoned]");
    let Some(root_dir) = fs::get_dir(commit, graph) else {return None};
    
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
                            fs_parents.retain(|filename| !filename.starts_with(&format!("{}/", path)));
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