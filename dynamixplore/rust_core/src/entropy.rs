// This module is dedicated to computing information-theoretic properties of time series data.

use numpy::PyReadonlyArray1;
use pyo3::prelude::*;
use std::collections::HashMap;

/// # `calculate_phi` (Internal Helper)
///
/// This is the core computational engine for Approximate Entropy (ApEn). It calculates φ^m(r),
/// a statistical measure of the logarithmic frequency of repeating patterns of length `m`
/// within a tolerance `r`.
fn calculate_phi(data: &[f64], m: usize, r: f64) -> f64 {
    let n = data.len();
    if m == 0 || n < m {
        return 0.0;
    }
    let num_vectors = n - m + 1;

    // Create embedding vectors (views into the original data, memory-efficient)
    let vectors: Vec<&[f64]> = (0..num_vectors).map(|i| &data[i..i + m]).collect();
    let mut log_counts_sum = 0.0;

    // For each vector, count its neighbors within tolerance `r`
    for i in 0..num_vectors {
        let template_vec = vectors[i];
        let mut count = 0;
        for j in 0..num_vectors {
            let compare_vec = vectors[j];

            // Calculate Chebyshev distance (maximum coordinate-wise distance)
            let mut max_dist = 0.0;
            for k in 0..m {
                let dist = (template_vec[k] - compare_vec[k]).abs();
                if dist > max_dist {
                    max_dist = dist;
                }
            }

            if max_dist <= r {
                count += 1;
            }
        }

        // Calculate and sum the log probability
        let probability: f64 = (count as f64) / (num_vectors as f64);
        if probability > 0.0 {
            log_counts_sum += probability.ln();
        }
    }
    // Return the average log probability
    log_counts_sum / (num_vectors as f64)
}

/// # Entropy Calculator
///
/// This class provides methods for computing various information-theoretic properties
/// of time series data, such as Approximate Entropy and Permutation Entropy.
#[pyclass]
pub struct Entropy;

impl Entropy {
    // Public constructor for use in main.rs test harness.
    pub fn new() -> Self {
        Entropy
    }
}

#[pymethods]
impl Entropy {
    #[new]
    fn __new__() -> Self {
        Entropy::new()
    }

    /// # Approximate Entropy (ApEn)
    ///
    /// ## Mathematical and Scientific Motivation
    ///
    /// Quantifies the likelihood that runs of patterns that are close for `m` observations
    /// will remain close on the next incremental observation. A low ApEn value indicates
    /// high regularity, while a high value indicates randomness. It is calculated as:
    /// `ApEn(m, r) = φ^m(r) - φ^(m+1)(r)`.
    #[pyo3(signature = (time_series, m, r))]
    fn compute_approximate(
        &self,
        py: Python,
        time_series: PyReadonlyArray1<f64>,
        m: usize,
        r: f64,
    ) -> PyResult<f64> {
        if m < 1 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Embedding dimension 'm' must be at least 1.",
            ));
        }
        if r < 0.0 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Tolerance 'r' cannot be negative.",
            ));
        }

        let data = time_series.as_slice()?;
        // Release the GIL for the potentially long-running computations
        let phi_m = py.allow_threads(|| calculate_phi(data, m, r));
        let phi_m_plus_1 = py.allow_threads(|| calculate_phi(data, m + 1, r));

        Ok(phi_m - phi_m_plus_1)
    }

    /// # Permutation Entropy (PE)
    ///
    /// ## Mathematical and Scientific Motivation
    ///
    /// A robust and computationally efficient method for quantifying the complexity of a
    /// time series based on the probability distribution of ordinal patterns (relative
    /// orderings of values). The result is a normalized entropy score between 0 and 1.
    /// The final value is the Shannon Entropy of the pattern distribution, normalized by log2(m!).
    #[pyo3(signature = (time_series, m, tau))]
    fn compute_permutation(
        &self,
        _py: Python,
        time_series: PyReadonlyArray1<f64>,
        m: usize,
        tau: usize,
    ) -> PyResult<f64> {
        if m < 2 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Embedding dimension 'm' must be at least 2.",
            ));
        }
        if tau < 1 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Time delay 'tau' must be at least 1.",
            ));
        }

        let data = time_series.as_slice()?;
        let n = data.len();
        let required_len = (m - 1) * tau + 1;
        if n < required_len {
            return Ok(0.0); // Not enough data
        }

        // --- 1. Iterate Through Time Series and Create Ordinal Patterns ---
        let mut pattern_counts: HashMap<Vec<usize>, usize> = HashMap::new();
        let num_windows = n - required_len + 1;

        for i in 0..num_windows {
            let window: Vec<f64> = (0..m).map(|j| data[i + j * tau]).collect();
            let mut indexed_window: Vec<(usize, f64)> = window
                .iter()
                .enumerate()
                .map(|(idx, &val)| (idx, val))
                .collect();
            // Sort by value to find the ordinal pattern
            indexed_window.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            let pattern: Vec<usize> = indexed_window.iter().map(|(idx, _)| *idx).collect();
            *pattern_counts.entry(pattern).or_insert(0) += 1;
        }

        if pattern_counts.is_empty() {
            return Ok(0.0);
        }

        // --- 2. Calculate Shannon Entropy from Frequencies ---
        let total_patterns = num_windows as f64;
        let mut entropy = 0.0;
        for count in pattern_counts.values() {
            let probability = (*count as f64) / total_patterns;
            if probability > 0.0 {
                entropy -= probability * probability.log2();
            }
        }

        // --- 3. Normalize the Entropy ---
        let m_factorial = (1..=m).map(|i| i as f64).product::<f64>();
        let max_entropy = m_factorial.log2();
        if max_entropy > 0.0 {
            Ok(entropy / max_entropy)
        } else {
            Ok(0.0)
        }
    }
}
