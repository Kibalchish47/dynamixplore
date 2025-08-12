// In src/entropy.rs
// This module is dedicated to computing information-theoretic properties of time series data.

use pyo3::prelude::*;
use numpy::PyReadonlyArray1;
use std::collections::HashMap;

// --- Helper function for Approximate Entropy ---

/// # `calculate_phi()`
///
/// ## Purpose
/// This is the core computational engine for Approximate Entropy (ApEn). It calculates a statistical
/// measure of regularity, denoted as φ^m(r).
///
/// ## Algorithm
/// It works by measuring the logarithmic frequency of repeating patterns of a given length (`m`)
/// within a tolerance radius (`r`).
///
/// 1.  Form Embedding Vectors: The input time series is converted into a set of `N-m+1`
///     overlapping vectors of length `m`. For example, the series `[1, 2, 3, 4]` with `m=2`
///     becomes `[[1, 2], [2, 3], [3, 4]]`.
///
/// 2.  Count Similar Patterns: For each embedding vector `x_i`, we count how many other
///     vectors `x_j` are "close" to it. Closeness is defined by the Chebyshev distance
///     (the maximum absolute difference between corresponding elements) being less than
///     the tolerance radius `r`. Let this count be `C_i(r)`.
///
/// 3.  Calculate Logarithmic Frequencies: For each vector `x_i`, we calculate the
///     logarithm of its pattern frequency: `log(C_i(r) / (N-m+1))`.
///
/// 4.  Average the Results: The final φ value is the average of these logarithmic
///     frequencies over all `i`.
///
/// φ^m(r) = (1 / (N-m+1)) * Σ [log(C_i(r) / (N-m+1))]

fn calculate_phi(data: &[f64], m: usize, r: f64) -> f64 {
    let n = data.len();
    if m == 0 || n < m {
        return 0.0;
    }
    let num_vectors = n - m + 1;

    // Create the embedding vectors
    let vectors: Vec<&[f64]> = (0..num_vectors).map(|i| &data[i..i + m]).collect();
    let mut log_counts_sum = 0.0;

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
        
        let probability = (count as f64) / (num_vectors as f64);
        if probability > 0.0 {
            log_counts_sum += probability.ln(); // Using natural log as is standard
        }
    }

    log_counts_sum / (num_vectors as f64)
}


// --- Public PyFunctions ---

/// # Approximate Entropy (ApEn)
///
/// ## Mathematical and Scientific Motivation
///
/// Approximate Entropy was introduced by Steve Pincus in 1991 as a measure of regularity
/// and predictability in a time series. It quantifies the likelihood that runs of patterns
/// that are close for `m` observations will remain close on the next incremental observation.
///
/// A low ApEn value indicates a high degree of regularity and predictability in the time series.
/// A high ApEn value indicates a high degree of randomness and unpredictability.
///
/// It answers the question: "Given a short pattern of length `m`, how much new information
/// (or uncertainty) is introduced when we extend the pattern to length `m+1`?"
///
/// The final value is calculated as: ApEn(m, r) = φ^m(r) - φ^(m+1)(r)
///
/// This difference represents the average increase in conditional probability from one
/// dimension to the next, providing a robust measure of the system's entropy. It is widely
/// used in biomedical signal processing (e.g., for heart rate variability).

#[pyfunction]
#[pyo3(signature = (time_series, m, r))]
pub fn compute_approximate_entropy(
    py: Python,
    time_series: PyReadonlyArray1<f64>,
    m: usize, // Embedding dimension
    r: f64,   // Tolerance radius
) -> PyResult<f64> {
    if m < 1 {
        return Err(pyo3::exceptions::PyValueError::new_err("Embedding dimension 'm' must be at least 1."));
    }
    if r < 0.0 {
        return Err(pyo3::exceptions::PyValueError::new_err("Tolerance 'r' cannot be negative."));
    }

    let data = time_series.as_slice()?;
    
    // 2. Invoke the helper procedure for dimension `m`.
    let phi_m = py.allow_threads(|| calculate_phi(data, m, r));
    
    // 3. Invoke it again for dimension `m+1`.
    let phi_m_plus_1 = py.allow_threads(|| calculate_phi(data, m + 1, r));

    // 4. The final result is the difference.
    Ok(phi_m - phi_m_plus_1)
}


/// # Permutation Entropy (PE)
///
/// ## Mathematical and Scientific Motivation
///
/// Permutation Entropy is a robust and computationally efficient method for quantifying the
/// complexity of a time series, introduced by Bandt and Pompe in 2002. It is based on
/// analyzing the probability distribution of ordinal patterns (relative orderings of values)
/// in a time series, making it highly resilient to noise.
///
/// The final value is a normalized entropy score between 0 (perfectly ordered) and 1 (maximally complex/random).
///
/// ## Shannon Entropy Formula
/// H(P) = - Σ [p_i * log2(p_i)] / log2(m!)

#[pyfunction]
#[pyo3(signature = (time_series, m, tau))]
pub fn compute_permutation_entropy(
    _py: Python,
    time_series: PyReadonlyArray1<f64>,
    m: usize, // Embedding dimension (pattern length)
    tau: usize, // Time delay
) -> PyResult<f64> {
    if m < 2 {
        return Err(pyo3::exceptions::PyValueError::new_err("Embedding dimension 'm' must be at least 2."));
    }
    if tau < 1 {
        return Err(pyo3::exceptions::PyValueError::new_err("Time delay 'tau' must be at least 1."));
    }

    let data = time_series.as_slice()?;
    let n = data.len();
    let required_len = (m - 1) * tau + 1;
    if n < required_len {
        return Ok(0.0);
    }

    let mut pattern_counts: HashMap<Vec<usize>, usize> = HashMap::new();
    let num_windows = n - required_len + 1;

    for i in 0..num_windows {
        let window: Vec<f64> = (0..m).map(|j| data[i + j * tau]).collect();
        let mut indexed_window: Vec<(usize, f64)> = window.iter().enumerate().map(|(idx, &val)| (idx, val)).collect();
        indexed_window.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        let pattern: Vec<usize> = indexed_window.iter().map(|(idx, _)| *idx).collect();
        *pattern_counts.entry(pattern).or_insert(0) += 1;
    }

    if pattern_counts.is_empty() {
        return Ok(0.0);
    }

    let total_patterns = num_windows as f64;
    let mut entropy = 0.0;
    for count in pattern_counts.values() {
        let probability = (*count as f64) / total_patterns;
        if probability > 0.0 {
            entropy -= probability * probability.log2();
        }
    }

    let m_factorial = (1..=m).map(|i| i as f64).product::<f64>();
    let max_entropy = m_factorial.log2();

    if max_entropy > 0.0 {
        Ok(entropy / max_entropy)
    } else {
        Ok(0.0)
    }
}
