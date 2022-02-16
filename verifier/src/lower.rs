
use nova_software_common as common;

pub fn verify(mid: crate::upper::ConfigFile) -> Result<common::ConfigFile, crate::Error> {

    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Used for format compatibility guarantees. Call with real encoded config files once we have
    /// a stable version to maintain
    fn assert_config_eq(bytes: Vec<u8>, config: common::ConfigFile) {
        let decoded: common::ConfigFile = postcard::from_bytes(bytes.as_slice()).unwrap();
        assert_eq!(decoded, config);
    }
}
