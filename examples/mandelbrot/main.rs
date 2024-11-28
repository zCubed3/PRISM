mod kernel;

use kernel::*;

use std::fs::*;
use std::path::Path;

use clap::Parser;

use prism::compute::buffer::*;
use prism::compute::dispatcher::*;
use prism::prelude::*;

#[derive(Parser)]
#[command(name = "Mandelbrot")]
#[command(author = "Liam Reese (zCubed)")]
#[command(version = "0.1")]
struct CmdArguments {
    /// Width of the output framebuffer / image
    #[arg(long, default_value_t = 512)]
    width: usize,

    /// Height of the output framebuffer / image
    #[arg(long, default_value_t = 512)]
    height: usize,

    /// Amount of threads used in rendering (-1 = all threads)
    #[arg(long, default_value_t = -1)]
    num_threads: i32,

    /// Max amount of iterations upon the mandelbrot set
    #[arg(long, default_value_t = 32)]
    max_iter: i32,

    /// Use the smoothed mandelbrot algorithm?
    #[arg(long, default_value_t = true)]
    smoothed: bool,

    /// Scale of the sampling area
    #[arg(long, default_value_t = 1.0)]
    scale: Real,
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

    // We need the aspect ratio for proper scale correction
    let aspect_x = args.width as Real / args.height as Real;

    let kernel = MandelbrotKernel {
        scale: Vector2::new(args.scale, args.scale * aspect_x),
        offset: Vector2::default(),

        max_iter: args.max_iter as Real,
        smoothbrot: args.smoothed,
    };

    let mut buffer = TexelBuffer::new(args.width, args.height);

    {
        let _total = prism::perf::dropwatch::Dropwatch::new_begin("TOTAL RENDER");

        dispatcher.do_tiles(&kernel, &mut buffer, tile_dim, tile_dim);

        let _save_time = prism::perf::dropwatch::Dropwatch::new_begin("IMAGE SAVE");

        buffer.save_as_image(Path::new("./output/mandelbrot.png"));

        println!("Rendered a mandelbrot set into mandelbrot.png!");
    }
}
