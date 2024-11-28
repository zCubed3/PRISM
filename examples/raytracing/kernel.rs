use prism::impl_buffer_data;
use prism::prelude::*;
use rand::*;
use serde::Serialize;
use std::fs::*;
use std::io::*;
use std::path::Path;

#[derive(Default, Debug, PartialEq, Copy, Clone, Serialize)]
pub struct RTVertex {
    pub position: Vector3,
    pub normal: Vector3,
    pub uv0: Vector2,
}

#[derive(Default, Debug, Clone)]
pub struct RTAABB {
    pub min: Vector3,
    pub max: Vector3,

    pub children: Vec<RTAABB>,
    pub cache_id: usize,

    #[cfg(debug_assertions)]
    pub debug_color: Vector3,
}

static mut AABB_ID: usize = 0usize;

impl RTAABB {
    pub fn new(min: Vector3, max: Vector3) -> RTAABB {
        #[cfg(debug_assertions)]
        let mut rng = thread_rng();

        unsafe {
            let bb = RTAABB {
                min,
                max,
                children: vec![],
                cache_id: AABB_ID,

                #[cfg(debug_assertions)]
                debug_color: Vector3::new(
                    rng.gen_range(0.0..1.0),
                    rng.gen_range(0.0..1.0),
                    rng.gen_range(0.0..1.0),
                ),
            };

            AABB_ID += 1;
            bb
        }
    }

    pub fn contains_point(&self, point: Vector3) -> bool {
        point >= self.min && point <= self.max
    }
}

#[derive(Default)]
pub struct RTMesh {
    pub vertices: Vec<RTVertex>,
    pub triangles: Vec<usize>,
    pub aabb: RTAABB,
    pub cache: Vec<Vec<(usize, usize, usize)>>,
}

#[derive(Default)]
pub struct RTRay {
    pub origin: Vector3,
    pub direction: Vector3,
}

impl RTAABB {
    pub fn union(&self, min: Vector3, max: Vector3) -> RTAABB {
        RTAABB {
            min: self.min.min(min),
            max: self.max.max(max),
            children: vec![],
            cache_id: self.cache_id,

            #[cfg(debug_assertions)]
            debug_color: self.debug_color,
        }
    }
}

impl RTMesh {
    pub fn load_obj<P: AsRef<Path>>(path: P) -> RTMesh {
        let mut mdl = RTMesh::default();

        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);

        let mut positions: Vec<Vector3> = Vec::new();
        let mut normals: Vec<Vector3> = Vec::new();
        let mut uvs: Vec<Vector2> = Vec::new();

        let mut faces: Vec<[usize; 3]> = Vec::new();

        for line in reader.lines() {
            let content = line.unwrap();

            if content.starts_with("v ") {
                let mut components: Vec<Real> = Vec::new();
                for word in content.split(" ") {
                    if let Ok(val) = word.parse::<Real>() {
                        components.push(val);
                    }
                }

                if components.len() == 3 {
                    let pos = Vector3::new(components[0], components[1], components[2]);
                    positions.push(pos);
                }
            }

            if content.starts_with("vn") {
                let mut components: Vec<Real> = Vec::new();
                for word in content.split(" ") {
                    if let Ok(val) = word.parse::<Real>() {
                        components.push(val);
                    }
                }

                if components.len() == 3 {
                    let normal = Vector3::new(components[0], components[1], components[2]);
                    normals.push(normal);
                }
            }

            if content.starts_with("vt") {
                let mut components: Vec<Real> = Vec::new();
                for word in content.split(" ") {
                    if let Ok(val) = word.parse::<Real>() {
                        components.push(val);
                    }
                }

                if components.len() == 2 {
                    let uv = Vector2::new(components[0], components[1]);
                    uvs.push(uv);
                }
            }

            if content.starts_with("f ") {
                let raw_face = content.replace("f ", "");

                for raw_indices in raw_face.split(" ") {
                    let mut face: [usize; 3] = [0, 0, 0];
                    let mut last_index = 0;
                    for raw_index in raw_indices.split("/") {
                        if let Ok(index) = raw_index.parse::<usize>() {
                            face[last_index] = index - 1;
                            last_index += 1;
                        }
                    }

                    faces.push(face);
                }
            }
        }

