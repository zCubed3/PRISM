use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

use crate::prelude::*;
use rgml::real::*;

/// Work distributor, simplifies the process of splitting workloads across threads
#[repr(C)]
pub struct Dispatcher {
    #[cfg(feature = "threading")]
    _pool: rayon::ThreadPool,
    _async: bool,
    _threads: usize,

    _progress_fn: Option<fn(f32)>,
}

/// Arguments passed to [JobModel] to define how work should be dispatched within the model
#[derive(Copy, Clone)]
#[repr(C)]
pub struct DispatchJobArgs {
    pub min_x: usize,
    pub min_y: usize,
    pub min_z: usize,

    pub max_x: usize,
    pub max_y: usize,
    pub max_z: usize,

    pub shift_x: usize,
    pub shift_y: usize,
    pub shift_z: usize,
}

/// Abstraction over how work is processed within a thread
pub trait JobModel<TIn, TOut, TK, TBIn, TBOut>
where
    TIn: BufferData,
    TOut: BufferData,
    TK: Kernel<TOut, TBIn, TIn>,
    TBIn: Buffer<TIn>,
    TBOut: Buffer<TOut>,
{
    /// Does job work using the provided inputs
    fn do_job(&self, kernel: &TK, buffer: &mut TBOut, input: KernelInput, args: DispatchJobArgs);
}

/// Default job model, expects the Kernel inputs to match outputs
///
/// Consider using a custom [JobModel] implementation if additional data is required during dispatching
#[derive(Default)]
pub struct DefaultJobModel {}

impl<TD, TK, TB> JobModel<TD, TD, TK, TB, TB> for DefaultJobModel
where
    TD: BufferData,
    TK: Kernel<TD, TB>,
    TB: Buffer<TD>,
{
    fn do_job(&self, kernel: &TK, buffer: &mut TB, input: KernelInput, args: DispatchJobArgs) {
        for x in args.min_x..args.max_x {
            for y in args.min_y..args.max_y {
                for z in args.min_z..args.max_z {
                    let mut input_copy = input;
                    input_copy.thread_x = x;
                    input_copy.thread_y = y;
                    input_copy.thread_z = z;

                    buffer.buffer_write(x, y, z, kernel.kernel_exec(input_copy, buffer));
                }
            }
        }
    }
}

pub mod semaphore;
pub use semaphore::*;

impl Dispatcher {
    /// Creates a new dispatcher with the provided number of threads (if < 0, will use system thread count)
    pub fn new(num_threads: i32) -> Dispatcher {
        #[cfg(feature = "threading")]
        {
            let mut thread_count = num_threads as usize;

            if num_threads < 0 {
                thread_count = std::thread::available_parallelism().unwrap().get();
            }

            let build_result = rayon::ThreadPoolBuilder::new()
                .num_threads(thread_count)
                .build();
            let pool = build_result.expect("Failed to create thread pool!");

            return Dispatcher {
                _pool: pool,
                _async: thread_count != 0,
                _threads: thread_count,
                _progress_fn: None,
            };
        }

        #[cfg(not(feature = "threading"))]
        {
            return Dispatcher {
                _async: false,
                _threads: 0,
            };
        }
    }

    /// Returns the amount of allocated threads
    pub fn get_thread_count(&self) -> usize {
        return self._threads;
    }

    /// Provides a default input for use within dispatch functions
    fn get_default_input<TD: BufferData, TB: Buffer<TD>>(buffer: &mut TB) -> KernelInput {
        return KernelInput {
            buffer_width: buffer.get_buffer_width(),
            buffer_height: buffer.get_buffer_height(),
            buffer_depth: buffer.get_buffer_depth(),

            thread_x: 0,
            thread_y: 0,
            thread_z: 0,
        };
    }

    /// Copies a slice into the target buffer based on the provided job arguments
    fn copy_slices<TD, TB>(target: &mut TB, slice: &TB, slice_args: DispatchJobArgs)
    where
        TD: BufferData,
        TB: Buffer<TD> + Send + Sync,
    {
        for x in slice_args.min_x..slice_args.max_x {
            let origin_x = x + slice_args.shift_x;
            for y in slice_args.min_y..slice_args.max_y {
                let origin_y = y + slice_args.shift_y;
                for z in slice_args.min_z..slice_args.max_z {
                    let origin_z = z + slice_args.shift_z;
                    target.buffer_write(origin_x, origin_y, origin_z, slice.buffer_read(x, y, z));
                }
            }
        }
    }

