use std::fs::*;
use std::path::Path;

use clap::Parser;

use prism::compute::buffer::*;
use prism::compute::dispatcher::*;
use prism::prelude::*;

use crate::kernel::*;

pub mod kernel;

#[derive(Parser)]
#[command(name = "Raytracer")]
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

    /// The distance from the camera that a model is offset by
    #[arg(long, default_value_t = -4.0)]
    camera_offset: Real,

    /// Field of view on the y-axis
    #[arg(long, default_value_t = 35.0)]
    fov_y: Real,

    /// The input model path (must be a triangulated wavefront OBJ!)
    #[arg(long, default_value = "test.obj")]
    in_model: String,
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
    
    let aspect_x = args.width as Real / args.height as Real;
    let mut buffer = TexelBuffer::new(args.width, args.height);

    let mut offset = Vector3::default();
    let mut cam_rotation = Vector3::default();
    let spin = Vector3::default();

    let output_path = Path::new("./output");

    if !output_path.exists() {
        create_dir_all(output_path).expect("Failed to create output path!");
    }

    let mut ckernel = RTCullPassKernel::default();
    let mut fkernel = RTFragPassKernel::default();

    let tile_dim = 16;

    println!("Loading {}...", args.in_model);
    let mut mesh = RTMesh::load_obj(args.in_model);

    println!("Optimizing (welding) mesh!");
    mesh.optimize();

    {
        let _total = prism::perf::dropwatch::Dropwatch::new_begin("TOTAL RENDER");

        let mut cbuffer = RTCullFramebuffer::new(args.width, args.height);
        let mut vbuffer = RTVertFramebuffer::new(args.width, args.height);

        offset[2] = args.camera_offset;

        cam_rotation[2] = 180.0;
        cam_rotation[1] = 180.0;

        let mat_m = Matrix4x4::rotation(spin);
        let mat_v = Matrix4x4::translation(offset) * Matrix4x4::rotation(cam_rotation);
        let mat_p = Matrix4x4::perspective(args.fov_y, aspect_x, 0.001, 100.0);

        let mat_m_i = mat_m.inverse();
        let mat_m_it = mat_m_i.transpose();

        let mat_p_i = mat_p.inverse();

        let mat_vp_i = (mat_p * mat_v).inverse();

        let origin = Vector3::from(mat_v * Vector4::new(0.0, 0.0, 0.0, 1.0));

        // CKernel
        ckernel.mat_m_i = mat_m_i;
        ckernel.mat_vp_i = mat_vp_i;
        ckernel.origin = origin;
        ckernel.aabb = Some(&mesh.aabb);

        // FKernel
        fkernel.mat_m = mat_m;
        fkernel.mat_v = mat_v;
        fkernel.mat_p = mat_p;

        fkernel.mat_vp_i = mat_vp_i;
        fkernel.mat_m_it = mat_m_it;

        fkernel.origin = origin;

        {
            let _save_time = prism::perf::dropwatch::Dropwatch::new_begin("DISPATCH");

            // Cull pass
            {
                dispatcher.do_tiles(&ckernel, &mut cbuffer, tile_dim, tile_dim);
            }

            // Vertex pass
            {
                let vert_model = RTVertJobModel::default();
                let mut vkernel = RTVertPassKernel::default();

                vkernel.cull = Some(&cbuffer);
                vkernel.mesh = Some(&mesh);

                vkernel.mat_m = mat_m;
                vkernel.mat_v = mat_v;
                vkernel.mat_p = mat_p;

                vkernel.mat_p_i = mat_p_i;
                vkernel.mat_vp_i = mat_vp_i;

                vkernel.mat_m_i = mat_m_i;
                vkernel.mat_m_it = mat_m_it;

                vkernel.origin = origin;

                dispatcher.do_model_tiles(&vkernel, &mut vbuffer, &vert_model, tile_dim, tile_dim);
            }

            // Fragment pass
            {
                let frag_model = RTFragJobModel::new(&vbuffer);
                dispatcher.do_model_tiles(&fkernel, &mut buffer, &frag_model, tile_dim, tile_dim);
            }
        }

        {
            let _save_time = prism::perf::dropwatch::Dropwatch::new_begin("IMAGE SAVE");
            
            buffer.save_as_image("./output/raytracer.png");
        }

        println!("Rendered a scene using the prism raytracer!");
    }
}
