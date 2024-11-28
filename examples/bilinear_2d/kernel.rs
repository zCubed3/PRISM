use prism::compute::kernel::*;
use prism::prelude::*;

#[derive(Default)]
#[doc(hidden)]
pub struct BilinearKernel<'a> {
    pub img: Option<&'a TexelBuffer<Vector4>>,
}

impl<'a> Kernel<Vector4, TexelBuffer<Vector4>> for BilinearKernel<'a> {
    fn kernel_exec(&self, input: KernelInput, _buffer: &TexelBuffer<Vector4>) -> Vector4 {
        let u = input.thread_x as Real / (input.buffer_width as Real - Real::ONE);
        let v = input.thread_y as Real / (input.buffer_height as Real - Real::ONE);

        if let Some(img) = self.img {
            let samp = img.bilinear_sample(u, v);
            return samp;
        }

        Vector4::default()
    }
}
