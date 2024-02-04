use bollard::{
    image::{BuildImageOptions, CreateImageOptions},
    Docker,
};
use futures::StreamExt;

use super::profiles::ContainerPreset;
use crate::Result;

pub async fn build_image_from_preset(preset: impl ContainerPreset) -> Result<()> {
    info!("Building image from preset: {}", preset.name());
    let docker = Docker::connect_with_local_defaults().expect("Failed to connect to docker");

    let image = docker.inspect_image(preset.name()).await;
    if let Ok(_) = image {
        return Ok(()); // Image already exists
    }

    let options: CreateImageOptions<String> = preset.create_image_options();
    let stream = docker.create_image(Some(options), None, None);
    stream
        .for_each(|message| async {
            match message {
                Ok(msg) => {
                    if (msg.progress.is_some()) {
                        println!("{:?}", msg.progress);
                    }
                }
                Err(e) => {
                    error!("Build error: {:?}", e);
                }
            }
        })
        .await;
    Ok(())
}
