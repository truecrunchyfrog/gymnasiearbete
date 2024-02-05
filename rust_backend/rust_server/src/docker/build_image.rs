use bollard::{
    image::{self, BuildImageOptions, CreateImageOptions},
    service::ImageInspect,
    Docker,
};
use futures::StreamExt;

use super::profiles::ContainerPreset;

pub async fn get_image(
    preset: impl ContainerPreset,
) -> Result<ImageInspect, bollard::errors::Error> {
    let preset_name = preset.info().name;
    let image_name = preset.info().image;

    info!("Building image from preset: {}", preset_name);
    let docker = Docker::connect_with_local_defaults().expect("Failed to connect to docker");

    let image = docker.inspect_image(&image_name).await;
    if let Ok(i) = image {
        return Ok(i); // Image already exists
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
    return docker.inspect_image(&image_name).await;
}
