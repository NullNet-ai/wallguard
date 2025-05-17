use rand::Rng;
use rand::distr::Alphanumeric;

/// Generates a random alphanumeric string of the given length.
///
/// # Arguments
/// * `length` - The length of the string to generate.
///
/// # Returns
/// A `String` containing random alphanumeric characters.
pub fn generate_random_string(length: usize) -> String {
    rand::rngs::ThreadRng::default()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}
