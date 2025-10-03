use statrs::distribution::{ContinuousCDF, Normal};
use std::f64;

pub enum Bucket {
    Lte(usize),
    Between(usize, usize),
    Gte(usize),
}

pub fn bucket_probs(buckets: Vec<Bucket>, mu: f64, sigma: f64) -> Vec<f64> {
    let normal = Normal::new(mu, sigma).unwrap();

    let mut probs = Vec::with_capacity(buckets.len());
    for bucket in buckets {
        let prob = match bucket {
            Bucket::Lte(lt) => normal.cdf(lt as f64 + 1.0 - f64::EPSILON),
            Bucket::Between(start, stop) => {
                normal.cdf(stop as f64 + 1.0 - f64::EPSILON) - normal.cdf(start as f64)
            }
            Bucket::Gte(gt) => 1.0 - normal.cdf(gt as f64),
        };
        probs.push(prob);
    }
    probs
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_normal_probabilities() {
        let buckets = vec![
            Bucket::Lte(60),
            Bucket::Between(61, 62),
            Bucket::Between(63, 64),
            Bucket::Between(65, 66),
            Bucket::Gte(67),
        ];
        let probs = bucket_probs(buckets, 64.0, 1.5);
        eprintln!("{:?}", probs);
        assert_relative_eq!(probs[0] * 100.0, 2.27, epsilon = 1e-2);
        assert_relative_eq!(probs[1] * 100.0, 22.97, epsilon = 1e-2);
        assert_relative_eq!(probs[2] * 100.0, 49.50, epsilon = 1e-2);
        assert_relative_eq!(probs[3] * 100.0, 22.97, epsilon = 1e-2);
        assert_relative_eq!(probs[4] * 100.0, 2.27, epsilon = 1e-2);

        assert_relative_eq!(probs.iter().sum::<f64>(), 1.0, epsilon = 1e-6);
    }

    #[test]
    fn test_normal_no_std() {
        let buckets = vec![
            Bucket::Lte(60),
            Bucket::Between(61, 62),
            Bucket::Between(63, 64),
            Bucket::Between(65, 66),
            Bucket::Gte(67),
        ];
        let probs = bucket_probs(buckets, 64.0, 0.000001);
        eprintln!("{:?}", probs);
        assert_relative_eq!(probs[0] * 100.0, 0., epsilon = 1e-2);
        assert_relative_eq!(probs[1] * 100.0, 0., epsilon = 1e-2);
        assert_relative_eq!(probs[2] * 100.0, 100., epsilon = 1e-2);
        assert_relative_eq!(probs[3] * 100.0, 0., epsilon = 1e-2);
        assert_relative_eq!(probs[4] * 100.0, 0., epsilon = 1e-2);
    }
}
