pub trait EstimateGeometricMean {
    type Error: std::error::Error;
    fn estimate_geometric_mean(values: &[f64]) -> Result<f64, Self::Error>;
}