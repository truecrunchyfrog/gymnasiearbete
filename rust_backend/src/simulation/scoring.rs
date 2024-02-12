use crate::docker::api::Metrics;

enum EasingFunctions {
    Linear,
    Quadratic,
    Cubic,
    Quartic,
    Quintic,
    Sinusoidal,
    Exponential,
    Circular,
    Elastic,
    Back,
    Bounce,
}

pub async fn caluclate_score(metrics: Metrics) -> i32 {
    let memory_weight: i32 = 2;
    let memory_max: i32 = 100;
    let memory_easing_function: EasingFunctions = EasingFunctions::Quadratic;
    0
}

async fn calculate_memory_score(memory: i32, max: i32, easing_function: EasingFunctions) -> i32 {
    
}

// Easing goes from 0 to 1 given a function
async fn run_easing(func: EasingFunctions)