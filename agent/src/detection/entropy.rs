use std::path::Path;
use crate::error::{AgentError, Result};

/// Compute Shannon entropy of the first `sample_bytes` of the file at `path`.
/// Returns a value in [0.0, 8.0] bits/byte.
/// Returns `None` if the file is empty or cannot be read.
pub async fn file_entropy(
    path: &Path,
    sample_bytes: usize,
    max_file_bytes: u64,
) -> Result<Option<f64>> {
    // Skip files that are too large
    let metadata = tokio::fs::metadata(path).await.map_err(|e| AgentError::io(path.display().to_string(), e))?;
    if metadata.len() > max_file_bytes || metadata.len() == 0 {
        return Ok(None);
    }

    let bytes = read_sample(path, sample_bytes).await?;
    if bytes.is_empty() {
        return Ok(None);
    }

    Ok(Some(shannon(&bytes)))
}

/// Read up to `limit` bytes from a file.
async fn read_sample(path: &Path, limit: usize) -> Result<Vec<u8>> {
    use tokio::io::AsyncReadExt;
    let mut f = tokio::fs::File::open(path)
        .await
        .map_err(|e| AgentError::io(path.display().to_string(), e))?;

    let mut buf = vec![0u8; limit];
    let n = f.read(&mut buf)
        .await
        .map_err(|e| AgentError::io(path.display().to_string(), e))?;
    buf.truncate(n);
    Ok(buf)
}

/// Pure Shannon entropy calculation over a byte slice.
/// H = -∑ p(b) * log₂(p(b))
pub fn shannon(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    let mut counts = [0u32; 256];
    for &b in data {
        counts[b as usize] += 1;
    }
    let len = data.len() as f64;
    counts.iter().fold(0.0_f64, |acc, &c| {
        if c == 0 {
            acc
        } else {
            let p = c as f64 / len;
            acc - p * p.log2()
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_data_is_zero() {
        assert_eq!(shannon(&[]), 0.0);
    }

    #[test]
    fn uniform_byte_is_zero() {
        let data = vec![0xABu8; 1000];
        assert!(shannon(&data) < 0.001);
    }

    #[test]
    fn all_values_is_eight() {
        // 256 distinct bytes → entropy = 8.0
        let data: Vec<u8> = (0u8..=255).collect();
        let h = shannon(&data);
        assert!((h - 8.0).abs() < 0.01, "entropy was {h}");
    }

    #[test]
    fn random_like_is_high() {
        // pseudo-random — just counting distinct values
        let data: Vec<u8> = (0..=255).cycle().take(4096).map(|x: u32| (x * 7 + 3) as u8).collect();
        let h = shannon(&data);
        assert!(h > 7.0, "expected high entropy, got {h}");
    }

    #[test]
    fn english_text_is_moderate() {
        let text = b"The quick brown fox jumps over the lazy dog. \
                     Pack my box with five dozen liquor jugs.";
        let h = shannon(text);
        // English text typically 4–5 bits/byte
        assert!(h > 3.5 && h < 6.0, "expected moderate entropy, got {h}");
    }
}
