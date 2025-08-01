// In src/entropy.rs
// This module is dedicated to computing information-theoretic properties of time series data.

use pyo3::prelude::*;
use numpy::PyReadonlyArray1;
use std::collections::HashMap;

/// # Permutation Entropy (PE)
///
/// ## Mathematical and Scientific Motivation
///
/// Permutation Entropy is a robust and computationally efficient method for quantifying the
/// complexity of a time series. It was introduced by Bandt and Pompe in 2002.
///
/// The core idea is to move from the raw values of a time series to the "symbolic" space
/// of ordinal patterns. Instead of asking "what are the values?", we ask "what is the
/// relative ordering of the values?". This makes the measure highly resilient to noise and
/// observational errors, as a small amount of noise is unlikely to change the ordering
/// of values in a local window.
///
/// PE measures the complexity of a system by analyzing the probability distribution of these
/// ordinal patterns.
///   - A **simple, regular system** (like a sine wave) will only ever produce a few distinct
///     patterns, leading to a low-entropy, concentrated probability distribution.
///   - A **complex, chaotic, or random system** will produce many different patterns with
///     similar probabilities, leading to a high-entropy, uniform probability distribution.
///
/// The final value is a normalized entropy score between 0 (perfectly ordered) and 1 (maximally complex/random).
///
/// ## Shannon Entropy Formula
///
/// The calculation uses the classic Shannon entropy formula, normalized by the maximum possible entropy:
///
///     H(P) = - Σ [p_i * log2(p_i)] / log2(m!)
///
/// Where:
/// - `P` is the probability distribution of the observed ordinal patterns.
/// - `p_i` is the probability of the i-th pattern.
/// - `m` is the embedding dimension (the length of the patterns).
/// - `m!` (m factorial) is the total number of possible patterns of length `m`.

#[pyfunction]
#[pyo3(signature = (time_series, m, tau))]
pub fn compute_permutation_entropy(
    py: Python,
    time_series: PyReadonlyArray1<f64>,
    m: usize, // Embedding dimension (pattern length)
    tau: usize, // Time delay
) -> PyResult<f64> {
    // --- Input Validation ---
    if m < 2 {
        return Err(pyo3::exceptions::PyValueError::new_err("Embedding dimension 'm' must be at least 2."));
    }
    if tau < 1 {
        return Err(pyo3::exceptions::PyValueError::new_err("Time delay 'tau' must be at least 1."));
    }

    let data = time_series.as_slice()?;
    let n = data.len();

    // The number of points required to form at least one window of length 'm' with delay 'tau'.
    let required_len = (m - 1) * tau + 1;
    if n < required_len {
        return Ok(0.0); // Not enough data to form any patterns, entropy is zero.
    }

    // A HashMap to store the frequency of each unique ordinal pattern.
    // The key is a Vec<usize> representing the permutation (e.g., [1, 2, 0] for the pattern in `[5.1, 8.3, 2.7]`).
    let mut pattern_counts: HashMap<Vec<usize>, usize> = HashMap::new();

    // --- 1. Iterate Through Time Series and Create Ordinal Patterns ---
    let num_windows = n - required_len + 1;
    for i in 0..num_windows {
        // Create a window (sub-vector) of the time series data.
        // This window is of length 'm' with a delay of 'tau' between elements.
        let window: Vec<f64> = (0..m).map(|j| data[i + j * tau]).collect();

        // --- 2. Determine the Ordinal Pattern for the Window ---
        // To find the ordinal pattern, we create a vector of (index, value) pairs,
        // sort it by value, and then extract the original indices.
        let mut indexed_window: Vec<(usize, f64)> = window.iter().enumerate().map(|(idx, &val)| (idx, val)).collect();
        
        // Sort by value (the f64 part of the tuple).
        // `sort_by` is used instead of `sort_unstable_by` to maintain order for equal elements,
        // which is the standard convention for permutation entropy.
        indexed_window.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        // The ordinal pattern is the sequence of original indices after sorting.
        let pattern: Vec<usize> = indexed_window.iter().map(|(idx, _)| *idx).collect();

        // --- 3. Update Pattern Frequencies ---
        // Use the pattern as a key in the HashMap and increment its count.
        *pattern_counts.entry(pattern).or_insert(0) += 1;
    }

    // --- 4. Calculate Shannon Entropy from Frequencies ---
    if pattern_counts.is_empty() {
        return Ok(0.0);
    }

    let total_patterns = num_windows as f64;
    let mut entropy = 0.0;

    for count in pattern_counts.values() {
        // Calculate the probability of the current pattern.
        let probability = (*count as f64) / total_patterns;
        if probability > 0.0 {
            entropy -= probability * probability.log2();
        }
    }

    // --- 5. Normalize the Entropy ---
    // The maximum possible entropy for a given dimension 'm' is log2(m!).
    // We calculate m! using a simple loop.
    let m_factorial = (1..=m).map(|i| i as f64).product::<f64>();
    let max_entropy = m_factorial.log2();

    // Avoid division by zero if max_entropy is 0 (e.g., if m=1, though we validate against that).
    if max_entropy > 0.0 {
        Ok(entropy / max_entropy)
    } else {
        Ok(0.0)
    }
}
