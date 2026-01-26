// For God so loved the world that He gave His only begotten Son that all who believe in Him should not perish but have everlasting life.
//! GPU acceleration for bit matrix operations ☧
//!
//! Uses WebGPU compute shaders to accelerate:
//! - Bulk bitwise AND (domain intersection)
//! - Bulk bitwise OR (branch union)
//! - Boolean matrix multiplication
//! - Transitive closure (occurs check)
//!
//! Feature-gated: enable with `gpu_chirho` feature.

use bytemuck::{Pod, Zeroable};
use std::sync::Arc;
use wgpu::util::DeviceExt;

/// GPU context for bit matrix operations
pub struct GpuContextChirho {
    device_chirho: Arc<wgpu::Device>,
    queue_chirho: Arc<wgpu::Queue>,
    and_pipeline_chirho: wgpu::ComputePipeline,
    or_pipeline_chirho: wgpu::ComputePipeline,
    matmul_pipeline_chirho: wgpu::ComputePipeline,
}

/// Shader source for bitwise AND
const AND_SHADER_CHIRHO: &str = r#"
@group(0) @binding(0) var<storage, read> a_chirho: array<u32>;
@group(0) @binding(1) var<storage, read> b_chirho: array<u32>;
@group(0) @binding(2) var<storage, read_write> result_chirho: array<u32>;

@compute @workgroup_size(256)
fn main_chirho(@builtin(global_invocation_id) id_chirho: vec3<u32>) {
    let idx_chirho = id_chirho.x;
    if (idx_chirho < arrayLength(&a_chirho)) {
        result_chirho[idx_chirho] = a_chirho[idx_chirho] & b_chirho[idx_chirho];
    }
}
"#;

/// Shader source for bitwise OR
const OR_SHADER_CHIRHO: &str = r#"
@group(0) @binding(0) var<storage, read> a_chirho: array<u32>;
@group(0) @binding(1) var<storage, read> b_chirho: array<u32>;
@group(0) @binding(2) var<storage, read_write> result_chirho: array<u32>;

@compute @workgroup_size(256)
fn main_chirho(@builtin(global_invocation_id) id_chirho: vec3<u32>) {
    let idx_chirho = id_chirho.x;
    if (idx_chirho < arrayLength(&a_chirho)) {
        result_chirho[idx_chirho] = a_chirho[idx_chirho] | b_chirho[idx_chirho];
    }
}
"#;

/// Shader source for Boolean matrix multiplication
/// Uses u32 chunks (32 bits per element)
const MATMUL_SHADER_CHIRHO: &str = r#"
struct ParamsChirho {
    n_chirho: u32,
    words_per_row_chirho: u32,
}

@group(0) @binding(0) var<uniform> params_chirho: ParamsChirho;
@group(0) @binding(1) var<storage, read> a_chirho: array<u32>;
@group(0) @binding(2) var<storage, read> b_chirho: array<u32>;
@group(0) @binding(3) var<storage, read_write> result_chirho: array<u32>;

fn get_bit_chirho(matrix_chirho: ptr<storage, array<u32>, read>, row_chirho: u32, col_chirho: u32, words_per_row_chirho: u32) -> bool {
    let word_idx_chirho = row_chirho * words_per_row_chirho + col_chirho / 32u;
    let bit_idx_chirho = col_chirho % 32u;
    return ((*matrix_chirho)[word_idx_chirho] & (1u << bit_idx_chirho)) != 0u;
}

@compute @workgroup_size(16, 16)
fn main_chirho(@builtin(global_invocation_id) id_chirho: vec3<u32>) {
    let row_chirho = id_chirho.y;
    let word_col_chirho = id_chirho.x;

    if (row_chirho >= params_chirho.n_chirho || word_col_chirho >= params_chirho.words_per_row_chirho) {
        return;
    }

    var result_word_chirho: u32 = 0u;

    for (var bit_chirho: u32 = 0u; bit_chirho < 32u; bit_chirho = bit_chirho + 1u) {
        let col_chirho = word_col_chirho * 32u + bit_chirho;
        if (col_chirho >= params_chirho.n_chirho) {
            break;
        }

        var dot_chirho: bool = false;
        for (var k_chirho: u32 = 0u; k_chirho < params_chirho.n_chirho; k_chirho = k_chirho + 1u) {
            if (get_bit_chirho(&a_chirho, row_chirho, k_chirho, params_chirho.words_per_row_chirho) &&
                get_bit_chirho(&b_chirho, k_chirho, col_chirho, params_chirho.words_per_row_chirho)) {
                dot_chirho = true;
                break;
            }
        }

        if (dot_chirho) {
            result_word_chirho = result_word_chirho | (1u << bit_chirho);
        }
    }

    let out_idx_chirho = row_chirho * params_chirho.words_per_row_chirho + word_col_chirho;
    result_chirho[out_idx_chirho] = result_word_chirho;
}
"#;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct MatmulParamsChirho {
    n_chirho: u32,
    words_per_row_chirho: u32,
}

