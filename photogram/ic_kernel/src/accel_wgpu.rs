//a Imports
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{Accelerate, KernelArgs};
use ic_base::utils::rtc::run_to_completion as rtc;

//a Support types
//tp ShaderDesc
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ShaderDesc {
    kernel: String,
    shader: String,
    x_worksize: usize,
    binary: bool,
}

//tp ShaderFileDesc
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ShaderFileDesc {
    path: PathBuf,
    shader_descs: Vec<ShaderDesc>,
}

impl ShaderFileDesc {
    //cp from_json
    pub fn from_json(path: PathBuf, json: &str) -> Result<Self, serde_json::Error> {
        let shader_descs = serde_json::from_str::<Vec<ShaderDesc>>(json)?;
        Ok(Self { path, shader_descs })
    }
}

//tp Pipeline
#[derive(Debug, Clone, Copy)]
pub struct Pipeline(usize);

//ip Pipeline
impl Pipeline {
    fn as_usize(&self) -> usize {
        self.0
    }
}

//tp From<usize> for Pipeline
impl From<usize> for Pipeline {
    fn from(n: usize) -> Pipeline {
        Pipeline(n)
    }
}

//tp Shader
#[derive(Debug, Clone, Copy)]
pub struct Shader(usize);

//ip Shader
impl Shader {
    fn as_usize(&self) -> usize {
        self.0
    }
}

//ip From<usize> for Shader
impl From<usize> for Shader {
    fn from(n: usize) -> Shader {
        Shader(n)
    }
}

//tp BufferType
pub enum BufferType {
    HostSrc,
    GpuData,
    GpuUniform,
    HostDst,
}

//a AccelWgpu
//tp AccelWgpu
#[derive(Debug)]
pub struct AccelWgpu {
    #[allow(dead_code)]
    instance: wgpu::Instance,
    #[allow(dead_code)]
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    shaders: Vec<wgpu::ShaderModule>,
    pipelines: Vec<(wgpu::ComputePipeline, Vec<wgpu::BindGroupLayout>)>,
}

//ip AccelWgpu
impl AccelWgpu {
    //cp new
    pub fn new() -> Self {
        // Instantiates instance of WebGPU
        let instance = wgpu::Instance::default();

        // `request_adapter` instantiates the general connection to the GPU
        let adapter =
            rtc(instance.request_adapter(&wgpu::RequestAdapterOptions::default())).unwrap();

        // `request_device` instantiates the feature specific connection to the GPU, defining some parameters,
        //  `features` being the available features.
        let (device, queue) = rtc(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
            },
            None,
        ))
        .unwrap();
        let shaders = vec![];
        let pipelines = vec![];
        Self {
            instance,
            adapter,
            device,
            queue,
            shaders,
            pipelines,
        }
    }

    //mp add_shader
    pub fn add_shader(
        &mut self,
        code: &str,
        opt_label: Option<&'static str>,
    ) -> Result<Shader, String> {
        let shader_desc = wgpu::ShaderModuleDescriptor {
            label: opt_label,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(code)),
        };

        // Loads the shader from WGSL
        let sm = self.device.create_shader_module(shader_desc);
        let n = self.shaders.len();
        self.shaders.push(sm);
        Ok(n.into())
    }

    //ap device
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    //ap queue
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    //ap shader
    pub fn shader(&self, shader: Shader) -> Option<&wgpu::ShaderModule> {
        self.shaders.get(shader.as_usize())
    }

    //ap shader_err
    pub fn shader_err(&self, shader: Shader) -> Result<&wgpu::ShaderModule, String> {
        self.shader(shader).ok_or("Bad shader index".into())
    }

    //ap pipeline
    pub fn pipeline(&self, pipeline: Pipeline) -> Option<&wgpu::ComputePipeline> {
        self.pipelines.get(pipeline.as_usize()).map(|(p, _bg)| p)
    }

    //ap pipeline_err
    pub fn pipeline_err(&self, pipeline: Pipeline) -> Result<&wgpu::ComputePipeline, String> {
        self.pipeline(pipeline).ok_or("Bad pipeline index".into())
    }

    //mp create_pipeline_layout
    /// A pipeline layout can be used by many pipelines
    pub fn create_pipeline_layout(
        &self,
        bind_group_layouts: &[&wgpu::BindGroupLayout],
    ) -> wgpu::PipelineLayout {
        let l = wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts,
            push_constant_ranges: &[],
        };
        self.device.create_pipeline_layout(&l)
    }

    //mp create_pipeline
    pub fn create_pipeline(
        &mut self,
        shader: Shader,
        entry_point: &str,
        layout: Option<&wgpu::PipelineLayout>,
        num_bind_groups: usize,
    ) -> Result<Pipeline, String> {
        let compilation_options = Default::default();
        let pipeline_desc = wgpu::ComputePipelineDescriptor {
            label: None,
            layout,
            module: self.shader_err(shader)?,
            entry_point,
            compilation_options,
        };
        let pipeline = self.device().create_compute_pipeline(&pipeline_desc);
        let n = self.pipelines.len();
        let mut bgl = vec![];
        for i in 0..num_bind_groups {
            bgl.push(pipeline.get_bind_group_layout(i as u32));
        }
        self.pipelines.push((pipeline, bgl));
        Ok(n.into())
    }

    //mp create_buffer
    pub fn create_buffer(
        &self,
        buffer_type: BufferType,
        buffer_size: usize,
        opt_label: Option<&'static str>,
    ) -> wgpu::Buffer {
        let usage = match buffer_type {
            BufferType::HostDst => wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            BufferType::HostSrc => wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC,
            BufferType::GpuData => {
                wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST
            }
            BufferType::GpuUniform => wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        };
        let bd = wgpu::BufferDescriptor {
            label: opt_label,
            size: buffer_size as u64,
            usage,
            mapped_at_creation: false,
        };
        self.device.create_buffer(&bd)
    }

    //mp block
    pub fn block(&self) {
        self.device()
            .poll(wgpu::Maintain::wait())
            .panic_on_timeout();
    }

    //zz All done
}

