mod kernel;

use kernel::*;

use std::fs::*;
use std::path::Path;

use clap::Parser;
use image::{GenericImageView, ImageReader};

use prism::compute::buffer::*;
use prism::compute::dispatcher::*;
use prism::prelude::*;

#[derive(Parser)]
#[command(name = "Bilinear 2D")]
#[command(author = "Liam Reese (zCubed)")]
#[command(version = "0.1")]
struct CmdArguments {
    /// Width of the output framebuffer / image
    #[arg(long, default_value_t = 256)]
    width: usize,

    /// Height of the output framebuffer / image
    #[arg(long, default_value_t = 256)]
    height: usize,

    /// Amount of threads used in rendering (-1 = all threads)
    #[arg(long, default_value_t = -1, )]
    num_threads: i32,

    /// Input file used for bilinear sampling
    #[arg(long, default_value = "test.png")]
    in_file: String,
}

#[doc(hidden)]
fn main() {
    let args: CmdArguments = CmdArguments::parse();
    let dispatcher: Dispatcher = Dispatcher::new(args.num_threads);

    let threads = dispatcher.get_thread_count();
    if threads > 0 {
        println!("Using {} threads to render!", threads);
    } else {
        println!("Using synchronous rendering!");
    }

    let output_path = Path::new("./output");

    if !output_path.exists() {
        create_dir_all(output_path).expect("Failed to create output path!");
    }

    let tile_dim = 16;

    let mut buffer = TexelBuffer::new(args.width, args.height);
    let mut kernel = BilinearKernel::default();

    println!("Loading test.png...");

    let reader = ImageReader::open("test.png").expect("Please provide test.png!");
    let image = reader.decode().expect("Failed to decode image");

    let mut img = TexelBuffer::<Vector4>::new(image.width() as usize, image.height() as usize);

    for x in 0..img.get_buffer_width() {
        for y in 0..img.get_buffer_height() {
            let pixel = image.get_pixel(x as u32, y as u32).0;

            let color = Vector4::new(
                pixel[0] as Real / 255.0,
                pixel[1] as Real / 255.0,
                pixel[2] as Real / 255.0,
                pixel[3] as Real / 255.0,
            );

            img.buffer_write(x, y, 0, color);
        }
    }

    kernel.img = Some(&img);

    {
        let _total = prism::perf::dropwatch::Dropwatch::new_begin("TOTAL RENDER");

        dispatcher.do_tiles(&kernel, &mut buffer, tile_dim, tile_dim);

        let _save_time = prism::perf::dropwatch::Dropwatch::new_begin("IMAGE SAVE");

        buffer.save_as_image(Path::new("./output/bilinear_output.png"));

        println!(
            "Resampled {} from {}x{} to {}x{}!",
            args.in_file,
            img.get_buffer_width(),
            img.get_buffer_height(),
            args.width,
            args.height
        );
    }
}
