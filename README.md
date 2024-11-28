# Prism - A Rust Rendering Library

### NOTE - This is the "public release" version!

History has been squashed since the last time this repo was last public

This is partly due to poor copyright / ownership attributions in previous examples. All offending examples have been removed and/or altered

prism is not complete! However, it is functional with an unstable API. One day I may pick this project up again and finish it out to 1.0, but today's not that day :P

### What is this?
* Prism is an attempt to create a powerful enough math library to make distributed CPU workloads possible, mainly for shading
* The API handles complex operations, like threading, for you! While also providing traits for you to implement your own features!

**Please consult the math library [readme](https://github.com/zCubed3/rgml) for more on the math section**

### Features
* CPU Only Rendering
  * Rendering is a 100% CPU bound, this is not a drawback seeing as the CPU is capable of certain types of data manipulation GPUs may struggle with.
  * Note: If you're looking for GPU acceleration, use CUDA or OpenCL as those are professionally made APIs for GPUs
* Buffers
  * A buffer is a block of memory that represents a certain "shaped" piece of data, be it 1D, 2D, or even 3D!
* Kernels
  * A kernel is a type of structure that ensures safety during distributed workloads, you can think of it as a "program" that is ran multiple times
  * A kernel is essentially a wrapped function, so data goes in and data comes out without needing to change data elsewhere!
* Math Library
  * A custom and generic math library is used to make the API extensible if you need different datatypes
  * The ability to switch between f32 and f64 by simply changing a feature flag!

### Fun Fact

Prism was used internally for my job working on Strayed by Crustacean Interactive!

It came in handy for generating LUTs and inspired the source release :)