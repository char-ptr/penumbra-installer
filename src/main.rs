use std::path::PathBuf;

use anyhow::Ok;
use clap::Parser;
use path_absolutize::Absolutize;
use xdialog::{
    show_message, show_message_error_ok, show_message_info_ok, show_message_yes_no, XDialogBuilder,
};
static ASSETS_DIR: include_dir::Dir = include_dir::include_dir!("./assets");
#[derive(clap::Parser, Debug)]
struct Args {
    #[arg(short, long)]
    mod_path: Option<PathBuf>,
    #[arg(short, long)]
    install_ext: bool,
}

#[cfg(not(windows))]
fn install_exts() -> anyhow::Result<()> {
    use std::{env::current_exe, fs, io::Write, path::Path};

    use anyhow::anyhow;
    use directories::ProjectDirs;

    let base_dirs = directories::BaseDirs::new().ok_or(anyhow!("failed to get user dirs"))?;
    let project_dirs = ProjectDirs::from("ong", "waow", "penumbra-installer")
        .ok_or(anyhow!("failed to get project dir"))?;
    let data_dir = project_dirs.data_local_dir();
    println!("data dir at {data_dir:?}");

    fs::create_dir_all(data_dir)?;
    ASSETS_DIR.extract(data_dir)?;
    let this_program = current_exe()?;

    let sym_linked = data_dir.join("penumbra-installer");
    std::os::unix::fs::symlink(this_program, sym_linked)?;

    let desktop_file_str = r#"[Desktop Entry]
Name=Penumbra Installer
Comment=Penumbra installer
Icon=$STORE/pen.png
Path=$STORE
Exec=$STORE/penumbra-installer -m %u
Terminal=false
Type=Application
StartupNotify=true
StartupWMClass=penumbra-installer
MimeType=application/zip
        "#
    .replace(
        "$STORE",
        data_dir.to_str().ok_or(anyhow!("unable to get data_dir"))?,
    );
    let desktop_path = data_dir.join("penumbra-installer.desktop");
    let mut desktop_file = fs::File::create_new(&desktop_path)?;
    desktop_file.write_all(desktop_file_str.as_bytes())?;
    let to_symlink = base_dirs
        .data_local_dir()
        .join("applications/penumbra-installer.desktop");

    std::os::unix::fs::symlink(desktop_path, to_symlink)?;

    Ok(())
}
#[cfg(windows)]
fn install_exts() -> anyhow::Result<()> {
    use winreg::{enums::HKEY_CLASSES_ROOT, RegKey};

    let cr = RegKey::predef(HKEY_CLASSES_ROOT);
    let pi = cr.create_subkey("penumbra-installer")?;
    pi.0.set_value("", &"penumbra-installer")?;
    let ico = cr.create_subkey("penumbra-installer\\DefaultIcon")?;
    cr.create_subkey("penumbra-installer\\shell")?
        .0
        .set_value("", &"open")?;
    let cmd = cr.create_subkey("penumbra-installer\\shell\\open\\command")?;
    let pather = std::env::current_exe()?;
    let pather = pather.absolutize()?;
    cmd.0
        .set_value("", &format!("\"{}\" -m \"%1\" ", pather.to_string_lossy()))?;

    ico.0
        .set_value("", &format!("{},0", pather.to_string_lossy()))?;
    let tt2 = cr.create_subkey(".ttmp2")?;
    tt2.0.set_value("", &"penumbra-installer")?;
    let tt2 = cr.create_subkey(".ttmp")?;
    tt2.0.set_value("", &"penumbra-installer")?;
    let tt2 = cr.create_subkey(".pmp")?;
    tt2.0.set_value("", &"penumbra-installer")?;
    Ok(())
}
fn logic() -> anyhow::Result<()> {
    let confirmation = show_message_yes_no(
        "https://github.com/pozm/penumbra-installer - file ext installer",
        " install .desktop or extensions in registry",
        "Would you like to register the file extensions?",
        xdialog::XDialogIcon::Information,
    )?;
    if !confirmation {
        return Ok(());
    }
    let install_res = install_exts();
    println!("install outcome = {install_res:?}");
    if let Err(res) = install_res {
        println!("failed to install : {:?}", res);
        println!("if u want to re run on linux delete .desktop at ~/.local/share/applications/penumbra-installer.desktop and ~/.local/share/penumbra-installer");
        show_message_error_ok(
            "https://github.com/pozm/penumbra-installer - Unable to install file ext",
            "failed to install :c",
            "if ur on windows try run as admin",
        );
        // return Ok(xdialog::MessageDialog::new()
        //     .set_type(xdialog::MessageType::Info)
        //     .set_title("https://github.com/pozm/penumbra-installer - Unable to install file ext")
        //     .set_text("To register this program as the default program for .ttmp2, .ttmp and .pmp files, please rerun this program as an admin.")
        //     .show_alert()?);
    } else {
        show_message_yes_no(
            "Successfully Installed",
            "Successfully Installed",
            "successfully installed - close to exit",
            xdialog::XDialogIcon::Information,
        )?;
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if args.install_ext {
        install_exts().expect("unable to install file exts")
    } else if args.mod_path.is_none() {
        return XDialogBuilder::new().run_result(logic);
    }

    let Some(path) = args.mod_path else {
        return Ok(());
    };
    let cli = reqwest::blocking::Client::new();
    let path = path.absolutize()?;
    let jsn = serde_json::json!( {
        "Path" : path.to_string_lossy()
    });
    cli.post("http://localhost:42069/api/installmod")
        .body(jsn.to_string())
        .send()?;
    Ok(())
}
