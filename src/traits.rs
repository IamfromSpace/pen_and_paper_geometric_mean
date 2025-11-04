pub trait EstimateGeometricMean {
    type Error: std::error::Error;
    fn estimate_geometric_mean(values: &[f64]) -> Result<f64, Self::Error>;
}

pub trait FinalAnswer {
    fn final_answer(&self) -> f64;
}

pub trait EstimateGeometricMeanStepByStep {
    type StepByStep;
    type Error: std::error::Error;

    fn estimate_geometric_mean_steps(values: &[f64]) -> Result<Self::StepByStep, Self::Error>;
}