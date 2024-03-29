use std::{
    fs::File,
    io::Write,
    rc::Rc
};
use magic_crypt::{new_magic_crypt, MagicCryptTrait};

extern crate directories;
use directories::UserDirs;

extern crate clipboard;

use clipboard::ClipboardProvider;
use clipboard::ClipboardContext;

use slint::{Model, SharedString};

slint::include_modules!();

pub fn main() {
    let data_model = read_from_file();

    let ui = AppWindow::new().unwrap();
    
    ui.on_add_to_database({
        let data_model = data_model.clone();
        move |platform, password| {
            if platform.is_empty() || password.is_empty() {
                return;
            }
            let indx = data_model.iter()
                .position(| d | d.platform.to_lowercase() > platform.to_lowercase())
                .unwrap_or(data_model.row_count());
            data_model.insert(indx, DataItem {platform: SharedString::from(platform), password: SharedString::from(password)});
            write_to_file(data_model.clone());
        }
    });

    ui.on_copy_password_to_clipboard({
        move |password| {
            let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
            ctx.set_contents(password.to_string()).unwrap();
        }
    });

    ui.on_clear_clipboard({
        move || {
            let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
            ctx.set_contents("".to_string()).unwrap();
        }
    });    

    ui.on_delete_from_database({
        let data_model = data_model.clone();
        move |platform| {
            if let Some(indx) = data_model.iter().position(|item| item.platform == platform) {
                data_model.remove(indx);
            }
            write_to_file(data_model.clone()); 
        }
    });

    ui.set_data_model(data_model.into());
    ui.run().unwrap();
}

fn read_from_file() -> Rc<slint::VecModel<DataItem>> {
    let user_dirs = UserDirs::new().unwrap(); 
    let path = user_dirs.document_dir().unwrap().to_str().unwrap().to_string() + "\\PasswordManager\\data.txt";
    let mc = new_magic_crypt!("MyVerySecureKey", 256);
    match std::fs::read_to_string(path) {
        Ok(data) => {
            let data = mc.decrypt_base64_to_string(&data).unwrap();
            Rc::new(slint::VecModel::<DataItem>::from(data.lines()
                .filter_map(|l| l.split_once("\r"))
                .map(|(platform, password)| DataItem {platform: SharedString::from(platform), password: SharedString::from(password)})
                .collect::<Vec<DataItem>>()))
        },
        Err(_) => {
            return Rc::new(slint::VecModel::<DataItem>::from(vec![]))
        } 
    }
}

fn write_to_file(data: Rc<slint::VecModel<DataItem>>) {
    let mc = new_magic_crypt!("MyVerySecureKey", 256);
    let user_dirs = UserDirs::new().unwrap(); 
    let path = user_dirs.document_dir().unwrap().to_str().unwrap().to_string() + "\\PasswordManager\\data.txt";
    if let Err(e) = std::fs::create_dir_all(user_dirs.document_dir().unwrap().to_str().unwrap().to_string() + "\\PasswordManager") {
        println!("Error creating directory: {}", e);
    }
    if let Ok(mut file) = File::create(path) {
        let string: String = data.iter().map(|d| {
            d.platform.to_string() + "\r" + &d.password + "\n"
        }).collect();
        let encrypted = mc.encrypt_str_to_base64(&string);
        if let Err(e) = file.write_all(encrypted.as_bytes()) {
            println!("Error writing to file: {}", e);
        }
    }
}