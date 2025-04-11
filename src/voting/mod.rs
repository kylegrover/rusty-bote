pub mod star;
pub mod plurality;
pub mod ranked;
pub mod approval;

pub struct PollResults {
    pub winner: String,      // Name of winning option
    pub summary: String,     // Summary of results
    pub raw_results: String, // Detailed results data (could be JSON)
}