impl GpuContextChirho {
    /// Create a new GPU context (blocks until GPU is ready)
    pub fn new_chirho() -> Option<Self> {
        pollster::block_on(Self::new_async_chirho())
    }

    /// Create a new GPU context asynchronously
    pub async fn new_async_chirho() -> Option<Self> {
        let instance_chirho = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter_chirho = instance_chirho
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await?;

        let (device_chirho, queue_chirho) = adapter_chirho
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("minikanren_1bit_chirho"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .ok()?;

        let device_chirho = Arc::new(device_chirho);
        let queue_chirho = Arc::new(queue_chirho);

        let and_pipeline_chirho = Self::create_pipeline_chirho(&device_chirho, AND_SHADER_CHIRHO, "and_chirho");
        let or_pipeline_chirho = Self::create_pipeline_chirho(&device_chirho, OR_SHADER_CHIRHO, "or_chirho");
        let matmul_pipeline_chirho = Self::create_matmul_pipeline_chirho(&device_chirho);

        Some(Self {
            device_chirho,
            queue_chirho,
            and_pipeline_chirho,
            or_pipeline_chirho,
            matmul_pipeline_chirho,
        })
    }

    fn create_pipeline_chirho(device_chirho: &wgpu::Device, source_chirho: &str, label_chirho: &str) -> wgpu::ComputePipeline {
        let shader_chirho = device_chirho.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label_chirho),
            source: wgpu::ShaderSource::Wgsl(source_chirho.into()),
        });

        device_chirho.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(label_chirho),
            layout: None,
            module: &shader_chirho,
            entry_point: "main_chirho",
        })
    }

    fn create_matmul_pipeline_chirho(device_chirho: &wgpu::Device) -> wgpu::ComputePipeline {
        let shader_chirho = device_chirho.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("matmul_chirho"),
            source: wgpu::ShaderSource::Wgsl(MATMUL_SHADER_CHIRHO.into()),
        });

        device_chirho.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("matmul_chirho"),
            layout: None,
            module: &shader_chirho,
            entry_point: "main_chirho",
        })
    }

    /// Bulk bitwise AND on GPU
    pub fn bulk_and_chirho(&self, a_chirho: &[u32], b_chirho: &[u32]) -> Vec<u32> {
        assert_eq!(a_chirho.len(), b_chirho.len());
        if a_chirho.is_empty() {
            return Vec::new();
        }

        let size_chirho = (a_chirho.len() * std::mem::size_of::<u32>()) as wgpu::BufferAddress;

        let buf_a_chirho = self.device_chirho.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("a_chirho"),
            contents: bytemuck::cast_slice(a_chirho),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let buf_b_chirho = self.device_chirho.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("b_chirho"),
            contents: bytemuck::cast_slice(b_chirho),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let buf_result_chirho = self.device_chirho.create_buffer(&wgpu::BufferDescriptor {
            label: Some("result_chirho"),
            size: size_chirho,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let buf_staging_chirho = self.device_chirho.create_buffer(&wgpu::BufferDescriptor {
            label: Some("staging_chirho"),
            size: size_chirho,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_chirho = self.device_chirho.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("and_bind_chirho"),
            layout: &self.and_pipeline_chirho.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: buf_a_chirho.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: buf_b_chirho.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: buf_result_chirho.as_entire_binding() },
            ],
        });

        let mut encoder_chirho = self.device_chirho.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("and_encoder_chirho"),
        });

        {
            let mut pass_chirho = encoder_chirho.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("and_pass_chirho"),
                timestamp_writes: None,
            });
            pass_chirho.set_pipeline(&self.and_pipeline_chirho);
            pass_chirho.set_bind_group(0, &bind_group_chirho, &[]);
            pass_chirho.dispatch_workgroups((a_chirho.len() as u32 + 255) / 256, 1, 1);
        }

        encoder_chirho.copy_buffer_to_buffer(&buf_result_chirho, 0, &buf_staging_chirho, 0, size_chirho);
        self.queue_chirho.submit(Some(encoder_chirho.finish()));

        let slice_chirho = buf_staging_chirho.slice(..);
        slice_chirho.map_async(wgpu::MapMode::Read, |_| {});
        self.device_chirho.poll(wgpu::Maintain::Wait);

        let data_chirho = slice_chirho.get_mapped_range();
        let result_chirho: Vec<u32> = bytemuck::cast_slice(&data_chirho).to_vec();
        drop(data_chirho);
        buf_staging_chirho.unmap();

        result_chirho
    }

    /// Bulk bitwise OR on GPU
    pub fn bulk_or_chirho(&self, a_chirho: &[u32], b_chirho: &[u32]) -> Vec<u32> {
        assert_eq!(a_chirho.len(), b_chirho.len());
        if a_chirho.is_empty() {
            return Vec::new();
        }

        let size_chirho = (a_chirho.len() * std::mem::size_of::<u32>()) as wgpu::BufferAddress;

        let buf_a_chirho = self.device_chirho.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("a_chirho"),
            contents: bytemuck::cast_slice(a_chirho),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let buf_b_chirho = self.device_chirho.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("b_chirho"),
            contents: bytemuck::cast_slice(b_chirho),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let buf_result_chirho = self.device_chirho.create_buffer(&wgpu::BufferDescriptor {
            label: Some("result_chirho"),
            size: size_chirho,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let buf_staging_chirho = self.device_chirho.create_buffer(&wgpu::BufferDescriptor {
            label: Some("staging_chirho"),
            size: size_chirho,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_chirho = self.device_chirho.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("or_bind_chirho"),
            layout: &self.or_pipeline_chirho.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: buf_a_chirho.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: buf_b_chirho.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: buf_result_chirho.as_entire_binding() },
            ],
        });

        let mut encoder_chirho = self.device_chirho.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("or_encoder_chirho"),
        });

        {
            let mut pass_chirho = encoder_chirho.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("or_pass_chirho"),
                timestamp_writes: None,
            });
            pass_chirho.set_pipeline(&self.or_pipeline_chirho);
            pass_chirho.set_bind_group(0, &bind_group_chirho, &[]);
            pass_chirho.dispatch_workgroups((a_chirho.len() as u32 + 255) / 256, 1, 1);
        }

        encoder_chirho.copy_buffer_to_buffer(&buf_result_chirho, 0, &buf_staging_chirho, 0, size_chirho);
        self.queue_chirho.submit(Some(encoder_chirho.finish()));

        let slice_chirho = buf_staging_chirho.slice(..);
        slice_chirho.map_async(wgpu::MapMode::Read, |_| {});
        self.device_chirho.poll(wgpu::Maintain::Wait);

        let data_chirho = slice_chirho.get_mapped_range();
        let result_chirho: Vec<u32> = bytemuck::cast_slice(&data_chirho).to_vec();
        drop(data_chirho);
        buf_staging_chirho.unmap();

        result_chirho
    }

    /// Boolean matrix multiplication on GPU
    pub fn matmul_chirho(&self, a_chirho: &[u32], b_chirho: &[u32], n_chirho: u32) -> Vec<u32> {
        let words_per_row_chirho = (n_chirho + 31) / 32;
        let total_words_chirho = (n_chirho * words_per_row_chirho) as usize;

        assert_eq!(a_chirho.len(), total_words_chirho);
        assert_eq!(b_chirho.len(), total_words_chirho);

        if n_chirho == 0 {
            return Vec::new();
        }

        let params_chirho = MatmulParamsChirho { n_chirho, words_per_row_chirho };
        let size_chirho = (total_words_chirho * std::mem::size_of::<u32>()) as wgpu::BufferAddress;

        let buf_params_chirho = self.device_chirho.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("params_chirho"),
            contents: bytemuck::bytes_of(&params_chirho),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let buf_a_chirho = self.device_chirho.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("a_chirho"),
            contents: bytemuck::cast_slice(a_chirho),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let buf_b_chirho = self.device_chirho.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("b_chirho"),
            contents: bytemuck::cast_slice(b_chirho),
            usage: wgpu::BufferUsages::STORAGE,
        });

        let buf_result_chirho = self.device_chirho.create_buffer(&wgpu::BufferDescriptor {
            label: Some("result_chirho"),
            size: size_chirho,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let buf_staging_chirho = self.device_chirho.create_buffer(&wgpu::BufferDescriptor {
            label: Some("staging_chirho"),
            size: size_chirho,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_chirho = self.device_chirho.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("matmul_bind_chirho"),
            layout: &self.matmul_pipeline_chirho.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: buf_params_chirho.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: buf_a_chirho.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: buf_b_chirho.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 3, resource: buf_result_chirho.as_entire_binding() },
            ],
        });

        let mut encoder_chirho = self.device_chirho.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("matmul_encoder_chirho"),
        });

        {
            let mut pass_chirho = encoder_chirho.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("matmul_pass_chirho"),
                timestamp_writes: None,
            });
            pass_chirho.set_pipeline(&self.matmul_pipeline_chirho);
            pass_chirho.set_bind_group(0, &bind_group_chirho, &[]);
            pass_chirho.dispatch_workgroups((words_per_row_chirho + 15) / 16, (n_chirho + 15) / 16, 1);
        }

        encoder_chirho.copy_buffer_to_buffer(&buf_result_chirho, 0, &buf_staging_chirho, 0, size_chirho);
        self.queue_chirho.submit(Some(encoder_chirho.finish()));

        let slice_chirho = buf_staging_chirho.slice(..);
        slice_chirho.map_async(wgpu::MapMode::Read, |_| {});
        self.device_chirho.poll(wgpu::Maintain::Wait);

        let data_chirho = slice_chirho.get_mapped_range();
        let result_chirho: Vec<u32> = bytemuck::cast_slice(&data_chirho).to_vec();
        drop(data_chirho);
        buf_staging_chirho.unmap();

        result_chirho
    }

    /// Compute transitive closure on GPU using repeated squaring
    pub fn transitive_closure_chirho(&self, matrix_chirho: &[u32], n_chirho: u32) -> Vec<u32> {
        if n_chirho == 0 {
            return Vec::new();
        }

        let words_per_row_chirho = ((n_chirho + 31) / 32) as usize;

        let mut current_chirho = matrix_chirho.to_vec();
        let mut accumulator_chirho = matrix_chirho.to_vec();

        // Add identity matrix (reflexive closure)
        for i_chirho in 0..n_chirho as usize {
            let word_idx_chirho = i_chirho * words_per_row_chirho + i_chirho / 32;
            let bit_idx_chirho = i_chirho % 32;
            if word_idx_chirho < accumulator_chirho.len() {
                accumulator_chirho[word_idx_chirho] |= 1u32 << bit_idx_chirho;
            }
        }

        // Repeated squaring until convergence
        let max_iters_chirho = (32 - n_chirho.leading_zeros()) as usize + 1;
        for _ in 0..max_iters_chirho {
            let squared_chirho = self.matmul_chirho(&current_chirho, &current_chirho, n_chirho);
            let new_accum_chirho = self.bulk_or_chirho(&accumulator_chirho, &squared_chirho);

            if new_accum_chirho == accumulator_chirho {
                break;
            }

            accumulator_chirho = new_accum_chirho;
            current_chirho = squared_chirho;
        }

        accumulator_chirho
    }
}

