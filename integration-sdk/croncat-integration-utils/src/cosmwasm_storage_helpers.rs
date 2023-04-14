// The helper methods in this file are taken from the cosmwasm-storage crate.
// At the time of this writing, the end-of-life is nigh, and due to dependency
// architecture problems () we're copying the logic here.
// Full attribution to Confio and the contributors who authored this logic.

pub fn to_length_prefixed_nested(namespaces: &[&[u8]]) -> Vec<u8> {
  let mut size = 0;
  for &namespace in namespaces {
    size += namespace.len() + 2;
  }

  let mut out = Vec::with_capacity(size);
  for &namespace in namespaces {
    out.extend_from_slice(&encode_length(namespace));
    out.extend_from_slice(namespace);
  }
  out
}

fn encode_length(namespace: &[u8]) -> [u8; 2] {
  if namespace.len() > 0xFFFF {
    panic!("only supports namespaces up to length 0xFFFF")
  }
  let length_bytes = (namespace.len() as u32).to_be_bytes();
  [length_bytes[2], length_bytes[3]]
}