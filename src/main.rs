use std::path::PathBuf;

use anyhow::Ok;
use clap::Parser;
use native_dialog::{MessageDialog, MessageType};
use path_absolutize::Absolutize;

#[derive(clap::Parser, Debug)]
struct Args {
    #[arg(short, long)]
    mod_path: Option<PathBuf>,
    #[arg(short, long)]
    install_ext: bool,
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
    let tt2 = cr.create_subkey(".pmp")?;
    tt2.0.set_value("", &"penumbra-installer")?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if args.install_ext {
        install_exts().expect("unable to install file exts")
    } else if args.mod_path.is_none() {
        let confirmation = MessageDialog::new()
            .set_type(MessageType::Info)
            .set_title("https://github.com/pozm/penumbra-installer - file ext installer")
            .set_text("Would you like to register the file extensions?")
            .show_confirm()?;
        if !confirmation {
            return Ok(());
        }
        let install_res = install_exts();
        if let Err(res) = install_res {
            println!("failed to install : {:?}", res);
            return Ok(MessageDialog::new().set_type(MessageType::Info).set_title("https://github.com/pozm/penumbra-installer - Unable to install file ext").set_text("To regiser this program as the default program for .ttmp2 and .pmp files, please rerun this program as an admin.").show_alert()?);
        } else {
            MessageDialog::new()
                .set_type(MessageType::Info)
                .set_title("Successfully installed.")
                .set_text("successfully installed - close to exit")
                .show_alert()?;
        }
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
