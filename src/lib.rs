mod merge_analysis;
pub mod env;
pub mod fs;

pub fn merge_analysis_call(){
    const AMOUNT_MERGE_TEASER: usize = 40_603_865;
    merge_analysis::merge_analysis_multi_thread(Some(AMOUNT_MERGE_TEASER));
}