//a ImageAccelerator
//tp ImageAccelerator
#[derive(Debug)]
pub struct ImageAccelerator {
    accelerator: AccelWgpu,
    buffer_size: usize,
    pipeline_layout: wgpu::PipelineLayout,
    pipelines: HashMap<String, (Pipeline, ShaderDesc)>,
    /// Buffer for data that gets copied TO the GPU shader_in_out_data_buffer
    input_buffer: wgpu::Buffer,
    /// Buffer for data that gets copied TO the GPU shader_in_data_b_buffer
    input_b_buffer: wgpu::Buffer,
    /// Buffer for data that gets copied FROM the GPU shader_in_out_data_buffer
    output_buffer: wgpu::Buffer,
    /// Buffer for data that gets copied TO the GPU shader_uniform_buffer
    uniform_buffer: wgpu::Buffer,
    /// The buffer in the GPU for *pure* input data - this is 'rs2' as it were; img_a if required
    shader_in_data_buffer: wgpu::Buffer,
    /// The buffer in the GPU for *pure* input data - this is 'rs2' as it were; img_b if required
    shader_in_data_b_buffer: wgpu::Buffer,
    /// The buffer in the GPU for in-out data
    shader_in_out_data_buffer: wgpu::Buffer,
    /// The buffer in the GPU for in-out data
    shader_uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

//ip ImageAccelerator
impl ImageAccelerator {
    //fp new
    pub fn new(accelerator: AccelWgpu, buffer_size: usize) -> Result<Self, String> {
        let uniform_bgle = wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let in_out_data_bgle = wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let in_data_bgle = wgpu::BindGroupLayoutEntry {
            binding: 2,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let in_data_b_bgle = wgpu::BindGroupLayoutEntry {
            binding: 3,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let bgl = accelerator
            .device()
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[in_out_data_bgle, in_data_bgle, in_data_b_bgle, uniform_bgle],
            });
        let input_buffer =
            accelerator.create_buffer(BufferType::HostSrc, buffer_size, Some("Input"));
        let input_b_buffer =
            accelerator.create_buffer(BufferType::HostSrc, buffer_size, Some("Input B"));
        let shader_in_data_buffer =
            accelerator.create_buffer(BufferType::GpuData, buffer_size, Some("Shader In Data"));
        let shader_in_data_b_buffer =
            accelerator.create_buffer(BufferType::GpuData, buffer_size, Some("Shader In B Data"));
        let shader_in_out_data_buffer =
            accelerator.create_buffer(BufferType::GpuData, buffer_size, Some("Shader InOut Data"));
        let shader_uniform_buffer = accelerator.create_buffer(
            BufferType::GpuUniform,
            std::mem::size_of::<KernelArgs>(),
            Some("Shader Uniform"),
        );
        let uniform_buffer = accelerator.create_buffer(
            BufferType::HostSrc,
            std::mem::size_of::<KernelArgs>(),
            Some("Uniform"),
        );
        let output_buffer =
            accelerator.create_buffer(BufferType::HostDst, buffer_size, Some("Output"));
        let bind_group = accelerator
            .device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: shader_uniform_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: shader_in_out_data_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: shader_in_data_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: shader_in_data_b_buffer.as_entire_binding(),
                    },
                ],
            });

        let pipeline_layout = accelerator.create_pipeline_layout(&[&bgl]);

        let pipelines = HashMap::new();
        Ok(Self {
            accelerator,
            buffer_size,
            pipeline_layout,
            pipelines,
            input_buffer,
            input_b_buffer,
            uniform_buffer,
            output_buffer,
            shader_in_out_data_buffer,
            shader_in_data_buffer,
            shader_in_data_b_buffer,
            shader_uniform_buffer,
            bind_group,
        })
    }

    //mp create_pipelines
    pub fn create_pipelines(&mut self, shader_file: ShaderFileDesc) -> Result<(), String> {
        let file_text = std::fs::read_to_string(&shader_file.path)
            .map_err(|e| format!("Error reading WGSL shader file {e}"))?;
        let cs_module = self.accelerator.add_shader(&file_text, None)?;
        for sd in shader_file.shader_descs.into_iter() {
            let pipeline = self.accelerator.create_pipeline(
                cs_module,
                &sd.shader,
                Some(&self.pipeline_layout),
                1,
            )?;
            self.pipelines.insert(sd.kernel.clone(), (pipeline, sd));
        }
        Ok(())
    }

    //mp create_cmd_buffer
    fn create_cmd_buffer(
        &self,
        number_of_ops: [usize; 3],
        pipeline: Pipeline,
        buffers_in: &[((&wgpu::Buffer, usize), (&wgpu::Buffer, usize), usize)],
        buffers_out: &[((&wgpu::Buffer, usize), (&wgpu::Buffer, usize), usize)],
    ) -> Result<wgpu::CommandBuffer, String> {
        let mut encoder = self
            .accelerator
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        for (bi, bo, size) in buffers_in {
            encoder.copy_buffer_to_buffer(bi.0, bi.1 as u64, bo.0, bo.1 as u64, *size as u64);
        }
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        cpass.set_bind_group(0, &self.bind_group, &[]);
        cpass.set_pipeline(self.accelerator.pipeline_err(pipeline)?);
        cpass.dispatch_workgroups(
            number_of_ops[0] as u32,
            number_of_ops[1] as u32,
            number_of_ops[2] as u32,
        );
        drop(cpass); // so encoder can be mutable again
        for (bi, bo, size) in buffers_out {
            encoder.copy_buffer_to_buffer(bi.0, bi.1 as u64, bo.0, bo.1 as u64, *size as u64);
        }

        Ok(encoder.finish())
    }

    //mp copy_to_buffer
    fn copy_to_buffer(&self, src_data: &[u8], buffer: &wgpu::Buffer) -> Result<(), String> {
        let byte_size = src_data.len() as u64;
        let (sender, receiver) = std::sync::mpsc::channel();
        let buffer_slice = buffer.slice(0..byte_size);
        buffer_slice.map_async(wgpu::MapMode::Write, move |v| sender.send(v).unwrap());
        self.accelerator.block();
        let Ok(Ok(())) = receiver.recv() else {
            return Err("Failed to run compute on gpu!".into());
        };
        buffer_slice
            .get_mapped_range_mut()
            .copy_from_slice(bytemuck::cast_slice(src_data));
        buffer.unmap();
        Ok(())
    }

    //mp map_output
    fn map_output(&self, byte_size: u64) -> Result<wgpu::BufferView, String> {
        let (sender, receiver) = std::sync::mpsc::channel();
        let buffer_slice = self.output_buffer.slice(0..byte_size);
        buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());
        self.accelerator.block();
        let Ok(Ok(())) = receiver.recv() else {
            return Err("Failed to run compute on gpu!".into());
        };
        Ok(buffer_slice.get_mapped_range())
    }

    //mp run
    fn run<F: Fn(&KernelArgs, &[f32], &mut [f32]) -> Result<(), String>>(
        &self,
        args: &KernelArgs,
        src_data: Option<&[f32]>,
        out_data: &mut [f32],
        cmd_buffer: wgpu::CommandBuffer,
        callback: &F,
    ) -> Result<(), String> {
        let byte_size = std::mem::size_of_val(out_data) as u64;
        if let Some(src_data) = src_data {
            self.copy_to_buffer(bytemuck::cast_slice(src_data), &self.input_buffer)?;
            self.copy_to_buffer(bytemuck::cast_slice(out_data), &self.input_b_buffer)?;
        } else {
            self.copy_to_buffer(bytemuck::cast_slice(out_data), &self.input_buffer)?;
        }
        self.copy_to_buffer(bytemuck::cast_slice(&[*args]), &self.uniform_buffer)?;
        self.accelerator.queue().submit(Some(cmd_buffer));
        {
            let data = self.map_output(byte_size)?;
            callback(args, bytemuck::cast_slice(&data), out_data)?;
            // Must drop data so that self.output_buffer has no BufferView's
        }
        // Must unmap self.output_buffer so it can be used in the future
        self.output_buffer.unmap();
        Ok(())
    }

    //zz All done
}