        for face in faces {
            let position = positions[face[0]];
            let uv = uvs[face[1]];
            let normal = normals[face[2]];

            let vertex = RTVertex {
                position,
                normal,
                uv0: uv,
            };

            let mut similar = false;
            for test_vertex_index in 0..mdl.vertices.len() {
                let test_vertex = mdl.vertices[test_vertex_index];
                if vertex == test_vertex {
                    similar = true;
                    mdl.triangles.push(test_vertex_index);
                    break;
                }
            }

            if !similar {
                mdl.triangles.push(mdl.vertices.len());
                mdl.vertices.push(vertex);
            }
        }

        mdl
    }

    pub fn chunk_aabb_recursive(aabb: &mut RTAABB, depth: usize) {
        let size = (aabb.min - aabb.max).abs();
        let cell = size / 2.0;

        // TODO: Fix densely chunked culling
        if depth > 0 {
            return;
        }

        // We then get 4 new AABBs from this cell
        let shift_x = cell * Vector3::new(1.0, 0.0, 0.0);
        let shift_y = cell * Vector3::new(0.0, 1.0, 0.0);
        let shift_z = cell * Vector3::new(0.0, 0.0, 1.0);

            aabb.children = vec![
                RTAABB::new(aabb.min, aabb.min + cell),
                RTAABB::new(aabb.min + shift_x, aabb.min + shift_x + cell),
                RTAABB::new(aabb.min + shift_y, aabb.min + shift_y + cell),
                RTAABB::new(aabb.min + shift_z, aabb.min + shift_z + cell),
                RTAABB::new(
                    aabb.min + shift_z + shift_x,
                    aabb.min + shift_z + shift_x + cell,
                ),
                RTAABB::new(
                    aabb.min + shift_y + shift_x,
                    aabb.min + shift_y + shift_x + cell,
                ),
                RTAABB::new(
                    aabb.min + shift_y + shift_z,
                    aabb.min + shift_y + shift_z + cell,
                ),
                RTAABB::new(
                    aabb.min + shift_x + shift_y + shift_z,
                    aabb.min + shift_x + shift_y + shift_z + cell,
                ),
            ];

        aabb.children
            .iter_mut()
            .for_each(|bb| RTMesh::chunk_aabb_recursive(bb, depth + 1));
    }

    // Caches triangles within this AABB and its children that may also contain it
    pub fn cache_aabb_recursive(
        aabb: &mut RTAABB,
        cache: &mut Vec<Vec<(usize, usize, usize)>>,
        polygon: (usize, usize, usize),
        (p0, p1, p2): (Vector3, Vector3, Vector3),
    ) {
        let c = (p0 + p1 + p2) / 3.0;

        if aabb.contains_point(p0)
            || aabb.contains_point(p1)
            || aabb.contains_point(p2)
            || aabb.contains_point(c)
        {
            cache[aabb.cache_id].push(polygon);
            aabb.children
                .iter_mut()
                .for_each(|a| RTMesh::cache_aabb_recursive(a, cache, polygon, (p0, p1, p2)));
        }
    }

    pub fn prune_aabb_recursive(aabb: &mut RTAABB, cache: &mut Vec<Vec<(usize, usize, usize)>>) {
        aabb.children.retain(|aabb| {
            let retain = !cache[aabb.cache_id].is_empty();

            if !retain {
                println!("PRUNED: {}", aabb.cache_id);
            }

            retain
        });

        aabb.children
            .iter_mut()
            .for_each(|bb| RTMesh::prune_aabb_recursive(bb, cache));
    }

    /// Builds acceleration structures for this mesh
    pub fn optimize(&mut self) {
        self.aabb = RTAABB::new(Vector3::default(), Vector3::default());

        for t in (0..self.triangles.len()).step_by(3) {
            let p0 = self.vertices[self.triangles[t]].position;
            let p1 = self.vertices[self.triangles[t + 1]].position;
            let p2 = self.vertices[self.triangles[t + 2]].position;

            self.aabb = self.aabb.union(p0.min(p1.min(p2)), p0.max(p1.max(p2)));
        }

        // We subdivide the AABB down and check which triangles exist within each AABB
        // This operation can get slow at high recurse amounts!

        RTMesh::chunk_aabb_recursive(&mut self.aabb, 0);

        unsafe {
            self.cache = vec![Vec::new(); AABB_ID];

            for t in (0..self.triangles.len()).step_by(3) {
                let t0 = self.triangles[t];
                let t1 = self.triangles[t + 1];
                let t2 = self.triangles[t + 2];

                let p0 = self.vertices[t0].position;
                let p1 = self.vertices[t1].position;
                let p2 = self.vertices[t2].position;

                RTMesh::cache_aabb_recursive(
                    &mut self.aabb,
                    &mut self.cache,
                    (t0, t1, t2),
                    (p0, p1, p2),
                );
            }

            // We then get rid of any AABBs that are empty
            RTMesh::prune_aabb_recursive(&mut self.aabb, &mut self.cache);
        }

        //println!("{:?}", self.aabb);
    }
}

