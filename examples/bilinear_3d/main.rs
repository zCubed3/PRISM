mod kernel;

use kernel::*;

use std::fs::*;
use std::path::Path;

use clap::Parser;

use prism::compute::buffer::*;
use prism::compute::dispatcher::*;
use prism::prelude::*;

#[derive(Parser)]
#[command(name = "Bilinear 3D")]
#[command(author = "Liam Reese (zCubed)")]
#[command(version = "0.1")]
struct CmdArguments {
    /// Width of the output framebuffer / image
    #[arg(long, default_value_t = 32)]
    width: usize,

    /// Height of the output framebuffer / image
    #[arg(long, default_value_t = 32)]
    height: usize,

    /// Depth of the output framebuffer / image
    #[arg(long, default_value_t = 32)]
    depth: usize,

    /// Amount of threads used in rendering (-1 = all threads)
    #[arg(long, default_value_t = -1, )]
    num_threads: i32,

    /// Input file used for bilinear sampling
    #[arg(long, default_value = "test.png")]
    in_file: String,
}

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

    let mut buffer = VoxelBuffer::new(args.width, args.height, args.depth);
    let mut kernel = Bilinear3DKernel::default();

    let mut img = VoxelBuffer::<Vector4>::new(2, 2, 2);

    // Sampled image is just a 2x2x2 buffer with primary colors on each axis
    img.buffer_write(1, 0, 0, Vector4::new(1.0, 0.0, 0.0, 1.0));
    img.buffer_write(0, 1, 0, Vector4::new(0.0, 1.0, 0.0, 1.0));
    img.buffer_write(0, 0, 1, Vector4::new(0.0, 0.0, 1.0, 1.0));

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