//ip Accelerate for ImageAccelerator
impl Accelerate for ImageAccelerator {
    //mp run_shader
    fn run_shader(
        &self,
        shader: &str,
        args: &KernelArgs,
        work_items: usize,
        src_data: Option<&[f32]>,
        out_data: &mut [f32],
    ) -> Result<bool, String> {
        if let Some((p, sd)) = self.pipelines.get(shader) {
            match sd.binary {
                false => {
                    let cmd_buffer = self.create_cmd_buffer(
                        [work_items / sd.x_worksize, 1, 1],
                        *p,
                        &[
                            (
                                (&self.input_buffer, 0),
                                (&self.shader_in_data_buffer, 0),
                                self.buffer_size,
                            ),
                            (
                                (&self.uniform_buffer, 0),
                                (&self.shader_uniform_buffer, 0),
                                std::mem::size_of::<KernelArgs>(),
                            ),
                        ],
                        &[(
                            (&self.shader_in_out_data_buffer, 0),
                            (&self.output_buffer, 0),
                            self.buffer_size,
                        )],
                    )?;
                    self.run(args, src_data, out_data, cmd_buffer, &|_a, s, o| {
                        o.copy_from_slice(s);
                        Ok(())
                    })?;
                    Ok(true)
                }
                true => {
                    let cmd_buffer = self.create_cmd_buffer(
                        [work_items / sd.x_worksize, 1, 1],
                        *p,
                        &[
                            (
                                (&self.input_buffer, 0),
                                (&self.shader_in_data_buffer, 0),
                                self.buffer_size,
                            ),
                            (
                                (&self.input_b_buffer, 0),
                                (&self.shader_in_data_b_buffer, 0),
                                self.buffer_size,
                            ),
                            (
                                (&self.uniform_buffer, 0),
                                (&self.shader_uniform_buffer, 0),
                                std::mem::size_of::<KernelArgs>(),
                            ),
                        ],
                        &[(
                            (&self.shader_in_out_data_buffer, 0),
                            (&self.output_buffer, 0),
                            self.buffer_size,
                        )],
                    )?;
                    self.run(args, src_data, out_data, cmd_buffer, &|_a, s, o| {
                        o.copy_from_slice(s);
                        Ok(())
                    })?;
                    Ok(true)
                }
            }
        } else {
            Ok(false)
        }
    }

    //zz All done
}
