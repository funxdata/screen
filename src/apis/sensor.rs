use xcores::application::app::Application;
use crate::service::sensor::sensor;

pub fn signal(app:&mut Application){
    let sen = sensor::new();
    let signals = sen.get_signal();
    match signals {
        Some(uuid)=>{
            app.reponse.success(uuid);
        }
        None=>{
            app.reponse.success("no".to_string());
        }
    }
}