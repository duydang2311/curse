use interprocess::local_socket::{
    GenericFilePath, GenericNamespaced, Name, NameType, ToNsName, prelude::*,
};

const SOCKET_NAME: &str = "curse-daemon.sock";

pub fn get_socket_name() -> eyre::Result<Name<'static>> {
    if GenericNamespaced::is_supported() {
        Ok(SOCKET_NAME.to_ns_name::<GenericNamespaced>()?)
    } else {
        let mut temp_dir = std::env::temp_dir();
        temp_dir.set_file_name(SOCKET_NAME);
        Ok(temp_dir
            .into_string()
            .expect(&format!("failed to get {} file in temp dir", SOCKET_NAME))
            .to_fs_name::<GenericFilePath>()?)
    }
}
