use naga_oil::compose::{ComposableModuleDefinition, ComposableModuleDescriptor, Composer, ComposerError, NagaModuleDescriptor};
use spark_gap::gpu_context::GpuContext;
use std::borrow::Cow;
use std::io::Read;
use wgpu::util::DeviceExt;
use wgpu::Buffer;

pub mod buffers;
mod bullet_render;
pub mod enemy_render;
pub mod floor_render;
pub mod main_render;
pub mod player_render;
mod shader_loader;
mod sprite_render;
mod textures;
mod debug_render;
mod shadow_map;
