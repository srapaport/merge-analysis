use std::collections::{HashMap, HashSet, VecDeque};
use swh_graph::{graph::*, labels::EdgeLabel, NodeType};

#[derive(Debug, Clone)]
pub struct FileSystemTree {
    pub directories: HashMap<String, DirectoryInfo>,
    pub files: HashMap<String, FileInfo>,
    // Reverse lookups for performance
    pub node_to_dir_path: HashMap<usize, String>,
    pub node_to_file_path: HashMap<usize, String>,
}

#[derive(Debug, Clone)]
pub struct DirectoryInfo {
    pub path: String,
    pub node_id: usize,
    pub parent: Option<String>,
    pub children: Vec<String>, // paths of children
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: String,
    pub filename: String,
    pub node_id: usize,
    pub parent_dir: String,
}

impl FileSystemTree {
    pub fn new() -> Self {
        Self {
            directories: HashMap::new(),
            files: HashMap::new(),
            node_to_dir_path: HashMap::new(),
            node_to_file_path: HashMap::new(),
        }
    }
    
    pub fn add_directory(&mut self, dir_info: DirectoryInfo) {
        let path = dir_info.path.clone();
        let node_id = dir_info.node_id;
        self.node_to_dir_path.insert(node_id, path.clone());
        self.directories.insert(path, dir_info);
    }
    
    pub fn add_file(&mut self, file_info: FileInfo) {
        let path = file_info.path.clone();
        let node_id = file_info.node_id;
        self.node_to_file_path.insert(node_id, path.clone());
        self.files.insert(path, file_info);
    }
    
    // Check if a node_id with specific name exists
    pub fn has_node_with_name(&self, node_id: usize, name: &str) -> bool {
        // Check if it's a directory
        if let Some(dir_path) = self.node_to_dir_path.get(&node_id) {
            if let Some(dir) = self.directories.get(dir_path) {
                return dir.path.split('/').last().unwrap_or("") == name;
            }
        }
        
        // Check if it's a file
        if let Some(file_path) = self.node_to_file_path.get(&node_id) {
            if let Some(file) = self.files.get(file_path) {
                return file.filename == name;
            }
        }
        
        false
    }
    
    // Get node info by node_id
    pub fn get_node_info(&self, node_id: usize) -> Option<(&str, bool)> {
        // Returns (path, is_directory)
        if let Some(path) = self.node_to_dir_path.get(&node_id) {
            return Some((path, true));
        }
        if let Some(path) = self.node_to_file_path.get(&node_id) {
            return Some((path, false));
        }
        None
    }
    
    // Check if a file exists at a specific path
    pub fn has_file_at_path(&self, path: &str) -> Option<usize> {
        self.files.get(path).map(|file| file.node_id)
    }
    
    // Check if a directory exists at a specific path
    pub fn has_directory_at_path(&self, path: &str) -> Option<usize> {
        self.directories.get(path).map(|dir| dir.node_id)
    }
}

pub fn get_dir<G: SwhLabeledForwardGraph + SwhGraphWithProperties>(
    commit: usize,
    graph: &G,
) -> Option<usize>
where
    <G as SwhGraphWithProperties>::Maps: swh_graph::properties::Maps,
    <G as SwhGraphWithProperties>::LabelNames: swh_graph::properties::LabelNames,
{
    for succ in graph.successors(commit) {
        if graph.properties().node_type(succ) == NodeType::Directory {
            return Some(succ);
        }
    }
    None
}

fn get_dirs<G: SwhLabeledForwardGraph + SwhGraphWithProperties>(
    commits: HashSet<usize>,
    graph: &G,
) -> HashSet<usize>
where
    <G as SwhGraphWithProperties>::Maps: swh_graph::properties::Maps,
    <G as SwhGraphWithProperties>::LabelNames: swh_graph::properties::LabelNames,
{
    let mut dirs = HashSet::new();
    commits.into_iter().for_each(|commit|{
        if let Some(dir) = get_dir(commit, graph){
            dirs.insert(dir);
        }
    });
    dirs
}

fn get_parents<G: SwhLabeledForwardGraph + SwhGraphWithProperties + SwhLabeledBackwardGraph>(
    commit: usize,
    graph: &G,
) -> HashSet<usize>
where
    <G as SwhGraphWithProperties>::Maps: swh_graph::properties::Maps,
    <G as SwhGraphWithProperties>::LabelNames: swh_graph::properties::LabelNames,
    <G as SwhGraphWithProperties>::Strings: swh_graph::properties::Strings,
    <G as SwhGraphWithProperties>::Persons: swh_graph::properties::Persons,
    <G as SwhGraphWithProperties>::Timestamps: swh_graph::properties::Timestamps,
{
    let mut res = HashSet::new();
    for node in graph.successors(commit){
        res.insert(node);
    }
    res
}

pub fn get_list_of_content_parents<G: SwhLabeledForwardGraph + SwhGraphWithProperties + SwhLabeledBackwardGraph>(
    commit: usize,
    graph: &G,
) -> Option<(HashSet<String>, HashSet<usize>)>
where
    <G as SwhGraphWithProperties>::Maps: swh_graph::properties::Maps,
    <G as SwhGraphWithProperties>::LabelNames: swh_graph::properties::LabelNames,
    <G as SwhGraphWithProperties>::Strings: swh_graph::properties::Strings,
    <G as SwhGraphWithProperties>::Persons: swh_graph::properties::Persons,
    <G as SwhGraphWithProperties>::Timestamps: swh_graph::properties::Timestamps,
{
    let mut filenames = HashSet::new();
    let parents = get_parents(commit, graph);
    if parents.len() < 2{
        return None;
    }
    let root_dirs = get_dirs(parents, graph);
    let mut visited_dir = HashSet::new();
    
    root_dirs.into_iter().for_each(|dir|{
        let mut path_node: HashMap<usize, String> = HashMap::new();
        path_node.insert(dir, ".".to_string());

        let mut to_visit = VecDeque::new();
        to_visit.push_back(dir);
        let mut visited = HashSet::new();

        while let Some(node) = to_visit.pop_front(){
            if visited.contains(&node) {
                continue;
            }
            visited.insert(node);
            let current_path = path_node.get(&node).expect("couldn't find path in path_node").clone();
            for (succ, labels) in graph.labeled_successors(node) {
                for label in labels {
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
                            filenames.insert(path);
                        }
                        NodeType::Directory => {                            
                            path_node.insert(succ, path);
                            if !visited_dir.contains(&succ){
                                visited_dir.insert(succ);
                                to_visit.push_back(succ);
                            }
                        }
                        _ => continue,
                    }
                }
            }
        }
    });

    Some((filenames, visited_dir))
}