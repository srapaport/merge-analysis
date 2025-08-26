use merge_analysis::env;

fn main() {
    let opts = env::Options{
        graph: String::from(env::GRAPH_NAME_TEASER),
        results: format!("results/merge_res_teaser.csv"),
        amount_merge: Some(env::AMOUNT_MERGE_TEASER),
    };
    
    merge_analysis::merge_analysis_call(&opts);
}
