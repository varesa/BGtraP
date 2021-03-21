use std::fmt;

#[derive(PartialEq)]
pub struct Prefix {
    length: u8,
    prefix: [u8; 4],
}

impl fmt::Debug for Prefix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!("{}.{}.{}.{}/{}", self.prefix[0], self.prefix[1], self.prefix[2], self.prefix[3], self.length))
    }
}

pub(crate) fn extract_prefixes(data: &[u8]) -> Vec<Prefix> {
    let mut routes: Vec<Prefix> = Vec::new();

    let mut bytes_left = data.len();
    let mut i = 0; // Start after the "withdrawn length" field

    while bytes_left > 0 {
        let prefix_length = data[i];
        let prefix_octets = (prefix_length as f32 / 8f32).ceil() as usize;
        let mut prefix = [0 as u8; 4];
        prefix[0..prefix_octets].copy_from_slice(&data[i+1 .. i+1+prefix_octets]);
        routes.push(Prefix {
            prefix: prefix,
            length: prefix_length,
        });
        i += 1 + prefix_octets;
        bytes_left -= 1 + prefix_octets as usize;
    }

    return routes;
}

pub(crate) fn compile_prefixes(prefixes: Vec<Prefix>) -> Vec<u8> {
    let mut data = Vec::new();

    for prefix in prefixes {
        data.push(prefix.length);
        for octet in 0 .. (prefix.length as f32 / 8f32).ceil() as usize {
            data.push(prefix.prefix[octet]);
        }
    }

    return data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_prefixes1() {
        let data = [32 as u8, 1, 2, 3, 4];
        let prefixes = extract_prefixes(&data[..]);
        assert_eq!(prefixes.len(), 1);
        assert_eq!(prefixes[0], Prefix { length: 32, prefix: [1, 2, 3, 4]});
    }

    #[test]
    fn test_extract_prefixes2() {
        let data = [32 as u8, 1, 2, 3, 4, 12, 172, 16];
        let prefixes = extract_prefixes(&data[..]);
        assert_eq!(prefixes.len(), 2);
        assert_eq!(prefixes[1], Prefix { length: 12, prefix: [172, 16, 0, 0]});
    }

    #[test]
    fn compile_prefixes1() {
        let prefixes = vec![Prefix { length: 32, prefix: [1, 2, 3, 4]}];
        let data = compile_prefixes(prefixes);
        assert_eq!(data, vec![32 as u8, 1, 2, 3, 4]);
    }

    #[test]
    fn compile_prefixes2() {
        let prefixes = vec![
            Prefix { length: 32, prefix: [1, 2, 3, 4]},
            Prefix { length: 12, prefix: [172, 16, 0, 0]},
        ];
        let data = compile_prefixes(prefixes);
        assert_eq!(data, vec![32 as u8, 1, 2, 3, 4, 12, 172, 16]);
    }
}