    /// Spawns a fn to be dispatched by the dispatcher
    ///
    /// # Returns:
    ///   [Semaphore] indicating task state, use this for handling order dependent tasks!
    ///
    /// # Note:
    ///   If the dispatcher is working in synchronous mode, this will be synchronous too!
    pub fn spawn_fn<TF>(&self, func: TF) -> Option<Semaphore>
    where
        TF: FnOnce() + Send + 'static,
    {
        if self._async {
            let semaphore = Semaphore::new(self);

            let our_semaphore = semaphore.clone();
            #[cfg(feature = "threading")]
            self._pool.spawn(move || {
                our_semaphore.set_flag(SemaphoreState::Working);
                func();
                our_semaphore.set_flag(SemaphoreState::Finished);
            });

            return Some(semaphore);

            #[cfg(not(feature = "threading"))]
            panic!("Attempted to do spawn an async fn when compiled without async support!");
        } else {
            func();
        }

        return None;
    }

    /// Assigns a progress callback function, use this for displaying progress somewhere
    pub fn set_progress_callback(&mut self, func: fn(f32)) {
        self._progress_fn = Some(func);
    }

    /// Clears the progress callback function
    pub fn clear_progress_callback(&mut self) {
        self._progress_fn = None;
    }

    /// Dispatches thread blocks (explicit width, height, and depth)
    pub fn do_model_blocks<TIn, TOut, TK, TBIn, TBOut, TM>(
        &self,
        kernel: &TK,
        buffer: &mut TBOut,
        model: &TM,
        width: usize,
        height: usize,
        depth: usize,
    ) where
        TIn: BufferData,
        TOut: BufferData,
        TK: Kernel<TOut, TBIn, TIn> + Send + Sync,
        TBIn: Buffer<TIn> + Send + Sync,
        TBOut: Buffer<TOut> + Send + Sync,
        TM: JobModel<TIn, TOut, TK, TBIn, TBOut> + Send + Sync,
    {
        //let _time = perf::dropwatch::Dropwatch::new_begin("DISPATCH".to_string());

        let input = Dispatcher::get_default_input(buffer);

        let tiles_x = (buffer.get_buffer_width() as Real / width as Real).rl_ceil() as usize;
        let tiles_y = (buffer.get_buffer_height() as Real / height as Real).rl_ceil() as usize;
        let tiles_z = (buffer.get_buffer_depth() as Real / depth as Real).rl_ceil() as usize;

        let args = DispatchJobArgs {
            min_x: 0,
            max_x: width,

            min_y: 0,
            max_y: height,

            min_z: 0,
            max_z: depth,

            shift_x: 0,
            shift_y: 0,
            shift_z: 0,
        };

        if self._async {
            #[cfg(feature = "threading")]
            unsafe {
                let num_jobs = tiles_x * tiles_y * tiles_z;
                let mut tx: Option<Sender<usize>> = None;
                let mut rx: Option<Receiver<usize>> = None;

                if self._progress_fn.is_some() {
                    let (txi, rxi) = mpsc::channel::<usize>();
                    tx = Some(txi);
                    rx = Some(rxi);
                }

                self._pool.scope(move |s| {
                    let bptr: *mut TBOut = buffer;
                    for x in 0..tiles_x {
                        let x_origin = x * width;

                        for y in 0..tiles_y {
                            let y_origin = y * height;

                            for z in 0..tiles_z {
                                let z_origin = z * depth;

                                let mut block_args = args;
                                block_args.min_x = x_origin;
                                block_args.max_x = x_origin + width;

                                block_args.min_y = y_origin;
                                block_args.max_y = y_origin + height;

                                block_args.min_z = z_origin;
                                block_args.max_z = z_origin + depth;

                                if block_args.max_x > buffer.get_buffer_width() {
                                    block_args.max_x =
                                        width - (block_args.max_x - buffer.get_buffer_width());
                                }

                                if block_args.max_y > buffer.get_buffer_height() {
                                    block_args.max_y =
                                        height - (block_args.max_y - buffer.get_buffer_height());
                                }

                                if block_args.max_z > buffer.get_buffer_depth() {
                                    block_args.max_z =
                                        depth - (block_args.max_z - buffer.get_buffer_depth());
                                }

                                let bref = &mut *bptr;

                                let mut tx_clone: Option<Sender<usize>> = None;
                                if let Some(tx) = &tx {
                                    tx_clone = Some(tx.clone());
                                }

                                s.spawn(move |_s| {
                                    model.do_job(kernel, bref, input, block_args);

                                    if let Some(tx) = tx_clone {
                                        tx.send(1).expect("Failed to send!");
                                    }
                                });
                            }
                        }
                    }

                    if let Some(callback) = self._progress_fn {
                        let rx = rx.expect("Failed to unwrap receiver");

                        for j in 0..num_jobs {
                            rx.recv().expect("Failed to receive message!");

                            // Artificial delay to prevent overwhelming the callback
                            if j % 128 == 0 {
                                callback(j as f32 / num_jobs as f32);
                            }
                        }

                        callback(1.0);
                    }
                });
            }

            #[cfg(not(feature = "threading"))]
            {
                panic!("Attempted to do async dispatching when compiled without async support!");
            }
        } else {
            for x in 0..tiles_x {
                let x_origin = x * width;
                for y in 0..tiles_y {
                    let y_origin = y * height;

                    let mut block_args = args;
                    block_args.shift_x = x_origin;
                    block_args.shift_y = y_origin;

                    model.do_job(kernel, buffer, input, block_args);
                }
            }
        }
    }

