pub mod merge_utils;
pub mod env;
pub mod fs;

pub fn merge_analysis_call(opts: &env::Options){
    merge_utils::merge_analysis_multi_thread(opts);
}