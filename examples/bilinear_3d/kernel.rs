use prism::compute::kernel::*;
use prism::prelude::*;

#[derive(Default)]
#[doc(hidden)]
pub struct Bilinear3DKernel<'a> {
    pub img: Option<&'a VoxelBuffer<Vector4>>,
}

impl<'a> Kernel<Vector4, VoxelBuffer<Vector4>> for Bilinear3DKernel<'a> {
    fn kernel_exec(&self, input: KernelInput, _buffer: &VoxelBuffer<Vector4>) -> Vector4 {
        let u = input.thread_x as Real / (input.buffer_width as Real - Real::ONE);
        let v = input.thread_y as Real / (input.buffer_height as Real - Real::ONE);
        let w = input.thread_z as Real / (input.buffer_depth as Real - Real::ONE);

        if let Some(img) = self.img {
            let samp = img.trilinear_sample(u, v, w);
            return samp;
        }

        Vector4::default()
    }
}
