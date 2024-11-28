use crate::prelude::*;
use std::fs::File;
use std::path::Path;

#[cfg(feature = "serialization")]
use bincode::serialize_into;

#[cfg(feature = "serialization")]
use serde::*;

/// Denotes the shape of this buffer
#[repr(u8)]
pub enum BufferShape {
    // 1D, aka "linear" (DX / GL = "Texture1D")
    Shape1D,
    // 2D, aka "texel" (DX / GL = "Texture2D")
    Shape2D,
    // 3D, aka "voxel" (DX / GL = "Texture3D")
    Shape3D,
}

#[cfg(feature = "serialization")]
pub trait DataBounds: Serialize {}

#[cfg(not(feature = "serialization"))]
pub trait DataBounds {}

/// Bounds required for a type to be allowed inside a buffer as data
pub trait BufferData: Sized + Clone + Copy + Default + DataBounds {}

/// Allows this data to be interpolated by [Real]
pub trait LerpableData: BufferData {
    fn linear_interpolate(&self, to: Self, alpha: Real) -> Self;
}

/// Implements BufferData and required traits for the given type
/// This works best with arbitrary types
///
/// Refer to [BufferData] for trait bounds
#[macro_export]
macro_rules! impl_buffer_data {
    ($tipe:ty) => {
        impl DataBounds for $tipe {}
        impl BufferData for $tipe {}
    };
}

macro_rules! impl_rl_lerpable_data {
    ($tipe:ty, $lerp_call:ident) => {
        impl LerpableData for $tipe {
            fn linear_interpolate(&self, to: $tipe, alpha: Real) -> $tipe {
                return self.$lerp_call(to, alpha);
            }
        }
    };
}

impl_buffer_data!(f32);
impl_buffer_data!(f64);
impl_buffer_data!(Vector2F32);
impl_buffer_data!(Vector3F32);
impl_buffer_data!(Vector4F32);
impl_buffer_data!(Vector2F64);
impl_buffer_data!(Vector3F64);
impl_buffer_data!(Vector4F64);

impl_rl_lerpable_data!(Real, rl_lerp);
impl_rl_lerpable_data!(Vector2, lerp);
impl_rl_lerpable_data!(Vector3, lerp);
impl_rl_lerpable_data!(Vector4, lerp);

/// Defines a shaped buffer
///
/// Refer to: [LinearBuffer], [TexelBuffer], [VoxelBuffer]
pub trait Buffer<T: BufferData>: Clone + DataBounds {
    /// Returns the shape of this buffer (Kernels expect a certain shape)
    fn get_buffer_shape(&self) -> BufferShape;

    /// Returns the width of this buffer
    fn get_buffer_width(&self) -> usize {
        return 1;
    }

    /// Returns the height of this buffer (If buffer is [BufferShape::Shape1D], it will be equal to 1)
    fn get_buffer_height(&self) -> usize {
        return 1;
    }

    /// Returns the depth of this buffer (If buffer is [BufferShape::Shape2D] or [BufferShape::Shape1D], it will be equal to 1)
    fn get_buffer_depth(&self) -> usize {
        return 1;
    }

    fn buffer_new(width: usize, height: usize, depth: usize) -> Self;

    fn buffer_write(&mut self, x: usize, y: usize, z: usize, value: T);
    fn buffer_read(&self, x: usize, y: usize, z: usize) -> T;

    /// Saves this buffer to disk
    #[cfg(feature = "serialization")]
    fn buffer_save(&self, path: &Path) {
        let mut file = File::open(path).expect("Failed to open file!");
        serialize_into(&mut file, self).expect("Failed to write buffer!");
    }
}

/// Linear buffer implementation
pub mod linear_buffer;
// Texel buffer implementation
pub mod texel_buffer;
// Voxel buffer implementation
pub mod voxel_buffer;

pub use linear_buffer::*;
pub use texel_buffer::*;
pub use voxel_buffer::*;
