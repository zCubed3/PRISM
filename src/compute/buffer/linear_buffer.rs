use super::*;
use rgml::vector::Vector;

#[cfg(feature = "image")]
use image::*;

/// A linear "line" shaped buffer
#[cfg_attr(feature = "serialization", derive(Serialize))]
#[derive(Clone)]
#[repr(C)]
pub struct LinearBuffer<T: BufferData> {
    _backing: Vec<T>,
    _size: usize,
}

impl<T: BufferData> LinearBuffer<T> {
    pub fn new(size: usize) -> Self {
        return Self {
            _backing: vec![T::default(); size],
            _size: size,
        };
    }

    fn copy_slice(&self, x: usize, width: usize) -> Self {
        return LinearBuffer {
            _size: width,
            _backing: Vec::from(&self._backing[x..x + width]),
        };
    }
}

impl<T: BufferData> DataBounds for LinearBuffer<T> {}
impl<T: BufferData> Buffer<T> for LinearBuffer<T> {
    fn get_buffer_shape(&self) -> BufferShape {
        return BufferShape::Shape1D;
    }

    fn get_buffer_width(&self) -> usize {
        return self._size;
    }

    fn buffer_new(width: usize, _height: usize, _depth: usize) -> Self {
        return LinearBuffer::new(width);
    }

    fn buffer_write(&mut self, x: usize, _y: usize, _z: usize, value: T) {
        assert!(x < self._size);
        self._backing[x] = value;
    }

    fn buffer_read(&self, x: usize, _y: usize, _z: usize) -> T {
        assert!(x < self._size);
        return self._backing[x];
    }
}

/// Allows saving this [LinearBuffer] as a colored texture
#[cfg(feature = "image")]
impl LinearBuffer<Vector<Real, 4>> {
    pub fn save_as_image<P: AsRef<Path>>(&self, path: P) {
        let mut image = RgbaImage::new(self._size as u32, 1);

        for x in 0..self._size {
            let raw_color = self.buffer_read(x, 0, 0);
            image.put_pixel(
                x as u32,
                0,
                Rgba([
                    (raw_color[0].rl_saturate() * 255.0).rl_round() as u8,
                    (raw_color[1].rl_saturate() * 255.0).rl_round() as u8,
                    (raw_color[2].rl_saturate() * 255.0).rl_round() as u8,
                    (raw_color[3].rl_saturate() * 255.0).rl_round() as u8,
                ]),
            );
        }

        image.save(path).expect("Failed to save image");
    }
}
