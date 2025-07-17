use base64::{Engine as _, engine::general_purpose};
use wallguard_common::protobuf::wallguard_service::FileSnapshot;

/// Encodes binary data into a Base64-encoded string.
///
/// # Type Parameters
/// * `T` - Any type that implements `AsRef<[u8]>`, allowing flexible input types such as `Vec<u8>`, `&[u8]`, and `String`.
///
/// # Arguments
/// * `data` - The input data to be encoded. It can be any type that can be referenced as a byte slice (`AsRef<[u8]>`).
///
/// # Returns
/// A `String` containing the Base64-encoded representation of the input data.
pub fn encode_base64<T>(data: T) -> String
where
    T: AsRef<[u8]>,
{
    general_purpose::STANDARD.encode(data)
}

/// Finds a file entry in a given snapshot by its filename.
///
/// # Arguments
/// - `snapshot`: A reference to a collection of `FileSnapshot` entries.
/// - `filename`: The name of the file to search for.
///
/// # Returns
/// - `Some(&FileSnapshot)`: A reference to the matching `FileSnapshot` entry if found.
/// - `None`: If no file with the specified filename exists in the snapshot.
pub fn find_in_snapshot<'a>(
    snapshot: &'a [FileSnapshot],
    filename: &str,
) -> Option<&'a FileSnapshot> {
    snapshot
        .iter()
        .find(|file_data| file_data.filename == filename)
}