#[cfg(test)]
mod tests_chirho {
    use super::*;

    #[test]
    fn test_bulk_and_chirho() {
        let ctx_chirho = match GpuContextChirho::new_chirho() {
            Some(c) => c,
            None => {
                eprintln!("No GPU available, skipping test");
                return;
            }
        };

        let a_chirho = vec![0b1111u32, 0b1010, 0b0101];
        let b_chirho = vec![0b1100u32, 0b0110, 0b1111];
        let result_chirho = ctx_chirho.bulk_and_chirho(&a_chirho, &b_chirho);

        assert_eq!(result_chirho, vec![0b1100, 0b0010, 0b0101]);
    }

    #[test]
    fn test_bulk_or_chirho() {
        let ctx_chirho = match GpuContextChirho::new_chirho() {
            Some(c) => c,
            None => {
                eprintln!("No GPU available, skipping test");
                return;
            }
        };

        let a_chirho = vec![0b1100u32, 0b0010];
        let b_chirho = vec![0b0011u32, 0b0100];
        let result_chirho = ctx_chirho.bulk_or_chirho(&a_chirho, &b_chirho);

        assert_eq!(result_chirho, vec![0b1111, 0b0110]);
    }

    #[test]
    fn test_matmul_identity_chirho() {
        let ctx_chirho = match GpuContextChirho::new_chirho() {
            Some(c) => c,
            None => {
                eprintln!("No GPU available, skipping test");
                return;
            }
        };

        let identity_chirho = vec![0b01u32, 0b10u32];
        let result_chirho = ctx_chirho.matmul_chirho(&identity_chirho, &identity_chirho, 2);

        assert_eq!(result_chirho, identity_chirho);
    }

    #[test]
    fn test_transitive_closure_chain_chirho() {
        let ctx_chirho = match GpuContextChirho::new_chirho() {
            Some(c) => c,
            None => {
                eprintln!("No GPU available, skipping test");
                return;
            }
        };

        // 3x3 chain: 0->1->2
        let chain_chirho = vec![0b010u32, 0b100u32, 0b000u32];
        let tc_chirho = ctx_chirho.transitive_closure_chirho(&chain_chirho, 3);

        assert_eq!(tc_chirho[0] & 0b111, 0b111); // 0 reaches all
        assert_eq!(tc_chirho[1] & 0b111, 0b110); // 1 reaches 1,2
        assert_eq!(tc_chirho[2] & 0b111, 0b100); // 2 reaches only 2
    }
}
