use futures::StreamExt;
use shiplift::{BuildOptions, Docker};

enum Language {
    C,
    CPP,
}

struct BuildInstructions {
    language: Language, // language of the code
    path: String,       // folder of code
}

async fn build_image(instructions: BuildInstructions) -> Result<bool, shiplift::Error> {
    let docker = Docker::new();
    let options = BuildOptions::builder(instructions.path)
        .tag("shiplift")
        .build();
    let mut stream = docker.images().build(&options);
    while let Some(build_result) = stream.next().await {
        match build_result {
            Ok(_) => return Ok(true),
            Err(e) => return Err(e),
        }
    }
    return Ok(true);
}
