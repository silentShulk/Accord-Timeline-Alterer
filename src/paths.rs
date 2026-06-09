use std::path::PathBuf;

use std::sync::LazyLock;

use dirs::{config_dir, data_local_dir, home_dir};



pub struct Paths {
    pub executable: PathBuf,
    pub data_file: PathBuf,
    pub settings_file: PathBuf,
    pub uis_dir: PathBuf,
    pub apps_dir: PathBuf,
}

pub static PATHS: LazyLock<Paths> = LazyLock::new(|| {
    return Paths {
        #[cfg(target_os = "linux")]
        executable: home_dir()
            .unwrap()
            .join(".local")
            .join("bin")
            .join("ATA")
            .join("ATA"),
        #[cfg(target_os = "windows")]
        executable: data_local_dir()
            .unwrap()
            .join("Programs")
            .join("ATA")
            .join("ATA.exe"),
        data_file: data_local_dir().unwrap().join("ATA").join("data.json"),
        settings_file: config_dir().unwrap().join("ATA").join("settings.json"),
        uis_dir: data_local_dir().unwrap().join("ATA").join("UIs"),
        apps_dir: data_local_dir().unwrap().join("ATA").join("Apps"),
    };

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    compile_error!("ATA only supports Linux and Windows");
});