    /// Dispatches thread tiles (explicit width and height, inferred depth)
    pub fn do_model_tiles<TIn, TOut, TK, TBIn, TBOut, TM>(
        &self,
        kernel: &TK,
        buffer: &mut TBOut,
        model: &TM,
        width: usize,
        height: usize,
    ) where
        TIn: BufferData,
        TOut: BufferData,
        TK: Kernel<TOut, TBIn, TIn> + Send + Sync,
        TBIn: Buffer<TIn> + Send + Sync,
        TBOut: Buffer<TOut> + Send + Sync,
        TM: JobModel<TIn, TOut, TK, TBIn, TBOut> + Send + Sync,
    {
        let depth = buffer.get_buffer_depth();
        self.do_model_blocks::<TIn, TOut, TK, TBIn, TBOut, TM>(
            kernel, buffer, model, width, height, depth,
        );
    }

    /// Dispatches thread strips (explicit width only, inferred height and depth)
    pub fn do_model_strips<TIn, TOut, TK, TBIn, TBOut, TM>(
        &self,
        kernel: &TK,
        buffer: &mut TBOut,
        model: &TM,
        width: usize,
    ) where
        TIn: BufferData,
        TOut: BufferData,
        TK: Kernel<TOut, TBIn, TIn> + Send + Sync,
        TBIn: Buffer<TIn> + Send + Sync,
        TBOut: Buffer<TOut> + Send + Sync,
        TM: JobModel<TIn, TOut, TK, TBIn, TBOut> + Send + Sync,
    {
        let depth = buffer.get_buffer_depth();
        let height = buffer.get_buffer_height();
        self.do_model_blocks::<TIn, TOut, TK, TBIn, TBOut, TM>(
            kernel, buffer, model, width, height, depth,
        );
    }

    /// Dispatches thread blocks using [DefaultJobModel]
    /// Use do_model_tiles() to use your own [JobModel]
    pub fn do_blocks<TD, TK, TB>(
        &self,
        kernel: &TK,
        buffer: &mut TB,
        width: usize,
        height: usize,
        depth: usize,
    ) where
        TD: BufferData,
        TK: Kernel<TD, TB> + Send + Sync,
        TB: Buffer<TD> + Send + Sync,
    {
        let model = DefaultJobModel::default();
        self.do_model_blocks::<TD, TD, TK, TB, TB, DefaultJobModel>(
            kernel, buffer, &model, width, height, depth,
        );
    }

    /// Dispatches thread tiles using [DefaultJobModel]
    /// Use do_model_tiles() to use your own [JobModel]
    pub fn do_tiles<TD, TK, TB>(&self, kernel: &TK, buffer: &mut TB, width: usize, height: usize)
    where
        TD: BufferData,
        TK: Kernel<TD, TB> + Send + Sync,
        TB: Buffer<TD> + Send + Sync,
    {
        let model = DefaultJobModel::default();
        self.do_model_tiles::<TD, TD, TK, TB, TB, DefaultJobModel>(
            kernel, buffer, &model, width, height,
        );
    }

    /// Dispatches thread strips using [DefaultJobModel]
    /// Use do_model_tiles() to use your own [JobModel]
    pub fn do_strips<TD, TK, TB>(&self, kernel: &TK, buffer: &mut TB, width: usize)
    where
        TD: BufferData,
        TK: Kernel<TD, TB> + Send + Sync,
        TB: Buffer<TD> + Send + Sync,
    {
        let model = DefaultJobModel::default();
        self.do_model_strips::<TD, TD, TK, TB, TB, DefaultJobModel>(kernel, buffer, &model, width);
    }
}
