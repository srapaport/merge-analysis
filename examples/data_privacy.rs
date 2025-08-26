use std::collections::{HashMap, HashSet};
use indicatif::{ProgressBar, ProgressStyle};
use serde::Serialize;
use swh_graph::{graph::*, NodeType};
use std::path::PathBuf;
use swh_graph::mph::DynMphf;
use regex::Regex;
use log::{info, debug};
// use orc_rust::projection::ProjectionMask;
// use orc_rust::ArrowReaderBuilder;
use rayon::prelude::*;
use std::fs::File;
use rayon::prelude::*;

use merge_analysis::env;

fn main(){
    let map = HashMap::from([
        ("USA", vec![
            "California Consumer Privacy Act", "CCPA", "California Privacy Rights Act", "CPRA",
            //"Colorado Privacy Act", "CPA",
            "Colorado Privacy Act",
            "Connecticut Data Privacy Act", "CTDPA",
            "Delaware Personal Data Privacy Act",
            "Florida Data Privacy and Security Act",
            "Indiana Consumer Data Protection Act",
            "Iowa Consumer Data Protection Act", "ICDPA",
            "Kentucky Consumer Data Act", "KCDPA",
            "Maryland Online Data Privacy Act", "MODPA",
            "Minnesota Consumer Data Privacy Act", "MCDPA",
            "Montana Consumer Data Privacy Act",
            "New Hampshire Privacy Act", "NHPA",
            "Nebraska Data Privacy Act", "NDPA",
            "New Jersey Data Privacy Act", "NJDPA",
            "Oregon Consumer Privacy Act", "OCPA",
            "Tennessee Information Protection Act",
            "Texas Data Privacy and Security Act", "TDPSA",
            "Utah Consumer Privacy Act", "UCPA",
            "Virginia Consumer Data Protection Act", "VCDPA"
        ]),
        ("EU", vec!["GDPR", "General Data Protection Regulation"]),
        //("China", vec!["Personal Information Protection Law", "PIPL", "Cybersecurity Law", "CSL", "Data Security Law", "DSL"]),
        ("China", vec!["Personal Information Protection Law", "PIPL", "Cybersecurity Law", "Data Security Law"]),
        ("Germany", vec!["German Federal Data Protection Act", "BDSG"]),
        ("India", vec!["Information Technology Act", "IT Act", "Digital Personal Data Protection Act", "DPDP Act"]),
        ("Japan", vec!["Act on the Protection of Personal Information", "APPI"]),
        //("United Kingdom", vec!["UK GDP", "Data Protection Act", "DPA"]),
        ("United Kingdom", vec!["UK GDP", "Data Protection Act"]),
        ("Canada", vec!["Personal Information Protection and Electronic Documents Act", "PIPEDA", "Personal Information Protection Act", "PIPA Alberta", "Personal Information Protection Act", "PIPA BC", "Act Respecting the Protection of Personal Information in the Private Sector", "Quebec Private Sector Act", "Canadian Privacy Statutes"]),
        ("Brazil", vec!["General Data Protection Law", "LGPD"]),
        ("Russia", vec!["Data Protection Act", "Information Law"]),
        //("South Korea", vec!["Personal Information Protection Act", "PIPA", "Protection of Credit Information", "CIA", "Location Information", "LIA"]),
        ("South Korea", vec!["Personal Information Protection Act", "PIPA", "Protection of Credit Information", "Location Information"]),
        ("Australia", vec!["Privacy Act", "Australian Privacy Principles", "APPs"]),
        ("Mexico", vec!["Ley Federal de Protección de Datos Personales en Posesión de los Particulares", "Mexican Privacy Laws"]),
        ("Other", vec!["Privacy and Electronic Communications", "PECR"]),
    ]);
    

}

fn data_privacy_graph_extraction(map: HashMap<&str, Vec<&str>>){
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

    let keywords: HashSet<String> = map.values()
        .flat_map(|words| words.into_iter())
        .map(|word| String::from(*word))
        .collect();
    let keyword_patterns: Vec<Regex> = keywords.iter()
        .map(|keyword| {
            let pattern = format!(r"\b{}\b", regex::escape(keyword));
            Regex::new(&pattern).unwrap()
        })
        .collect();

    #[derive(Serialize)]
    struct Row{
        swhid: String,
        keyword: String,
        message: String,
    }

    let mut csv_wrt = csv::WriterBuilder::new()
        .from_path(format!("data_privacy_v2.csv"))
        .unwrap();
    let bar_wrt = ProgressBar::new(graph.num_nodes() as u64);
    bar_wrt.set_style(
        ProgressStyle::with_template(
            "{msg} {wide_bar} {pos} {percent_precise}% {elapsed_precise} {eta}",
        )
        .unwrap(),
    );
    bar_wrt.set_message("going through graph"); 
    for node in 0..graph.num_nodes(){
        bar_wrt.inc(1);
        if graph.properties().node_type(node) != NodeType::Revision{
            continue;
        }
        if let Some(utf) = graph.properties().message(node){
            if let Ok(msg) = String::from_utf8(utf){
                // I can read the commit message
                for (keyword, pattern) in keywords.iter().zip(keyword_patterns.iter()){
                    if pattern.is_match(&msg){
                        csv_wrt.serialize(Row{
                            swhid: graph.properties().swhid(node).to_string(),
                            keyword: keyword.clone(),
                            message: msg,
                        }).expect(&format!("couldn't serialize for {}", node));
                       break; // Break to avoid multiple matches per message
                    }
                }
            }
        }
    }
    bar_wrt.finish();
    csv_wrt.flush().unwrap();
}

// fn data_privacy_orc_extraction(){
//     let files = std::fs::read_dir("/home/infres/rapaport/datasets/2024-08-23-popular-500-python/orc/revision")
//         .unwrap()
//         .collect::<Result<Vec<_>, _>>()
//         .unwrap();
//     files.into_par_iter().for_each(|dir_entry| {
//         let orc_path = dir_entry.path();
//         info!(
//             "orc_path : {}",
//             orc_path
//                 .file_name()
//                 .unwrap()
//                 .to_os_string()
//                 .into_string()
//                 .unwrap()
//         );
//         debug!("Extension orc OK");
//         let file = File::open(dir_entry.path())
//             .expect(format!("could not open .orc {:?}", orc_path).as_str());
//         let builder = ArrowReaderBuilder::try_new(file).expect("could not make builder");
//         let projection = ProjectionMask::named_roots(
//             builder.file_metadata().root_data_type(),
//             &[
//                 "id",
//                 "message",
//                 "author",
//                 "date",
//                 "date_offset",
//                 "committer",
//                 "committer_date",
//                 "committer_offset",
//                 "directory",
//             ],
//         );
//         let reader_builder = builder
//             .with_projection(projection.clone())
//             .with_batch_size(env::ORC_BATCH_SIZE);
//         let schema = env::transform_schema(&reader_builder.schema());
//         let reader = reader_builder.with_schema(schema).build();
//         reader.for_each(|batch| {
//             debug!("Entering reader");
//             let batch = batch.unwrap();
//             for revision in <env::Revision>::from_record_batch(batch).unwrap() {

//             }
//         });
//     });
// }
