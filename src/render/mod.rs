use naga_oil::compose::{ComposableModuleDefinition, ComposableModuleDescriptor, Composer, ComposerError, NagaModuleDescriptor};
use spark_gap::gpu_context::GpuContext;
use std::borrow::Cow;
use std::io::Read;
use wgpu::util::DeviceExt;
use wgpu::Buffer;

pub mod buffers;
pub mod enemy_render;
pub mod floor_render;
pub mod main_render;
pub mod player_render;
mod sprite_render;
mod bullet_render;
mod shader_loader;
mod textures;