// The RT system is one of the few examples that is made up of multiple kernels
// We first to per-triangle, per-pixel traces
// Dispatching occurs once per triangle, and we wait for pixels to process it
// Then afterward we store the depth and triangle info the buffer which is then later used
// This modeled similarly to a GPUs "vertex" + "fragment" pipeline

//
// RT Job models
//
#[derive(Default)]
pub struct RTVertJobModel {}

impl<'a>
    JobModel<
        RTVertPassData,
        RTVertPassData,
        RTVertPassKernel<'a>,
        RTVertFramebuffer,
        RTVertFramebuffer,
    > for RTVertJobModel
{
    fn do_job(
        &self,
        kernel: &RTVertPassKernel,
        buffer: &mut RTVertFramebuffer,
        input: KernelInput,
        args: DispatchJobArgs,
    ) {
        for x in args.min_x..args.max_x {
            for y in args.min_y..args.max_y {
                for z in args.min_z..args.max_z {
                    //let cull = kernel.cull.buffer_read(args.shift_x + x, args.shift_y + y, args.shift_z + z);

                    let mut input_copy = input;
                    input_copy.thread_x = args.shift_x + x;
                    input_copy.thread_y = args.shift_y + y;
                    input_copy.thread_z = args.shift_z + z;

                    let result = kernel.kernel_exec(input_copy, buffer);

                    if result.h {
                        buffer.buffer_write(x, y, z, result);
                    }
                }
            }
        }
    }
}

pub struct RTFragJobModel<'a> {
    pub buffer: &'a RTVertFramebuffer,
}

impl<'a> RTFragJobModel<'a> {
    pub fn new(buffer: &'a RTVertFramebuffer) -> RTFragJobModel {
        RTFragJobModel { buffer }
    }
}

impl<'a>
    JobModel<
        RTVertPassData,
        Vector4,
        RTFragPassKernel,
        TexelBuffer<RTVertPassData>,
        TexelBuffer<Vector4>,
    > for RTFragJobModel<'a>
{
    fn do_job(
        &self,
        kernel: &RTFragPassKernel,
        buffer: &mut TexelBuffer<Vector4>,
        input: KernelInput,
        args: DispatchJobArgs,
    ) {
        for x in args.min_x..args.max_x {
            for y in args.min_y..args.max_y {
                for z in args.min_z..args.max_z {
                    let mut input_copy = input;
                    input_copy.thread_x = args.shift_x + x;
                    input_copy.thread_y = args.shift_y + y;
                    input_copy.thread_z = args.shift_z + z;

                    buffer.buffer_write(x, y, z, kernel.kernel_exec(input_copy, self.buffer));
                }
            }
        }
    }
}

//
// RT Kernels
//
#[derive(Default, Clone, Copy, Serialize)]
pub struct RTCullPassData {
    pub c: bool,
    pub cache_id: usize,

    #[cfg(debug_assertions)]
    pub debug_color: Vector3,
}

#[derive(Clone, Copy, Serialize)]
pub struct RTVertPassData {
    pub v0: RTVertex,
    pub v1: RTVertex,
    pub v2: RTVertex,
    pub d: Real,
    pub u: Real,
    pub v: Real,
    pub h: bool,

    #[cfg(debug_assertions)]
    pub debug_color: Vector3,
}

impl Default for RTVertPassData {
    fn default() -> Self {
        Self {
            v0: RTVertex::default(),
            v1: RTVertex::default(),
            v2: RTVertex::default(),
            d: 999999.0,
            u: 0.0,
            v: 0.0,
            h: false,

            #[cfg(debug_assertions)]
            debug_color: Vector3::default(),
        }
    }
}

