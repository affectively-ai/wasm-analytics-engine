use std::collections::HashMap;

/// Statistical result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatisticsResult {
    pub mean: f64,
    pub median: f64,
    pub min: f64,
    pub max: f64,
    pub percentiles: HashMap<String, f64>,
}

/// Compute statistical aggregations
pub fn compute_statistics(values: &[f64]) -> StatisticsResult {
    if values.is_empty() {
        return StatisticsResult {
            mean: 0.0,
            median: 0.0,
            min: 0.0,
            max: 0.0,
            percentiles: HashMap::new(),
        };
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let sum: f64 = values.iter().sum();
    let mean = sum / values.len() as f64;
    let min = sorted[0];
    let max = sorted[sorted.len() - 1];

    // Calculate median
    let median = if sorted.len() % 2 == 0 {
        (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
    } else {
        sorted[sorted.len() / 2]
    };

    // Calculate percentiles
    let mut percentiles = HashMap::new();
    let percentiles_to_calc = [10, 25, 50, 75, 90, 95, 99];
    
    for p in &percentiles_to_calc {
        let index = ((*p as f64 / 100.0) * (sorted.len() - 1) as f64).round() as usize;
        let value = sorted[index.min(sorted.len() - 1)];
        percentiles.insert(format!("p{}", p), value);
    }

    StatisticsResult {
        mean,
        median,
        min,
        max,
        percentiles,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_statistics() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = compute_statistics(&values);

        assert_eq!(result.mean, 3.0);
        assert_eq!(result.median, 3.0);
        assert_eq!(result.min, 1.0);
        assert_eq!(result.max, 5.0);
        assert!(result.percentiles.contains_key("p50"));
    }

    #[test]
    fn test_compute_statistics_empty() {
        let result = compute_statistics(&[]);
        assert_eq!(result.mean, 0.0);
        assert_eq!(result.median, 0.0);
    }
}
