use std::borrow::Cow;
use std::io::Read;
use naga_oil::compose::{ComposableModuleDefinition, ComposableModuleDescriptor, Composer, ComposerError, NagaModuleDescriptor};

pub mod main_render;
mod player_render;
mod floor_render;

fn try_every_shader_file(
    composer: &mut Composer,
    for_shader: &str,
    shader_dir: &str,
    max_iters: usize,
) -> anyhow::Result<()> {
    let mut try_again = true;
    let mut iters = 0;
    while try_again {
        try_again = false;
        let shader_dir = std::fs::read_dir(shader_dir)?;
        for entry in shader_dir {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if path.extension().unwrap() != "wgsl" {
                    continue;
                }
                if path.to_str().unwrap() == for_shader {
                    continue;
                }

                let mut file = std::fs::File::open(&path)?;
                let mut shader = String::new();

                file.read_to_string(&mut shader)?;

                let result =  composer.add_composable_module(ComposableModuleDescriptor {
                        file_path: path.to_str().unwrap(),
                        source: shader.as_str(),
                        ..Default::default()
                    });

                match result {
                    Ok(_) => {try_again = false}
                    Err(e) => {
                        println!("composer error: {:#?}\n", e);
                        try_again = true
                    }
                }

            } else if path.is_dir() {
                try_every_shader_file(composer, for_shader, path.to_str().unwrap(), max_iters)?;
            }
        }

        iters += 1;

        if iters > max_iters {
            return Err(anyhow::anyhow!("Max iterations reached"));
        }
    }

    Ok(())
}

pub fn preprocess_shader(
    file_path: &'static str,
    base_include_path: &'static str,
) -> wgpu::ShaderModuleDescriptor<'static> {
    let mut composer = Composer::non_validating();

    println!("file_path: {:?}", &file_path);
    let shader = std::fs::read_to_string(file_path).unwrap();

    try_every_shader_file(&mut composer, file_path, base_include_path, 100).unwrap();

    let module = composer
        .make_naga_module(NagaModuleDescriptor {
            file_path,
            source: shader.as_str(),
            ..Default::default()
        })
        .unwrap_or_else(|e| {
            log::error!("Failed to compile shader {}: {}", file_path, e.inner);
            panic!("{}", e.inner);
        });

    wgpu::ShaderModuleDescriptor {
        label: Some(file_path),
        source: wgpu::ShaderSource::Naga(Cow::Owned(module)),
    }
}

#[macro_export]
macro_rules! load_shader {
    ($file_path:literal) => {
        $crate::render::preprocess_shader(
            concat!("shaders/", $file_path),
            "shaders",
        )
    };
}