impl_buffer_data!(RTCullPassData);
impl_buffer_data!(RTVertPassData);

pub type RTCullFramebuffer = TexelBuffer<RTCullPassData>;
pub type RTVertFramebuffer = TexelBuffer<RTVertPassData>;

#[derive(Default)]
pub struct RTCullPassKernel<'a> {
    pub mat_m_i: Matrix4x4,
    pub mat_vp_i: Matrix4x4,

    pub aabb: Option<&'a RTAABB>,

    pub origin: Vector3,
}

impl<'a> RTCullPassKernel<'a> {
    fn ray_aabb(
        origin: Vector3,
        direction: Vector3,
        aabb: &RTAABB,
        tmin: Real,
        tmax: Real,
    ) -> Option<&RTAABB> {
        let inv_d = Vector3::from_scalar(1.0) / direction;
        let t0s = (aabb.min - origin) * inv_d;
        let t1s = (aabb.max - origin) * inv_d;

        let tsmaller = t0s.min(t1s);
        let tbigger = t0s.max(t1s);

        let otmin = tmin;
        let otmax = tmax;

        let tmin = tmin.rl_max(tsmaller[0].rl_max(tsmaller[1].rl_max(tsmaller[2])));
        let tmax = tmax.rl_min(tbigger[0].rl_min(tbigger[1].rl_min(tbigger[2])));

        let hit = tmin < tmax;

        if hit {
            let mut closest = aabb;
            let mut closest_origin = (origin - (aabb.min + aabb.max) / 2.0).magnitude_sqr();

            for child in &aabb.children {
                if let Some(bb) = RTCullPassKernel::ray_aabb(origin, direction, child, otmin, otmax)
                {
                    let dist = (origin - (bb.min + bb.max) / 2.0).magnitude();

                    if dist < closest_origin {
                        closest = bb;
                        closest_origin = dist;
                    }
                }
            }

            //return Some(aabb.clone());
            return Some(closest);
        }

        None
    }
}

impl<'a> Kernel<RTCullPassData, TexelBuffer<RTCullPassData>> for RTCullPassKernel<'a> {
    fn kernel_exec(
        &self,
        input: KernelInput,
        _buffer: &TexelBuffer<RTCullPassData>,
    ) -> RTCullPassData {
        let u_proj = (input.thread_x as Real / input.buffer_width as Real) * 2.0 - 1.0;
        let v_proj = (input.thread_y as Real / input.buffer_height as Real) * 2.0 - 1.0;

        let mut ray = Vector4::new(u_proj, v_proj, 1.0, 1.0);
        ray = self.mat_vp_i * ray;

        let origin = Vector3::from(self.mat_m_i * Vector4::from_w(self.origin, 1.0));
        let direction = (self.mat_m_i * Vector4::from_w(ray.xyz(), 0.0).normalize()).xyz();

        match RTCullPassKernel::ray_aabb(
            origin,
            direction,
            self.aabb.unwrap(),
            -100.0,
            100.0,
        ) {
            Some(bb) => {
                 RTCullPassData {
                    c: true,
                    cache_id: bb.cache_id,

                    #[cfg(debug_assertions)]
                    debug_color: bb.debug_color,
                }
            }
            _ => RTCullPassData::default(),
        }
    }
}

#[derive(Default)]
pub struct RTVertPassKernel<'a> {
    pub mat_m: Matrix4x4,
    pub mat_v: Matrix4x4,
    pub mat_p: Matrix4x4,

    pub mat_p_i: Matrix4x4,
    pub mat_vp_i: Matrix4x4,

    pub mat_m_i: Matrix4x4,
    pub mat_m_it: Matrix4x4,

    pub origin: Vector3,
    pub face_n: Vector3,

    pub v0: RTVertex,
    pub v1: RTVertex,
    pub v2: RTVertex,

    pub w0: Vector3,
    pub w1: Vector3,
    pub w2: Vector3,

    pub cull: Option<&'a RTCullFramebuffer>,
    pub mesh: Option<&'a RTMesh>,
}

