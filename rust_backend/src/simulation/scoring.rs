use crate::docker::api::Metrics;
use crate::Error;
use anyhow;
use easing;

pub async fn calculate_score() -> Result<u32, anyhow::Error> {
    let cpu_time: f64 = 0.2;
    let cpu_time_max: f64 = 1.0;

    let cpu_score = cpu_score(cpu_time, cpu_time_max).expect("Failed");
    let memory_score: f64 = 0.1;

    // Define weights for CPU and memory scores
    let cpu_weight = 0.7;
    let memory_weight = 0.3;

    // Apply weights to the scores
    let weighted_cpu = cpu_score * cpu_weight;
    let weighted_memory = memory_score * memory_weight;

    // Combine weighted scores
    let combined_score = weighted_cpu + weighted_memory;

    // Apply easing function to make higher values rarer
    let eased_score = ease_out(combined_score);

    // Map the eased score to the range [1, 100]
    let final_score = (eased_score * 99.0 + 1.0).round() as u32;

    Ok(final_score)
}

fn cpu_score(time: f64, max: f64) -> Option<f64> {
    let steps = 100;
    easing::quad_out(time, max, steps).last()
}

// Easing function (simple quadratic ease out)
fn ease_out(t: f64) -> f64 {
    1.0 - (1.0 - t).powi(2)
}