impl<'a> RTVertPassKernel<'a> {
    const EPSILON: Real = 0.00000001;
    fn intersect_tri(
        origin: Vector3,
        direction: Vector3,
        (p1, p2, p3): (Vector3, Vector3, Vector3),
    ) -> Option<(Real, Real, Real)> {
        let e1 = p2 - p1;
        let e2 = p3 - p1;

        let h = direction.cross(e2);
        let a = e1.dot(h);

        // Is parallel?
        if a > -RTVertPassKernel::EPSILON && a < RTVertPassKernel::EPSILON {
            return None;
        }

        let f = 1.0 / a;
        let s = origin - p1;
        let u = f * s.dot(h);

        if !(0.0..=1.0).contains(&u) {
            return None;
        }

        let q = s.cross(e1);
        let v = f * direction.dot(q);

        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = f * e2.dot(q);

        if t > RTVertPassKernel::EPSILON {
            return Some((u, v, t));
        }

        None
    }

    /*
    pub fn new(buffer: &'a RTCullFramebuffer) -> Self {
        Self {
            mat_m: Matrix4x4::default(),
            mat_v: Matrix4x4::default(),
            mat_p: Matrix4x4::default(),

            mat_p_i : Matrix4x4::default(),
            mat_vp_i : Matrix4x4::default(),

            mat_m_i : Matrix4x4::default(),
            mat_m_it : Matrix4x4::default(),

            origin: Vector3::default(),
            face_n: Vector3::default(),

            v0: RTVertex::default(),
            v1: RTVertex::default(),
            v2: RTVertex::default(),

            w0: Vector3::default(),
            w1: Vector3::default(),
            w2: Vector3::default(),

            cull: buffer
        }
    }

     */
}

impl<'a> Kernel<RTVertPassData, TexelBuffer<RTVertPassData>> for RTVertPassKernel<'a> {
    fn kernel_exec(&self, input: KernelInput, buffer: &RTVertFramebuffer) -> RTVertPassData {
        let empty = RTVertPassData::default();

        if let Some(cull) = self.cull {
            let data = cull.buffer_read(input.thread_x, input.thread_y, input.thread_z);

            /*
            #[cfg(debug_assertions)] {
                empty.debug_color = data.debug_color;
                empty.h = data.c;

                return empty;
            }

             */

            if data.c {
                let u_proj = (input.thread_x as Real / input.buffer_width as Real) * 2.0 - 1.0;
                let v_proj = (input.thread_y as Real / input.buffer_height as Real) * 2.0 - 1.0;

                let mut ray = Vector4::new(u_proj, v_proj, 1.0, 1.0);
                ray = self.mat_vp_i * ray;

                let direction = Vector3::from(ray).normalize();

                let source = buffer.buffer_read(input.thread_x, input.thread_y, input.thread_z);

                if let Some(mesh) = self.mesh {
                    let mut closest = source;

                    for base in &mesh.cache[data.cache_id] {
                        let v0 = mesh.vertices[base.0];
                        let v1 = mesh.vertices[base.1];
                        let v2 = mesh.vertices[base.2];

                        let w0 = Vector3::from(self.mat_m * Vector4::from_w(v0.position, 1.0));
                        let w1 = Vector3::from(self.mat_m * Vector4::from_w(v1.position, 1.0));
                        let w2 = Vector3::from(self.mat_m * Vector4::from_w(v2.position, 1.0));

                        if let Some(h) =
                            RTVertPassKernel::intersect_tri(self.origin, direction, (w0, w1, w2))
                        {
                            if h.2 < closest.d {
                                closest = RTVertPassData {
                                    v0,
                                    v1,
                                    v2,
                                    d: h.2,
                                    u: h.0,
                                    v: h.1,
                                    h: true,

                                    #[cfg(debug_assertions)]
                                    debug_color: source.debug_color,
                                };
                            }
                        }
                    }

                    return closest;
                }
            }
        }

        empty
    }
}

#[derive(Default)]
pub struct RTFragPassKernel {
    pub mat_m: Matrix4x4,
    pub mat_v: Matrix4x4,
    pub mat_p: Matrix4x4,

    pub mat_vp_i: Matrix4x4,

    pub mat_m_i: Matrix4x4,
    pub mat_m_it: Matrix4x4,

    pub origin: Vector3,

    pub mesh: RTMesh,
}

impl RTFragPassKernel {
    /// Interpolates a Vector3 across a triangle
    fn bary_interpolate(u: Real, v: Real, (v0, v1, v2): (Vector3, Vector3, Vector3)) -> Vector3 {
        (v0 * (1.0 - u - v)) + (v1 * u) + (v2 * v)
    }

    fn fresnel_schlick(cos_theta: Real, f0: Vector3) -> Vector3 {
        f0
            + (Vector3::from_scalar(1.0) - f0) * (1.0 - cos_theta).rl_clamp(0.0, 1.0).rl_pow(5.0)
    }

    fn distribution_ggx(n: Vector3, h: Vector3, roughness: Real) -> Real {
        let a = roughness * roughness;
        let a2 = a * a;
        let n_dot_h = n.dot(h).max(0.0);
        let n_dot_h2 = n_dot_h * n_dot_h;

        let num = a2;
        let mut denom = n_dot_h2 * (a2 - 1.0) + 1.0;
        denom = Real::PI * denom * denom;

        num / denom
    }

    fn geometry_schlick_ggx(n_dot_v: Real, roughness: Real) -> Real {
        let r = roughness + 1.0;
        let k = (r * r) / 8.0;

        let num = n_dot_v;
        let denom = n_dot_v * (1.0 - k) + k;

        num / denom
    }

    fn geometry_smith(n: Vector3, v: Vector3, l: Vector3, roughness: Real) -> Real {
        let n_dot_v = n.dot(v).max(0.0);
        let n_dot_l = n.dot(l).max(0.0);
        let ggx2 = RTFragPassKernel::geometry_schlick_ggx(n_dot_v, roughness);
        let ggx1 = RTFragPassKernel::geometry_schlick_ggx(n_dot_l, roughness);

        ggx1 * ggx2
    }
}

impl Kernel<Vector4, TexelBuffer<RTVertPassData>, RTVertPassData> for RTFragPassKernel {
    fn kernel_exec(&self, input: KernelInput, buffer: &RTVertFramebuffer) -> Vector4 {
        let frag = buffer.buffer_read(input.thread_x, input.thread_y, input.thread_z);

        if frag.h {
            let u_proj = (input.thread_x as Real / input.buffer_width as Real) * 2.0 - 1.0;
            let v_proj = (input.thread_y as Real / input.buffer_height as Real) * 2.0 - 1.0;

            let mut ray = Vector4::new(u_proj, v_proj, 1.0, 1.0);
            ray = self.mat_vp_i * ray;

            let direction = ray.xyz().normalize();

            let n_raw = RTFragPassKernel::bary_interpolate(
                frag.u,
                frag.v,
                (frag.v0.normal, frag.v1.normal, frag.v2.normal),
            );
            let n = Vector3::from(self.mat_m_it * Vector4::from_w(n_raw, 0.0)).normalize();

            let v = -direction;

            let l = Vector3::new(-1.0, 0.5, -1.0).normalize();
            let h = (v + l).normalize();

            let n_dot_l = n.dot(l);
            let radiance = n_dot_l.max(0.0);

            let albedo_color = Vector3::new(1.0, 1.0, 1.0);
            let metallic = 0.0;
            let roughness = 0.3;

            let f0 = Vector3::from_scalar(0.04).lerp(albedo_color, metallic);

            let ndf = RTFragPassKernel::distribution_ggx(n, h, roughness);
            let g = RTFragPassKernel::geometry_smith(n, v, l, roughness);
            let f = RTFragPassKernel::fresnel_schlick(h.dot(v).max(0.0), f0);

            let numerator = Vector3::from_scalar(ndf) * Vector3::from_scalar(g) * f;
            let denominator = 4.0 * n.dot(v).max(0.0) * n.dot(l).max(0.0) + 0.0001;
            let specular = numerator / denominator;

            let ks = f;
            let mut kd = Vector3::from_scalar(1.0) - ks;
            kd *= 1.0 - metallic;

            let ambient = Vector4::from_scalar(0.01);

            let mut result;

            result = Vector4::from(kd * albedo_color / Real::PI + specular) * radiance;
            result += ambient;

            //result = Vector4::from(n * Vector3::new(1.0, 1.0, -1.0));

            result[3] = 1.0;

            result
        } else {
            Vector4::default()
        }
    }
}
